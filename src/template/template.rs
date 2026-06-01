use std::path::PathBuf;

use serde::Deserialize;

// define structure for template parsing
// template will be given in workflow.toml
#[derive(Deserialize, Debug)]
pub struct WorkflowTOML {
    pub workflow: WorkflowMeta,
    pub nodes: Vec<NodeTOML>,
    pub edges: Vec<EdgeTOML>,
}

#[derive(Deserialize, Debug)]
pub struct WorkflowMeta {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct NodeTOML {
    pub name: String,
    pub script_type: String,
    pub script_path: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct EdgeTOML {
    pub label: Option<String>,
    pub from_node: String,
    pub to_node: String,
}
