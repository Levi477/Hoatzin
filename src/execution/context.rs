use crate::{
    execution::{helper::execute_node, node_output::NodeOutput, node_status::NodeStatus},
    workflow::workflow::Workflow,
};
use serde_json::{Map, Value, json};
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::task::JoinSet;
use uuid::Uuid;

pub struct ExecutionContext {
    // Every New Workflow Trigger will result in new execution context and hence execution id
    pub id: String,
    // Unique ID of Workflow
    pub workflow_id: String,
    // Workflow read only thread safe pointer
    pub workflow: Arc<Workflow>,
    // Store Hashmap of Node ID to Node Output
    pub node_outputs: HashMap<String, NodeOutput>,
    // Store Hashmap of Node ID to Node Status
    pub node_status: HashMap<String, NodeStatus>,
}

impl ExecutionContext {
    pub fn new(workflow: Arc<Workflow>) -> Self {
        Self {
            id: Uuid::new_v4().simple().to_string(),
            workflow_id: workflow.id.clone(),
            workflow,
            node_outputs: HashMap::new(),
            node_status: HashMap::new(),
        }
    }

    fn make_input_context(&self, front_node_id: &str) -> Value {
        // make input context map (later convert to context json object) using reverse adjacency list
        let mut ctx_map = Map::new();
        // for trigger node put input context as empty vector
        // for othe nodes when [node2,node3,node4....] -> node1
        // first concatinate all the outputs of node2,3,4...
        // as {"nodei":{json output},...}
        // and pass it as the input context to the node1 for execution

        if let Some(incoming_edges) = self.workflow.reverse_adjacency_list.get(front_node_id) {
            for edge_id in incoming_edges {
                // Hop through the edge to find the previous Node ID
                let edge = self.workflow.edges.get(edge_id).unwrap();
                let prev_node_id = &edge.from_node_id;
                let prev_node_name = self.workflow.nodes.get(prev_node_id).unwrap().name.clone();

                match &self.node_outputs.get(prev_node_id).unwrap().payload {
                    Ok(op) => {
                        ctx_map.insert(prev_node_name, op.clone());
                    }
                    Err(er) => {
                        ctx_map.insert(prev_node_name, json!({"error": er}));
                    }
                }
            }
        } // make json object from input context map
        Value::Object(ctx_map)
    }

    fn make_indegree_vec(&self) -> HashMap<String, usize> {
        let mut indegree: HashMap<String, usize> = HashMap::new();

        for node_id in self.workflow.nodes.keys() {
            indegree.insert(node_id.clone(), 0);
        }
        for (to_node_id, incoming_edges_vector) in &self.workflow.reverse_adjacency_list {
            indegree.insert(to_node_id.clone(), incoming_edges_vector.len());
        }
        indegree
    }

    fn save_node_output(&mut self, front_node_id: &String, node_op: Result<Value, String>) {
        self.node_outputs.insert(
            front_node_id.clone(),
            NodeOutput {
                payload: node_op.clone(),
            },
        );
    }

    fn init_node_status(&mut self) {
        for node_id in self.workflow.nodes.keys() {
            self.node_status
                .insert(node_id.clone(), NodeStatus::Pending);
        }
    }

    pub async fn run_workflow(&mut self) {
        // run kahn's algorithm for execution of nodes
        // use adjacency list to track incoming edges count
        // store node id to indegree
        let mut indegree = self.make_indegree_vec();
        // initialize all node status
        self.init_node_status();
        // global queue for nodes to be executed
        let mut ready_queue: VecDeque<String> = VecDeque::new();

        // find starting node with 0 indegree
        for (node_id, indegree_count) in &indegree {
            if (*indegree_count) == 0 {
                ready_queue.push_back(node_id.clone());
            }
        }

        // init joinset for parallel execution
        let mut join_set = JoinSet::new();

        while !ready_queue.is_empty() || !join_set.is_empty() {
            // keep executing nodes from queue
            while let Some(front_node_id) = ready_queue.pop_front() {
                //prepare input context and node object to be passed
                let ctx = self.make_input_context(&front_node_id);
                let node = self.workflow.nodes.get(&front_node_id).unwrap();
                let thread_node_id = front_node_id.clone();

                //keep node status to running
                self.node_status
                    .insert(front_node_id.clone(), NodeStatus::Running);
                // start running the node in a new thread
                join_set.spawn(execute_node(node.clone(), ctx, thread_node_id));
            }

            // wait for output for executed nodes
            if let Some(result) = join_set.join_next().await {
                let (node_id, node_op) = result.unwrap();

                // update status according to node_op
                if node_op.is_ok() {
                    self.node_status
                        .insert(node_id.clone(), NodeStatus::Success);
                } else {
                    self.node_status.insert(node_id.clone(), NodeStatus::Failed);
                }

                // add node output to the history
                self.save_node_output(&node_id, node_op);
                // reduce indegrees of connected nodes
                if let Some(outgoing_edges) = self.workflow.adjacency_list.get(&node_id) {
                    for edge_id in outgoing_edges {
                        // Hop through the edge to find the target Node ID
                        let edge = self.workflow.edges.get(edge_id).unwrap();
                        let target_node_id = &edge.to_node_id;
                        if let Some(count) = indegree.get_mut(target_node_id) {
                            *count -= 1;
                            if *count == 0 {
                                ready_queue.push_back(target_node_id.clone());
                            }
                        }
                    }
                }
            }
        }
    }
}
