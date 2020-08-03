#! /usr/bin/env python3

from binascii import hexlify
import json
from os import path
import subprocess
import shlex
import base64

HERE = path.dirname(__file__)
TEST_VECTORS_PATH = path.join(HERE, "..", "test_vectors", "test_vectors.json")
TEST_VECTORS = json.load(open(TEST_VECTORS_PATH))

def TIS_MAKE_TEST(test_no, machdep, test_name, expected_name, args, tis_config_file):
    print("===", str(test_no), ":", test_name, "===")
    # print("RUN", [path.join(HERE, "blake3")] + args)

    name = "Test vector %02d: %s" % (test_no, test_name)
    print("  \"name\": \"%s\"" % name)

    val_args_full=""
    if args:
        # val_args = "%" + "%".join(map(shlex.quote, args))
        val_args = "%" + "%".join(args)
        print("  \"-val-args\": \"%s\"" % val_args)
        val_args_full=",\n\
    \"val-args\": \"%s\"" % val_args

    input_filename = "tis-ci/test_vectors/%02d_input.bin" % test_no
    expected_filename = "tis-ci/test_vectors/%02d_%s" % (test_no, expected_name)
    print("  INPUT FILE = %s" % input_filename)
    print("  EXPECTED FILE = %s" % expected_filename)

    maybe_no_results=""
    if test_no >= 22:
        maybe_no_results=",\n\
    \"no-results\": true"

    tis_config_file.write(
"  {\n\
    \"name\": \"%s (%s)\",\n\
    \"files\": [\n\
      \"tis-ci/test.c\",\n\
      \"c/main.c\",\n\
      \"c/blake3.c\",\n\
      \"c/blake3_dispatch.c\",\n\
      \"c/blake3_portable.c\"\n\
    ],\n\
    \"main\": \"main_wrapper\",\n\
    \"compilation_cmd\": \"-I . -I c -DBLAKE3_TESTING -DBLAKE3_NO_SSE41 -DBLAKE3_NO_AVX2 -DBLAKE3_NO_AVX512 -U__clang__ -U__GNUC__ -U__x86_64__ -U__i386__\",\n\
    \"machdep\": \"%s\",\n\
    \"filesystem\": {\n\
      \"files\": [\n\
        {\n\
          \"name\": \"tis-mkfs-stdin\",\n\
          \"from\": \"%s\"\n\
        },\n\
        {\n\
          \"name\": \"expected\",\n\
          \"from\": \"%s\"\n\
        }\n\
      ]\n\
    }%s%s\n\
  }" % (name, machdep, machdep, input_filename, expected_filename, val_args_full, maybe_no_results)
    )


def run_blake3(args, input):
    output = subprocess.run([path.join(HERE, "blake3")] + args,
                            input=input,
                            stdout=subprocess.PIPE,
                            check=True)
    return output.stdout.decode().strip()


# Fill the input with a repeating byte pattern. We use a cycle length of 251,
# because that's the largets prime number less than 256. This makes it unlikely
# to swapping any two adjacent input blocks or chunks will give the same
# answer.
def make_test_input(length):
    i = 0
    buf = bytearray()
    while len(buf) < length:
        buf.append(i)
        i = (i + 1) % 251
    return buf

def write_test_vector_file(test_no, name, content):
    print("-<", name, ">-")
    file_name = "../tis-ci/test_vectors/%02d_%s" % (test_no, name)
    # print(content)
    file = open(file_name, "w")
    file.write(content)
    file.close()

def write_test_vector_file_binary(test_no, name, content):
    print("-<", name, ">-")
    file_name = "../tis-ci/test_vectors/%02d_%s.bin" % (test_no, name)
    # print(content[:32], "...")
    file = open(file_name, "wb")
    file.write(content)
    file.close()

