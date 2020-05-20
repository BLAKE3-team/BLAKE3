//! GPU acceleration for BLAKE3.
//!
//! This module allows accelerating a [`Hasher`] through SPIR-V shaders.
//!
//! [`Hasher`]: ../struct.Hasher.html

use super::*;
use core::mem;
use core::ops::{Deref, DerefMut};
use core::slice;

/// Control uniform for the BLAKE3 shader.
///
/// This uniform contains the information necessary for a BLAKE3 shader to
/// correctly hash one level of the BLAKE3 tree structure.
#[repr(C)]
#[derive(Clone)]
pub struct GpuControl {
    k: [u32; 8],
    t: [u32; 2],
    d: u32,
}

impl GpuControl {
    fn new(key: &CVWords, chunk_counter: u64, flags: u8) -> Self {
        Self {
            k: *key,
            t: [counter_low(chunk_counter), counter_high(chunk_counter)],
            d: flags.into(),
        }
    }

    fn plus_chunks(&self, chunks: u64) -> Self {
        let t = self.chunk_counter() + chunks;
        Self {
            k: self.k,
            t: [counter_low(t), counter_high(t)],
            d: self.d,
        }
    }

    #[inline]
    fn key(&self) -> &CVWords {
        &self.k
    }

    #[inline]
    fn chunk_counter(&self) -> u64 {
        self.t[0] as u64 | (self.t[1] as u64) << 32
    }

    #[inline]
    fn flags(&self) -> u8 {
        self.d as u8
    }

    /// Returns the bytes to be copied to the control uniform in the GPU.
    ///
    /// The contents of the returned slice are opaque and should be interpreted
    /// only by the shader.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // According to the specification, the host and the device must have
        // the same endianness, so no endian conversion is necessary even on
        // big-endian hosts.
        debug_assert_eq!(
            mem::size_of_val(self),
            shaders::blake3::CONTROL_UNIFORM_SIZE,
            "must not have padding"
        );
        unsafe { slice::from_raw_parts(self as *const Self as *const u8, mem::size_of_val(self)) }
    }
}

// Variant of compress_subtree_wide which takes parents as input.
fn compress_parents_wide<J: Join>(
    input: &[u8],
    key: &CVWords,
    flags: u8,
    platform: Platform,
    out: &mut [u8],
) -> usize {
    debug_assert!(input.len().is_power_of_two());

    // Note that the single block case does *not* bump the SIMD degree up to 2
    // when it is 1. This allows Rayon the option of multi-threading even the
    // 2-block case, which can help performance on smaller platforms.
    if input.len() <= platform.simd_degree() * BLOCK_LEN {
        return compress_parents_parallel(input, key, flags, platform, out);
    }

    // With more than simd_degree blocks, we need to recurse. Start by dividing
    // the input into left and right subtrees. (Note that this is only optimal
    // as long as the SIMD degree is a power of 2. If we ever get a SIMD degree
    // of 3 or something, we'll need a more complicated strategy.)
    debug_assert_eq!(platform.simd_degree().count_ones(), 1, "power of 2");
    let (left, right) = input.split_at(input.len() / 2);

    // Make space for the child outputs. Here we use MAX_SIMD_DEGREE_OR_2 to
    // account for the special case of returning 2 outputs when the SIMD degree
    // is 1.
    let mut cv_array = [0; 2 * MAX_SIMD_DEGREE_OR_2 * OUT_LEN];
    let degree = if left.len() == BLOCK_LEN {
        // The "simd_degree=1 and we're at the leaf nodes" case.
        debug_assert_eq!(platform.simd_degree(), 1);
        1
    } else {
        cmp::max(platform.simd_degree(), 2)
    };
    let (left_out, right_out) = cv_array.split_at_mut(degree * OUT_LEN);

    // Recurse! This uses multiple threads if the "rayon" feature is enabled.
    let (left_n, right_n) = J::join(
        || compress_parents_wide::<J>(left, key, flags, platform, left_out),
        || compress_parents_wide::<J>(right, key, flags, platform, right_out),
        left.len(),
        right.len(),
    );

    // The special case again. If simd_degree=1, then we'll have left_n=1 and
    // right_n=1. Rather than compressing them into a single output, return
    // them directly, to make sure we always have at least two outputs.
    debug_assert_eq!(left_n, degree);
    debug_assert!(right_n >= 1 && right_n <= left_n);
    if left_n == 1 {
        out[..2 * OUT_LEN].copy_from_slice(&cv_array[..2 * OUT_LEN]);
        return 2;
    }

    // Otherwise, do one layer of parent node compression.
    let num_children = left_n + right_n;
    compress_parents_parallel(
        &cv_array[..num_children * OUT_LEN],
        key,
        flags,
        platform,
        out,
    )
}

