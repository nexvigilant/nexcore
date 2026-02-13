//! # Network Analysis for Systemic Risk Assessment
//!
//! Models complex relationships between drugs, adverse events, patients,
//! and healthcare systems to identify systemic risks and contagion effects.
//!
//! ## Key Metrics
//!
//! | Metric | Purpose |
//! |--------|---------|
//! | Centrality | Node importance |
//! | Clustering | Local connectivity |
//! | Path Length | Network traversal |
//! | Contagion | Risk spread |

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Node type in the network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Drug,
    AdverseEvent,
    Patient,
    HealthcareFacility,
    RegulatoryAuthority,
}

/// Edge type in the network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    DrugInteraction,
    AdverseEventAssociation,
    PatientExposure,
    RegulatoryLink,
    TemporalSequence,
    CausalRelationship,
}

/// Edge direction
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeDirection {
    #[default]
    Undirected,
    Directed,
}

/// Node criticality level
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Criticality {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Analysis type
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisType {
    Centrality,
    CommunityDetection,
    ContagionSimulation,
    #[default]
    SystemicRisk,
    CascadeAnalysis,
}

/// Intervention type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionType {
    Isolation,
    Monitoring,
    Reinforcement,
    Redundancy,
}

/// Network node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub node_type: NodeType,
    pub attributes: HashMap<String, String>,
    #[serde(default = "default_risk")]
    pub risk_level: f64,
    #[serde(default = "default_exposure")]
    pub exposure_size: f64,
    #[serde(default)]
    pub criticality: Criticality,
}

fn default_risk() -> f64 {
    0.5
}
fn default_exposure() -> f64 {
    1.0
}

/// Network edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    #[serde(default = "default_weight")]
    pub weight: f64,
    pub edge_type: EdgeType,
    #[serde(default = "default_strength")]
    pub strength: f64,
    #[serde(default)]
    pub direction: EdgeDirection,
}

fn default_weight() -> f64 {
    0.5
}
fn default_strength() -> f64 {
    0.5
}

/// Contagion simulation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContagionParameters {
    #[serde(default = "default_transmission")]
    pub transmission_rate: f64,
    #[serde(default = "default_recovery")]
    pub recovery_rate: f64,
    #[serde(default)]
    pub initial_infection: Vec<String>,
    #[serde(default = "default_steps")]
    pub time_steps: u32,
    #[serde(default = "default_threshold")]
    pub threshold: f64,
}

fn default_transmission() -> f64 {
    0.1
}
fn default_recovery() -> f64 {
    0.05
}
fn default_steps() -> u32 {
    100
}
fn default_threshold() -> f64 {
    0.5
}

/// Stress test scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressScenario {
    pub name: String,
    pub affected_nodes: Vec<String>,
    #[serde(default = "default_multiplier")]
    pub risk_multiplier: f64,
}

fn default_multiplier() -> f64 {
    2.0
}

/// Input for network analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInput {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub analysis_type: AnalysisType,
    pub contagion_parameters: Option<ContagionParameters>,
    pub stress_test_scenarios: Option<Vec<StressScenario>>,
}

/// Centrality metrics for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityMetrics {
    pub degree: f64,
    pub betweenness: f64,
    pub closeness: f64,
    pub eigenvector: f64,
    pub pagerank: f64,
}

/// Risk metrics for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRiskMetrics {
    pub local_risk: f64,
    pub systemic_risk: f64,
    pub contagion_potential: f64,
    pub vulnerability_score: f64,
}

/// Node neighborhoods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neighborhoods {
    pub first_order: Vec<String>,
    pub second_order: Vec<String>,
    pub risk_neighborhood: Vec<String>,
}

/// Complete node metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub id: String,
    pub node_type: String,
    pub centrality: CentralityMetrics,
    pub risk_metrics: NodeRiskMetrics,
    pub neighborhoods: Neighborhoods,
}

/// Network-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub density: f64,
    pub average_degree: f64,
    pub clustering: f64,
    pub path_length: f64,
    pub diameter: f64,
    pub small_world_index: f64,
}

/// Community in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: u32,
    pub nodes: Vec<String>,
    pub size: usize,
    pub modularity: f64,
    pub internal_risk: f64,
    pub external_connections: u32,
    pub risk_concentration: f64,
}

/// Community structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityStructure {
    pub communities: Vec<Community>,
    pub modularity_score: f64,
    pub intercommunity_risk: Vec<Vec<f64>>,
}

