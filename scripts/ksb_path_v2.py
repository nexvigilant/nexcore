import json

# Multipliers from CLAUDE.md: T1(1.0), T2-P(0.9), T2-C(0.7), T3(0.4)
TIER_MAP = {
    "T2-C": 0.7,
    "T3": 0.4
}

domains = [
    {"id": "D01", "name": "Foundations", "ksbs": 63, "transfer": 0.72, "tier": "T2-C"},
    {"id": "D02", "name": "Clinical ADRs", "ksbs": 83, "transfer": 0.68, "tier": "T2-C"},
    {"id": "D03", "name": "Important ADRs", "ksbs": 97, "transfer": 0.74, "tier": "T2-C"},
    {"id": "D04", "name": "ICSRs", "ksbs": 104, "transfer": 0.65, "tier": "T2-C"},
    {"id": "D05", "name": "Clinical Trials", "ksbs": 109, "transfer": 0.62, "tier": "T2-C"},
    {"id": "D06", "name": "Med Errors", "ksbs": 105, "transfer": 0.82, "tier": "T2-C"},
    {"id": "D07", "name": "SRS", "ksbs": 99, "transfer": 0.76, "tier": "T2-C"},
    {"id": "D08", "name": "Signal Detection", "ksbs": 132, "transfer": 0.70, "tier": "T3"},
    {"id": "D09", "name": "Post-Auth Studies", "ksbs": 109, "transfer": 0.71, "tier": "T2-C"},
    {"id": "D10", "name": "Benefit-Risk", "ksbs": 110, "transfer": 0.84, "tier": "T3"},
    {"id": "D11", "name": "Risk Mgmt", "ksbs": 108, "transfer": 0.80, "tier": "T2-C"},
    {"id": "D12", "name": "Regulatory", "ksbs": 105, "transfer": 0.75, "tier": "T2-C"},
    {"id": "D13", "name": "Global PV", "ksbs": 98, "transfer": 0.73, "tier": "T2-C"},
    {"id": "D14", "name": "Communication", "ksbs": 70, "transfer": 0.81, "tier": "T2-C"},
    {"id": "D15", "name": "Sources", "ksbs": 70, "transfer": 0.85, "tier": "T2-C"},
]

# η = (TC * TierMultiplier) / KSBCount
for d in domains:
    d['efficiency'] = (d['transfer'] * TIER_MAP[d['tier']]) / d['ksbs']

# Sort by efficiency
sorted_domains = sorted(domains, key=lambda x: x['efficiency'], reverse=True)

print("| ID | Domain | η (Efficiency) | Transfer | KSBs | Tier |")
print("|----|--------|----------------|----------|------|------|")
for d in sorted_domains:
    print(f"| {d['id']} | {d['name'][:15]:<15} | {d['efficiency']:.5f} | {d['transfer']:.2f} | {d['ksbs']:>4} | {d['tier']} |")
