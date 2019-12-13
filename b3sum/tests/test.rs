use duct::cmd;
use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

pub fn b3sum_exe() -> PathBuf {
    assert_cmd::cargo::cargo_bin("b3sum")
}

#[test]
fn test_hash_one() {
    let expected = blake3::hash(b"foo").to_hex();
    let output = cmd!(b3sum_exe()).stdin_bytes("foo").read().unwrap();
    assert_eq!(&*expected, &*output);
}

#[test]
fn test_hash_many() {
    let dir = tempfile::tempdir().unwrap();
    let file1 = dir.path().join("file1");
    fs::write(&file1, b"foo").unwrap();
    let file2 = dir.path().join("file2");
    fs::write(&file2, b"bar").unwrap();

    let output = cmd!(b3sum_exe(), &file1, &file2).read().unwrap();
    let foo_hash = blake3::hash(b"foo");
    let bar_hash = blake3::hash(b"bar");
    let expected = format!(
        "{}  {}\n{}  {}",
        foo_hash.to_hex(),
        file1.to_string_lossy(),
        bar_hash.to_hex(),
        file2.to_string_lossy(),
    );
    assert_eq!(expected, output);

    let output_no_names = cmd!(b3sum_exe(), "--no-names", &file1, &file2)
        .read()
        .unwrap();
    let expected_no_names = format!("{}\n{}", foo_hash.to_hex(), bar_hash.to_hex(),);
    assert_eq!(expected_no_names, output_no_names);
}

#[test]
fn test_hash_length() {
    let mut buf = [0; 100];
    blake3::Hasher::new()
        .update(b"foo")
        .finalize_xof()
        .fill(&mut buf);
    let expected = hex::encode(&buf[..]);
    let output = cmd!(b3sum_exe(), "--length=100")
        .stdin_bytes("foo")
        .read()
        .unwrap();
    assert_eq!(&*expected, &*output);
}

#[test]
fn test_keyed() {
    let key = [42; blake3::KEY_LEN];
    let f = tempfile::NamedTempFile::new().unwrap();
    f.as_file().write_all(b"foo").unwrap();
    f.as_file().flush().unwrap();
    let expected = blake3::keyed_hash(&key, b"foo").to_hex();
    let output = cmd!(b3sum_exe(), "--keyed", "--no-names", f.path())
        .stdin_bytes(&key[..])
        .read()
        .unwrap();
    assert_eq!(&*expected, &*output);
}

#[test]
fn test_derive_key() {
    let key = [99; blake3::KEY_LEN];
    let f = tempfile::NamedTempFile::new().unwrap();
    f.as_file().write_all(b"context").unwrap();
    f.as_file().flush().unwrap();
    let expected = hex::encode(blake3::derive_key(&key, b"context"));
    let output = cmd!(b3sum_exe(), "--derive-key", "--no-names", f.path())
        .stdin_bytes(&key[..])
        .read()
        .unwrap();
    assert_eq!(&*expected, &*output);
}
