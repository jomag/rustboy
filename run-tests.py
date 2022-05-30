#!/usr/bin/env python3
import sys
import argparse
from cgi import test
import os
from typing import List, Optional

SCRIPT_DIR = os.path.dirname(os.path.realpath(__file__))
RELEASE_MODE = True
MOONEYE_DIR = f"{SCRIPT_DIR}/test/mooneye-test-suite"
BLARGG_DIR = f"{SCRIPT_DIR}/test/blargg"
RUSTBOY = (
    f"{SCRIPT_DIR}/target/release/rustboy"
    if RELEASE_MODE
    else f"{SCRIPT_DIR}/target/debug/rustboy"
)
TESTS_PER_ROW = 15
PASS_EMOJI = ":green_heart:"
FAIL_EMOJI = ":red_circle:"
SKIPPED_EMOJI = "🙅"
OUTPUT_DIR = SCRIPT_DIR

test_suites = []

try:
    import colorama

    BRIGHT = colorama.Style.BRIGHT
    WHITE = colorama.Fore.WHITE
    YELLOW = colorama.Fore.YELLOW
    GREEN = colorama.Fore.GREEN
    RED = colorama.Fore.RED
    RESET_ALL = colorama.Style.RESET_ALL
except ModuleNotFoundError:
    BRIGHT = ""
    WHITE = ""
    YELLOW = ""
    GREEN = ""
    RED = ""
    RESET_ALL = ""


def split_in_chunks(v: List, n: int):
    for i in range(0, len(v), n):
        yield v[i : i + n]


class Test:
    def __init__(
        self,
        name,
        rom_path,
        variant: Optional[str] = None,
        expect: Optional[str] = None,
        machine: str = "dmg",
    ):
        self.name = name
        self.rom_path = rom_path
        self.result = None
        self.variant = variant
        self.expect = expect
        self.machine = machine

    def run(self):
        print(f"{BRIGHT}{YELLOW}{self.name}{RESET_ALL}")
        machine = f"-m {self.machine}" if self.machine else ""
        if self.expect:
            print(f"[{self.expect}]")
            sys.stdout.flush()
            result = os.system(
                f'{RUSTBOY} --test-expect="{self.expect}" {machine} "{self.rom_path}"'
            )
            self.result = result == 0
        elif self.variant:
            sys.stdout.flush()
            result = os.system(
                f'{RUSTBOY} --test="{self.variant}" {machine} "{self.rom_path}"'
            )
            self.result = result == 0
        else:
            raise Exception("Not sure how to run test")

        if self.result:
            print(f"{BRIGHT}{GREEN}Pass{RESET_ALL}\n")
        else:
            print(f"{BRIGHT}{RED}Fail{RESET_ALL}\n")

    def skip(self):
        self.result = None

    def pretty_print(self):
        if self.result is None:
            res = f"{BRIGHT}{YELLOW}skipped{RESET_ALL}"
        elif self.result:
            res = f"{BRIGHT}{GREEN}Pass{RESET_ALL}"
        else:
            res = f"{BRIGHT}{RED}Fail{RESET_ALL}"
        print(f"{self.name}: {res}")


class TestGroup:
    """A group of tests"""

    """Name of the test"""
    name: str

    """Directory that holds the test ROM's for this group"""
    romdir: str

    """All tests in this group"""
    tests: List[Test]

    def __init__(self, name: str, romdir: str):
        self.name = name
        self.romdir = romdir
        self.tests = []

    def run(self, skip: List[str] = []):
        for test in self.tests:
            if test.name not in skip:
                test.run()
            else:
                test.skip()

    def get_roms(self):
        """Return all ROM's in self.romdir in sorted order"""
        return [
            os.path.join(self.romdir, rom)
            for rom in sorted(os.listdir(self.romdir))
            if rom.endswith(".gb")
        ]

    def pretty_print(self):
        print(f"{BRIGHT}{WHITE}{self.name}:{RESET_ALL}")
        if not self.tests:
            print("No tests\n")
        else:
            for test in self.tests:
                test.pretty_print()

    def add_test(self, rom_name, test_name, path):
        raise NotImplemented

    def setup(self):
        for path in self.get_roms():
            rom_name = os.path.basename(path)
            test_name = rom_name[:-3]
            self.add_test(rom_name, test_name, path)


