use hoatzin::{
    edge::edge::Edge, execution::context::ExecutionContext, node::node::Node,
    workflow::workflow::Workflow,
};
use serde_json::{Value, json};
use std::sync::Arc;

/*
 =========================================================================
  FINANCIAL TRANSACTION DAG
 =========================================================================

                            [ 1. init_transfer ]
                              /       |        \
                             /        |         \
                            v         v          v
                 [ 2. check_kyc ] [ 3. ml_fraud ] [ 4. lock_funds ]
                            \         |          /
                             \        |         /
                              v       v        v
                         [ 5. consensus_evaluator ]
                                /           \
                               v             v
                     [ 6. commit_ledger ]  [ 7. alert_compliance ]
                                \            /
                                 v          v
                            [ 8. finalize_audit ]
*/

fn init_transfer(_input: Value) -> Result<Value, String> {
    println!("[INFO] init_transfer: Starting transaction T-8992-B");
    Ok(json!({
        "tx_id": "T-8992-B",
        "sender_id": "USR_992",
        "receiver_id": "USR_401",
        "amount": 25000.00,
        "currency": "USD"
    }))
}

fn check_kyc(input: Value) -> Result<Value, String> {
    let sender = input["init_transfer"]["sender_id"].as_str().unwrap_or("");
    println!("[INFO] check_kyc: Verifying identity for {}", sender);

    // Simulate database lookup
    Ok(json!({
        "kyc_cleared": true,
        "risk_tier": "low"
    }))
}

fn ml_fraud(input: Value) -> Result<Value, String> {
    let amount = input["init_transfer"]["amount"].as_f64().unwrap_or(0.0);
    println!(
        "[INFO] ml_fraud: Running heuristic model on amount {:.2}",
        amount
    );

    Ok(json!({
        "fraud_score": 0.12,
        "flagged": false
    }))
}

fn lock_funds(input: Value) -> Result<Value, String> {
    let amount = input["init_transfer"]["amount"].as_f64().unwrap_or(0.0);
    println!("[INFO] lock_funds: Placing hold on {:.2} USD", amount);

    Ok(json!({
        "funds_locked": true,
        "hold_id": "HLD_110"
    }))
}

fn consensus_evaluator(input: Value) -> Result<Value, String> {
    println!("[INFO] consensus_evaluator: Aggregating risk signals");

    let kyc_ok = input["check_kyc"]["kyc_cleared"].as_bool().unwrap_or(false);
    let fraud_score = input["ml_fraud"]["fraud_score"].as_f64().unwrap_or(1.0);
    let locked = input["lock_funds"]["funds_locked"]
        .as_bool()
        .unwrap_or(false);

    if !kyc_ok || fraud_score > 0.85 || !locked {
        println!("[WARN] consensus_evaluator: Transaction flagged for review");
        return Ok(json!({ "approved": false, "reason": "High risk or fund lock failure" }));
    }

    println!("[INFO] consensus_evaluator: Transaction approved");
    Ok(json!({ "approved": true }))
}

fn commit_ledger(input: Value) -> Result<Value, String> {
    let approved = input["consensus_evaluator"]["approved"]
        .as_bool()
        .unwrap_or(false);
    if !approved {
        println!("[DEBUG] commit_ledger: Skipped due to rejection");
        return Ok(json!({ "status": "skipped" }));
    }

    let tx_id = input["init_transfer"]["tx_id"]
        .as_str()
        .unwrap_or("UNKNOWN");
    println!("[INFO] commit_ledger: Committing {} to PostgreSQL", tx_id);
    Ok(json!({ "status": "committed", "db_offset": 849201 }))
}

fn alert_compliance(input: Value) -> Result<Value, String> {
    let approved = input["consensus_evaluator"]["approved"]
        .as_bool()
        .unwrap_or(true);
    if approved {
        println!("[DEBUG] alert_compliance: Skipped due to approval");
        return Ok(json!({ "status": "skipped" }));
    }

    println!("[WARN] alert_compliance: Pushing alert to Kafka topic 'aml.alerts'");
    Ok(json!({ "status": "alert_sent" }))
}

fn finalize_audit(input: Value) -> Result<Value, String> {
    println!("[INFO] finalize_audit: Generating compliance receipt");

    let ledger_status = input["commit_ledger"]["status"].as_str().unwrap_or("none");
    let alert_status = input["alert_compliance"]["status"]
        .as_str()
        .unwrap_or("none");

    Ok(json!({
        "final_state": if ledger_status == "committed" { "SETTLED" } else { "REJECTED" },
        "audit_timestamp": 1718392019,
        "ledger_ack": ledger_status,
        "alert_ack": alert_status
    }))
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let n1 = Node::new_native_node("init_transfer".into(), None, Arc::new(init_transfer));
    let n2 = Node::new_native_node("check_kyc".into(), None, Arc::new(check_kyc));
    let n3 = Node::new_native_node("ml_fraud".into(), None, Arc::new(ml_fraud));
    let n4 = Node::new_native_node("lock_funds".into(), None, Arc::new(lock_funds));
    let n5 = Node::new_native_node(
        "consensus_evaluator".into(),
        None,
        Arc::new(consensus_evaluator),
    );
    let n6 = Node::new_native_node("commit_ledger".into(), None, Arc::new(commit_ledger));
    let n7 = Node::new_native_node("alert_compliance".into(), None, Arc::new(alert_compliance));
    let n8 = Node::new_native_node("finalize_audit".into(), None, Arc::new(finalize_audit));

    let edges = vec![
        Edge::new(&n1, &n2, None),
        Edge::new(&n1, &n3, None),
        Edge::new(&n1, &n4, None),
        Edge::new(&n2, &n5, None),
        Edge::new(&n3, &n5, None),
        Edge::new(&n4, &n5, None),
        Edge::new(&n5, &n6, None),
        Edge::new(&n5, &n7, None),
        Edge::new(&n6, &n8, None),
        Edge::new(&n7, &n8, None),
    ];

    let mut ctx = ExecutionContext::new(Arc::new(Workflow::new(
        vec![n1, n2, n3, n4, n5, n6, n7, n8],
        edges,
        "Microservice".to_string(),
        "Final Transaction Evaluation".to_string(),
    )));

    ctx.run_workflow().await;
}
