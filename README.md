# Hoatzin

A workflow execution engine written in Rust. You define a graph of tasks,nodes and edges and Hoatzin figures out the execution order, runs independent tasks in parallel, and passes each node's output downstream as JSON.

The name is pronounced *hwaat-zin*.

---

## What it does

You give Hoatzin a directed acyclic graph (DAG). Each node is either a Rust closure or a Python script. Edges define dependencies between them. When you run it, nodes with no pending dependencies are dispatched immediately onto Tokio worker threads. When a node finishes, its output is made available to all downstream nodes that were waiting on it.

The whole thing is built on `tokio` with a multi-threaded scheduler, so independent branches of your workflow run at the same time without you having to think about it.

---

## What works today

- **Rust native nodes** — wrap any `Fn(Value) -> Result<Value, String>` closure as a node
- **Python nodes** — point a node at a `.py` file and Hoatzin runs the `main(input)` function inside it (sync and async both work)
- **Fan-out and fan-in** — a node can feed multiple downstream nodes, and a node can wait for multiple upstream nodes before it runs
- **JSON context passing** — each node receives a JSON object keyed by the names of its immediate predecessors, containing their outputs
- **TOML workflow definitions** — describe your entire graph in a `.toml` file without touching Rust code
- **Parallel workflow loading** — point the CLI at a directory and it picks up every `.toml` file and runs them all concurrently
- **Free-threaded Python** — uses PyO3 with Python 3.14t to run multiple Python nodes simultaneously without GIL contention

---

## What's on the roadmap

- **JavaScript / TypeScript nodes** — the `ScriptType::JavaScript` variant exists in the code but isn't implemented yet; the plan is to embed V8 via `deno_core`
- **Daemon mode** — right now Hoatzin is a run-once process; the goal is a persistent background daemon with an event loop that can accept webhook and cron triggers
- **Visual interface** — a web frontend for building workflows with drag-and-drop, watching execution in real-time, and inspecting the JSON state at each step

---

## Quick start

### Option A: TOML workflow (recommended)

Write a `workflow.toml` file:

```toml
[workflow]
name = "E-Commerce Pipeline"
description = "Workflow Description here"

[[nodes]]
name = "receive_order"
script_type = "Python"
script_path = "scripts/receive_order.py"
description = "Node Description here"

[[nodes]]
name = "process_payment"
script_type = "Python"
script_path = "scripts/process_payment.py"

[[nodes]]
name = "check_inventory"
script_type = "Python"
script_path = "scripts/check_inventory.py"

[[nodes]]
name = "dispatch_order"
script_type = "Python"
script_path = "scripts/dispatch_order.py"

[[edges]]
from_node = "receive_order"
to_node = "process_payment"

[[edges]]
from_node = "receive_order"
to_node = "check_inventory"

[[edges]]
from_node = "process_payment"
to_node = "dispatch_order"

[[edges]]
from_node = "check_inventory"
to_node = "dispatch_order"
```

Run it:

```bash
hoatzin --workflows ./path/to/toml/dir
```

### Option B: Native Rust API

Wire up closures directly:

```rust
use hoatzin::{
    edge::edge::Edge, execution::context::ExecutionContext,
    node::node::Node, workflow::workflow::Workflow,
};
use serde_json::{Value, json};
use std::sync::Arc;

fn receive_order(_input: Value) -> Result<Value, String> {
    Ok(json!({ "order_id": "ORD-7781", "item": "Keyboard", "amount": 150.00 }))
}

fn process_payment(input: Value) -> Result<Value, String> {
    let amount = input["receive_order"]["amount"].as_f64().unwrap_or(0.0);
    println!("Charging ${:.2}", amount);
    Ok(json!({ "payment_status": "success" }))
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let n1 = Node::new_native_node("receive_order".into(), Some("Node Description".to_string()),Arc::new(receive_order));
    let n2 = Node::new_native_node("process_payment".into(), None,Arc::new(process_payment));

    let edges = vec![Edge::new(&n1, &n2, None)];
    let workflow = Workflow::new(vec![n1, n2], edges,"Enter your description here".to_String());

    let mut ctx = ExecutionContext::new(Arc::new(workflow));
    ctx.run_workflow().await;
}
```

---

## How context passing works

Each node receives a JSON object whose keys are the **names** of its direct predecessors. So if `dispatch_order` sits downstream of both `process_payment` and `check_inventory`, it gets:

```json
{
  "process_payment": {
    "payment_status": "success",
    "transaction_id": "tx_99210"
  },
  "check_inventory": {
    "stock_available": true,
    "warehouse_zone": "Zone-A"
  }
}
```

Trigger nodes (no predecessors) receive an empty object `{}`.

In Python, you access it like a regular dict:

```python
def main(input):
    order = input.get("receive_order", {})
    amount = order.get("amount", 0.0)
    print(f"Processing payment of ${amount}")
    return {"payment_status": "success"}
```

---

## Examples

### E-Commerce order pipeline

A classic fan-out / fan-in pattern. The order comes in, payment and inventory checks run in parallel, then dispatch waits for both.

```
              [ receive_order ]
                /             \
               /               \
              v                 v
  [ process_payment ]   [ check_inventory ]
              \                 /
               \               /
                v             v
            [ dispatch_order ]
```

The full working example is in [`examples/ecommerce.rs`](examples/ecommerce.rs).

---

### Financial transaction pipeline

A more involved graph — three independent checks feed a consensus node, which then branches again into ledger commit and compliance alert, before a final audit step collects both results.

```
              [ init_transfer ]
            /        |         \
           v         v          v
    [ check_kyc ] [ ml_fraud ] [ lock_funds ]
           \         |         /
            v        v        v
        [ consensus_evaluator ]
              /             \
             v               v
    [ commit_ledger ]  [ alert_compliance ]
             \               /
              v             v
           [ finalize_audit ]
```

`check_kyc`, `ml_fraud`, and `lock_funds` all run at the same time. `consensus_evaluator` waits for all three, reads their outputs, and decides whether the transaction is approved. Depending on that decision, either `commit_ledger` or `alert_compliance` does the real work (the other skips itself gracefully).

The full working example is in [`examples/microservice.rs`](examples/microservice.rs).

---

### ML dataset pipeline (TOML)

A linear chain — acquire data, preprocess it, train. Defined entirely in [`templates/ml_pipeline.toml`](templates/ml_pipeline.toml).

```
[ acquire_dataset ] → [ preprocess_data ] → [ train_model ]
```

---

### Video upscaler pipeline (TOML)

Audio and video are extracted in parallel, frames are upscaled, then everything is merged back together. Defined in [`templates/video_upscaler.toml`](templates/video_upscaler.toml).

```
[ extract_audio ]          [ extract_frames ]
       \                          |
        \                         v
         \               [ upscale_frames ]
          \                       /
           v                     v
              [ merge_output ]
```

---

## Writing Python nodes

Your script needs a `main` function that accepts one argument (the input dict) and returns something JSON-serializable. Both sync and async are fine:

```python
# sync
def main(input):
    data = input.get("some_upstream_node", {})
    return {"result": data["value"] * 2}

# async
async def main(input):
    import asyncio
    await asyncio.sleep(0.1)
    return {"done": True}
```

If your script throws an exception, Hoatzin catches it and passes `{"error": "..."}` as the node's output to any downstream nodes.

---

## Contributing

Issues and PRs are open. If something is broken or a feature you want isn't there yet, open an issue.
