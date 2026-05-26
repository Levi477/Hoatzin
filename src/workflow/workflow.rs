use crate::{edge::edge::Edge, node::node::Node};
use std::collections::HashMap;
use uuid::Uuid;

// This is a global struct for all current workflow triggers
// Workflow Owns all Edges and Nodes
// All executions can read this workflow wrapped in Arc<>
pub struct Workflow {
    pub id: String,
    // Map Node ID to Node
    pub nodes: HashMap<String, Node>,
    // Map Edge ID to Edge
    pub edges: HashMap<String, Edge>,
    // Map Node to outgoing Edges
    pub adjacency_list: HashMap<String, Vec<String>>,
    // Map Node to incoming Edges
    pub reverse_adjacency_list: HashMap<String, Vec<String>>,
}

impl Workflow {
    // Take a list of Nodes and Edges as input to build adjacency list and reverse adjacency list
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Self {
        let mut nodes_map: HashMap<String, Node> = HashMap::new();
        for node in nodes {
            nodes_map.insert(node.id.clone(), node);
        }

        let mut edges_map: HashMap<String, Edge> = HashMap::new();
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        let mut rev_adj_list: HashMap<String, Vec<String>> = HashMap::new();

        Self {
            id: Uuid::new_v4().simple().to_string(),
            nodes: nodes_map,
            edges: edges_map,
            adjacency_list: adj_list,
            reverse_adjacency_list: rev_adj_list,
        }
    }
}