/// Critical node in systemic risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalNode {
    pub id: String,
    pub systemic_importance: f64,
    pub failure_impact: f64,
    pub interconnectedness: f64,
}

/// Systemic risk analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemicRiskAnalysis {
    pub overall_systemic_risk: f64,
    pub risk_concentration: f64,
    pub network_fragility: f64,
    pub cascade_vulnerability: f64,
    pub contagion_risk: f64,
    pub critical_nodes: Vec<CriticalNode>,
}

/// Contagion timeline entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub time_step: u32,
    pub infected_nodes: Vec<String>,
    pub risk_levels: HashMap<String, f64>,
    pub total_system_risk: f64,
}

/// Peak risk info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakRisk {
    pub time_step: u32,
    pub risk_level: f64,
    pub affected_nodes: Vec<String>,
}

/// Recovery pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryPattern {
    pub recovery_time: u32,
    pub residual_risk: f64,
    pub permanent_damage: Vec<String>,
}

/// Contagion simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContagionSimulation {
    pub initial_state: HashMap<String, f64>,
    pub final_state: HashMap<String, f64>,
    pub timeline: Vec<TimelineEntry>,
    pub peak_risk: PeakRisk,
    pub recovery_pattern: RecoveryPattern,
}

/// Critical path in network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalPath {
    pub path: Vec<String>,
    pub risk_amplification: f64,
    pub breaking_points: Vec<String>,
}

/// Network resilience metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResilience {
    pub robustness: f64,
    pub redundancy: f64,
    pub adaptability: f64,
}

/// Vulnerability assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityAssessment {
    pub single_points_of_failure: Vec<String>,
    pub critical_paths: Vec<CriticalPath>,
    pub network_resilience: NetworkResilience,
}

/// Stress test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTestResult {
    pub name: String,
    pub systemic_risk_increase: f64,
    pub cascade_length: u32,
    pub recovery_time: u32,
    pub affected_communities: Vec<u32>,
    pub risk_amplification: f64,
}

/// Worst case scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorstCaseScenario {
    pub name: String,
    pub total_risk_increase: f64,
    pub system_failure_probability: f64,
}

/// Stress testing results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressTesting {
    pub scenarios: Vec<StressTestResult>,
    pub worst_case_scenario: WorstCaseScenario,
}

/// Targeted intervention
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetedIntervention {
    pub node_id: String,
    pub intervention_type: InterventionType,
    pub risk_reduction: f64,
    pub implementation_cost: f64,
    pub effectiveness: f64,
}

/// Network redesign recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRedesign {
    pub optimal_topology: String,
    pub risk_reduction: f64,
    pub implementation_feasibility: f64,
}

/// Risk mitigation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMitigation {
    pub targeted_interventions: Vec<TargetedIntervention>,
    pub network_redesign: NetworkRedesign,
}

/// Complete network analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResult {
    pub network_metrics: NetworkMetrics,
    pub node_metrics: Vec<NodeMetrics>,
    pub community_structure: CommunityStructure,
    pub systemic_risk_analysis: SystemicRiskAnalysis,
    pub contagion_simulation: Option<ContagionSimulation>,
    pub vulnerability_assessment: VulnerabilityAssessment,
    pub stress_testing: Option<StressTesting>,
    pub risk_mitigation: RiskMitigation,
}

/// Analyze network for systemic risk
#[must_use]
pub fn analyze_network(input: &NetworkInput) -> NetworkResult {
    if input.nodes.is_empty() {
        return empty_result();
    }
    let graph = build_graph(&input.nodes, &input.edges);
    let network_metrics = calculate_network_metrics(&graph);
    let node_metrics = calculate_node_metrics(&graph, &input.nodes);
    let community_structure = detect_communities(&graph, &input.nodes);
    let systemic_risk_analysis = analyze_systemic_risk(&graph, &input.nodes, &node_metrics);
    let contagion_simulation = input
        .contagion_parameters
        .as_ref()
        .map(|params| simulate_contagion(&graph, &input.nodes, params));
    let vulnerability_assessment = assess_vulnerabilities(&graph, &node_metrics);
    let stress_testing = input
        .stress_test_scenarios
        .as_ref()
        .map(|scenarios| perform_stress_testing(&graph, &input.nodes, scenarios));
    let risk_mitigation = generate_risk_mitigation(&systemic_risk_analysis);
    NetworkResult {
        network_metrics,
        node_metrics,
        community_structure,
        systemic_risk_analysis,
        contagion_simulation,
        vulnerability_assessment,
        stress_testing,
        risk_mitigation,
    }
}

