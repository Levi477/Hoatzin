use hoatzin::{execution::context::ExecutionContext, template::parser::parse_workflow_from_toml};
use std::{path::PathBuf, str::FromStr, sync::Arc};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    // import workflow from a template
    let workflow = parse_workflow_from_toml(
        PathBuf::from_str("/Users/levi/Desktop/hoatzin/templates/pyexmaple.toml").unwrap(),
    )
    .await
    .unwrap();

    //make new execution context
    let mut ctx = ExecutionContext::new(Arc::new(workflow));

    // run this workflow
    ctx.run_workflow().await;
}
