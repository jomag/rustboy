#!/usr/bin/env python3
import os

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
RELEASE_MODE = True
MOONEYE_DIR = f"{SCRIPT_DIR}/../test/mooneye-test-suite"
RUSTBOY = (
    f"{SCRIPT_DIR}/../target/release/rustboy"
    if RELEASE_MODE
    else f"{SCRIPT_DIR}/../target/debug/rustboy"
)
TESTS_PER_ROW = 10
PASS_EMOJI = ":green_heart:"
FAIL_EMOJI = ":red_circle:"
OUTPUT_DIR = SCRIPT_DIR

test_suites = []


def split_in_chunks(v: list, n: int):
    for i in range(0, len(v), n):
        yield v[i : i + n]


def run_mooneye_test(path):
    result = os.system(f"{RUSTBOY} --test=mooneye {path}")
    return result == 0


# Run the Mooneye Test Suite tests
mooneye_tests = {}
mooneye_whitelist = set(["acceptance"])

for dir_name in sorted(os.listdir(MOONEYE_DIR)):
    full_path = os.path.join(MOONEYE_DIR, dir_name)
    if os.path.isdir(full_path):
        if dir_name in mooneye_whitelist:
            mooneye_tests[dir_name] = {}
            print(f"Running {dir_name} tests:")
            roms = [n for n in os.listdir(full_path) if n.endswith(".gb")]
            for rom in roms:
                print(f"- Run test {rom}")
                result = run_mooneye_test(os.path.join(full_path, rom))
                mooneye_tests[dir_name][rom] = result

print(mooneye_tests)

mooneye_report = "## Mooneye Test Suite\n\n"

for group in mooneye_tests:
    mooneye_report += f"### {group.title()} tests\n\n"
    mooneye_report += "|" + "|".join(["       "] * TESTS_PER_ROW) + "|\n"
    mooneye_report += "|" + "|".join([" :---: "] * TESTS_PER_ROW) + "|\n"
    test_names = mooneye_tests[group].keys()
    chunks = list(split_in_chunks(list(test_names), TESTS_PER_ROW))

    for chunk in chunks:
        for test in chunk:
            if mooneye_tests[group][test]:
                status = PASS_EMOJI
            else:
                status = FAIL_EMOJI
            mooneye_report += f'| [{status}](x "{test}") '
        mooneye_report += "|\n"

with open(os.path.join(SCRIPT_DIR, "mooneye.md"), "w") as f:
    f.write(mooneye_report)
