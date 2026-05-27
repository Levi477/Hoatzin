use crate::node::node::Node;
use serde_json::Value;

pub async fn execute_node(
    node: Node,
    ctx: Value,
    front_node_id: String,
) -> (String, Result<Value, String>) {
    println!(
        "Executing Node with ID {} with input : {:#?}",
        &front_node_id, &ctx
    );
    let node_output = node.execute(ctx);

    match &node_output {
        Ok(op) => {
            println!("Success - Output for Node {} : {:#?}", front_node_id, &op);
        }
        Err(e) => {
            println!("Error - Output for Node {} : {:#?}", front_node_id, &e);
        }
    }
    (front_node_id, node_output)
}
