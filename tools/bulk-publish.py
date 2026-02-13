#!/usr/bin/env python3
"""Bulk publish all crates to Kellnr in DAG phase order."""

import os
import re
import subprocess
import sys
import time
from datetime import datetime

NEXCORE_ROOT = "/home/matthew/Projects/nexcore"
CRATES_DIR = os.path.join(NEXCORE_ROOT, "crates")
LOG_FILE = "/tmp/bulk-publish.log"

ENV = {
    **os.environ,
    "ZDOTDIR": "/dev/null",
    "CARGO_REGISTRIES_NEXCORE_INDEX": "sparse+http://localhost:8000/api/v1/crates/",
    "CARGO_REGISTRIES_NEXCORE_TOKEN": "WUq8ItiDTqiWtcYnKD29FgCrVw3YoAPH",
}


def log(msg):
    """Print and log to file."""
    print(msg, flush=True)
    with open(LOG_FILE, "a") as f:
        f.write(f"[{datetime.now().strftime('%H:%M:%S')}] {msg}\n")


def get_phases():
    """Run dag-publish to get phase ordering."""
    r = subprocess.run(
        ["cargo", "run", "--manifest-path",
         os.path.join(NEXCORE_ROOT, "tools/dag-publish/Cargo.toml"),
         "--", "--crates-dir", CRATES_DIR, "--show-phases", "--dry-run"],
        capture_output=True, text=True, timeout=60, env=ENV
    )

    phases = {}
    current_phase = None
    for line in r.stdout.split('\n'):
        m = re.match(r'^Phase (\d+) \((\d+) crates\):', line)
        if m:
            current_phase = int(m.group(1))
            phases[current_phase] = []
        elif current_phase is not None and line.startswith('  '):
            crate_name = line.strip()
            if crate_name:
                phases[current_phase].append(crate_name)
        elif line.startswith('Total:'):
            break

    return phases


def find_crate_dir(crate_name):
    """Find the directory for a crate (name might differ from dir)."""
    # First try exact match
    d = os.path.join(CRATES_DIR, crate_name)
    if os.path.exists(os.path.join(d, "Cargo.toml")):
        return d
    # Search all dirs for matching package name
    for entry in os.scandir(CRATES_DIR):
        if not entry.is_dir():
            continue
        ct = os.path.join(entry.path, "Cargo.toml")
        if not os.path.exists(ct):
            continue
        with open(ct) as f:
            for line in f:
                m = re.match(r'^name\s*=\s*"([^"]+)"', line.strip())
                if m and m.group(1) == crate_name:
                    return entry.path
    return None


def publish_crate(crate_dir):
    """Publish a single crate to Kellnr."""
    try:
        r = subprocess.run(
            ["cargo", "publish", "--manifest-path",
             os.path.join(crate_dir, "Cargo.toml"),
             "--registry", "nexcore", "--allow-dirty", "--no-verify"],
            capture_output=True, text=True, timeout=600, env=ENV
        )
        return r
    except subprocess.TimeoutExpired:
        class R:
            returncode = 1
            stderr = "TIMEOUT: publish took >600s"
            stdout = ""
        return R()


def main():
    # Clear log file
    with open(LOG_FILE, "w") as f:
        f.write(f"=== Bulk publish started at {datetime.now()} ===\n")

    log("Getting DAG phase ordering...")
    phases = get_phases()
    total = sum(len(crates) for crates in phases.values())
    log(f"  {len(phases)} phases, {total} crates\n")

    published = 0
    skipped = 0
    failed = []
    count = 0

    for phase_num in sorted(phases.keys()):
        crates = phases[phase_num]
        log(f"{'='*60}")
        log(f"Phase {phase_num}: {len(crates)} crates")
        log(f"{'='*60}")

        for crate_name in crates:
            count += 1
            crate_dir = find_crate_dir(crate_name)
            if not crate_dir:
                log(f"  [{count}/{total}] [SKIP] {crate_name}: directory not found")
                skipped += 1
                continue

            # Check for publish = false
            with open(os.path.join(crate_dir, "Cargo.toml")) as f:
                content = f.read()
            if re.search(r'publish\s*=\s*false', content):
                log(f"  [{count}/{total}] [SKIP] {crate_name}: publish = false")
                skipped += 1
                continue

            log(f"  [{count}/{total}] Publishing {crate_name}...")
            t0 = time.time()
            r = publish_crate(crate_dir)
            elapsed = time.time() - t0

            if r.returncode == 0:
                log(f"    -> OK ({elapsed:.1f}s)")
                published += 1
            elif "already uploaded" in r.stderr or "already exists" in r.stderr:
                log(f"    -> already published ({elapsed:.1f}s)")
                skipped += 1
            else:
                err = r.stderr.strip().split('\n')[-1][:120]
                log(f"    -> FAILED ({elapsed:.1f}s): {err}")
                failed.append((crate_name, r.stderr.strip()))

        # Small delay between phases to let index update
        if phase_num < max(phases.keys()):
            log(f"  Waiting 3s for index to update...")
            time.sleep(3)

    log(f"\n{'='*60}")
    log(f"SUMMARY")
    log(f"{'='*60}")
    log(f"  Published: {published}")
    log(f"  Skipped:   {skipped}")
    log(f"  Failed:    {len(failed)}")
    if failed:
        log(f"\n  Failures:")
        for name, err in failed:
            log(f"    {name}:")
            for line in err.split('\n')[-5:]:
                log(f"      {line}")


if __name__ == "__main__":
    main()
