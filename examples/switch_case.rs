use hoatzin::{
    edge::edge::Edge, execution::context::ExecutionContext, node::node::Node,
    workflow::workflow::Workflow,
};
use serde_json::{Value, json};
use std::sync::Arc;

// ==========================================
// 1. Router Node (The Switch)
// ==========================================
fn node_a(_input: Value) -> Result<Value, String> {
    // We explicitly route ONLY to "PathC".
    let output = json!({
        "data": "Initial Payload",
        "__route__": ["PathC"]
    });
    Ok(output)
}

// ==========================================
// 2. Branch B (Labeled "PathB")
// ==========================================
fn node_b(input: Value) -> Result<Value, String> {
    // This should NOT print because "PathB" is not in the route array.
    Ok(json!({ "branch_executed": "B" }))
}

// ==========================================
// 3. Branch C (Labeled "PathC")
// ==========================================
fn node_c(input: Value) -> Result<Value, String> {
    // This SHOULD print because "PathC" is in the route array.
    Ok(json!({ "branch_executed": "C" }))
}

// ==========================================
// 4. Branch D (Unlabeled / Default)
// ==========================================
fn node_d(input: Value) -> Result<Value, String> {
    // This SHOULD print because it has NO label, so it runs unconditionally.
    Ok(json!({ "branch_executed": "D" }))
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // ---------------------------------------------------------
    // STEP 1: Create the Nodes
    // ---------------------------------------------------------
    let n_a = Node::new_native_node("Node A".to_string(), None, Arc::new(node_a));
    let n_b = Node::new_native_node("Node B".to_string(), None, Arc::new(node_b));
    let n_c = Node::new_native_node("Node C".to_string(), None, Arc::new(node_c));
    let n_d = Node::new_native_node("Node D".to_string(), None, Arc::new(node_d));

    // ---------------------------------------------------------
    // STEP 2: Wire the Edges
    // ---------------------------------------------------------
    let edge_a_to_b = Edge::new(&n_a, &n_b, Some("PathB".to_string()));
    let edge_a_to_c = Edge::new(&n_a, &n_c, Some("PathC".to_string()));
    let edge_a_to_d = Edge::new(&n_a, &n_d, None);

    // ---------------------------------------------------------
    // STEP 3: Build & Execute
    // ---------------------------------------------------------
    let workflow = Workflow::new(
        vec![n_a.clone(), n_b.clone(), n_c.clone(), n_d.clone()],
        vec![edge_a_to_b, edge_a_to_c, edge_a_to_d],
        "MixedRoutingWorkflow".to_string(),
        "Testing Labeled vs Unlabeled Paths".to_string(),
    );

    let mut execution_context = ExecutionContext::new(Arc::new(workflow));
    execution_context.run_workflow().await;
}
