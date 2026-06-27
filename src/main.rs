use std::sync::Arc;

use clap::Parser;
use tokio::task::JoinSet;
use walkdir::WalkDir;

use crate::{
    args::args::Args, execution::context::ExecutionContext,
    template::parser::parse_workflow_from_toml,
};

mod args;
mod db;
mod edge;
mod execution;
mod node;
mod template;
mod workflow;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();
    let dir = args.workflows;

    // Track all spawned workflows in JoinSet
    let mut join_set = JoinSet::new();

    // walk through all .toml files and spawn up workflows
    for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        // check if the file is .toml format
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
            // if valid .toml then read content
            match parse_workflow_from_toml(path.to_path_buf()).await {
                Ok(workflow) => {
                    println!("Spawning workflow from path : {}", path.to_str().unwrap());
                    join_set.spawn(async move {
                        let mut ctx = ExecutionContext::new(Arc::new(workflow));
                        ctx.run_workflow().await;
                    });
                }
                Err(err) => {
                    println!("Error Occured parsing workflow : {}", err);
                }
            }
        }
    }

    // wait for all workflows to finish
    while let Some(result) = join_set.join_next().await {
        if let Err(e) = result {
            eprintln!("A workflow task panicked: {}", e);
        }
    }
}
