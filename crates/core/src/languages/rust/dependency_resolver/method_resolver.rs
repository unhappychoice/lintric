use crate::models::{Definition, Dependency, InferenceContext, Type, Usage, UsageKind};
use std::collections::HashMap;
use tree_sitter::Node;

#[derive(Debug, Clone)]
pub struct MethodResolutionResult {
    pub resolved_method: Definition,
    pub receiver_type: Type,
    pub resolution_path: ResolutionPath,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub enum ResolutionPath {
    InherentMethod { impl_block_id: ImplBlockId },
    TraitMethod { trait_impl_id: TraitImplId },
    Associated { type_name: String },
}

pub type ImplBlockId = u32;
pub type TraitImplId = u32;
pub type TraitId = u32;

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub id: ImplBlockId,
    pub target_type: Type,
    pub trait_impl: Option<TraitImplId>,
    pub methods: Vec<Definition>,
    pub associated_types: Vec<Definition>,
    pub generic_params: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TraitDef {
    pub id: TraitId,
    pub name: String,
    pub methods: Vec<Definition>,
    pub associated_types: Vec<Definition>,
    pub super_traits: Vec<TraitId>,
}

#[derive(Debug, Clone)]
pub struct TraitImpl {
    pub id: TraitImplId,
    pub trait_def: TraitId,
    pub target_type: Type,
    pub implemented_methods: Vec<Definition>,
}

pub struct MethodResolver {
    pub type_inference_engine: TypeInferenceEngine,
    pub impl_block_analyzer: ImplBlockAnalyzer,
    pub trait_resolver: TraitResolver,
}

pub struct TypeInferenceEngine {
    symbol_table: HashMap<String, Type>,
    #[allow(dead_code)]
    type_cache: HashMap<u32, Type>,
}

pub struct ImplBlockAnalyzer {
    impl_blocks: HashMap<ImplBlockId, ImplBlock>,
    type_to_impls: HashMap<String, Vec<ImplBlockId>>,
}

pub struct TraitResolver {
    traits: HashMap<TraitId, TraitDef>,
    trait_impls: HashMap<TraitImplId, TraitImpl>,
}

impl MethodResolver {
    pub fn new() -> Self {
        Self {
            type_inference_engine: TypeInferenceEngine::new(),
            impl_block_analyzer: ImplBlockAnalyzer::new(),
            trait_resolver: TraitResolver::new(),
        }
    }

    pub fn resolve_method_call(
        &self,
        usage: &Usage,
        _source_code: &str,
        _root_node: Node,
        definitions: &[Definition],
    ) -> Option<MethodResolutionResult> {
        if usage.kind != UsageKind::CallExpression {
            return None;
        }

        // Check if it's an associated function call (Type::function)
        if let Some((type_name, function_name)) = self.parse_associated_function_call(&usage.name) {
            return self.resolve_associated_function_call(type_name, function_name, definitions);
        }

        // Extract receiver and method name from method call expression
        let (receiver_name, method_name) = self.parse_method_call(&usage.name)?;

        // Infer receiver type
        let receiver_type = self
            .type_inference_engine
            .infer_receiver_type(&receiver_name, &InferenceContext::new())?;

        // Find method definitions for this type
        let method_candidates =
            self.find_method_candidates(&receiver_type, &method_name, definitions);

        // Select best candidate
        self.select_best_method_candidate(method_candidates, receiver_type)
    }

    fn parse_method_call(&self, call_name: &str) -> Option<(String, String)> {
        if let Some(dot_pos) = call_name.rfind('.') {
            let receiver = call_name[..dot_pos].to_string();
            let method = call_name[dot_pos + 1..].to_string();
            Some((receiver, method))
        } else {
            None
        }
    }

    fn parse_associated_function_call(&self, call_name: &str) -> Option<(String, String)> {
        if let Some(double_colon_pos) = call_name.find("::") {
            let type_name = call_name[..double_colon_pos].to_string();
            let function_name = call_name[double_colon_pos + 2..].to_string();
            Some((type_name, function_name))
        } else {
            None
        }
    }

