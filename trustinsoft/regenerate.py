#! /usr/bin/env python3

# This script regenerates TrustInSoft CI configuration.

# Run from the root of the BLAKE3 project:
# $ python3 trustinsoft/regenerate.py

import re # sub
import json # dumps, load
import os # path, makedirs
import binascii # hexlify

# Following function copied from c/test.py :
# -----------------------------------------------------------------------------
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
# -----------------------------------------------------------------------------

# Following line copied from c/test.py :
# -----------------------------------------------------------------------------
TEST_VECTORS_PATH = os.path.join("test_vectors", "test_vectors.json")
# -----------------------------------------------------------------------------

# Outputting JSON.
def string_of_json(obj):
    # Output standard pretty-printed JSON (RFC 7159) with 4-space indentation.
    s = json.dumps(obj, indent=4)
    # Sometimes we need to have multiple "include" fields in the outputted
    # JSON, which is unfortunately impossible in the internal python
    # representation (OK, it is technically possible, but too cumbersome to
    # bother implementing it here), so we can name these fields 'include_',
    # 'include__', etc, and they are all converted to 'include' before
    # outputting as JSON.
    s = re.sub(r'"include_+"', '"include"', s)
    return s

# --------------------------------------------------------------------------- #
# ---------------------------------- CHECKS --------------------------------- #
# --------------------------------------------------------------------------- #

def check_dir(dir):
    if os.path.isdir(dir):
        print("   > OK! Directory '%s' exists." % dir)
    else:
        exit("Directory '%s' not found." % dir)

def check_file(file):
    if os.path.isfile(file):
        print("   > OK! File '%s' exists." % file)
    else:
        exit("File '%s' not found." % file)

# Initial check.
print("1. Check if needed directories and files exist...")
check_dir("trustinsoft")
check_file(TEST_VECTORS_PATH)

# --------------------------------------------------------------------------- #
# -------------------- GENERATE trustinsoft/common.config ------------------- #
# --------------------------------------------------------------------------- #

common_config_path = os.path.join("trustinsoft", "common.config")

def string_of_options(options):
    s = ''
    beginning = True
    for option_prefix in options:
        for option_val in options[option_prefix]:
            if beginning:
                beginning = False # No need for a separator at the beginning.
            else:
                s += ' '
            s += option_prefix + option_val
    return s

def make_common_config():
    # C files.
    c_files = [
        "blake3.c",
        "blake3_dispatch.c",
        "blake3_portable.c",
    ]
    # Compilation options.
    compilation_cmd = (
        {
            "-I": [
                "..",
                os.path.join("..", "c"),
            ],
            "-D": [
                "BLAKE3_TESTING",
                "BLAKE3_NO_SSE41",
                "BLAKE3_NO_AVX2",
                "BLAKE3_NO_AVX512",
            ],
            "-U": [
                "__clang__",
                "__GNUC__",
                "__x86_64__",
                "__i386__",
            ],
        }
    )
    # Whole common.config JSON.
    return {
        "files": [ "test.c" ] +
                 list(map(lambda file: os.path.join("..", "c", file), c_files)),
        "compilation_cmd": string_of_options(compilation_cmd),
    }

common_config = make_common_config()
with open(common_config_path, "w") as file:
    print("2. Generate the 'trustinsoft/common.config' file.")
    file.write(string_of_json(common_config))

# --------------------------------------------------------------------------- #
# -------------------------------- tis.config ------------------------------- #
# --------------------------------------------------------------------------- #

# Following line copied from c/test.py :
# -----------------------------------------------------------------------------
TEST_VECTORS = json.load(open(TEST_VECTORS_PATH))
hex_key = binascii.hexlify(TEST_VECTORS["key"].encode())
context_string = TEST_VECTORS["context_string"]
# -----------------------------------------------------------------------------

machdeps = [
    "gcc_x86_32",
    "gcc_x86_64",
    "gcc_ppc_32",
    "gcc_ppc_64",
]

test_vectors_dir = os.path.join("trustinsoft", "test_vectors")

