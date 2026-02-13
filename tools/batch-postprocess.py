#!/usr/bin/env python3
"""Post-process converted Cargo.toml files:
1. Add [workspace] marker at top
2. Add path = "../<dirname>" for internal deps with registry = "nexcore"
"""

import os
import re

NEXCORE_ROOT = "/home/matthew/Projects/nexcore"
CRATES_DIR = os.path.join(NEXCORE_ROOT, "crates")


def build_dep_to_dir_mapping():
    """Build mapping: dependency_key -> crate_directory_name."""
    mapping = {}

    # Source 1: workspace deps from root Cargo.toml
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
                m = re.match(r'^(\S+)\s*=\s*\{.*path\s*=\s*"crates/([^"]+)"', stripped)
                if m:
                    mapping[m.group(1)] = m.group(2)

    # Source 2: scan all crate directories for package names
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


def postprocess_cargo_toml(cargo_toml_path, dep_to_dir):
    """Add [workspace] marker and path fallbacks for internal deps."""
    with open(cargo_toml_path) as f:
        content = f.read()

    modified = False

    # Add [workspace] if not present
    if not content.lstrip().startswith("[workspace]"):
        content = "[workspace]\n\n" + content
        modified = True

    # Add path deps alongside registry deps
    lines = content.split('\n')
    result = []
    for line in lines:
        if 'registry = "nexcore"' in line and 'path =' not in line:
            m = re.match(r'^(\s*)([a-zA-Z0-9_-]+)\s*=\s*\{(.+)\}\s*$', line)
            if m:
                indent = m.group(1)
                dep_name = m.group(2)
                inner = m.group(3)
                dirname = dep_to_dir.get(dep_name, dep_name)
                inner = inner.replace(
                    'registry = "nexcore"',
                    f'path = "../{dirname}", registry = "nexcore"'
                )
                line = f'{indent}{dep_name} = {{{inner}}}'
                modified = True
        result.append(line)

    return '\n'.join(result), modified


def main():
    print("Building dependency → directory mapping...")
    dep_to_dir = build_dep_to_dir_mapping()
    print(f"  Mapped {len(dep_to_dir)} names")

    print("\nPost-processing Cargo.toml files...")
    processed = 0
    added_ws = 0
    added_paths = 0
    already_ok = 0
    still_workspace = []
    all_crate_dirs = []

    for entry in sorted(os.scandir(CRATES_DIR), key=lambda e: e.name):
        if not entry.is_dir():
            continue
        cargo_toml = os.path.join(entry.path, "Cargo.toml")
        if not os.path.exists(cargo_toml):
            continue

        all_crate_dirs.append(entry.name)

        with open(cargo_toml) as f:
            content = f.read()

        # Skip crates that still have workspace = true (converter errors)
        if "workspace = true" in content:
            still_workspace.append(entry.name)
            continue

        new_content, modified = postprocess_cargo_toml(cargo_toml, dep_to_dir)

        if modified:
            with open(cargo_toml, 'w') as f:
                f.write(new_content)
            processed += 1
            if not content.lstrip().startswith("[workspace]"):
                added_ws += 1
            if 'registry = "nexcore"' in content and 'path =' not in content:
                added_paths += 1
        else:
            already_ok += 1

    print(f"\n  Post-processed: {processed}")
    print(f"  Already OK: {already_ok}")
    print(f"  Still workspace-dependent: {len(still_workspace)}")
    if still_workspace:
        for s in still_workspace:
            print(f"    - {s}")

    # Write exclude list (all crates except still-workspace-dependent)
    exclude_crates = sorted(set(all_crate_dirs) - set(still_workspace))
    exclude_file = os.path.join(NEXCORE_ROOT, "tools", "exclude-list.txt")
    with open(exclude_file, 'w') as f:
        for d in exclude_crates:
            f.write(f'    "crates/{d}",\n')
    print(f"\n  Exclude list ({len(exclude_crates)} crates) written to: {exclude_file}")


if __name__ == "__main__":
    main()