    fn resolve_associated_function_call(
        &self,
        type_name: String,
        function_name: String,
        _definitions: &[Definition],
    ) -> Option<MethodResolutionResult> {
        // Look for associated functions (no self parameter) in impl blocks
        if let Some(function_def) = self
            .impl_block_analyzer
            .resolve_associated_function(&type_name, &function_name)
        {
            let receiver_type = Type::Concrete(type_name.clone());
            return Some(MethodResolutionResult {
                resolved_method: function_def,
                receiver_type,
                resolution_path: ResolutionPath::Associated { type_name },
                confidence: 1.0,
            });
        }

        // TODO: Add fallback to regular function definitions if needed

        None
    }

    fn find_method_candidates(
        &self,
        receiver_type: &Type,
        method_name: &str,
        _definitions: &[Definition],
    ) -> Vec<Definition> {
        let mut candidates = Vec::new();

        // Find inherent methods
        if let Some(inherent_methods) = self
            .impl_block_analyzer
            .find_methods_for_type(&receiver_type.name())
        {
            for method in inherent_methods {
                if method.name == method_name {
                    candidates.push(method);
                }
            }
        }

        // Find trait methods
        candidates.extend(
            self.trait_resolver
                .find_trait_methods_for_type(receiver_type, method_name),
        );

        candidates
    }

    fn select_best_method_candidate(
        &self,
        candidates: Vec<Definition>,
        receiver_type: Type,
    ) -> Option<MethodResolutionResult> {
        if candidates.is_empty() {
            return None;
        }

        // Prioritize inherent methods over trait methods
        let inherent_methods: Vec<_> = candidates
            .iter()
            .filter(|def| {
                matches!(
                    def.definition_type,
                    crate::models::DefinitionType::MethodDefinition
                        | crate::models::DefinitionType::FunctionDefinition
                )
            })
            .collect();

        let best_candidate = if !inherent_methods.is_empty() {
            // Prefer inherent methods
            inherent_methods[0].clone()
        } else {
            // Fall back to trait methods
            candidates[0].clone()
        };

        let resolution_path = if inherent_methods.contains(&&best_candidate) {
            ResolutionPath::InherentMethod { impl_block_id: 0 } // TODO: Get actual impl block ID
        } else {
            ResolutionPath::TraitMethod { trait_impl_id: 0 } // TODO: Get actual trait impl ID
        };

        let confidence = if candidates.len() == 1 {
            1.0 // Unambiguous
        } else if !inherent_methods.is_empty() {
            0.9 // Inherent method preferred
        } else {
            0.7 // Multiple trait methods, less certain
        };

        Some(MethodResolutionResult {
            resolved_method: best_candidate,
            receiver_type,
            resolution_path,
            confidence,
        })
    }
}

impl TypeInferenceEngine {
    pub fn new() -> Self {
        Self {
            symbol_table: HashMap::new(),
            type_cache: HashMap::new(),
        }
    }

    pub fn infer_receiver_type(
        &self,
        receiver_name: &str,
        _context: &InferenceContext,
    ) -> Option<Type> {
        // Look up in symbol table first
        if let Some(type_) = self.symbol_table.get(receiver_name) {
            return Some(type_.clone());
        }

        // Basic heuristic-based type inference
        // TODO: Implement proper type inference
        if receiver_name == "self" {
            Some(Type::Concrete("Self".to_string()))
        } else {
            // Try to infer from variable naming patterns
            Some(Type::Unknown)
        }
    }

    pub fn with_symbols(mut self, symbols: HashMap<String, Type>) -> Self {
        self.symbol_table = symbols;
        self
    }
}

impl ImplBlockAnalyzer {
    pub fn new() -> Self {
        Self {
            impl_blocks: HashMap::new(),
            type_to_impls: HashMap::new(),
        }
    }

    pub fn find_methods_for_type(&self, type_name: &str) -> Option<Vec<Definition>> {
        let impl_block_ids = self.type_to_impls.get(type_name)?;
        let mut methods = Vec::new();

        for &impl_id in impl_block_ids {
            if let Some(impl_block) = self.impl_blocks.get(&impl_id) {
                methods.extend(impl_block.methods.clone());
            }
        }

        if methods.is_empty() {
            None
        } else {
            Some(methods)
        }
    }

