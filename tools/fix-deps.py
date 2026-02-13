#!/usr/bin/env python3
"""Fix all path dependency issues: wrong versions and missing registry."""

import os
import re

CRATES_DIR = "/home/matthew/Projects/nexcore/crates"

# Build a map of crate_name -> version from all Cargo.toml files
crate_versions = {}
crate_dirs = {}
for entry in os.scandir(CRATES_DIR):
    if not entry.is_dir():
        continue
    ct = os.path.join(entry.path, "Cargo.toml")
    if not os.path.exists(ct):
        continue
    with open(ct) as f:
        content = f.read()
    m = re.search(r'^name\s*=\s*"([^"]+)"', content, re.MULTILINE)
    v = re.search(r'^version\s*=\s*"([^"]+)"', content, re.MULTILINE)
    if m and v:
        crate_versions[m.group(1)] = v.group(1)
        crate_dirs[m.group(1)] = entry.name

print(f"Found {len(crate_versions)} crates")

fixed_count = 0

for entry in sorted(os.scandir(CRATES_DIR), key=lambda e: e.name):
    if not entry.is_dir():
        continue
    ct = os.path.join(entry.path, "Cargo.toml")
    if not os.path.exists(ct):
        continue

    with open(ct) as f:
        content = f.read()
    original = content

    # Fix path deps: ensure version matches and registry = "nexcore" is present
    def fix_dep(m):
        global fixed_count
        full = m.group(0)
        dep_name = m.group(1)
        body = m.group(2)

        if 'path' not in body or '../' not in body:
            return full

        changed = False

        # Fix version to match actual crate version
        if dep_name in crate_versions:
            actual_ver = crate_versions[dep_name]
            ver_match = re.search(r'version\s*=\s*"([^"]+)"', body)
            if ver_match and ver_match.group(1) != actual_ver:
                body = body.replace(f'version = "{ver_match.group(1)}"', f'version = "{actual_ver}"')
                changed = True
            elif not ver_match:
                body = f'version = "{actual_ver}", ' + body
                changed = True

        # Add registry = "nexcore" if missing
        if 'registry' not in body:
            body = body.rstrip()
            if body.endswith(','):
                body += ' registry = "nexcore"'
            else:
                body += ', registry = "nexcore"'
            changed = True

        if changed:
            fixed_count += 1
            print(f"  [FIX] {entry.name}: dep {dep_name}")

        return f'{dep_name} = {{{body}}}'

    content = re.sub(
        r'([\w][\w-]*)\s*=\s*\{([^}]+)\}',
        fix_dep,
        content
    )

    # Also fix publish = false -> true for nexcore-domain-primitives
    if entry.name == 'nexcore-domain-primitives':
        content = content.replace('publish = false', 'publish = true')
        if content != original:
            print(f"  [FIX] {entry.name}: publish = true")

    if content != original:
        with open(ct, 'w') as f:
            f.write(content)

print(f"\nTotal fixes: {fixed_count}")