// Variant of compress_subtree_to_parent_node which takes parents as input.
fn compress_parents_to_parent_node<J: Join>(
    input: &[u8],
    key: &CVWords,
    flags: u8,
    platform: Platform,
) -> [u8; BLOCK_LEN] {
    debug_assert!(input.len() > BLOCK_LEN);
    let mut cv_array = [0; 2 * MAX_SIMD_DEGREE_OR_2 * OUT_LEN];
    let mut num_cvs = compress_parents_wide::<J>(input, &key, flags, platform, &mut cv_array);
    debug_assert!(num_cvs >= 2);

    // If MAX_SIMD_DEGREE is greater than 2 and there's enough input,
    // compress_parents_wide() returns more than 2 chaining values. Condense
    // them into 2 by forming parent nodes repeatedly.
    let mut out_array = [0; MAX_SIMD_DEGREE_OR_2 * OUT_LEN / 2];
    while num_cvs > 2 {
        let cv_slice = &cv_array[..num_cvs * OUT_LEN];
        num_cvs = compress_parents_parallel(cv_slice, key, flags, platform, &mut out_array);
        cv_array[..num_cvs * OUT_LEN].copy_from_slice(&out_array[..num_cvs * OUT_LEN]);
    }
    *array_ref!(cv_array, 0, 2 * OUT_LEN)
}

/// GPU-accelerated Hasher.
///
/// This is a wrapper around a [`Hasher`] which also allows exporting the key
/// and flags to be used by a GPU shader, and importing the shader's result.
///
/// This wrapper should be used with care, since incorrect use can lead to a
/// wrong hash output. It also allows extracting the key from the state, which
/// would otherwise not be allowed in safe code.
///
/// This wrapper can be freely converted to its inner [`Hasher`], through the
/// `Deref`, `DerefMut`, and `Into` traits. Prefer to use the inner [`Hasher`]
/// wherever the extra functionality from this wrapper is not needed.
///
/// [`Hasher`]: ../struct.Hasher.html
#[derive(Clone, Debug, Default)]
pub struct GpuHasher {
    inner: Hasher,
}

