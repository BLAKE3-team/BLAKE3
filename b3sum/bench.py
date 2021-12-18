#! /usr/bin/env python3

import subprocess
import sys
import time

NUM_RUNS = 5


def one_run():
    start = time.monotonic()
    subprocess.run(
        sys.argv[1:],
        stdout=subprocess.DEVNULL,
        check=True,
    )
    end = time.monotonic()
    assert end > start
    return end - start


def median_run():
    assert NUM_RUNS % 2 == 1, "NUM_RUNS should be odd"
    times = []
    for _ in range(NUM_RUNS):
        t = one_run()
        times.append(t)
    times.sort()
    return times[len(times) // 2]


def main():
    t = median_run()
    print("{:.3f}".format(t))


if __name__ == "__main__":
    main()
