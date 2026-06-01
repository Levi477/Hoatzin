use crate::node::script_type::ScriptType;
use pyo3::{
    Python,
    types::{PyAnyMethods, PyDict, PyDictMethods},
};
use serde_json::{Value, json};
use std::{ffi::CString, str::FromStr, sync::Arc};
use tokio::fs;
use uuid::Uuid;

pub type NativeFunction = Arc<dyn Fn(Value) -> Result<Value, String> + Sync + Send>;

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

    pub async fn execute(&self, input: Value) -> Result<Value, String> {
        match self.script_type {
            ScriptType::Native => {
                if let Some(func) = &self.native_script {
                    func(input)
                } else {
                    Err("Function Not Provided For Native Type !".to_string())
                }
            }
            ScriptType::Python => {
                let py_script_path = self.script_path.as_ref().unwrap();
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
                    );

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