def main():
    tis_config_file = open("test.py_tis.config", "w")
    tis_config_file.write("[")

    test_no = 0
    beginning = 0
    machdeps = ["gcc_x86_32", "gcc_x86_64", "ppc_32", "ppc_64"]
    for case in TEST_VECTORS["cases"]:

        test_no += 1
        print("--- Test case", test_no, "---")

        input_len = case["input_len"]
        input = make_test_input(input_len)
        key = TEST_VECTORS["key"]
        hex_key = hexlify(TEST_VECTORS["key"].encode())
        context_string = TEST_VECTORS["context_string"]
        expected_hash_xof = case["hash"]
        expected_hash = expected_hash_xof[:64]
        expected_keyed_hash_xof = case["keyed_hash"]
        expected_keyed_hash = expected_keyed_hash_xof[:64]
        expected_derive_key_xof = case["derive_key"]
        expected_derive_key = expected_derive_key_xof[:64]

        write_test_vector_file_binary(test_no, "input", input)
        write_test_vector_file(test_no, "expected_hash_xof", expected_hash_xof)
        write_test_vector_file(test_no, "expected_hash", expected_hash)
        write_test_vector_file(test_no, "expected_keyed_hash_xof", expected_keyed_hash_xof)
        write_test_vector_file(test_no, "expected_keyed_hash", expected_keyed_hash)
        write_test_vector_file(test_no, "expected_derive_key_xof", expected_derive_key_xof)
        write_test_vector_file(test_no, "expected_derive_key", expected_derive_key)

        # Test the default hash.
        test_hash = run_blake3([], input)
        for machdep in machdeps:
            if beginning != 0:
                tis_config_file.write(",\n")
            else:
                beginning = 1
                tis_config_file.write("\n")
            TIS_MAKE_TEST(test_no,
                          machdep,
                          "test_hash",
                          "expected_hash",
                          [],
                          tis_config_file)
        for line in test_hash.splitlines():
            assert expected_hash == line, \
                "hash({}): {} != {}".format(input_len, expected_hash, line)

        # Test the extended hash.
        xof_len = len(expected_hash_xof) // 2
        test_hash_xof = run_blake3(["--length", str(xof_len)], input)
        for machdep in machdeps:
            tis_config_file.write(",\n")
            TIS_MAKE_TEST(test_no,
                          machdep,
                          "test_hash_xof",
                          "expected_hash_xof",
                          ["--length", str(xof_len)],
                          tis_config_file)
        for line in test_hash_xof.splitlines():
            assert expected_hash_xof == line, \
                "hash_xof({}): {} != {}".format(
                    input_len, expected_hash_xof, line)

        # Test the default keyed hash.
        test_keyed_hash = run_blake3(["--keyed", hex_key], input)
        for machdep in machdeps:
            tis_config_file.write(",\n")
            TIS_MAKE_TEST(test_no,
                          machdep,
                          "test_keyed_hash",
                          "expected_keyed_hash",
                          ["--keyed", hex_key.decode()],
                          tis_config_file)
        for line in test_keyed_hash.splitlines():
            assert expected_keyed_hash == line, \
                "keyed_hash({}): {} != {}".format(
                    input_len, expected_keyed_hash, line)

        # Test the extended keyed hash.
        xof_len = len(expected_keyed_hash_xof) // 2
        test_keyed_hash_xof = run_blake3(
            ["--keyed", hex_key, "--length",
             str(xof_len)], input)
        for machdep in machdeps:
            tis_config_file.write(",\n")
            TIS_MAKE_TEST(test_no,
                          machdep,
                          "test_keyed_hash_xof",
                          "expected_keyed_hash_xof",
                          ["--keyed", hex_key.decode(), "--length", str(xof_len)],
                          tis_config_file)
        for line in test_keyed_hash_xof.splitlines():
            assert expected_keyed_hash_xof == line, \
                "keyed_hash_xof({}): {} != {}".format(
                    input_len, expected_keyed_hash_xof, line)

        # Test the default derive key.
        test_derive_key = run_blake3(["--derive-key", context_string], input)
        for machdep in machdeps:
            tis_config_file.write(",\n")
            TIS_MAKE_TEST(test_no,
                          machdep,
                          "test_derive_key",
                          "expected_derive_key",
                          ["--derive-key", context_string],
                          tis_config_file)
        for line in test_derive_key.splitlines():
            assert expected_derive_key == line, \
                "derive_key({}): {} != {}".format(
                    input_len, expected_derive_key, line)

        # Test the extended derive key.
        xof_len = len(expected_derive_key_xof) // 2
        test_derive_key_xof = run_blake3(
            ["--derive-key", context_string, "--length",
             str(xof_len)], input)
        for machdep in machdeps:
            tis_config_file.write(",\n")
            TIS_MAKE_TEST(test_no,
                          machdep,
                          "test_derive_key_xof",
                          "expected_derive_key_xof",
                          ["--derive-key", context_string, "--length", str(xof_len)],
                          tis_config_file)
        for line in test_derive_key_xof.splitlines():
            assert expected_derive_key_xof == line, \
                "derive_key_xof({}): {} != {}".format(
                    input_len, expected_derive_key_xof, line)

    tis_config_file.write("\n]")
    tis_config_file.close()


if __name__ == "__main__":
    main()