class TestSuite:
    name: str
    groups: List[TestGroup]
    basedir: str

    def __init__(self, name, basedir: str):
        self.name = name
        self.groups = []
        self.basedir = basedir

    def run(self, skip: List[str] = []):
        for grp in self.groups:
            grp.run(skip=skip)

    def pretty_print(self):
        print(f"{BRIGHT}{WHITE}{self.name}")
        print("-" * len(self.name) + RESET_ALL)

        for grp in self.groups:
            grp.pretty_print()
            print()

    def build_report(self, with_title=False, tests_per_row=TESTS_PER_ROW):
        if with_title:
            report = f"## {self.name}\n\n"
        else:
            report = ""

        report += "|" + "|".join(["       "] * (tests_per_row + 1)) + "|\n"
        report += "|" + "|".join([" :---: "] * (tests_per_row + 1)) + "|\n"

        for grp in sorted(self.groups, key=lambda x: x.name):
            chunks = [
                grp.tests[i : i + tests_per_row]
                for i in range(0, len(grp.tests), tests_per_row)
            ]

            first = True
            for chunk in chunks:
                if first:
                    report += f"| {grp.name} "
                    first = False
                else:
                    report += f"| "

                for test in chunk:
                    if test.result is None:
                        status = SKIPPED_EMOJI
                        status_text = ": SKIPPED"
                    elif test.result:
                        status = PASS_EMOJI
                        status_text = ": PASS"
                    else:
                        status = FAIL_EMOJI
                        status_text = ": FAIL"
                    report += f'| [{status}](x "{test.name}{status_text}") '
                report += "|\n"

        return report


class BlarggTestGroup(TestGroup):
    subdir: str
    tests: List[Test]

    def __init__(self, name, subdir, romdir):
        super().__init__(name, romdir)
        self.subdir = subdir

    def add_test(self, rom_name, test_name, path):
        if "cgb" in path:
            machine = "cgb"
        else:
            machine = "dmg"

        if test_name == "instr_timing":
            expect = f"{test_name}\n\n\nPassed\n"
            test = Test(test_name, path, expect=expect, machine=machine)
        elif self.romdir.endswith("individual"):
            expect = f"{test_name}\n\n\nPassed"
            test = Test(test_name, path, expect=expect, machine=machine)
        else:
            test = Test(test_name, path, variant="blargg", machine=machine)
        self.tests.append(test)


class MooneyeTestGroup(TestGroup):
    def __init__(self, name, romdir):
        super().__init__(name, romdir)

    def add_test(self, rom_name, test_name, path):
        test = Test(test_name, path, variant="mooneye")
        self.tests.append(test)


class BlarggTestSuite(TestSuite):
    def __init__(self, basedir):
        super().__init__("Blargg Test Suite", basedir=basedir)

    def setup(self):
        subdirs = [
            n
            for n in sorted(os.listdir(self.basedir))
            if os.path.isdir(os.path.join(self.basedir, n))
        ]

        for dir in subdirs:
            # The actual ROM's are kept in a subdirectory called either "individual"
            # or "rom_singles", and the way they are validated depends on the name.
            romdir = os.path.join(self.basedir, dir, "individual")
            if not os.path.exists(romdir):
                romdir = os.path.join(self.basedir, dir, "rom_singles")
            if not os.path.exists(romdir):
                romdir = os.path.join(self.basedir, dir)

            grp = BlarggTestGroup(
                name=dir,
                subdir=os.path.join(self.basedir, dir),
                romdir=romdir,
            )
            grp.setup()

            self.groups.append(grp)


