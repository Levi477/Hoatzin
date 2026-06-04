use std::path::PathBuf;

use crate::node::node::NativeFunction;

#[derive(Clone)]
pub enum NodeType {
    Script(ScriptType),
    Conditional(ScriptType),
}

// define all possible script type
// where native stands for Rust functions
// and encapsulate needed parameters for different script types
// NativeFunction -> signature for rust function
// String in JS and Py represent script_path
#[derive(Clone)]
pub enum ScriptType {
    Native(NativeFunction),
    JavaScript(PathBuf),
    Python(PathBuf),
    BuiltIn,
}
