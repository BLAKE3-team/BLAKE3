//! The multi-threading abstractions used by [`Hasher::update_with_join`].
//!
//! Different implementations of the `Join` trait determine whether
//! [`Hasher::update_with_join`] performs multi-threading on sufficiently large
//! inputs. The `SerialJoin` implementation is single-threaded, and the
//! `RayonJoin` implementation (gated by the `rayon` feature) is
//! multi-threaded. Interfaces other than [`Hasher::update_with_join`], like
//! [`hash`] and [`Hasher::update`], always use `SerialJoin` internally.
//!
//! The `Join` trait is an almost exact copy of the [`rayon::join`] API, and
//! `RayonJoin` is the only non-trivial implementation provided. The only
//! difference between the function signature in the `Join` trait and the
//! underlying one in Rayon, is that the trait method includes two length
//! parameters. This gives an implementation the option of e.g. setting a
//! subtree size threshold below which it keeps splits on the same thread.
//! However, neither of the two provided implementations currently makes use of
//! those parameters. Note that in Rayon, the very first `join` call is more
//! expensive than subsequent calls, because it moves work from the calling
//! thread into the thread pool. That makes a coarse-grained input length
//! threshold in the caller more effective than a fine-grained subtree size
//! threshold after the implementation has already started recursing.
//!
//! # Example
//!
//! ```
//! // Hash a large input using multi-threading. Note that multi-threading
//! // comes with some overhead, and it can actually hurt performance for small
//! // inputs. The meaning of "small" varies, however, depending on the
//! // platform and the number of threads. (On x86_64, the cutoff tends to be
//! // around 128 KiB.) You should benchmark your own use case to see whether
//! // multi-threading helps.
//! # #[cfg(feature = "rayon")]
//! # {
//! # fn some_large_input() -> &'static [u8] { b"foo" }
//! let input: &[u8] = some_large_input();
//! let mut hasher = blake3::Hasher::new();
//! hasher.update_with_join::<blake3::join::RayonJoin>(input);
//! let hash = hasher.finalize();
//! # }
//! ```
//!
//! [`Hasher::update_with_join`]: ../struct.Hasher.html#method.update_with_join
//! [`Hasher::update`]: ../struct.Hasher.html#method.update
//! [`hash`]: ../fn.hash.html
//! [`rayon::join`]: https://docs.rs/rayon/1.3.0/rayon/fn.join.html

/// The trait that abstracts over single-threaded and multi-threaded recursion.
///
/// See the [`join` module docs](index.html) for more details.
pub trait Join {
    fn join<A, B, RA, RB>(oper_a: A, oper_b: B, len_a: usize, len_b: usize) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send;
}

/// The trivial, serial implementation of `Join`. The left and right sides are
/// executed one after the other, on the calling thread. The standalone hashing
/// functions and the `Hasher::update` method use this implementation
/// internally.
///
/// See the [`join` module docs](index.html) for more details.
pub enum SerialJoin {}

impl Join for SerialJoin {
    #[inline]
    fn join<A, B, RA, RB>(oper_a: A, oper_b: B, _len_a: usize, _len_b: usize) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        (oper_a(), oper_b())
    }
}

/// The Rayon-based implementation of `Join`. The left and right sides are
/// executed on the Rayon thread pool, potentially in parallel. This
/// implementation is gated by the `rayon` feature, which is off by default.
///
/// See the [`join` module docs](index.html) for more details.
#[cfg(feature = "rayon")]
pub enum RayonJoin {}

#[cfg(feature = "rayon")]
impl Join for RayonJoin {
    #[inline]
    fn join<A, B, RA, RB>(oper_a: A, oper_b: B, _len_a: usize, _len_b: usize) -> (RA, RB)
    where
        A: FnOnce() -> RA + Send,
        B: FnOnce() -> RB + Send,
        RA: Send,
        RB: Send,
    {
        rayon::join(oper_a, oper_b)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_serial_join() {
        let oper_a = || 1 + 1;
        let oper_b = || 2 + 2;
        assert_eq!((2, 4), SerialJoin::join(oper_a, oper_b, 3, 4));
    }

    #[test]
    #[cfg(feature = "rayon")]
    fn test_rayon_join() {
        let oper_a = || 1 + 1;
        let oper_b = || 2 + 2;
        assert_eq!((2, 4), RayonJoin::join(oper_a, oper_b, 3, 4));
    }
}