/// Batch process multiple network inputs
#[must_use]
pub fn batch_network_analysis(inputs: &[NetworkInput]) -> Vec<NetworkResult> {
    inputs.iter().map(analyze_network).collect()
}

fn empty_result() -> NetworkResult {
    NetworkResult {
        network_metrics: NetworkMetrics {
            total_nodes: 0,
            total_edges: 0,
            density: 0.0,
            average_degree: 0.0,
            clustering: 0.0,
            path_length: 0.0,
            diameter: 0.0,
            small_world_index: 0.0,
        },
        node_metrics: vec![],
        community_structure: CommunityStructure {
            communities: vec![],
            modularity_score: 0.0,
            intercommunity_risk: vec![],
        },
        systemic_risk_analysis: SystemicRiskAnalysis {
            overall_systemic_risk: 0.0,
            risk_concentration: 0.0,
            network_fragility: 0.0,
            cascade_vulnerability: 0.0,
            contagion_risk: 0.0,
            critical_nodes: vec![],
        },
        contagion_simulation: None,
        vulnerability_assessment: VulnerabilityAssessment {
            single_points_of_failure: vec![],
            critical_paths: vec![],
            network_resilience: NetworkResilience {
                robustness: 0.0,
                redundancy: 0.0,
                adaptability: 0.0,
            },
        },
        stress_testing: None,
        risk_mitigation: RiskMitigation {
            targeted_interventions: vec![],
            network_redesign: NetworkRedesign {
                optimal_topology: "none".into(),
                risk_reduction: 0.0,
                implementation_feasibility: 0.0,
            },
        },
    }
}

struct Graph {
    adj: Vec<Vec<f64>>,
    node_ids: Vec<String>,
    node_map: HashMap<String, usize>,
}

fn build_graph(nodes: &[Node], edges: &[Edge]) -> Graph {
    let n = nodes.len();
    let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
    let node_map: HashMap<String, usize> = node_ids
        .iter()
        .enumerate()
        .map(|(i, id)| (id.clone(), i))
        .collect();
    let mut adj = vec![vec![0.0; n]; n];
    edges.iter().for_each(|e| {
        if let (Some(&src), Some(&tgt)) = (node_map.get(&e.source), node_map.get(&e.target)) {
            adj[src][tgt] = e.weight;
            if matches!(e.direction, EdgeDirection::Undirected) {
                adj[tgt][src] = e.weight;
            }
        }
    });
    Graph {
        adj,
        node_ids,
        node_map,
    }
}

fn calculate_network_metrics(graph: &Graph) -> NetworkMetrics {
    let n = graph.node_ids.len();
    let m = graph
        .adj
        .iter()
        .flat_map(|row| row.iter())
        .filter(|&&w| w > 0.0)
        .count()
        / 2;
    let density = if n > 1 {
        (2 * m) as f64 / (n * (n - 1)) as f64
    } else {
        0.0
    };
    let degrees: Vec<usize> = graph
        .adj
        .iter()
        .map(|row| row.iter().filter(|&&w| w > 0.0).count())
        .collect();
    let average_degree = degrees.iter().sum::<usize>() as f64 / n.max(1) as f64;
    let clustering = calculate_clustering(&graph.adj);
    let (path_length, diameter) = calculate_path_metrics(graph);
    let small_world_index = if path_length > 0.0 {
        (clustering / (average_degree / n as f64).max(0.01))
            / (path_length / (n as f64).ln() / average_degree.max(0.01).ln())
    } else {
        0.0
    };
    NetworkMetrics {
        total_nodes: n,
        total_edges: m,
        density,
        average_degree,
        clustering,
        path_length,
        diameter: diameter as f64,
        small_world_index,
    }
}

fn calculate_clustering(adj: &[Vec<f64>]) -> f64 {
    let n = adj.len();
    (0..n)
        .map(|i| {
            let neighbors: Vec<usize> = adj[i]
                .iter()
                .enumerate()
                .filter(|&(_, &w)| w > 0.0)
                .map(|(j, _)| j)
                .collect();
            if neighbors.len() < 2 {
                return 0.0;
            }
            let triangles = neighbors
                .iter()
                .enumerate()
                .flat_map(|(idx, &j)| {
                    neighbors
                        .iter()
                        .skip(idx + 1)
                        .filter(move |&&k| adj[j][k] > 0.0)
                })
                .count();
            let possible = neighbors.len() * (neighbors.len() - 1) / 2;
            if possible > 0 {
                triangles as f64 / possible as f64
            } else {
                0.0
            }
        })
        .sum::<f64>()
        / n.max(1) as f64
}

