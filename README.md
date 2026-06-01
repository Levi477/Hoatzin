# Hoatzin

**A high-performance, asynchronous workflow execution engine written in Rust.**

Hoatzin (pronounced *hwaat-zin*) is a lightweight, fast orchestrator for executing complex workflows. Powered by `tokio`, it automatically resolves task dependencies, passes JSON context dynamically between steps, and scales parallel execution across CPU cores.

---

## Features

* **True Asynchronous Parallelism:** Tasks with satisfied prerequisites are spawned directly onto Tokio worker threads.
* **Dynamic Context Passing:** Nodes receive `serde_json::Value` payloads aggregated from all immediate upstream dependencies.
* **Declarative TOML Workflows:** Build and modify workflow topologies without recompiling the engine.
* **Free-Threaded Python Support:** Run Python nodes in true parallel. By leveraging PyO3 and Python 3.14t (free-threaded), Hoatzin bypasses the GIL, allowing multiple Python scripts to execute simultaneously in the same memory space.

---

## Quick Start & Examples

Hoatzin supports both hardcoded Rust execution and declarative TOML files. Here is an example of an **E-Commerce Order Pipeline** demonstrating Fan-Out (running payment and inventory checks simultaneously) and Fan-In (waiting for both to finish).

### The Workflow Graph

```text
                           [ Node 1: receive_order ]
                               /               \
                     edge1    /                 \   edge2
                             /                   \
                            v                     v
            [ Node 2: process_payment ]      [ Node 3: check_inventory ]
                            \                     /
                     edge3   \                   /   edge4
                              \                 /
                               v               v
                         [ Node 4: dispatch_order ]


```

### Option A: Declarative TOML (Recommended)

You can define the entire workflow in a `workflow.toml` file.

```toml
[workflow]
name = "E-Commerce Pipeline"

# 1. Trigger Node
[[nodes]]
name = "receive_order"
script_type = "Python"
script_path = "scripts/receive_order.py"

# 2. Parallel Tasks
[[nodes]]
name = "process_payment"
script_type = "Python"
script_path = "scripts/process_payment.py"

[[nodes]]
name = "check_inventory"
script_type = "Python"
script_path = "scripts/check_inventory.py"

# 3. Fan-In Task
[[nodes]]
name = "dispatch_order"
script_type = "Python"
script_path = "scripts/dispatch_order.py"

# Define Edges
[[edges]]
from = "receive_order"
to = "process_payment"

[[edges]]
from = "receive_order"
to = "check_inventory"

[[edges]]
from = "process_payment"
to = "dispatch_order"

[[edges]]
from = "check_inventory"
to = "dispatch_order"


```

#### Writing the Python Nodes

Hoatzin injects the workflow state directly into a `main(input)` function in your Python scripts. You can use standard sync functions or `async def`.

**Understanding Node Inputs:**
The `input` payload is a JSON object where the keys are the **names of the immediately preceding nodes**, and the values are the outputs from those nodes.

For example, when the workflow reaches `dispatch_order` (Node 4), it waits for both Node 2 and Node 3 to finish. It then merges their outputs and passes the following JSON into the `dispatch_order` script:

```json
{
  "process_payment": {
    "payment_status": "success"
  },
  "check_inventory": {
    "stock_available": true
  }
}

```

**Example: `scripts/process_payment.py**`

```python
def main(input):
    # Read data from the upstream 'receive_order' node
    order_data = input.get("receive_order", {})
    amount = order_data.get("amount", 0.0)
    
    print(f"[INFO] Processing payment of ${amount}")
    
    return {"payment_status": "success"}


```

### Option B: Native Rust API

You can also wire up native Rust closures directly in code.

```rust
use hoatzin::{
    edge::edge::Edge, execution::context::ExecutionContext, node::node::Node,
    workflow::workflow::Workflow,
};
use serde_json::{Value, json};
use std::sync::Arc;

fn receive_order(_input: Value) -> Result<Value, String> {
    Ok(json!({ "order_id": "ORD-7781", "item": "Keyboard", "amount": 150.00 }))
}

fn process_payment(input: Value) -> Result<Value, String> {
    let amount = input["receive_order"]["amount"].as_f64().unwrap_or(0.0);
    Ok(json!({ "payment_status": "success" }))
}

#[tokio::main]
async fn main() {
    let n1 = Node::new_native_node("receive_order".into(), Arc::new(receive_order));
    let n2 = Node::new_native_node("process_payment".into(), Arc::new(process_payment));
    
    let edges = vec![Edge::new(&n1, &n2, None)];

    let workflow = Workflow::new(vec![n1, n2], edges);
    let mut ctx = ExecutionContext::new(Arc::new(workflow));
    
    ctx.run_workflow().await;
}


```

---

## Future Roadmap

Hoatzin is actively evolving into a full-fledged orchestration platform.

### 1. The Engine Daemon & Workflow Manager

Transitioning from a run-once script model to a continuous background daemon.

* Multi-Producer Single-Consumer (`mpsc`) event loop.
* Webhook, Cron, and manual trigger event ingestion.
* Concurrent execution of thousands of distinct workflow instances.

### 2. JavaScript / TypeScript Support

Expanding polyglot support for frontend tooling and lightweight API mapping using V8 isolation (via `deno_core`).

### 3. Visual Interface

A modern web frontend to visualize, build, and monitor workflows in real-time.

* Drag-and-drop node builder.
* Live telemetry and execution visualization.
* Context inspection (viewing JSON state at any step in the workflow).

---

## Contributing

Contributions, issues, and feature requests are welcome! Check the issues page if you want to get involved.
