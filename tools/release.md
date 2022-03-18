# Release checklist

- Make sure `cargo outdated -R` is clean in the root and in b3sum/.
- Bump the version in the root Cargo.toml.
- Bump the version in b3sum/Cargo.toml.
- Delete b3sum/Cargo.lock and recreate it with `cargo build` or similar.
- Update the `--help` output (including the version number) in b3sum/README.md.
- Bump `BLAKE3_VERSION_STRING` in c/blake3.h.
- Make a version bump commit with change notes.
- `git push` and make sure CI is green.
- `git tag` the version bump commit with the new version number.
- `git push --tags`
- `cargo publish` in the root.
- `cargo publish --dry-run` in b3sum/ and make sure it fetches the just-published library version.
- `cargo publish` in b3sum/.
