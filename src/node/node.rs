use crate::node::node_type::{NodeType, ScriptType};
use pyo3::{
    Python,
    types::{PyAnyMethods, PyDict, PyDictMethods},
};
use serde_json::Value;
use std::{ffi::CString, path::PathBuf, str::FromStr, sync::Arc};
use tokio::fs;
use uuid::Uuid;

pub type NativeFunction = Arc<dyn Fn(Value) -> Result<Value, String> + Sync + Send>;

#[derive(Clone)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub node_type: NodeType,
}

impl Node {
    fn new(name: String, description: Option<String>, node_type: NodeType) -> Self {
        Self {
            id: Uuid::new_v4().simple().to_string(),
            name,
            description,
            node_type,
        }
    }

    pub fn new_native_node(
        name: String,
        description: Option<String>,
        native_script: NativeFunction,
    ) -> Self {
        Self::new(
            name,
            description,
            NodeType::Script(ScriptType::Native(native_script)),
        )
    }

    pub fn new_python_node(name: String, description: Option<String>, script_path: &str) -> Self {
        Self::new(
            name,
            description,
            NodeType::Script(ScriptType::Python(PathBuf::from_str(script_path).unwrap())),
        )
    }

    pub fn new_javascript_node(
        name: String,
        description: Option<String>,
        script_path: &str,
    ) -> Self {
        Self::new(
            name,
            description,
            NodeType::Script(ScriptType::JavaScript(
                PathBuf::from_str(script_path).unwrap(),
            )),
        )
    }

    pub async fn execute(&self, input: Value) -> Result<Value, String> {
        match &self.node_type {
            // run execute function for NodeType Script
            NodeType::Script(lang) => {
                match lang {
                    ScriptType::Native(func) => func(input),
                    ScriptType::Python(script_path) => {
                        let py_script_path = script_path;
                        let user_script = fs::read_to_string(py_script_path).await.unwrap();

                        let input_json = serde_json::to_string(&input).unwrap();

                        Python::attach(|py| -> Result<Value, String> {
                            // make a space for new dictionary in python memory to add input json to it
                            let locals = PyDict::new(py);
                            // add input json to the variable rust_input_json in python
                            locals
                                .set_item("rust_input_json", input_json)
                                .map_err(|e| e.to_string())?;
                            //wrapper code to be passed to python
                            let code = format!(
                                r#"
import asyncio
import inspect
import json

# insert user script main function
{}

# prepare input
input = json.loads(rust_input_json)

try:
    if inspect.iscoroutinefunction(main):
        result = asyncio.run(main(input))
    else:
        result = main(input)
    rust_output_json = json.dumps(result)
except Exception as e:
    rust_output_json = json.dumps({{"error": str(e)}})
"#,
                                user_script
                            );

                            // execute python script and get output
                            py.run(
                                &CString::from_str(&code).unwrap(),
                                Some(&locals),
                                Some(&locals),
                            )
                            .unwrap();

                            // extract output string
                            let output_str: String = locals
                                .get_item("rust_output_json")
                                .unwrap()
                                .unwrap()
                                .extract()
                                .unwrap();

                            let output_value: Value = serde_json::from_str(&output_str).unwrap();

                            return Ok(output_value);
                        })
                    }
                    _ => Err("Script Type Not Supported Yet !".to_string()),
                }
            }
        }
    }
}
