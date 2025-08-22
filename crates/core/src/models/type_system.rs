use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Concrete(String),
    Generic(String, Vec<Type>),
    Reference(Box<Type>),
    TypeParameter(String),
    Unknown,
}

impl Type {
    pub fn name(&self) -> String {
        match self {
            Type::Concrete(name) => name.clone(),
            Type::Generic(name, _) => name.clone(),
            Type::Reference(inner) => format!("&{}", inner.name()),
            Type::TypeParameter(name) => name.clone(),
            Type::Unknown => "unknown".to_string(),
        }
    }

    pub fn is_reference(&self) -> bool {
        matches!(self, Type::Reference(_))
    }

    pub fn deref(&self) -> Type {
        match self {
            Type::Reference(inner) => (**inner).clone(),
            other => other.clone(),
        }
    }

    pub fn is_generic(&self) -> bool {
        matches!(self, Type::Generic(_, _))
    }

    pub fn is_type_parameter(&self) -> bool {
        matches!(self, Type::TypeParameter(_))
    }

    pub fn substitute_type_parameter(&self, param_name: &str, concrete_type: &Type) -> Type {
        match self {
            Type::TypeParameter(name) if name == param_name => concrete_type.clone(),
            Type::Generic(name, args) => {
                let substituted_args: Vec<Type> = args
                    .iter()
                    .map(|arg| arg.substitute_type_parameter(param_name, concrete_type))
                    .collect();
                Type::Generic(name.clone(), substituted_args)
            }
            Type::Reference(inner) => Type::Reference(Box::new(
                inner.substitute_type_parameter(param_name, concrete_type),
            )),
            other => other.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InferenceContext {
    pub symbol_table: HashMap<String, Type>,
    pub type_cache: HashMap<u32, Type>, // Node ID -> Type mapping
}

impl InferenceContext {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            type_cache: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, name: String, type_: Type) {
        self.symbol_table.insert(name, type_);
    }

    pub fn get_symbol_type(&self, name: &str) -> Option<&Type> {
        self.symbol_table.get(name)
    }

    pub fn cache_type(&mut self, node_id: u32, type_: Type) {
        self.type_cache.insert(node_id, type_);
    }

    pub fn get_cached_type(&self, node_id: u32) -> Option<&Type> {
        self.type_cache.get(&node_id)
    }
}

impl Default for InferenceContext {
    fn default() -> Self {
        Self::new()
    }
}
