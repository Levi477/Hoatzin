use crate::node::node::Node;
use serde_json::Value;

// own node,context and node id for executing in a separate threads
// return node id and it's output
pub async fn execute_node(
    node: Node,
    ctx: Value,
    node_id: String,
) -> (String, Result<Value, String>) {
    println!(
        "Executing Node with ID {} with input : {:#?}",
        &node_id, &ctx
    );
    let node_output = node.execute(ctx).await;

    match &node_output {
        Ok(op) => {
            println!("Success - Output for Node {} : {:#?}", node_id, &op);
        }
        Err(e) => {
            println!("Error - Output for Node {} : {:#?}", node_id, &e);
        }
    }
    (node_id, node_output)
}