impl GpuHasher {
    /// Wrapper for [`Hasher::new`](../struct.Hasher.html#method.new).
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Hasher::new(),
        }
    }

    /// Wrapper for [`Hasher::new_keyed`](../struct.Hasher.html#method.new_keyed).
    #[inline]
    pub fn new_keyed(key: &[u8; KEY_LEN]) -> Self {
        Self {
            inner: Hasher::new_keyed(key),
        }
    }

    /// Wrapper for [`Hasher::new_derive_key`](../struct.Hasher.html#method.new_derive_key).
    #[inline]
    pub fn new_derive_key(context: &str) -> Self {
        Self {
            inner: Hasher::new_derive_key(context),
        }
    }

    /// Obtain the [`GpuControl`](struct.GpuControl.html) to hash full chunks starting with `chunk_counter`
    /// or parent nodes.
    pub fn gpu_control(&self, chunk_counter: u64) -> GpuControl {
        GpuControl::new(&self.key, chunk_counter, self.chunk_state.flags)
    }

    /// GPU-accelerated version of [`update_with_join`].
    ///
    /// Unlike [`update_with_join`], this method receives the parents computed
    /// by one or more applications of the BLAKE3 shader.
    ///
    /// This method has several restrictions. The size of the shader input must
    /// be a power of two, it must be naturally aligned within the hash input,
    /// and the hasher state must not have any leftover bytes in its internal
    /// buffers. The simplest way to follow these invariants is to use this
    /// method, with the same chunk count and buffer size, for all of the input
    /// except for a variable-sized tail, which can use [`update_with_join`] or
    /// [`update`].
    ///
    /// Note: the chunk counter is implicit in this method, but it must be the
    /// same as the chunk counter in the [`GpuControl`] passed to the shader,
    /// otherwise it will lead to a wrong hash output.
    ///
    /// Note: on a big-endian host, this method will swap the endianness of the
    /// shader output in-place.
    ///
    /// [`update`]: #method.update
    /// [`update_with_join`]: #method.update_with_join
    /// [`GpuControl`]: struct.GpuControl.html
    pub fn update_from_gpu<J: Join>(&mut self, chunk_count: u64, parents: &mut [u8]) -> &mut Self {
        assert_eq!(self.chunk_state.len(), 0, "leftover buffered bytes");
        let chunk_counter = self.chunk_state.chunk_counter;

        // These three checks make sure the increment of t0 in the shader did not overflow.
        assert!(chunk_count.is_power_of_two(), "bad chunk count");
        assert!(chunk_count <= (1 << 32), "chunk count overflow");
        assert_eq!(chunk_counter % chunk_count, 0, "misaligned hash");

        assert_eq!(parents.len() % OUT_LEN, 0, "invalid hash size");
        let parent_count = (parents.len() / OUT_LEN) as u64;

        assert_eq!(chunk_count % parent_count, 0, "invalid child count");

        // The lazy merge of the CV stack needs at least 2 inputs.
        // And compress_parents_to_parent_node needs at least 2 blocks.
        assert!(parent_count > 2, "invalid parent count");

        // The shader inputs and outputs are 32-bit words, which are in native byte order.
        // The chunk shader byte swaps its input, but neither shader byte swaps its output.
        // Since the rest of the code assumes little endian, byte swap the buffer here.
        Self::swap_endian::<J>(parents);

        let cv_pair = compress_parents_to_parent_node::<J>(
            parents,
            &self.key,
            self.chunk_state.flags,
            self.chunk_state.platform,
        );
        let left_cv = array_ref!(cv_pair, 0, 32);
        let right_cv = array_ref!(cv_pair, 32, 32);
        // Push the two CVs we received into the CV stack in order. Because
        // the stack merges lazily, this guarantees we aren't merging the
        // root.
        self.push_cv(left_cv, chunk_counter);
        self.push_cv(right_cv, chunk_counter + (chunk_count / 2));
        self.chunk_state.chunk_counter += chunk_count;

        self
    }

    // CPU simulation of the BLAKE3 chunk shader.
    //
    // This can be used to test the real shader.
    //
    // Note: unlike the real shader, this simulation always uses little-endian
    // inputs and outputs.
    #[doc(hidden)]
    pub fn simulate_chunk_shader<J: Join>(
        &self,
        count: usize,
        input: &[u8],
        output: &mut [u8],
        control: &GpuControl,
    ) {
        assert_eq!(input.len(), count * CHUNK_LEN, "invalid input size");
        assert_eq!(output.len(), count * OUT_LEN, "invalid output size");

        if count > self.chunk_state.platform.simd_degree() {
            let mid = count / 2;
            let (left_in, right_in) = input.split_at(mid * CHUNK_LEN);
            let (left_out, right_out) = output.split_at_mut(mid * OUT_LEN);
            let control_r = control.plus_chunks(mid as u64);

            J::join(
                || self.simulate_chunk_shader::<J>(mid, left_in, left_out, control),
                || self.simulate_chunk_shader::<J>(count - mid, right_in, right_out, &control_r),
                left_in.len(),
                right_in.len(),
            );
        } else if count > 0 {
            let mut chunks = ArrayVec::<[&[u8; CHUNK_LEN]; MAX_SIMD_DEGREE]>::new();
            for chunk in input.chunks_exact(CHUNK_LEN) {
                chunks.push(array_ref!(chunk, 0, CHUNK_LEN));
            }
            self.chunk_state.platform.hash_many(
                &chunks,
                control.key(),
                control.chunk_counter(),
                IncrementCounter::Yes,
                control.flags(),
                CHUNK_START,
                CHUNK_END,
                output,
            );
        }
    }

    // CPU simulation of the BLAKE3 parent shader.
    //
    // This can be used to test the real shader.
    //
    // Note: unlike the real shader, this simulation always uses little-endian
    // inputs and outputs.
    #[doc(hidden)]
    pub fn simulate_parent_shader<J: Join>(
        &self,
        count: usize,
        input: &[u8],
        output: &mut [u8],
        control: &GpuControl,
    ) {
        assert_eq!(input.len(), count * BLOCK_LEN, "invalid input size");
        assert_eq!(output.len(), count * OUT_LEN, "invalid output size");

        if count > self.chunk_state.platform.simd_degree() {
            let mid = count / 2;
            let (left_in, right_in) = input.split_at(mid * BLOCK_LEN);
            let (left_out, right_out) = output.split_at_mut(mid * OUT_LEN);
            let control_r = control.plus_chunks(mid as u64);

            J::join(
                || self.simulate_parent_shader::<J>(mid, left_in, left_out, control),
                || self.simulate_parent_shader::<J>(count - mid, right_in, right_out, &control_r),
                left_in.len(),
                right_in.len(),
            );
        } else if count > 0 {
            let mut parents = ArrayVec::<[&[u8; BLOCK_LEN]; MAX_SIMD_DEGREE]>::new();
            for parent in input.chunks_exact(BLOCK_LEN) {
                parents.push(array_ref!(parent, 0, BLOCK_LEN));
            }
            self.chunk_state.platform.hash_many(
                &parents,
                control.key(),
                0,
                IncrementCounter::No,
                control.flags() | PARENT,
                0,
                0,
                output,
            );
        }
    }

    #[doc(hidden)]
    #[cfg(target_endian = "big")]
    pub fn swap_endian<J: Join>(buffer: &mut [u8]) {
        debug_assert!(buffer.len().is_power_of_two(), "invalid buffer size");
        debug_assert_eq!(buffer.len() % OUT_LEN, 0, "invalid buffer size");

        if buffer.len() > OUT_LEN {
            let (left, right) = buffer.split_at_mut(buffer.len() / 2);
            let left_len = left.len();
            let right_len = right.len();

            J::join(
                || Self::swap_endian::<J>(left),
                || Self::swap_endian::<J>(right),
                left_len,
                right_len,
            );
        } else {
            for buf in buffer.chunks_exact_mut(4) {
                buf.swap(0, 3);
                buf.swap(1, 2);
            }
        }
    }

    #[doc(hidden)]
    #[inline(always)]
    #[cfg(target_endian = "little")]
    pub fn swap_endian<J: Join>(_buffer: &mut [u8]) {}
}