    pub fn resolve_associated_function(
        &self,
        type_name: &str,
        function_name: &str,
    ) -> Option<Definition> {
        let impl_block_ids = self.type_to_impls.get(type_name)?;

        for &impl_id in impl_block_ids {
            if let Some(impl_block) = self.impl_blocks.get(&impl_id) {
                for method in &impl_block.methods {
                    if method.name == function_name
                        && matches!(
                            method.definition_type,
                            crate::models::DefinitionType::FunctionDefinition
                        )
                    {
                        return Some(method.clone());
                    }
                }
            }
        }

        None
    }

    pub fn add_impl_block(&mut self, impl_block: ImplBlock) {
        let type_name = impl_block.target_type.name();
        let impl_id = impl_block.id;

        self.type_to_impls
            .entry(type_name)
            .or_default()
            .push(impl_id);

        self.impl_blocks.insert(impl_id, impl_block);
    }
}

impl TraitResolver {
    pub fn new() -> Self {
        Self {
            traits: HashMap::new(),
            trait_impls: HashMap::new(),
        }
    }

    pub fn find_trait_methods_for_type(
        &self,
        receiver_type: &Type,
        method_name: &str,
    ) -> Vec<Definition> {
        let mut trait_methods = Vec::new();
        let type_name = receiver_type.name();

        // Find all trait implementations for this type
        for trait_impl in self.trait_impls.values() {
            if trait_impl.target_type.name() == type_name {
                // Check if this trait implementation has the method we're looking for
                for method in &trait_impl.implemented_methods {
                    if method.name == method_name {
                        trait_methods.push(method.clone());
                    }
                }

                // Check the trait definition and its hierarchy for methods
                if let Some(trait_def) = self.traits.get(&trait_impl.trait_def) {
                    self.collect_trait_methods_recursive(
                        trait_def,
                        method_name,
                        &mut trait_methods,
                        &trait_impl.implemented_methods,
                    );
                }
            }
        }

        trait_methods
    }

    fn collect_trait_methods_recursive(
        &self,
        trait_def: &TraitDef,
        method_name: &str,
        trait_methods: &mut Vec<Definition>,
        implemented_methods: &[Definition],
    ) {
        // Check current trait for the method
        for method in &trait_def.methods {
            if method.name == method_name {
                // Only add if not already implemented in the impl block
                if !implemented_methods.iter().any(|m| m.name == method_name) {
                    trait_methods.push(method.clone());
                }
            }
        }

        // Recursively check super traits
        for &super_trait_id in &trait_def.super_traits {
            if let Some(super_trait) = self.traits.get(&super_trait_id) {
                self.collect_trait_methods_recursive(
                    super_trait,
                    method_name,
                    trait_methods,
                    implemented_methods,
                );
            }
        }
    }

    pub fn add_trait(&mut self, trait_def: TraitDef) {
        self.traits.insert(trait_def.id, trait_def);
    }

    pub fn add_trait_impl(&mut self, trait_impl: TraitImpl) {
        self.trait_impls.insert(trait_impl.id, trait_impl);
    }
}

impl Default for MethodResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TypeInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ImplBlockAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TraitResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl MethodResolver {
    /// Resolve struct field access dependencies for Rust
    pub fn resolve_struct_field_access(
        &self,
        usage_node: &Usage,
        definitions: &[Definition],
    ) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Only handle FieldExpression usage
        if usage_node.kind != UsageKind::FieldExpression {
            return dependencies;
        }

        // For field expressions like "p.x", extract the field name "x"
        let field_name = if usage_node.name.contains('.') {
            usage_node
                .name
                .split('.')
                .next_back()
                .unwrap_or(&usage_node.name)
                .to_string()
        } else {
            usage_node.name.clone()
        };

        // Find struct field definitions by the extracted field name
        for definition in definitions {
            if definition.name == field_name
                && matches!(
                    definition.definition_type,
                    crate::models::DefinitionType::StructFieldDefinition
                )
            {
                let source_line = usage_node.position.start_line;
                let target_line = definition.position.start_line;

                // Don't create self-referential dependencies
                if source_line != target_line {
                    let dependency = Dependency {
                        source_line,
                        target_line,
                        symbol: field_name.clone(),
                        dependency_type: crate::models::DependencyType::StructFieldAccess,
                        context: Some("field_access".to_string()),
                    };
                    dependencies.push(dependency);
                }
            }
        }

        dependencies
    }
}
