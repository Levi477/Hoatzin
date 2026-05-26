use std::collections::HashMap;

use crate::execution::{node_output::NodeOutput, node_status::NodeStatus};

pub struct ExecutionContext {
    // Every New Workflow Trigger will result in new execution context and hence execution id
    id: String,
    // Unique ID of Workflow
    workflow_id: String,
    // Store Hashmap of Node ID to Node Output
    node_outputs: HashMap<String, NodeOutput>,
    // Store Hashmap of Node ID to Node Status
    node_status: HashMap<String, NodeStatus>,
}