impl Deref for GpuHasher {
    type Target = Hasher;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for GpuHasher {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl From<GpuHasher> for Hasher {
    #[inline]
    fn from(hasher: GpuHasher) -> Hasher {
        hasher.inner
    }
}

/// SPIR-V shader modules.
pub mod shaders {
    /// Shader module for one level of the BLAKE3 tree.
    pub mod blake3 {
        /// Returns the SPIR-V code for the chunk shader module.
        #[cfg(target_endian = "big")]
        pub fn chunk_shader() -> &'static [u8] {
            include_bytes!("shaders/blake3-chunk-be.spv")
        }

        /// Returns the SPIR-V code for the chunk shader module.
        #[cfg(target_endian = "little")]
        pub fn chunk_shader() -> &'static [u8] {
            include_bytes!("shaders/blake3-chunk-le.spv")
        }

        /// Returns the SPIR-V code for the parent shader module.
        pub fn parent_shader() -> &'static [u8] {
            include_bytes!("shaders/blake3-parent.spv")
        }

        /// The local workgroup size.
        pub const WORKGROUP_SIZE: usize = 128;

        /// The descriptor binding for the input buffer.
        pub const INPUT_BUFFER_BINDING: u32 = 0;
        /// The descriptor binding for the output buffer.
        pub const OUTPUT_BUFFER_BINDING: u32 = 1;

        /// The size of the control uniform.
        pub const CONTROL_UNIFORM_SIZE: usize = 11 * 4;
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    fn selftest_seq(len: usize) -> Vec<u8> {
        let seed = len as u32;
        let mut out = Vec::with_capacity(len);

        let mut a = seed.wrapping_mul(0xDEAD4BAD);
        let mut b = 1;

        for _ in 0..len {
            let t = a.wrapping_add(b);
            a = b;
            b = t;
            out.push((t >> 24) as u8);
        }

        out
    }

