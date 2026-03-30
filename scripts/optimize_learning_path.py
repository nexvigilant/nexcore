import json

domains = [
    {"id": "D01", "name": "Foundations", "ksbs": 63, "transfer": 0.72, "tier": 4},
    {"id": "D02", "name": "Clinical ADRs", "ksbs": 83, "transfer": 0.68, "tier": 4},
    {"id": "D03", "name": "Important ADRs", "ksbs": 97, "transfer": 0.74, "tier": 4},
    {"id": "D04", "name": "ICSRs", "ksbs": 104, "transfer": 0.65, "tier": 4},
    {"id": "D05", "name": "Clinical Trials", "ksbs": 109, "transfer": 0.62, "tier": 4},
    {"id": "D06", "name": "Med Errors", "ksbs": 105, "transfer": 0.82, "tier": 4},
    {"id": "D07", "name": "SRS", "ksbs": 99, "transfer": 0.76, "tier": 4},
    {"id": "D08", "name": "Signal Detection", "ksbs": 132, "transfer": 0.70, "tier": 6},
    {"id": "D09", "name": "Post-Auth Studies", "ksbs": 109, "transfer": 0.71, "tier": 4},
    {"id": "D10", "name": "Benefit-Risk", "ksbs": 110, "transfer": 0.84, "tier": 6},
    {"id": "D11", "name": "Risk Mgmt", "ksbs": 108, "transfer": 0.80, "tier": 4},
    {"id": "D12", "name": "Regulatory", "ksbs": 105, "transfer": 0.75, "tier": 4},
    {"id": "D13", "name": "Global PV", "ksbs": 98, "transfer": 0.73, "tier": 4},
    {"id": "D14", "name": "Communication", "ksbs": 70, "transfer": 0.81, "tier": 4},
    {"id": "D15", "name": "Sources", "ksbs": 70, "transfer": 0.85, "tier": 4},
]

# η = (Transfer * Tier) / KSBs
for d in domains:
    d['efficiency'] = (d['transfer'] * d['tier']) / d['ksbs']

# Sorted by efficiency descending
sorted_domains = sorted(domains, key=lambda x: x['efficiency'], reverse=True)

print("ID | Name | Efficiency (η) | Transfer | KSBs")
print("-" * 55)
for d in sorted_domains:
    print(f"{d['id']} | {d['name'][:15]:<15} | {d['efficiency']:.4f} | {d['transfer']:.2f} | {d['ksbs']}")
