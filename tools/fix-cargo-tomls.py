#!/usr/bin/env python3
"""Fix all Cargo.toml issues blocking publish to Kellnr."""

import os
import re

CRATES_DIR = "/home/matthew/Projects/nexcore/crates"

fixed_license = 0
fixed_readme = 0
fixed_version = 0

for entry in sorted(os.scandir(CRATES_DIR), key=lambda e: e.name):
    if not entry.is_dir():
        continue
    ct = os.path.join(entry.path, "Cargo.toml")
    if not os.path.exists(ct):
        continue

    with open(ct) as f:
        content = f.read()
    original = content

    # Fix 1: license-file = "LICENSE" -> license = "LicenseRef-NexVigilant-Proprietary"
    if 'license-file' in content and not os.path.exists(os.path.join(entry.path, "LICENSE")):
        content = re.sub(
            r'license-file\s*=\s*"LICENSE"',
            'license = "LicenseRef-NexVigilant-Proprietary"',
            content
        )
        if content != original:
            fixed_license += 1
            print(f"  [FIX] {entry.name}: license-file -> license")

    # Fix 2: readme = "README.md" where README.md doesn't exist -> remove line
    if 'readme' in content and not os.path.exists(os.path.join(entry.path, "README.md")):
        before = content
        content = re.sub(r'\nreadme\s*=\s*"README\.md"\n', '\n', content)
        if content != before:
            fixed_readme += 1
            print(f"  [FIX] {entry.name}: removed missing readme reference")

    # Fix 3: path-only deps without version -> add version = "0.1.0"
    # Match: dep = { path = "../foo" } without version
    def add_version(m):
        global fixed_version
        dep_name = m.group(1)
        body = m.group(2)
        if 'path' in body and 'version' not in body and 'registry' not in body:
            # Add version before path
            body = re.sub(r'(path\s*=)', r'version = "0.1.0", \1', body)
            fixed_version += 1
            print(f"  [FIX] {entry.name}: added version to dep {dep_name}")
        return f'{dep_name} = {{{body}}}'

    content = re.sub(
        r'([\w][\w-]*)\s*=\s*\{([^}]+)\}',
        add_version,
        content
    )

    if content != original:
        with open(ct, 'w') as f:
            f.write(content)

print(f"\nTotal fixes: {fixed_license} license, {fixed_readme} readme, {fixed_version} version")