    #[cfg(not(feature = "rayon"))]
    type Join = join::SerialJoin;

    #[cfg(feature = "rayon")]
    type Join = join::RayonJoin;

    #[test]
    fn simulate_shader_one_level_once() {
        let len = CHUNK_LEN * 128;
        let input = selftest_seq(len);

        let expected = Hasher::new().update_with_join::<Join>(&input).finalize();

        let mut hasher = GpuHasher::new();
        let mut buffer = vec![0; OUT_LEN * 128];

        hasher.simulate_chunk_shader::<Join>(128, &input, &mut buffer, &hasher.gpu_control(0));
        GpuHasher::swap_endian::<Join>(&mut buffer);
        hasher.update_from_gpu::<Join>(128, &mut buffer);

        assert_eq!(hasher.finalize(), expected);
    }

    #[test]
    fn simulate_shader_one_level_twice() {
        let len = CHUNK_LEN * 128;
        let input = selftest_seq(2 * len);

        let expected = Hasher::new().update_with_join::<Join>(&input).finalize();

        let mut hasher = GpuHasher::new();
        let mut buffer = vec![0; OUT_LEN * 128];

        hasher.simulate_chunk_shader::<Join>(
            128,
            &input[..len],
            &mut buffer,
            &hasher.gpu_control(0),
        );
        GpuHasher::swap_endian::<Join>(&mut buffer);
        hasher.update_from_gpu::<Join>(128, &mut buffer);

        hasher.simulate_chunk_shader::<Join>(
            128,
            &input[len..],
            &mut buffer,
            &hasher.gpu_control(128),
        );
        GpuHasher::swap_endian::<Join>(&mut buffer);
        hasher.update_from_gpu::<Join>(128, &mut buffer);

        assert_eq!(hasher.finalize(), expected);
    }

    #[test]
    fn simulate_shader_two_levels_once() {
        let len = 2 * CHUNK_LEN * 128;
        let input = selftest_seq(len);

        let expected = Hasher::new().update_with_join::<Join>(&input).finalize();

        let mut hasher = GpuHasher::new();
        let mut buffer1 = vec![0; 2 * OUT_LEN * 128];
        let mut buffer2 = vec![0; OUT_LEN * 128];

        hasher.simulate_chunk_shader::<Join>(2 * 128, &input, &mut buffer1, &hasher.gpu_control(0));
        hasher.simulate_parent_shader::<Join>(128, &buffer1, &mut buffer2, &hasher.gpu_control(0));
        GpuHasher::swap_endian::<Join>(&mut buffer2);
        hasher.update_from_gpu::<Join>(2 * 128, &mut buffer2);

        assert_eq!(hasher.finalize(), expected);
    }

    #[test]
    fn simulate_shader_two_levels_twice() {
        let len = 2 * CHUNK_LEN * 128;
        let input = selftest_seq(2 * len);

        let expected = Hasher::new().update_with_join::<Join>(&input).finalize();

        let mut hasher = GpuHasher::new();
        let mut buffer1 = vec![0; 2 * OUT_LEN * 128];
        let mut buffer2 = vec![0; OUT_LEN * 128];

        hasher.simulate_chunk_shader::<Join>(
            2 * 128,
            &input[..len],
            &mut buffer1,
            &hasher.gpu_control(0),
        );
        hasher.simulate_parent_shader::<Join>(128, &buffer1, &mut buffer2, &hasher.gpu_control(0));
        GpuHasher::swap_endian::<Join>(&mut buffer2);
        hasher.update_from_gpu::<Join>(2 * 128, &mut buffer2);

        hasher.simulate_chunk_shader::<Join>(
            2 * 128,
            &input[len..],
            &mut buffer1,
            &hasher.gpu_control(2 * 128),
        );
        hasher.simulate_parent_shader::<Join>(
            128,
            &buffer1,
            &mut buffer2,
            &hasher.gpu_control(2 * 128),
        );
        GpuHasher::swap_endian::<Join>(&mut buffer2);
        hasher.update_from_gpu::<Join>(2 * 128, &mut buffer2);

        assert_eq!(hasher.finalize(), expected);
    }
}
