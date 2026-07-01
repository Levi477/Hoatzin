use serde_json::Value;

pub struct NodeOutput {
    // Extarct output to be passed down to further nodes
    pub payload: Result<Value, String>,
    // Extract route information from '__route__' variable in JSON output of the node
    pub route: Option<Vec<String>>,
}
