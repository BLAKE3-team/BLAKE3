use std::path::Path;

#[test]
fn test_parse_check_line() {
    // =========================
    // ===== Success Cases =====
    // =========================

    // the basic case
    let crate::CheckLine {
        file_string,
        is_escaped,
        file_path,
        expected_hash,
    } = "0909090909090909090909090909090909090909090909090909090909090909  foo"
        .parse()
        .unwrap();
    assert_eq!(expected_hash, blake3::Hash::from([0x09; 32]));
    assert!(!is_escaped);
    assert_eq!(file_string, "foo");
    assert_eq!(file_path, Path::new("foo"));

    // regular whitespace
    let crate::CheckLine {
        file_string,
        is_escaped,
        file_path,
        expected_hash,
    } = "fafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafafa  fo \to\n\n\n"
        .parse()
        .unwrap();
    assert_eq!(expected_hash, blake3::Hash::from([0xfa; 32]));
    assert!(!is_escaped);
    assert_eq!(file_string, "fo \to");
    assert_eq!(file_path, Path::new("fo \to"));

    // path is one space
    let crate::CheckLine {
        file_string,
        is_escaped,
        file_path,
        expected_hash,
    } = "4242424242424242424242424242424242424242424242424242424242424242   "
        .parse()
        .unwrap();
    assert_eq!(expected_hash, blake3::Hash::from([0x42; 32]));
    assert!(!is_escaped);
    assert_eq!(file_string, " ");
    assert_eq!(file_path, Path::new(" "));

    // *Unescaped* backslashes. Note that this line does *not* start with a
    // backslash, so something like "\" + "n" is interpreted as *two*
    // characters. We forbid all backslashes on Windows, so this test is
    // Unix-only.
    if cfg!(not(windows)) {
        let crate::CheckLine {
            file_string,
            is_escaped,
            file_path,
            expected_hash,
        } = "4343434343434343434343434343434343434343434343434343434343434343  fo\\a\\no"
            .parse()
            .unwrap();
        assert_eq!(expected_hash, blake3::Hash::from([0x43; 32]));
        assert!(!is_escaped);
        assert_eq!(file_string, "fo\\a\\no");
        assert_eq!(file_path, Path::new("fo\\a\\no"));
    }

    // escaped newline
    let crate::CheckLine {
        file_string,
        is_escaped,
        file_path,
        expected_hash,
    } = "\\4444444444444444444444444444444444444444444444444444444444444444  fo\\n\\no"
        .parse()
        .unwrap();
    assert_eq!(expected_hash, blake3::Hash::from([0x44; 32]));
    assert!(is_escaped);
    assert_eq!(file_string, "fo\\n\\no");
    assert_eq!(file_path, Path::new("fo\n\no"));

    // Escaped newline and backslash. Again because backslash is not allowed on
    // Windows, this test is Unix-only.
    if cfg!(not(windows)) {
        let crate::CheckLine {
            file_string,
            is_escaped,
            file_path,
            expected_hash,
        } = "\\4545454545454545454545454545454545454545454545454545454545454545  fo\\n\\\\o"
            .parse()
            .unwrap();
        assert_eq!(expected_hash, blake3::Hash::from([0x45; 32]));
        assert!(is_escaped);
        assert_eq!(file_string, "fo\\n\\\\o");
        assert_eq!(file_path, Path::new("fo\n\\o"));
    }

    // non-ASCII path
    let crate::CheckLine {
        file_string,
        is_escaped,
        file_path,
        expected_hash,
    } = "4646464646464646464646464646464646464646464646464646464646464646  否认"
        .parse()
        .unwrap();
    assert_eq!(expected_hash, blake3::Hash::from([0x46; 32]));
    assert!(!is_escaped);
    assert_eq!(file_string, "否认");
    assert_eq!(file_path, Path::new("否认"));

    // =========================
    // ===== Failure Cases =====
    // =========================

    // too short
    "".parse::<crate::CheckLine>().unwrap_err();
    "0".parse::<crate::CheckLine>().unwrap_err();
    "00".parse::<crate::CheckLine>().unwrap_err();
    "0000000000000000000000000000000000000000000000000000000000000000"
        .parse::<crate::CheckLine>()
        .unwrap_err();
    "0000000000000000000000000000000000000000000000000000000000000000  "
        .parse::<crate::CheckLine>()
        .unwrap_err();

    // not enough spaces
    "0000000000000000000000000000000000000000000000000000000000000000 foo"
        .parse::<crate::CheckLine>()
        .unwrap_err();

    // capital letter hex
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA  foo"
        .parse::<crate::CheckLine>()
        .unwrap_err();

    // non-hex hex
    "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx  foo"
        .parse::<crate::CheckLine>()
        .unwrap_err();

    // non-ASCII hex
    "你好, 我叫杰克. 认识你很高兴. 要不要吃个香蕉?  foo"
        .parse::<crate::CheckLine>()
        .unwrap_err();

    // invalid escape sequence
    "\\0000000000000000000000000000000000000000000000000000000000000000  fo\\o"
        .parse::<crate::CheckLine>()
        .unwrap_err();

    // truncated escape sequence
    "\\0000000000000000000000000000000000000000000000000000000000000000  foo\\"
        .parse::<crate::CheckLine>()
        .unwrap_err();

    // null char
    "0000000000000000000000000000000000000000000000000000000000000000  fo\0o"
        .parse::<crate::CheckLine>()
        .unwrap_err();

    // Unicode replacement char
    "0000000000000000000000000000000000000000000000000000000000000000  fo�o"
        .parse::<crate::CheckLine>()
        .unwrap_err();

    // On Windows only, backslashes are not allowed, escaped or otherwise.
    if cfg!(windows) {
        "0000000000000000000000000000000000000000000000000000000000000000  fo\\o"
            .parse::<crate::CheckLine>()
            .unwrap_err();
        "\\0000000000000000000000000000000000000000000000000000000000000000  fo\\\\o"
            .parse::<crate::CheckLine>()
            .unwrap_err();
    }
}
