#include <variant>

#include <llfio.hpp>

#include "blake3.h"
#include "blake3_impl.h"

namespace llfio = LLFIO_V2_NAMESPACE;

template <class... Ts> struct overloads : Ts... {
  using Ts::operator()...;
};
// NOTE: deduction guide only needed for Apple Clang now.
template <class... Ts> overloads(Ts...) -> overloads<Ts...>;

INLINE auto copy_wide(blake3_hasher *self, llfio::path_view path,
                      llfio::file_handle &file) noexcept
    -> llfio::result<std::uint64_t> {
  std::array<std::byte, 65535> buffer{};
  std::uint64_t total = 0;
  while (true) {
    auto result = llfio::read(file, total, {buffer});
    // Errors other than "interrupted" are immediately forwarded to caller.
    if (result.has_error() && result.error() != llfio::errc::interrupted) {
      return std::move(result).as_failure();
    }
    if (result.has_value()) {
      auto bytes_read = result.value();
      blake3_hasher_update(self, buffer.data(), bytes_read);
      total += bytes_read;
      if (bytes_read == 0) {
        break;
      }
    }
  }
  return total;
}

INLINE auto maybe_mmap_file(llfio::file_handle &&file) noexcept
    -> llfio::result<std::variant<
        std::pair<llfio::mapped_file_handle, llfio::file_handle::extent_type>,
        llfio::file_handle>> {
  OUTCOME_TRY(auto file_size, file.maximum_extent());
  if (!file.is_regular()) {
    // Not a real file.
    return file;
  } else if (file_size < 16 * 1024) {
    // Mapping small files is not worth it, and some special files that can't be
    // mapped report a size of zero.
    return file;
  } else {
    auto &&map = llfio::mapped_file_handle(
        std::move(file),                   // transfer ownership
        0,                                 // reserve maximum extent
        llfio::section_handle::flag::read, // map as read-only
        0                                  // map from the start
    );
    return std::pair{std::move(map), file_size};
  }
}

INLINE void blake3_hasher_update_mmap_base(blake3_hasher *self,
                                           char const *path,
                                           bool use_tbb) noexcept {
  auto result = [=]() -> llfio::result<void> {
    OUTCOME_TRY(auto file, llfio::file({}, path));
    OUTCOME_TRY(auto mmap, maybe_mmap_file(std::move(file)));
    OUTCOME_TRY(
        std::visit(overloads{
                       [=](std::pair<llfio::mapped_file_handle,
                                     llfio::file_handle::extent_type> &pair)
                           -> llfio::result<void> {
                         blake3_hasher_update_base(self, pair.first.address(),
                                                   pair.second, use_tbb);
                         return llfio::success();
                       },
                       [=](llfio::file_handle &file) -> llfio::result<void> {
                         OUTCOME_TRY(copy_wide(self, path, file));
                         return llfio::success();
                       },
                   },
                   mmap));
    return llfio::success();
  }();
  if (result.has_error()) {
    // Explicitly set errno on error since this doesn't always happen
    // automatically on some platforms such as Windows.
    errno = static_cast<int>(result.error().value().sc);
  }
}

void blake3_hasher_update_mmap(blake3_hasher *self, char const *path) noexcept {
  bool use_tbb = false;
  blake3_hasher_update_mmap_base(self, path, use_tbb);
}

#if defined(BLAKE3_USE_TBB)
void blake3_hasher_update_mmap_tbb(blake3_hasher *self,
                                   char const *path) noexcept {
  bool use_tbb = true;
  blake3_hasher_update_mmap_base(self, path, use_tbb);
}
#endif // BLAKE3_USE_TBB
