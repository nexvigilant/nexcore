import numpy as np
import collections

# Domain Data
TIER_MAP = {"T2-C": 0.7, "T3": 0.4}
domains_raw = {
    "D01": {"name": "Foundations", "ksbs": 63, "transfer": 0.72, "tier": "T2-C"},
    "D02": {"name": "Clinical ADRs", "ksbs": 83, "transfer": 0.68, "tier": "T2-C"},
    "D03": {"name": "Important ADRs", "ksbs": 97, "transfer": 0.74, "tier": "T2-C"},
    "D04": {"name": "ICSRs", "ksbs": 104, "transfer": 0.65, "tier": "T2-C"},
    "D05": {"name": "Clinical Trials", "ksbs": 109, "transfer": 0.62, "tier": "T2-C"},
    "D06": {"name": "Med Errors", "ksbs": 105, "transfer": 0.82, "tier": "T2-C"},
    "D07": {"name": "SRS", "ksbs": 99, "transfer": 0.76, "tier": "T2-C"},
    "D08": {"name": "Signal Detection", "ksbs": 132, "transfer": 0.70, "tier": "T3"},
    "D09": {"name": "Post-Auth Studies", "ksbs": 109, "transfer": 0.71, "tier": "T2-C"},
    "D10": {"name": "Benefit-Risk", "ksbs": 110, "transfer": 0.84, "tier": "T3"},
    "D11": {"name": "Risk Mgmt", "ksbs": 108, "transfer": 0.80, "tier": "T2-C"},
    "D12": {"name": "Regulatory", "ksbs": 105, "transfer": 0.75, "tier": "T2-C"},
    "D13": {"name": "Global PV", "ksbs": 98, "transfer": 0.73, "tier": "T2-C"},
    "D14": {"name": "Communication", "ksbs": 70, "transfer": 0.81, "tier": "T2-C"},
    "D15": {"name": "Sources", "ksbs": 70, "transfer": 0.85, "tier": "T2-C"},
}

# Prerequisites (A -> B means A must be learned before B)
adj = {
    "D01": ["D15", "D02", "D12", "D13", "D06", "D05", "D09"],
    "D15": ["D04", "D07"],
    "D04": ["D08"],
    "D07": ["D08"],
    "D02": ["D03", "D10"],
    "D03": ["D08"],
    "D12": ["D11"],
    "D11": ["D14"],
    "D13": ["D14"],
    "D08": ["D14"],
    "D10": ["D14"],
    "D05": [], "D06": [], "D09": [], "D14": []
}

def get_descendants(node, adj):
    desc = set()
    stack = [node]
    while stack:
        n = stack.pop()
        for m in adj.get(n, []):
            if m not in desc:
                desc.add(m)
                stack.append(m)
    return desc

iterations = 1000
path_results = collections.defaultdict(list) # domain -> list of ranks

for _ in range(iterations):
    # 1. Sample parameters with noise
    node_stats = {}
    for d_id, data in domains_raw.items():
        tc = np.random.normal(data["transfer"], 0.05)
        k = np.random.normal(data["ksbs"], 3)
        tm = TIER_MAP[data["tier"]]
        efficiency = (tc * tm) / k
        node_stats[d_id] = {"eta": efficiency}
    
    # 2. Calculate Recursive Weight (η + 0.5 * sum of descendant η)
    for d_id in node_stats:
        desc = get_descendants(d_id, adj)
        desc_eta = sum(node_stats[m]["eta"] for m in desc)
        node_stats[d_id]["weight"] = node_stats[d_id]["eta"] + (0.5 * desc_eta)
    
    # 3. Path selection using topological sort + greedy weight
    current_path = []
    in_degree = collections.defaultdict(int)
    for u in adj:
        for v in adj[u]:
            in_degree[v] += 1
            
    available = [d for d in domains_raw if in_degree[d] == 0]
    
    while available:
        # Pick available node with highest weight
        u = max(available, key=lambda d: node_stats[d]["weight"])
        available.remove(u)
        current_path.append(u)
        
        for v in adj.get(u, []):
            in_degree[v] -= 1
            if in_degree[v] == 0:
                available.append(v)
                
    # Record ranks
    for rank, d_id in enumerate(current_path):
        path_results[d_id].append(rank + 1)

# Summary
print("| ID | Domain | Mean Rank | 95% CI (Lower) | 95% CI (Upper) | Stability |")
print("|----|--------|-----------|----------------|----------------|-----------|")

for d_id in domains_raw:
    ranks = path_results[d_id]
    mean = np.mean(ranks)
    lower = np.percentile(ranks, 2.5)
    upper = np.percentile(ranks, 97.5)
    stability = 1.0 / (np.std(ranks) + 1e-6) # Higher = more stable position
    
    print(f"| {d_id} | {domains_raw[d_id]['name'][:15]:<15} | {mean:9.2f} | {lower:14.1f} | {upper:14.1f} | {stability:9.2f} |")

