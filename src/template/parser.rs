use std::{collections::HashMap, path::PathBuf};

use tokio::fs;

use crate::{
    edge::edge::Edge, node::node::Node, template::template::WorkflowTOML,
    workflow::workflow::Workflow,
};

pub async fn parse_workflow_from_toml(workflow_toml_file: PathBuf) -> Result<Workflow, String> {
    // read raw content in toml file
    let toml_content = fs::read_to_string(workflow_toml_file).await.unwrap();

    // parse config in template format
    let config: WorkflowTOML = toml::from_str(&toml_content).unwrap();

    //a temporary map to store instantiated Nodes by their name
    let mut node_map: HashMap<String, Node> = HashMap::new();
    let mut engine_nodes: Vec<Node> = Vec::new();

    // Instantiate the Nodes
    for node_config in config.nodes {
        let node = match node_config.script_type.as_str() {
            "Python" => Node::new_python_node(
                node_config.name.clone(),
                node_config.description,
                &node_config.script_path,
            ),
            "JavaScript" => Node::new_javascript_node(
                node_config.name.clone(),
                node_config.description,
                &node_config.script_path,
            ),
            _ => {
                return Err(format!(
                    "Unsupported script type: {}",
                    node_config.script_type
                ));
            }
        };

        node_map.insert(node_config.name.clone(), node.clone());
        engine_nodes.push(node);
    }

    let mut engine_edges: Vec<Edge> = Vec::new();

    // Instantiate the Edges
    for edge_config in config.edges {
        let from_node = node_map.get(&edge_config.from_node).ok_or_else(|| {
            format!(
                "Edge references missing 'from' node: {}",
                edge_config.from_node
            )
        })?;

        let to_node = node_map
            .get(&edge_config.to_node)
            .ok_or_else(|| format!("Edge references missing 'to' node: {}", edge_config.to_node))?;

        let edge = Edge::new(from_node, to_node, edge_config.label);
        engine_edges.push(edge);
    }

    // Build the final workflow
    let workflow = Workflow::new(
        engine_nodes,
        engine_edges,
        config.workflow.name,
        config.workflow.description,
    );

    Ok(workflow)
}
