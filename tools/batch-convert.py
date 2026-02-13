#!/usr/bin/env python3
"""Batch converter: transforms all remaining workspace crates to standalone packages.

Steps:
1. Run crate-converter --all (inlines workspace fields, deps, lints)
2. Add [workspace] marker to each converted Cargo.toml
3. Add path = "../<dir>" fallbacks for internal deps (for local dev)
4. Print exclude list for root Cargo.toml
"""

import os
import re
import subprocess
import sys

NEXCORE_ROOT = "/home/matthew/Projects/nexcore"
CRATES_DIR = os.path.join(NEXCORE_ROOT, "crates")

# Crates already converted (have [workspace] and standalone deps)
ALREADY_CONVERTED = {
    "nexcore-id", "nexcore-error", "nexcore-lex-primitiva",
    "nexcore-macros-core", "nexcore-macros", "nexcore-constants", "nexcore-config"
}


def build_dep_to_dir_mapping():
    """Build mapping: dependency_key -> crate_directory_name.

    Sources:
    1. Root Cargo.toml [workspace.dependencies] with path = "crates/X"
    2. Scan crates/ for package names
    """
    mapping = {}

    # Source 1: workspace deps (handles renamed deps like lex-primitiva -> nexcore-lex-primitiva)
    root_toml = os.path.join(NEXCORE_ROOT, "Cargo.toml")
    with open(root_toml) as f:
        in_ws_deps = False
        for line in f:
            stripped = line.strip()
            if stripped == "[workspace.dependencies]":
                in_ws_deps = True
                continue
            if stripped.startswith("[") and in_ws_deps:
                in_ws_deps = False
                continue
            if in_ws_deps and "path" in line:
                # Parse: dep_name = { path = "crates/dirname", ... }
                m = re.match(r'^(\S+)\s*=\s*\{.*path\s*=\s*"crates/([^"]+)"', stripped)
                if m:
                    dep_key = m.group(1)
                    dirname = m.group(2)
                    mapping[dep_key] = dirname

    # Source 2: scan crate directories for package names
    for entry in os.scandir(CRATES_DIR):
        if not entry.is_dir():
            continue
        cargo_toml = os.path.join(entry.path, "Cargo.toml")
        if not os.path.exists(cargo_toml):
            continue
        with open(cargo_toml) as f:
            for line in f:
                m = re.match(r'^name\s*=\s*"([^"]+)"', line.strip())
                if m:
                    mapping[m.group(1)] = entry.name
                    break

    return mapping


def add_workspace_marker_and_paths(cargo_toml_path, dep_to_dir):
    """Post-process a converted Cargo.toml:
    1. Add [workspace] at top if missing
    2. For deps with registry = "nexcore", add path = "../<dirname>"
    """
    with open(cargo_toml_path) as f:
        content = f.read()

    # Add [workspace] if not present
    if not content.lstrip().startswith("[workspace]"):
        content = "[workspace]\n\n" + content

    # Add path deps alongside registry deps
    lines = content.split('\n')
    result = []
    for line in lines:
        if 'registry = "nexcore"' in line:
            # Extract dep name from line like: dep_name = { version = "...", registry = "nexcore" }
            m = re.match(r'^(\s*)([a-zA-Z0-9_-]+)\s*=\s*\{(.+)\}\s*$', line)
            if m:
                indent = m.group(1)
                dep_name = m.group(2)
                inner = m.group(3)

                # Find directory for this dep
                dirname = dep_to_dir.get(dep_name, dep_name)

                # Only add path if not already present
                if 'path =' not in inner:
                    inner = inner.replace(
                        'registry = "nexcore"',
                        f'path = "../{dirname}", registry = "nexcore"'
                    )
                    line = f'{indent}{dep_name} = {{{inner}}}'
        result.append(line)

    return '\n'.join(result)


def main():
    print("=" * 60)
    print("Nexcore Batch Converter")
    print("=" * 60)

    # Step 1: Build mappings
    print("\n[1/4] Building dependency → directory mapping...")
    dep_to_dir = build_dep_to_dir_mapping()
    print(f"  Mapped {len(dep_to_dir)} dependency names to directories")

    # Step 2: Run converter
    print("\n[2/4] Running crate-converter --all ...")
    result = subprocess.run(
        [
            "cargo", "run", "--manifest-path",
            os.path.join(NEXCORE_ROOT, "tools/crate-converter/Cargo.toml"),
            "--", "--workspace", NEXCORE_ROOT, "--all"
        ],
        capture_output=True, text=True, timeout=120
    )

    # Converter may exit 1 if some crates fail — that's OK, successful ones are still written
    converted_dirs = []
    skipped = 0
    errors = []
    for line in (result.stdout + result.stderr).strip().split('\n'):
        if line.startswith("Converted:"):
            path = line.replace("Converted: ", "").strip()
            dirname = os.path.basename(os.path.dirname(path))
            converted_dirs.append(dirname)
        elif "[skip]" in line:
            skipped += 1
        elif "[error]" in line:
            errors.append(line)

    print(f"  Converted: {len(converted_dirs)} crates")
    print(f"  Skipped: {skipped} (already standalone)")
    if errors:
        print(f"  Errors: {len(errors)} crates (will handle separately)")
        for e in errors:
            print(f"    {e}")

    # Step 3: Post-process
    print(f"\n[3/4] Post-processing {len(converted_dirs)} Cargo.toml files...")
    for dirname in converted_dirs:
        cargo_toml_path = os.path.join(CRATES_DIR, dirname, "Cargo.toml")
        content = add_workspace_marker_and_paths(cargo_toml_path, dep_to_dir)
        with open(cargo_toml_path, 'w') as f:
            f.write(content)
    print("  Done")

    # Step 4: Print exclude list
    all_to_exclude = sorted(set(converted_dirs) | ALREADY_CONVERTED)

    print(f"\n[4/4] Summary")
    print(f"  Total converted (this run): {len(converted_dirs)}")
    print(f"  Previously converted: {len(ALREADY_CONVERTED)}")
    print(f"  Total standalone crates: {len(all_to_exclude)}")

    # Write exclude list to a file for easy copy-paste
    exclude_file = os.path.join(NEXCORE_ROOT, "tools", "exclude-list.txt")
    with open(exclude_file, 'w') as f:
        for d in all_to_exclude:
            f.write(f'    "crates/{d}",\n')
    print(f"\n  Exclude list written to: {exclude_file}")

    return converted_dirs


if __name__ == "__main__":
    main()