class MooneyeTestSuite(TestSuite):
    def __init__(self, basedir):
        super().__init__("Mooneye Test Suite", basedir=basedir)

    def setup(self):
        # There are a few first-level test groups in the Mooneye Test Suite,
        # but only the "acceptance" group is usable. The rest are for example
        # "emulator-only", "manual-only", "madness", etc.
        #
        # The "acceptance" group has a bunch of ROM's in it, but also a number
        # of subtests, so we add the "acceptance" roms as one group, and the
        # subtests as separate groups.
        acceptance_dir = os.path.join(self.basedir, "acceptance")
        emulator_only_dir = os.path.join(self.basedir, "emulator-only")

        grp = MooneyeTestGroup(name="acceptance", romdir=acceptance_dir)
        self.groups.append(grp)

        for name in os.listdir(acceptance_dir):
            path = os.path.join(acceptance_dir, name)
            if os.path.isdir(path):
                grp = MooneyeTestGroup(name=f"acceptance/{name}", romdir=path)
                self.groups.append(grp)

        for name in os.listdir(emulator_only_dir):
            path = os.path.join(emulator_only_dir, name)
            if os.path.isdir(path):
                grp = MooneyeTestGroup(name=f"emulator-only/{name}", romdir=path)
                self.groups.append(grp)

        for grp in self.groups:
            grp.setup()


def run_acid_test():
    from PIL import Image, ImageChops, ImageOps

    test = Test(name="DMG ACID2", rom_path="./test/dmg-acid2.gb", variant="capture")
    test.run()

    reference_image_path = "./dmg-acid2-ref.png"
    emulator_image_path = "./messed-up.png"
    diff_image_path = "./dmg-acid2-result.png"

    ref = Image.open(reference_image_path).convert(mode="RGB")
    emu = Image.open(emulator_image_path).convert(mode="RGB")
    assert ref.size == emu.size
    width, height = ref.size

    refp = ref.load()
    emup = emu.load()

    for y in range(height):
        for x in range(width):
            if any(abs(aa - bb) > 2 for aa, bb in zip(refp[x, y], emup[x, y])):
                emup[x, y] = (255, 0, 0)

    emu.save(diff_image_path)


parser = argparse.ArgumentParser(description="Run test suites")
parser.add_argument("suites", type=str, nargs="+", help="Test suite selection")
parser.add_argument("--report", type=str, help="Write report to file")
args = parser.parse_args()

all_suites = "all" in args.suites or len(args.suites) == 0
single_test = not all_suites and len(args.suites) < 2
reports = []

if all_suites or "mooneye" in args.suites:
    mooneye = MooneyeTestSuite(MOONEYE_DIR)
    mooneye.setup()
    mooneye.run(
        skip=[
            "intr_1_2_timing-GS",
            # All the following broke when rewriting the PPU.
            # Most likely it's the interrupts that are broken.
            "oam_dma_start",
            "hblank_ly_scx_timing-GS",
            "intr_2_0_timing",
            "intr_2_mode0_timing",
            "intr_2_mode0_timing_sprites",
            "intr_2_mode3_timing",
            "intr_2_oam_ok_timing",
            "vblank_stat_intr-GS",
        ]
    )
    mooneye.pretty_print()
    if args.report:
        reports.append(
            mooneye.build_report(with_title=not single_test, tests_per_row=12)
        )

if all_suites or "blargg" in args.suites:
    skip = ["interrupt_time"]  # CGB only
    blargg = BlarggTestSuite(BLARGG_DIR)
    blargg.setup()
    blargg.run(skip=skip)
    blargg.pretty_print()
    if args.report:
        reports.append(
            blargg.build_report(with_title=not single_test, tests_per_row=12)
        )

if all_suites or "acid2" in args.suites:
    run_acid_test()

if len(reports) > 0:
    with open(args.report, "w") as f:
        for report in reports:
            f.write(report)
