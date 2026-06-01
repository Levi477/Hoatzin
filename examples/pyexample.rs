use hoatzin::{
    edge::edge::Edge, execution::context::ExecutionContext, node::node::Node,
    workflow::workflow::Workflow,
};
use serde_json::{Value, json};
use std::sync::Arc;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // make nodes
    let node1 = Node::new_python_node(
        "node1".to_string(),
        "/Users/levi/Desktop/hoatzin/local-test/node1.py".to_string(),
    );
    let node2 = Node::new_python_node(
        "node2".to_string(),
        "/Users/levi/Desktop/hoatzin/local-test/node2.py".to_string(),
    );
    let node3 = Node::new_python_node(
        "node3".to_string(),
        "/Users/levi/Desktop/hoatzin/local-test/node3.py".to_string(),
    );

    // form edges
    let edge1 = Edge::new(&node1, &node2, None);
    let edge2 = Edge::new(&node3, &node2, None);

    // make new workflow
    let workflow = Workflow::new(vec![node1, node2, node3], vec![edge1, edge2]);

    //make new execution context
    let mut ctx = ExecutionContext::new(Arc::new(workflow));

    // run this workflow
    ctx.run_workflow().await;
}
