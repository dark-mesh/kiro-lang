use crate::grammar::grammar::{KiroType, StructNameVal};

pub fn compile_type(t: &KiroType) -> String {
    match t {
        KiroType::Num => compile_num(),
        KiroType::Str => compile_str(),
        KiroType::Bool => compile_bool(),
        KiroType::Adr => compile_adr(),
        KiroType::Pipe => compile_pipe(),
        KiroType::List(_, inner) => compile_list(inner),
        KiroType::Map(_, k, v) => compile_map(k, v),
        KiroType::Custom(s) => compile_custom(s),
    }
}

pub fn compile_num() -> String {
    "f64".to_string()
}

pub fn compile_str() -> String {
    "String".to_string()
}

pub fn compile_bool() -> String {
    "bool".to_string()
}

pub fn compile_adr() -> String {
    "std::sync::Arc<std::sync::Mutex<f64>>".to_string()
}

pub fn compile_pipe() -> String {
    "KiroPipe<f64>".to_string()
}

pub fn compile_custom(name: &StructNameVal) -> String {
    name.value.clone()
}

pub fn compile_list(inner: &KiroType) -> String {
    format!("Vec<{}>", compile_type(inner))
}

pub fn compile_map(key: &KiroType, value: &KiroType) -> String {
    format!(
        "std::collections::HashMap<{}, {}>",
        compile_type(key),
        compile_type(value)
    )
}
