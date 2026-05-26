use uuid::Uuid;

use crate::node::node::Node;

pub struct Edge {
    pub id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub label: Option<String>,
}

impl Edge {
    pub fn new(from_node: &Node, to_node: &Node, label: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().simple().to_string(),
            from_node_id: from_node.id.clone(),
            to_node_id: to_node.id.clone(),
            label: label,
        }
    }
}
