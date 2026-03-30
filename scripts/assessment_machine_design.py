import json

# Define the 15 states from curriculum
states = [
    {"id": 1, "domain": "D01", "name": "Identity", "primitive": "π", "activity": "Dissect Thalidomide", "assessment": "Case Decomposition Proof"},
    {"id": 2, "domain": "D15", "name": "Witness", "primitive": "∃", "activity": "Noise Filtering", "assessment": "Existence Verification Check"},
    {"id": 3, "domain": "D02", "name": "Classifier", "primitive": "κ", "activity": "MedDRA Mapping", "assessment": "Phenotype Match Fidelity"},
    {"id": 4, "domain": "D12", "name": "Guardian", "primitive": "∂", "activity": "7-Day Deadline Simulation", "assessment": "Boundary Compliance Audit"},
    {"id": 5, "domain": "D07", "name": "Aggregator", "primitive": "ν", "activity": "Signal Stream Mgmt", "assessment": "Reporting Rate Stability"},
    {"id": 6, "domain": "D03", "name": "Recognizer", "primitive": "∃", "activity": "IME/DME Triage", "assessment": "Sensitivity Sensitivity Test"},
    {"id": 7, "domain": "D04", "name": "Processor", "primitive": "σ", "activity": "E2B Transformation", "assessment": "Sequence Integrity Check"},
    {"id": 8, "domain": "D13", "name": "Strategist", "primitive": "λ", "activity": "Global Profile Mapping", "assessment": "Geospatial Alignment Proof"},
    {"id": 9, "domain": "D11", "name": "Actuator", "primitive": "∂", "activity": "REMS Design", "assessment": "Mitigation Effectiveness Score"},
    {"id": 10, "domain": "D10", "name": "Judge", "primitive": "κ", "activity": "Regulatory Committee", "assessment": "Benefit-Risk Consensus"},
    {"id": 11, "domain": "D08", "name": "Analyst", "primitive": "N", "activity": "FAERS Disproportionality", "assessment": "Statistical Significance Verification"},
    {"id": 12, "domain": "D14", "name": "Communicator", "primitive": "μ", "activity": "Plain Language Bridge", "assessment": "Transmission Fidelity Score"},
    {"id": 13, "domain": "D06", "name": "Auditor", "primitive": "∅", "activity": "Med Error RCA", "assessment": "Void Detection Accuracy"},
    {"id": 14, "domain": "D09", "name": "Lifecycle Mgr", "primitive": "π", "activity": "PASS Study Analysis", "assessment": "Continuity Persistence Proof"},
    {"id": 15, "domain": "D05", "name": "Genesis Witness", "primitive": "∃", "activity": "Trial SUSAR Management", "assessment": "GCP Protocol Adherence"}
]

# SMST Components for the Assessment Machine
assessment_machine = {
    "NAME": "Mastery-Verification-Machine",
    "INPUTS": {
        "TRIGGERS": ["State Change", "Experiential Completion", "/verify-mastery"],
        "CONTEXT": ["Student Work Product", "Target State Machine"],
        "PARAMETERS": ["Confidence Level (alpha)", "Time Spent (H)"]
    },
    "OUTPUTS": {
        "PRIMARY": "Mastery Score (0-100)",
        "ARTIFACTS": ["Verification Certificate", "State Transition Log"],
        "SIDE_EFFECTS": ["Transition to next state", "Refinement of priors"]
    },
    "PROCESS": {
        "ALGORITHM": [
            "1. Receive Experiential Data",
            "2. Compare with Primitive Gold Standard",
            "3. Calculate Error Delta (Δ)",
            "4. Compute Mastery Probability P(M|D)",
            "5. Emit Decision: Proceed/Remediate"
        ]
    }
}

print(json.dumps({"states": states, "machine": assessment_machine}, indent=2))
