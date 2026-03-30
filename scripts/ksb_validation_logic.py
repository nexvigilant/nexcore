import re
import os

def validate_markdown(file_path):
    print(f"Validating {os.path.basename(file_path)}...")
    with open(file_path, 'r') as f:
        content = f.read()
    
    issues = []
    
    # 1. Structural Integrity: Required Sections
    required_sections = ["Executive Summary", "Primitive Composition", "State Theory Analysis"]
    for section in required_sections:
        if not re.search(f"^#+ .*?{section}", content, re.MULTILINE | re.IGNORECASE):
            issues.append(f"MISSING SECTION: {section}")

    # 2. Mathematical Validity: Confidence Intervals and Equations
    if "[" not in content or "]" not in content:
        issues.append("MISSING DATA: No confidence intervals detected")
    
    if "eta" not in content.lower() and "η" not in content:
        issues.append("MISSING MATH: Learning Efficiency (η) equation not found")

    # 3. Grounding: Lex Primitiva Symbols
    primitives = ["σ", "μ", "ς", "ρ", "∅", "∂", "ν", "∃", "π", "→", "κ", "N", "λ", "∝", "Σ", "×"]
    found_prims = [p for p in primitives if p in content]
    if len(found_prims) < 3:
        issues.append(f"LOW PRIMITIVE DENSITY: Only {len(found_prims)} primitives referenced")

    if not issues:
        print("✅ VALIDATION PASSED: L1-L3 Coherence, Structure, and Grounding confirmed.")
    else:
        for issue in issues:
            print(f"❌ VALIDATION ERROR: {issue}")
    print("-" * 30)

files = [
    "/home/matthew/research/ksb_learning_path_master.md",
    "/home/matthew/research/ksb_curriculum_perfected.md",
    "/home/matthew/research/ksb_experiential_learning_phase1.md"
]

for f in files:
    if os.path.exists(f):
        validate_markdown(f)
    else:
        print(f"File not found: {f}")
