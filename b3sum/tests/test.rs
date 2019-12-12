use duct::cmd;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

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
    let dir = tempdir().unwrap();
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
fn test_hash_key() {
    let key = [42; blake3::KEY_LEN];
    let expected = blake3::keyed_hash(&key, b"foo").to_hex();
    let output = cmd!(b3sum_exe(), "--key", hex::encode(&key))
        .stdin_bytes("foo")
        .read()
        .unwrap();
    assert_eq!(&*expected, &*output);
}

#[test]
fn test_derive_key() {
    let key = &[99; blake3::KEY_LEN];
    let expected = hex::encode(&blake3::derive_key(key, b"context")[..]);
    let output = cmd!(b3sum_exe(), "--derive-key", hex::encode(key))
        .stdin_bytes("context")
        .read()
        .unwrap();
    assert_eq!(&*expected, &*output);
}
