// run using cargo run --exmaple ecommerce

use hoatzin::{
    edge::edge::Edge, execution::context::ExecutionContext, node::node::Node,
    workflow::workflow::Workflow,
};
use serde_json::{Value, json};
use std::sync::Arc;

/*
                           [ Node 1: receive_order ]
                           (Trigger: Outputs Order Data)
                               /                   \
                     edge1    /                     \   edge2
                             /                       \
                            v                         v
           [ Node 2: process_payment ]      [ Node 3: check_inventory ]
           (Reads 'amount' from Node 1)     (Reads 'item' from Node 1)
                            \                         /
                     edge3   \                       /   edge4
                              \                     /
                               v                   v
                          [ Node 4: dispatch_order ]
                (Waits for BOTH Node 2 and Node 3 to complete)

*/

// ==========================================
// 1. Trigger Node
// ==========================================
fn receive_order(_input: Value) -> Result<Value, String> {
    let output = json!({
        "order_id": "ORD-7781",
        "customer_name": "Levi",
        "item": "Mechanical Keyboard",
        "amount": 150.00
    });
    Ok(output)
}

// ==========================================
// 2. Charge the Card
// ==========================================
fn process_payment(input: Value) -> Result<Value, String> {
    // Read the data sent from the "receive_order" node
    let _amount = input["receive_order"]["amount"].as_f64().unwrap_or(0.0);
    let output = json!({
        "payment_status": "success",
        "transaction_id": "tx_99210"
    });
    Ok(output)
}

// ==========================================
// 3.Check Inventory
// ==========================================
fn check_inventory(input: Value) -> Result<Value, String> {
    // Read the data sent from the "receive_order" node
    let _item = input["receive_order"]["item"]
        .as_str()
        .unwrap_or("Unknown Item");

    let output = json!({
        "stock_available": true,
        "warehouse_zone": "Zone-A"
    });
    Ok(output)
}

// ==========================================
// 4. Final Node  for Payment & Inventory
// ==========================================
fn dispatch_order(input: Value) -> Result<Value, String> {
    let payment_status = input["process_payment"]["payment_status"]
        .as_str()
        .unwrap_or("failed");
    let in_stock = input["check_inventory"]["stock_available"]
        .as_bool()
        .unwrap_or(false);

    if payment_status == "success" && in_stock {
        Ok(json!({ "final_status": "Order Dispatched" }))
    } else {
        Err("Conditions not met for dispatch".to_string())
    }
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // ---------------------------------------------------------
    // STEP 1: Create the Nodes
    // ---------------------------------------------------------
    let node_start =
        Node::new_native_node("receive_order".to_string(), None, Arc::new(receive_order));
    let node_pay = Node::new_native_node(
        "process_payment".to_string(),
        None,
        Arc::new(process_payment),
    );
    let node_inv = Node::new_native_node(
        "check_inventory".to_string(),
        None,
        Arc::new(check_inventory),
    );
    let node_end =
        Node::new_native_node("dispatch_order".to_string(), None, Arc::new(dispatch_order));

    // ---------------------------------------------------------
    // STEP 2: Wire the Edges
    // ---------------------------------------------------------
    // receive_order points to BOTH process_payment and check_inventory
    let edge1 = Edge::new(&node_start, &node_pay, None);
    let edge2 = Edge::new(&node_start, &node_inv, None);

    // BOTH process_payment and check_inventory point to dispatch_order
    let edge3 = Edge::new(&node_pay, &node_end, None);
    let edge4 = Edge::new(&node_inv, &node_end, None);

    // ---------------------------------------------------------
    // STEP 3: Build & Execute
    // ---------------------------------------------------------
    let workflow = Workflow::new(
        vec![node_start, node_pay, node_inv, node_end],
        vec![edge1, edge2, edge3, edge4],
        "Ecommerce".to_string(),
        "Process Order".to_string(),
    );

    let mut execution_context = ExecutionContext::new(Arc::new(workflow));
    execution_context.run_workflow().await;
}
