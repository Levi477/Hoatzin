use crate::node::script_type::ScriptType;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

pub type NativeFunction = Arc<dyn Fn(Vec<Value>) -> Result<Vec<Value>, String> + Sync + Send>;

#[derive(Clone)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub script_type: ScriptType,
    pub script_path: Option<String>,
    pub native_script: Option<NativeFunction>,
}

impl Node {
    fn new(
        name: String,
        script_type: ScriptType,
        script_path: Option<String>,
        native_script: Option<NativeFunction>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().simple().to_string(),
            name,
            script_type,
            script_path,
            native_script,
        }
    }

    pub fn new_native_node(name: String, native_script: NativeFunction) -> Self {
        Self::new(name, ScriptType::Native, None, Some(native_script))
    }

    pub fn new_python_node(name: String, script_path: String) -> Self {
        Self::new(name, ScriptType::Python, Some(script_path), None)
    }

    pub fn new_javascript_node(name: String, script_path: String) -> Self {
        Self::new(name, ScriptType::JavaScript, Some(script_path), None)
    }

    pub fn execute(&self, input: Vec<Value>) -> Result<Vec<Value>, String> {
        match self.script_type {
            ScriptType::Native => {
                if let Some(func) = &self.native_script {
                    func(input)
                } else {
                    Err("Function Not Provided For Native Type !".to_string())
                }
            }
            _ => Err("Script Type Not Supported Yet !".to_string()),
        }
    }
}
