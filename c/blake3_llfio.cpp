#include <array>
#include <utility>
#include <variant>

#include <llfio.hpp>
#include <outcome.hpp>
#include <quickcpplib/signal_guard.hpp>

#include "blake3.h"
#include "blake3_impl.h"

#ifdef _WIN32
#include <windows.h>
#include <winternl.h>
#endif

namespace llfio = LLFIO_V2_NAMESPACE;
namespace quickcpp = QUICKCPPLIB_NAMESPACE;

namespace {
template <class... Ts> struct overloads : Ts... {
  using Ts::operator()...;
};
// NOTE: Deduction guide only needed for Apple Clang.
template <class... Ts> overloads(Ts...) -> overloads<Ts...>;

template <class T> auto error_code(T &&error) -> long long {
  const auto value = error.value();
#if LLFIO_EXPERIMENTAL_STATUS_CODE
  return value.sc;
#elif !defined(_WIN32)
  return value;
#else
  return NT_ERROR(value) ? RtlNtStatusToDosError(value) : value;
#endif
}

namespace {
const quickcpp::signal_guard::signalc_set guarded_signals =
    quickcpp::signal_guard::signalc_set::undefined_memory_access // SIGBUS
    | quickcpp::signal_guard::signalc_set::segmentation_fault    // SIGSEGV
    ;

// Install and enable global signal handlers for specified signals.
static const quickcpp::signal_guard::signal_guard_install
    signal_guard_install(guarded_signals);
} // namespace
} // namespace

INLINE auto copy_wide(blake3_hasher *self, llfio::file_handle &file) noexcept
    -> llfio::result<void> {
  std::array<std::byte, 65536> buffer;
  std::uint64_t offset = 0;
  while (true) {
    std::uint64_t bytes_read = 0;
    if (auto result = llfio::read(file, offset, {buffer})) {
      bytes_read = result.assume_value();
      blake3_hasher_update(self, buffer.data(), bytes_read);
      offset += bytes_read;
    } else {
#ifdef _WIN32
      if (error_code(result.assume_error()) == ERROR_HANDLE_EOF) {
        break;
      }
#endif
      if (result.assume_error() != llfio::errc::interrupted) {
        return std::move(result).as_failure();
      }

      continue;
    }

    if (bytes_read < buffer.size()) {
      break;
    }
  }
  return llfio::success();
}

INLINE auto maybe_mmap_path(char const *path) noexcept -> llfio::result<
    std::variant<llfio::mapped_file_handle, llfio::file_handle>> {
  const auto base_dir = llfio::path_handle{};
  if (auto mmap = llfio::mapped_file(base_dir, path); mmap.has_value()) {
    return std::move(mmap).value();
  }
  // Memory mapping may fail so fall back to normal file reading if necessary.
  OUTCOME_TRY(auto file, llfio::file(base_dir, path));
  return file;
}

INLINE int blake3_hasher_update_mmap_base(blake3_hasher *self, char const *path,
                                          bool use_tbb) noexcept {
  auto result = [=]() -> llfio::result<void> {
    OUTCOME_TRY(auto handle, maybe_mmap_path(path));
    OUTCOME_TRY(std::visit(
        overloads{
            [=](llfio::mapped_file_handle &mmap) -> llfio::result<void> {
              OUTCOME_TRY(const auto extent, mmap.maximum_extent());
              const auto result = quickcpp::signal_guard::signal_guard(
                  guarded_signals,
                  [&]() -> bool {
                    const auto data = mmap.address();
                    blake3_hasher_update_base(self, data, extent, use_tbb);
                    return true;
                  },
                  [=]([[maybe_unused]] const auto *info) -> bool {
                    return false;
                  });
              if (!result) {
                return llfio::errc::io_error;
              }
              return llfio::success();
            },
            [=](llfio::file_handle &file) -> llfio::result<void> {
              OUTCOME_TRY(copy_wide(self, file));
              return llfio::success();
            },
        },
        handle));
    return llfio::success();
  }();

  if (result.has_error()) {
    return error_code(result.assume_error());
  }

  return 0;
}

extern "C" int blake3_hasher_update_mmap(blake3_hasher *self,
                                         char const *path) noexcept {
  bool use_tbb = false;
  return blake3_hasher_update_mmap_base(self, path, use_tbb);
}

#if defined(BLAKE3_USE_TBB)
extern "C" int blake3_hasher_update_mmap_tbb(blake3_hasher *self,
                                             char const *path) noexcept {
  bool use_tbb = true;
  return blake3_hasher_update_mmap_base(self, path, use_tbb);
}
#endif // BLAKE3_USE_TBB