fn calculate_path_metrics(graph: &Graph) -> (f64, usize) {
    let n = graph.adj.len();
    let mut dist = vec![vec![usize::MAX / 2; n]; n];
    (0..n).for_each(|i| {
        dist[i][i] = 0;
        (0..n).for_each(|j| {
            if graph.adj[i][j] > 0.0 {
                dist[i][j] = 1;
            }
        });
    });
    (0..n).for_each(|k| {
        (0..n).for_each(|i| {
            (0..n).for_each(|j| {
                dist[i][j] = dist[i][j].min(dist[i][k].saturating_add(dist[k][j]));
            });
        });
    });
    let dist_ref = &dist;
    let (total, count, diameter) = (0..n)
        .flat_map(|i| (i + 1..n).map(move |j| dist_ref[i][j]))
        .filter(|&d| d < usize::MAX / 2)
        .fold((0usize, 0usize, 0usize), |(t, c, d), x| {
            (t + x, c + 1, d.max(x))
        });
    (
        if count > 0 {
            total as f64 / count as f64
        } else {
            0.0
        },
        diameter,
    )
}

fn calculate_node_metrics(graph: &Graph, nodes: &[Node]) -> Vec<NodeMetrics> {
    let n = graph.adj.len();
    let degree_cent: Vec<f64> = graph
        .adj
        .iter()
        .map(|row| row.iter().filter(|&&w| w > 0.0).count() as f64 / (n - 1).max(1) as f64)
        .collect();
    let betweenness = calculate_betweenness(graph);
    let closeness = calculate_closeness(graph);
    let eigenvector = calculate_eigenvector(graph);
    let pagerank = calculate_pagerank(graph);

    nodes
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let neighbors: Vec<String> = graph.adj[i]
                .iter()
                .enumerate()
                .filter(|&(_, &w)| w > 0.0)
                .map(|(j, _)| graph.node_ids[j].clone())
                .collect();
            let second_order: Vec<String> = neighbors
                .iter()
                .flat_map(|n| {
                    graph
                        .node_map
                        .get(n)
                        .map(|&idx| {
                            graph.adj[idx]
                                .iter()
                                .enumerate()
                                .filter(|&(_, &w)| w > 0.0)
                                .map(|(j, _)| graph.node_ids[j].clone())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                })
                .filter(|id| !neighbors.contains(id) && id != &node.id)
                .collect();
            let risk_neighborhood: Vec<String> = neighbors
                .iter()
                .filter(|n| {
                    graph
                        .node_map
                        .get(*n)
                        .and_then(|&idx| nodes.get(idx))
                        .map(|n| n.risk_level > 0.7)
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
            let degree = graph.adj[i].iter().filter(|&&w| w > 0.0).count();
            NodeMetrics {
                id: node.id.clone(),
                node_type: format!("{:?}", node.node_type).to_lowercase(),
                centrality: CentralityMetrics {
                    degree: degree_cent[i],
                    betweenness: betweenness[i],
                    closeness: closeness[i],
                    eigenvector: eigenvector[i],
                    pagerank: pagerank[i],
                },
                risk_metrics: NodeRiskMetrics {
                    local_risk: node.risk_level,
                    systemic_risk: node.risk_level * (1.0 + degree as f64 / n as f64),
                    contagion_potential: node.risk_level * degree as f64 * 0.1,
                    vulnerability_score: if neighbors.is_empty() {
                        0.0
                    } else {
                        risk_neighborhood.len() as f64 / neighbors.len() as f64
                    },
                },
                neighborhoods: Neighborhoods {
                    first_order: neighbors,
                    second_order,
                    risk_neighborhood,
                },
            }
        })
        .collect()
}

fn calculate_betweenness(graph: &Graph) -> Vec<f64> {
    let n = graph.adj.len();
    let mut betweenness = vec![0.0; n];
    (0..n).for_each(|s| {
        let mut stack = vec![];
        let mut pred: Vec<Vec<usize>> = vec![vec![]; n];
        let mut sigma: Vec<f64> = vec![0.0; n];
        let mut dist = vec![-1i64; n];
        sigma[s] = 1.0;
        dist[s] = 0;
        let mut queue = vec![s];
        let mut qi = 0;
        loop {
            if qi >= queue.len() {
                break;
            }
            let v = queue[qi];
            qi += 1;
            stack.push(v);
            (0..n).filter(|&w| graph.adj[v][w] > 0.0).for_each(|w| {
                if dist[w] < 0 {
                    queue.push(w);
                    dist[w] = dist[v] + 1;
                }
                if dist[w] == dist[v] + 1 {
                    sigma[w] += sigma[v];
                    pred[w].push(v);
                }
            });
        }
        let mut delta = vec![0.0; n];
        stack.iter().rev().for_each(|&w| {
            let sigma_w: f64 = sigma[w].max(1e-10);
            pred[w].iter().for_each(|&v| {
                delta[v] += (sigma[v] / sigma_w) * (1.0 + delta[w]);
            });
            if w != s {
                betweenness[w] += delta[w];
            }
        });
    });
    let norm = if n > 2 {
        2.0 / ((n - 1) * (n - 2)) as f64
    } else {
        1.0
    };
    betweenness.iter().map(|&b| b * norm).collect()
}

fn calculate_closeness(graph: &Graph) -> Vec<f64> {
    let n = graph.adj.len();
    let mut dist = vec![vec![usize::MAX / 2; n]; n];
    (0..n).for_each(|i| {
        dist[i][i] = 0;
        (0..n).for_each(|j| {
            if graph.adj[i][j] > 0.0 {
                dist[i][j] = 1;
            }
        });
    });
    (0..n).for_each(|k| {
        (0..n).for_each(|i| {
            (0..n).for_each(|j| {
                dist[i][j] = dist[i][j].min(dist[i][k].saturating_add(dist[k][j]));
            });
        });
    });
    (0..n)
        .map(|i| {
            let sum: usize = dist[i].iter().filter(|&&d| d < usize::MAX / 2).sum();
            if sum > 0 {
                (n - 1) as f64 / sum as f64
            } else {
                0.0
            }
        })
        .collect()
}

fn calculate_eigenvector(graph: &Graph) -> Vec<f64> {
    let n = graph.adj.len();
    let mut ev = vec![1.0 / (n as f64).sqrt(); n];
    (0..100).for_each(|_| {
        let new_ev: Vec<f64> = (0..n)
            .map(|i| (0..n).map(|j| graph.adj[i][j] * ev[j]).sum())
            .collect();
        let norm = new_ev.iter().map(|x| x * x).sum::<f64>().sqrt().max(1e-10);
        ev = new_ev.iter().map(|x| x / norm).collect();
    });
    ev
}

fn calculate_pagerank(graph: &Graph) -> Vec<f64> {
    let n = graph.adj.len();
    let d = 0.85;
    let mut pr = vec![1.0 / n as f64; n];
    (0..100).for_each(|_| {
        let new_pr: Vec<f64> = (0..n)
            .map(|i| {
                let contrib: f64 = (0..n)
                    .map(|j| {
                        let out_deg = graph.adj[j].iter().filter(|&&w| w > 0.0).count();
                        if out_deg > 0 && graph.adj[j][i] > 0.0 {
                            d * pr[j] / out_deg as f64
                        } else {
                            0.0
                        }
                    })
                    .sum();
                (1.0 - d) / n as f64 + contrib
            })
            .collect();
        pr = new_pr;
    });
    pr
}

fn detect_communities(graph: &Graph, nodes: &[Node]) -> CommunityStructure {
    let n = graph.adj.len();
    let mut assigned = vec![false; n];
    let mut communities = vec![];
    (0..n).for_each(|i| {
        if assigned[i] {
            return;
        }
        let mut comm = vec![i];
        assigned[i] = true;
        (i + 1..n).for_each(|j| {
            if !assigned[j] && graph.adj[i][j] > 0.5 {
                comm.push(j);
                assigned[j] = true;
            }
        });
        communities.push(comm);
    });
    let community_results: Vec<Community> = communities
        .iter()
        .enumerate()
        .map(|(id, comm)| {
            let comm_nodes: Vec<String> = comm
                .iter()
                .map(|&idx| graph.node_ids[idx].clone())
                .collect();
            let internal_risk: f64 = comm
                .iter()
                .map(|&idx| nodes.get(idx).map(|n| n.risk_level).unwrap_or(0.0))
                .sum::<f64>()
                / comm.len().max(1) as f64;
            let external: u32 = comm
                .iter()
                .flat_map(|&idx| {
                    graph.adj[idx]
                        .iter()
                        .enumerate()
                        .filter(|&(j, &w)| w > 0.0 && !comm.contains(&j))
                })
                .count() as u32;
            let risk_levels: Vec<f64> = comm
                .iter()
                .filter_map(|&idx| nodes.get(idx).map(|n| n.risk_level))
                .collect();
            let avg = risk_levels.iter().sum::<f64>() / risk_levels.len().max(1) as f64;
            let risk_concentration = risk_levels.iter().map(|r| (r - avg).powi(2)).sum::<f64>()
                / risk_levels.len().max(1) as f64;
            Community {
                id: id as u32,
                nodes: comm_nodes,
                size: comm.len(),
                modularity: 0.0,
                internal_risk,
                external_connections: external,
                risk_concentration,
            }
        })
        .collect();
    let modularity = calculate_modularity(graph, &communities);
    let intercommunity = calculate_intercommunity_risk(graph, &communities, nodes);
    CommunityStructure {
        communities: community_results,
        modularity_score: modularity,
        intercommunity_risk: intercommunity,
    }
}

fn calculate_modularity(graph: &Graph, communities: &[Vec<usize>]) -> f64 {
    let m = graph
        .adj
        .iter()
        .flat_map(|r| r.iter())
        .filter(|&&w| w > 0.0)
        .count() as f64
        / 2.0;
    if m == 0.0 {
        return 0.0;
    }
    communities
        .iter()
        .map(|comm| {
            let internal: usize = comm
                .iter()
                .flat_map(|&i| comm.iter().filter(move |&&j| graph.adj[i][j] > 0.0))
                .count()
                / 2;
            let total_deg: usize = comm
                .iter()
                .map(|&i| graph.adj[i].iter().filter(|&&w| w > 0.0).count())
                .sum();
            (internal as f64 / m) - (total_deg as f64 / (2.0 * m)).powi(2)
        })
        .sum()
}

fn calculate_intercommunity_risk(
    graph: &Graph,
    communities: &[Vec<usize>],
    nodes: &[Node],
) -> Vec<Vec<f64>> {
    let nc = communities.len();
    let mut risk = vec![vec![0.0; nc]; nc];
    (0..nc).for_each(|i| {
        (0..nc).for_each(|j| {
            if i == j {
                return;
            }
            let (conn, total) = communities[i]
                .iter()
                .flat_map(|&ni| {
                    communities[j]
                        .iter()
                        .filter(move |&&nj| graph.adj[ni][nj] > 0.0)
                        .map(move |&nj| {
                            (nodes.get(ni).map(|n| n.risk_level).unwrap_or(0.0)
                                + nodes.get(nj).map(|n| n.risk_level).unwrap_or(0.0))
                                / 2.0
                        })
                })
                .fold((0usize, 0.0), |(c, t), r| (c + 1, t + r));
            risk[i][j] = if conn > 0 { total / conn as f64 } else { 0.0 };
        });
    });
    risk
}

fn analyze_systemic_risk(
    graph: &Graph,
    nodes: &[Node],
    node_metrics: &[NodeMetrics],
) -> SystemicRiskAnalysis {
    let n = nodes.len();
    let risks: Vec<f64> = nodes.iter().map(|n| n.risk_level).collect();
    let avg = risks.iter().sum::<f64>() / n.max(1) as f64;
    let weighted: Vec<f64> = node_metrics
        .iter()
        .enumerate()
        .map(|(i, m)| {
            risks[i]
                * (1.0
                    + (m.centrality.degree + m.centrality.betweenness + m.centrality.eigenvector)
                        / 3.0)
        })
        .collect();
    let overall = weighted.iter().sum::<f64>() / n.max(1) as f64;
    let total_risk = risks.iter().sum::<f64>().max(0.01);
    let shares: Vec<f64> = risks.iter().map(|r| r / total_risk).collect();
    let concentration = shares.iter().map(|s| s * s).sum::<f64>();
    let density = graph
        .adj
        .iter()
        .flat_map(|r| r.iter())
        .filter(|&&w| w > 0.0)
        .count() as f64
        / (n * n.saturating_sub(1)).max(1) as f64;
    let fragility = avg * (1.0 - density);
    let cascade = node_metrics
        .iter()
        .map(|m| m.centrality.betweenness)
        .fold(0.0_f64, f64::max);
    let degrees: Vec<usize> = graph
        .adj
        .iter()
        .map(|r| r.iter().filter(|&&w| w > 0.0).count())
        .collect();
    let contagion = risks
        .iter()
        .zip(degrees.iter())
        .map(|(r, &d)| r * d as f64)
        .sum::<f64>()
        / degrees.iter().sum::<usize>().max(1) as f64;
    let mut critical: Vec<CriticalNode> = node_metrics
        .iter()
        .map(|m| CriticalNode {
            id: m.id.clone(),
            systemic_importance: (m.centrality.betweenness + m.centrality.eigenvector) / 2.0,
            failure_impact: m.risk_metrics.systemic_risk,
            interconnectedness: m.centrality.degree,
        })
        .collect();
    critical.sort_by(|a, b| {
        b.systemic_importance
            .partial_cmp(&a.systemic_importance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    critical.truncate(10);
    SystemicRiskAnalysis {
        overall_systemic_risk: overall,
        risk_concentration: concentration,
        network_fragility: fragility,
        cascade_vulnerability: cascade,
        contagion_risk: contagion,
        critical_nodes: critical,
    }
}

fn simulate_contagion(
    graph: &Graph,
    nodes: &[Node],
    params: &ContagionParameters,
) -> ContagionSimulation {
    let n = nodes.len();
    let mut state: HashMap<String, f64> = nodes
        .iter()
        .map(|n| {
            (
                n.id.clone(),
                if params.initial_infection.contains(&n.id) {
                    1.0
                } else {
                    n.risk_level
                },
            )
        })
        .collect();
    let initial = state.clone();
    let mut timeline = vec![];
    let mut peak = PeakRisk {
        time_step: 0,
        risk_level: 0.0,
        affected_nodes: vec![],
    };
    (1..=params.time_steps).for_each(|t| {
        let mut new_state = state.clone();
        nodes.iter().enumerate().for_each(|(i, node)| {
            let contagion: f64 = graph.adj[i]
                .iter()
                .enumerate()
                .filter(|&(_, &w)| w > 0.0)
                .map(|(j, &w)| {
                    w * state.get(&graph.node_ids[j]).copied().unwrap_or(0.0)
                        * params.transmission_rate
                })
                .sum();
            let mut new_risk = state.get(&node.id).copied().unwrap_or(0.0) + contagion;
            if new_risk > params.threshold {
                new_risk = (new_risk - params.recovery_rate).max(0.0);
            }
            new_state.insert(node.id.clone(), new_risk.min(1.0));
        });
        state = new_state;
        let infected: Vec<String> = state
            .iter()
            .filter(|&(_, &v)| v > params.threshold)
            .map(|(k, _)| k.clone())
            .collect();
        let total = state.values().sum::<f64>() / n as f64;
        timeline.push(TimelineEntry {
            time_step: t,
            infected_nodes: infected.clone(),
            risk_levels: state.clone(),
            total_system_risk: total,
        });
        if total > peak.risk_level {
            peak = PeakRisk {
                time_step: t,
                risk_level: total,
                affected_nodes: infected,
            };
        }
    });
    let residual = state.values().sum::<f64>() / n as f64;
    let recovery_time = timeline
        .iter()
        .find(|e| e.total_system_risk < peak.risk_level * 0.1)
        .map(|e| e.time_step)
        .unwrap_or(params.time_steps);
    let permanent: Vec<String> = state
        .iter()
        .filter(|&(_, &v)| v > 0.1)
        .map(|(k, _)| k.clone())
        .collect();
    ContagionSimulation {
        initial_state: initial,
        final_state: state,
        timeline,
        peak_risk: peak,
        recovery_pattern: RecoveryPattern {
            recovery_time,
            residual_risk: residual,
            permanent_damage: permanent,
        },
    }
}

fn assess_vulnerabilities(_graph: &Graph, node_metrics: &[NodeMetrics]) -> VulnerabilityAssessment {
    let spof: Vec<String> = node_metrics
        .iter()
        .filter(|m| m.centrality.betweenness > 0.5)
        .map(|m| m.id.clone())
        .collect();
    let high_risk: Vec<String> = node_metrics
        .iter()
        .filter(|m| m.risk_metrics.systemic_risk > 0.7)
        .map(|m| m.id.clone())
        .take(3)
        .collect();
    let critical_paths = vec![CriticalPath {
        path: high_risk.clone(),
        risk_amplification: 2.0,
        breaking_points: high_risk.into_iter().take(1).collect(),
    }];
    let avg_deg = node_metrics
        .iter()
        .map(|m| m.centrality.degree)
        .sum::<f64>()
        / node_metrics.len().max(1) as f64;
    let deg_var = node_metrics
        .iter()
        .map(|m| (m.centrality.degree - avg_deg).powi(2))
        .sum::<f64>()
        / node_metrics.len().max(1) as f64;
    VulnerabilityAssessment {
        single_points_of_failure: spof,
        critical_paths,
        network_resilience: NetworkResilience {
            robustness: 1.0 - deg_var.min(1.0),
            redundancy: avg_deg,
            adaptability: 0.5,
        },
    }
}

fn perform_stress_testing(
    _graph: &Graph,
    nodes: &[Node],
    scenarios: &[StressScenario],
) -> StressTesting {
    let results: Vec<StressTestResult> = scenarios
        .iter()
        .map(|s| {
            let baseline: f64 = nodes.iter().map(|n| n.risk_level).sum();
            let stressed: f64 = nodes
                .iter()
                .map(|n| {
                    if s.affected_nodes.contains(&n.id) {
                        (n.risk_level * s.risk_multiplier).min(1.0)
                    } else {
                        n.risk_level
                    }
                })
                .sum();
            let increase = (stressed - baseline) / baseline.max(0.01);
            StressTestResult {
                name: s.name.clone(),
                systemic_risk_increase: increase,
                cascade_length: 5,
                recovery_time: 10,
                affected_communities: vec![0, 1],
                risk_amplification: 1.5,
            }
        })
        .collect();
    let worst = results
        .iter()
        .max_by(|a, b| {
            a.systemic_risk_increase
                .partial_cmp(&b.systemic_risk_increase)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
        .unwrap_or(StressTestResult {
            name: "None".into(),
            systemic_risk_increase: 0.0,
            cascade_length: 0,
            recovery_time: 0,
            affected_communities: vec![],
            risk_amplification: 1.0,
        });
    StressTesting {
        scenarios: results,
        worst_case_scenario: WorstCaseScenario {
            name: worst.name,
            total_risk_increase: worst.systemic_risk_increase,
            system_failure_probability: (worst.systemic_risk_increase * 0.5).min(1.0),
        },
    }
}

fn generate_risk_mitigation(systemic: &SystemicRiskAnalysis) -> RiskMitigation {
    let interventions: Vec<TargetedIntervention> = systemic
        .critical_nodes
        .iter()
        .map(|cn| {
            let (intervention_type, risk_reduction, cost) = if cn.systemic_importance > 0.8 {
                (InterventionType::Reinforcement, 0.7, 0.8)
            } else if cn.interconnectedness > 0.7 {
                (InterventionType::Redundancy, 0.6, 0.9)
            } else {
                (InterventionType::Monitoring, 0.3, 0.2)
            };
            TargetedIntervention {
                node_id: cn.id.clone(),
                intervention_type,
                risk_reduction,
                implementation_cost: cost,
                effectiveness: risk_reduction / cost.max(0.01),
            }
        })
        .collect();
    RiskMitigation {
        targeted_interventions: interventions,
        network_redesign: NetworkRedesign {
            optimal_topology: "decentralized_mesh".into(),
            risk_reduction: 0.4,
            implementation_feasibility: 0.3,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_input() -> NetworkInput {
        NetworkInput {
            nodes: vec![
                Node {
                    id: "Drug1".into(),
                    node_type: NodeType::Drug,
                    attributes: HashMap::new(),
                    risk_level: 0.5,
                    exposure_size: 1000.0,
                    criticality: Criticality::Medium,
                },
                Node {
                    id: "AE1".into(),
                    node_type: NodeType::AdverseEvent,
                    attributes: HashMap::new(),
                    risk_level: 0.7,
                    exposure_size: 100.0,
                    criticality: Criticality::High,
                },
            ],
            edges: vec![Edge {
                source: "Drug1".into(),
                target: "AE1".into(),
                weight: 0.8,
                edge_type: EdgeType::AdverseEventAssociation,
                strength: 0.8,
                direction: EdgeDirection::Directed,
            }],
            analysis_type: AnalysisType::SystemicRisk,
            contagion_parameters: None,
            stress_test_scenarios: None,
        }
    }
    #[test]
    fn test_analyze_network() {
        let result = analyze_network(&sample_input());
        assert_eq!(result.network_metrics.total_nodes, 2);
        assert!(!result.node_metrics.is_empty());
    }
    #[test]
    fn test_empty_input() {
        let input = NetworkInput {
            nodes: vec![],
            edges: vec![],
            ..sample_input()
        };
        let result = analyze_network(&input);
        assert_eq!(result.network_metrics.total_nodes, 0);
    }
    #[test]
    fn test_batch_analysis() {
        let results = batch_network_analysis(&[sample_input(), sample_input()]);
        assert_eq!(results.len(), 2);
    }
    #[test]
    fn test_contagion() {
        let mut input = sample_input();
        input.contagion_parameters = Some(ContagionParameters {
            transmission_rate: 0.1,
            recovery_rate: 0.05,
            initial_infection: vec!["Drug1".into()],
            time_steps: 10,
            threshold: 0.5,
        });
        let result = analyze_network(&input);
        assert!(result.contagion_simulation.is_some());
    }
}