def test_vector_file(vector_no, name):
    filename = "%02d_%s" % (vector_no, name)
    return os.path.join(test_vectors_dir, filename)

def make_test(vector_no, test_case, machdep):
    name = test_case["name"]
    args = test_case["args"]
    # Base of the single tis.config entry.
    test = (
        {
            "name": ("Test vector %02d: %s (%s)" % (vector_no, name, machdep)),
            "include": common_config_path,
            "machdep": machdep,
            "filesystem": {
                "files": [
                    {
                        "name": "tis-mkfs-stdin",
                        "from": test_vector_file(vector_no, "input"),
                    },
                    {
                        "name": "expected",
                        "from": test_vector_file(vector_no, "expected_" + name),
                    },
                ],
            },
        }
    )
    # Add the field "val-args" if command line arguments are present.
    if args:
        test["val-args"] = ("%" + "%".join(args))
    # Add field "no-results" for longest tests.
    if vector_no >= 35:
        test["no-results"] = True
    # Done.
    return test

def test_cases_of_test_vector(test_vector):
    # Following lines copied from c/test.py :
    # -------------------------------------------------------------------------
    expected_hash_xof = test_vector["hash"]
    expected_keyed_hash_xof = test_vector["keyed_hash"]
    expected_hash = expected_hash_xof[:64]
    expected_keyed_hash = expected_keyed_hash_xof[:64]
    expected_derive_key_xof = test_vector["derive_key"]
    expected_derive_key = expected_derive_key_xof[:64]
    # -------------------------------------------------------------------------
    return (
        [
            # Test the default hash.
            {
                "name": "hash",
                "expected": expected_hash,
                "args": [],
            },
            # Test the extended hash.
            {
                "name": "hash_xof",
                "expected": expected_hash_xof,
                "args": ["--length", str(len(expected_hash_xof) // 2)],
            },
            # Test the default keyed hash.
            {
                "name": "keyed_hash",
                "expected": expected_keyed_hash,
                "args": ["--keyed", hex_key.decode()],
            },
            # Test the extended keyed hash.
            {
                "name": "keyed_hash_xof",
                "expected": expected_keyed_hash_xof,
                "args": ["--keyed", hex_key.decode(), "--length",
                         str(len(expected_keyed_hash_xof) // 2)],
            },
            # Test the default derive key.
            {
                "name": "derive_key",
                "expected": expected_derive_key,
                "args": ["--derive-key", context_string],
            },
            # Test the extended derive key.
            {
                "name": "derive_key_xof",
                "expected": expected_derive_key_xof,
                "args": ["--derive-key", context_string, "--length",
                         str(len(expected_derive_key_xof) // 2)],
            },
        ]
    )

def make_tis_config_and_generate_test_vector_files():
    # Prepare.
    tis_config = []
    os.makedirs(test_vectors_dir, exist_ok=True)
    vector_no = 0
    # Treat each test vector.
    for test_vector in TEST_VECTORS["cases"]:
        vector_no += 1
        print("   > Test vector %2d" % vector_no) # Debug.

        # Write the input file for this test vector.
        # Following lines copied from c/test.py :
        # ---------------------------------------------------------------------
        input_len = test_vector["input_len"]
        input = make_test_input(input_len)
        # ---------------------------------------------------------------------
        input_file = test_vector_file(vector_no, "input")
        with open(input_file, "wb") as file:
            file.write(input)

        # Treat each test case in this test vector.
        for test_case in test_cases_of_test_vector(test_vector):
            # Write the expected output file for this test case.
            expected_name = "expected_" + test_case["name"]
            expected_file = test_vector_file(vector_no, expected_name)
            with open(expected_file, "w") as file:
                file.write(test_case["expected"])
            # Generate an entry in the tis.config file.
            # (One entry for each vector * case * machdep combination.)
            for machdep in machdeps:
                test = make_test(vector_no, test_case, machdep)
                tis_config.append(test)

    # Done.
    return tis_config

tis_config = make_tis_config_and_generate_test_vector_files()
with open("tis.config", "w") as file:
    print("3. Generate the tis.config file and test vector files.")
    file.write(string_of_json(tis_config))
