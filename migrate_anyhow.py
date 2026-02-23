import os
import re

def migrate_crate(crate_dir):
    cargo_toml = os.path.join(crate_dir, "Cargo.toml")
    if not os.path.exists(cargo_toml):
        return
    with open(cargo_toml, "r") as f:
        content = f.read()
    
    if "anyhow" not in content:
        return
    
    # Check if nexcore-error is already present
    if "nexcore-error" not in content:
        content = re.sub(r'(?m)^anyhow\s*=\s*.*$', 'nexcore-error = { workspace = true }', content)
    else:
        content = re.sub(r'(?m)^anyhow\s*=\s*.\n', '', content)
        
    with open(cargo_toml, "w") as f:
        f.write(content)

    for root, dirs, files in os.walk(crate_dir):
        for file in files:
            if not file.endswith(".rs"):
                continue
            filepath = os.path.join(root, file)
            with open(filepath, "r") as f:
                code = f.read()
            
            if "anyhow" not in code:
                continue

            # Replace anyhow usages
            code = code.replace("anyhow::Result", "nexcore_error::Result")
            code = code.replace("anyhow::Error", "nexcore_error::NexError")
            code = code.replace("use anyhow::Result;", "use nexcore_error::Result;")
            code = code.replace("use anyhow::Error;", "use nexcore_error::NexError;")
            code = code.replace("use anyhow::anyhow;", "use nexcore_error::nexerror;")
            code = code.replace("use anyhow::bail;", "use nexcore_error::bail;")
            code = code.replace("use anyhow::ensure;", "use nexcore_error::ensure;")
            code = code.replace("use anyhow::{ ", "use nexcore_error::{ ")
            code = code.replace("anyhow::anyhow!", "nexcore_error::nexerror!")
            code = code.replace("anyhow::bail!", "nexcore_error::bail!")
            code = code.replace("anyhow::ensure!", "nexcore_error::ensure!")
            code = code.replace("anyhow!(", "nexerror!(")
            code = code.replace("use anyhow::Context;", "use nexcore_error::Context;")
            code = code.replace("anyhow::Context", "nexcore_error::Context")
            
            # For complex imports like `use anyhow::{ Result, Context, anyhow };`
            def replace_imports(m):
                inner = m.group(1)
                inner = inner.replace("Error", "NexError")
                inner = inner.replace("anyhow", "nexerror")
                return f"use nexcore_error::{{{inner}}};"
            
            code = re.sub(r'use anyhow::\{([^}]+)\};', replace_imports, code)

            with open(filepath, "w") as f:
                f.write(code)

if __name__ == "__main__":
    for crates_dir in ["crates", "tools"]:
        if not os.path.exists(crates_dir):
            continue
        for crate in os.listdir(crates_dir):
            crate_path = os.path.join(crates_dir, crate)
            if os.path.isdir(crate_path):
                migrate_crate(crate_path)
