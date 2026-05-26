use serde_json::Value;

pub struct NodeOutput {
    pub payload: Result<Value, String>,
}
