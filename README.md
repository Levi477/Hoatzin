# Hoatzin

A workflow execution engine written in Rust. You define a graph of tasks, nodes, and edges, and Hoatzin figures out the execution order, runs independent tasks in parallel, and passes each node's output downstream as JSON.

The name is pronounced *hwaat-zin*.

---

## What it does

You give Hoatzin a directed acyclic graph (DAG). Each node is either a Rust closure or a Python script. Edges define dependencies between them. When you run it, nodes with no pending dependencies are dispatched immediately onto Tokio worker threads. When a node finishes, its output is made available to all downstream nodes that were waiting on it.

The whole thing is built on `tokio` with a multi-threaded scheduler, so independent branches of your workflow run at the same time without you having to think about it.

---

## What works today

* **Rust native nodes** — wrap any `Fn(Value) -> Result<Value, String>` closure as a node
* **Python nodes** — point a node at a `.py` file and Hoatzin runs the `main(input)` function inside it (sync and async both work)
* **Conditional routing (Switch cases) [NEW]** — nodes can output a `__route__` array to dynamically select which labeled downstream branches execute. Unselected branches are automatically skipped, and that skipped status propagates downstream to dead-end paths. Unlabeled edges act as defaults and execute unconditionally.
* **Fan-out and fan-in** — a node can feed multiple downstream nodes, and a node can wait for multiple upstream nodes before it runs
* **JSON context passing** — each node receives a JSON object keyed by the names of its immediate predecessors, containing their outputs
* **TOML workflow definitions** — describe your entire graph in a `.toml` file without touching Rust code
* **Parallel workflow loading** — point the CLI at a directory and it picks up every `.toml` file and runs them all concurrently
* **Free-threaded Python** — uses PyO3 with Python 3.14t to run multiple Python nodes simultaneously without GIL contention

---

## What's on the roadmap

* **JavaScript / TypeScript nodes** — the `ScriptType::JavaScript` variant exists in the code but isn't implemented yet; the plan is to embed V8 via `deno_core`
* **Daemon mode** — right now Hoatzin is a run-once process; the goal is a persistent background daemon with an event loop that can accept webhook and cron triggers
* **Visual interface** — a web frontend for building workflows with drag-and-drop, watching execution in real-time, and inspecting the JSON state at each step

---

## Quick start

### Option A: Native Rust API (With Conditional Routing)

This example demonstrates how to wire up closures directly and use the switch-case routing feature. The router node outputs a `__route__` array to dynamically select whether the standard or high-value branch executes.

Here is the graph we are building:

```text
                      [ router ]
        (Evaluates amount, outputs __route__ = ["HighValue"])
                     /                  \
       "Standard"   /                    \   "HighValue"
                   /                      \
                  v                        v
        [ std_processor ]          [ high_processor ]
            (Skipped)                  (Executes)

```

And here is the code to run it:

```rust
use hoatzin::{
    edge::edge::Edge, execution::context::ExecutionContext,
    node::node::Node, workflow::workflow::Workflow,
};
use serde_json::{Value, json};
use std::sync::Arc;

// 1. Router Node
fn route_order(_input: Value) -> Result<Value, String> {
    let amount = 1500.00; // Simulated input data
    let route = if amount > 1000.0 { "HighValue" } else { "Standard" };

    // The __route__ key instructs the engine which labeled edges to follow
    Ok(json!({
        "status": "routed",
        "__route__": [route]
    }))
}

// 2. Branch A
fn process_standard(_input: Value) -> Result<Value, String> {
    println!("Processing standard order.");
    Ok(json!({ "branch": "standard_processed" }))
}

// 3. Branch B
fn process_high_value(_input: Value) -> Result<Value, String> {
    println!("Processing high-value order. Sending to priority queue.");
    Ok(json!({ "branch": "high_value_processed" }))
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let n_router = Node::new_native_node("router".into(), None, Arc::new(route_order));
    let n_std = Node::new_native_node("std_processor".into(), None, Arc::new(process_standard));
    let n_high = Node::new_native_node("high_processor".into(), None, Arc::new(process_high_value));

    // Connect edges with specific labels to enable switch-case routing
    let e1 = Edge::new(&n_router, &n_std, Some("Standard".into()));
    let e2 = Edge::new(&n_router, &n_high, Some("HighValue".into()));

    let workflow = Workflow::new(
        vec![n_router.clone(), n_std.clone(), n_high.clone()],
        vec![e1, e2],
        "RoutingWorkflow".into(),
        "Example of conditional execution".into(),
    );

    let mut ctx = ExecutionContext::new(Arc::new(workflow));
    ctx.run_workflow().await;

    // Because amount > 1000, n_high will succeed and n_std will be marked as Skipped.
}

```

### Option B: TOML workflow

You can also describe your entire graph in a `.toml` file. Notice how labels can be added to edges to support switch-case routing.

```toml
[workflow]
name = "Routing Pipeline"
description = "Basic routing structure"

[[nodes]]
name = "router"
script_type = "Python"
script_path = "scripts/route_order.py"

[[nodes]]
name = "std_processor"
script_type = "Python"
script_path = "scripts/process_standard.py"

[[nodes]]
name = "high_processor"
script_type = "Python"
script_path = "scripts/process_high_value.py"

[[edges]]
from_node = "router"
to_node = "std_processor"
label = "Standard"

[[edges]]
from_node = "router"
to_node = "high_processor"
label = "HighValue"

```

Run it:

```bash
hoatzin --workflows ./path/to/toml/dir

```

---

## Writing Python Nodes

Your script needs a `main` function that accepts one argument (the input dict) and returns something JSON-serializable. Both sync and async are fine.

Each node receives a JSON object whose keys are the **names** of its direct predecessors. Trigger nodes (no predecessors) receive an empty object `{}`.

You can also return a `__route__` array from a Python node to act as a switch case for downstream execution, exactly like in the Rust example.

```python
# sync example with conditional routing
def main(input):
    # Read output from an upstream node
    order = input.get("receive_order", {})
    amount = order.get("amount", 0.0)
    
    print(f"Evaluating order of ${amount}")
    
    # Decide which downstream branch should execute
    selected_path = "ReviewRequired" if amount > 10000 else "AutoApprove"

    return {
        "evaluated_amount": amount,
        "__route__": [selected_path]
    }

# async example
async def main(input):
    import asyncio
    await asyncio.sleep(0.1)
    return {"done": True}

```

If your script throws an exception, Hoatzin catches it, marks the node as failed, and prevents downstream execution for dependent nodes.

---

## Contributing

Issues and PRs are open. If something is broken or a feature you want isn't there yet, open an issue.
