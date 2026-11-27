#! /usr/bin/env python3

from pathlib import Path
import subprocess
import tempfile


def main():
    source_dir = Path(__file__).resolve().parent.as_posix()
    for shared in ("OFF", "ON"):
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            build_dir = root / "build"
            build_dir.mkdir()
            install_dir = root / "install"
            (root / "CMakeLists.txt").write_text(
                "cmake_minimum_required(VERSION 3.9)\n"
                "project(blake3_install_test LANGUAGES C CXX)\n"
                f'add_subdirectory("{source_dir}" blake3)\n')
            # A parent's file must not be installed in place of our generated one.
            (build_dir / "libblake3.pc").write_text("parent project sentinel\n")
            subprocess.run([
                "cmake", str(root),
                f"-DCMAKE_INSTALL_PREFIX={install_dir.as_posix()}",
                "-DCMAKE_INSTALL_LIBDIR=lib",
                "-DCMAKE_BUILD_TYPE=Release",
                f"-DBUILD_SHARED_LIBS={shared}",
                "-DBLAKE3_USE_TBB=OFF",
                "-DBLAKE3_FETCH_TBB=OFF",
            ], cwd=build_dir, check=True)
            subprocess.run([
                "cmake", "--build", str(build_dir),
                "--target", "install", "--config", "Release",
            ], check=True)
            generated = build_dir / "blake3" / "libblake3.pc"
            installed = install_dir / "lib" / "pkgconfig" / "libblake3.pc"
            assert installed.read_bytes() == generated.read_bytes()


if __name__ == "__main__":
    main()
