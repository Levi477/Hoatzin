# Hoatzin

**A high-performance, asynchronous workflow execution engine written in Rust.**

Hoatzin (pronounced *hwaat-zin*) is a lightweight but incredibly fast orchestrator designed to execute complex workflows. Powered by `tokio`, it automatically resolves task dependencies, dynamically passes JSON context between steps, and scales parallel execution seamlessly across CPU cores without lock contention.

---

## Features

* **True Asynchronous Parallelism:** Tasks with satisfied prerequisites are instantly spawned onto Tokio background worker threads.
* **Dynamic Context Passing:** Nodes seamlessly receive `serde_json::Value` payloads aggregated from all their immediate upstream dependencies.

---

## Quick Start & Example

Here is a practical example of how Hoatzin handles a staggered **E-Commerce Order Pipeline**. 

This workflow demonstrates **Fan-Out** (running payment and inventory checks simultaneously) and **Fan-In** (waiting for both to finish before dispatching).

### The Workflow 
```text
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
```

### The Code

```rust
use hoatzin::{
    edge::edge::Edge, execution::context::ExecutionContext, node::node::Node,
    workflow::workflow::Workflow,
};
use serde_json::{Value, json};
use std::sync::Arc;

// 1. Trigger Node
fn receive_order(_input: Value) -> Result<Value, String> {
    println!("[INFO] receive_order: Order received");
    Ok(json!({ "order_id": "ORD-7781", "item": "Keyboard", "amount": 150.00 }))
}

// 2. Parallel Task A
fn process_payment(input: Value) -> Result<Value, String> {
    let amount = input["receive_order"]["amount"].as_f64().unwrap_or(0.0);
    println!("[INFO] process_payment: Processing payment of ${:.2}", amount);
    Ok(json!({ "payment_status": "success" }))
}

// 3. Parallel Task B
fn check_inventory(input: Value) -> Result<Value, String> {
    let item = input["receive_order"]["item"].as_str().unwrap_or("Unknown");
    println!("[INFO] check_inventory: Checking inventory for {}", item);
    Ok(json!({ "stock_available": true }))
}

// 4. Fan-In Resolution Task
fn dispatch_order(input: Value) -> Result<Value, String> {
    let payment_status = input["process_payment"]["payment_status"].as_str().unwrap_or("failed");
    let in_stock = input["check_inventory"]["stock_available"].as_bool().unwrap_or(false);

    if payment_status == "success" && in_stock {
        println!("[INFO] dispatch_order: Conditions met, order dispatched");
        Ok(json!({ "final_status": "Order Dispatched!" }))
    } else {
        println!("[ERROR] dispatch_order: Cannot dispatch");
        Err("Payment failed or out of stock".into())
    }
}

#[tokio::main]
async fn main() {
    // 1. Create Nodes
    let n1 = Node::new_native_node("receive_order".into(), Arc::new(receive_order));
    let n2 = Node::new_native_node("process_payment".into(), Arc::new(process_payment));
    let n3 = Node::new_native_node("check_inventory".into(), Arc::new(check_inventory));
    let n4 = Node::new_native_node("dispatch_order".into(), Arc::new(dispatch_order));

    // 2. Wire Edges (Define Dependencies)
    let edges = vec![
        Edge::new(&n1, &n2, None), Edge::new(&n1, &n3, None), // n1 triggers n2 and n3
        Edge::new(&n2, &n4, None), Edge::new(&n3, &n4, None), // n2 and n3 must finish before n4
    ];

    // 3. Execute
    let workflow = Workflow::new(vec![n1, n2, n3, n4], edges);
    let mut ctx = ExecutionContext::new(Arc::new(workflow));
    
    ctx.run_workflow().await;
}

```

---

## Future Roadmap

Hoatzin is actively evolving from an embedded library into a full-fledged orchestration platform.

### 1. Declarative Workflow TOML

Writing workflows in pure Rust is fast, but recompiling for every structural change is slow. We are implementing a TOML parser to allow for fully declarative workflow definitions.

* Hot-reloading of workflow topologies.
* Standardized schema for nodes, edges, and retry logic.

### 2. The Engine Daemon & Workflow Manager

Transitioning from a run-once script model to a continuous background daemon.

* Multi-Producer Single-Consumer (`mpsc`) event loop.
* Webhook, Cron, and manual trigger event ingestion.
* Concurrent execution of thousands of distinct workflow instances.

### 3. Polyglot Node Support (Python & JavaScript)

To support data science and frontend tooling, Hoatzin will soon support executing non-Rust scripts natively within the workflow.

* **Python:** Integration via `PyO3` for machine learning and Pandas/NumPy tasks.
* **JavaScript:** V8 isolation (via `deno_core` or `v8`) for lightweight API mapping and data transformation.

### 4. Fully Featured Visual Interface

A modern web frontend to visualize, build, and monitor workflows in real-time.

* Drag-and-drop node builder.
* Live telemetry and execution visualization.
* Context inspection (viewing JSON state at any step in the workflow).

---

## Contributing

Contributions, issues, and feature requests are welcome!
Feel free to check the issues page if you want to contribute.
