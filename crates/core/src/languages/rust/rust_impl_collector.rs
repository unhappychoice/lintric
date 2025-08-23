use crate::dependency_resolver::method_resolver::{
    ImplBlock, ImplBlockId, TraitDef, TraitId, TraitImpl, TraitImplId,
};
use crate::models::{Definition, DefinitionType, Position, Type};
use tree_sitter::{Node, Query, QueryCursor};

pub struct RustImplCollector {
    next_impl_id: ImplBlockId,
    next_trait_id: TraitId,
    next_trait_impl_id: TraitImplId,
}

impl RustImplCollector {
    pub fn new() -> Self {
        Self {
            next_impl_id: 1,
            next_trait_id: 1,
            next_trait_impl_id: 1,
        }
    }

    pub fn collect_impl_blocks(
        &mut self,
        source_code: &str,
        root_node: Node,
    ) -> Result<Vec<ImplBlock>, String> {
        let mut impl_blocks = Vec::new();

        // Query for impl blocks
        let query_str = r#"
            (impl_item
              type: (type_identifier) @type_name
              body: (declaration_list) @body
            ) @impl_block
        "#;

        let language = tree_sitter_rust::language();
        let query = Query::new(&language, query_str)
            .map_err(|e| format!("Failed to create impl query: {}", e))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, root_node, source_code.as_bytes());

        for match_ in matches {
            if let Some(impl_block) = self.parse_impl_block(&match_, source_code) {
                impl_blocks.push(impl_block);
            }
        }

        Ok(impl_blocks)
    }

    pub fn collect_trait_impl_blocks(
        &mut self,
        source_code: &str,
        root_node: Node,
    ) -> Result<Vec<TraitImpl>, String> {
        let mut trait_impls = Vec::new();

        // Query for trait impl blocks
        let query_str = r#"
            (impl_item
              trait: (type_identifier) @trait_name
              type: (type_identifier) @type_name
              body: (declaration_list) @body
            ) @trait_impl
        "#;

        let language = tree_sitter_rust::language();
        let query = Query::new(&language, query_str)
            .map_err(|e| format!("Failed to create trait impl query: {}", e))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, root_node, source_code.as_bytes());

        for match_ in matches {
            if let Some(trait_impl) = self.parse_trait_impl(&match_, source_code) {
                trait_impls.push(trait_impl);
            }
        }

        Ok(trait_impls)
    }

    pub fn collect_traits(
        &mut self,
        source_code: &str,
        root_node: Node,
    ) -> Result<Vec<TraitDef>, String> {
        let mut traits = Vec::new();

        // Query for trait definitions
        let query_str = r#"
            (trait_item
              name: (type_identifier) @trait_name
              body: (declaration_list) @body
            ) @trait_def
        "#;

        let language = tree_sitter_rust::language();
        let query = Query::new(&language, query_str)
            .map_err(|e| format!("Failed to create trait query: {}", e))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, root_node, source_code.as_bytes());

        for match_ in matches {
            if let Some(trait_def) = self.parse_trait_def(&match_, source_code) {
                traits.push(trait_def);
            }
        }

        Ok(traits)
    }

    fn parse_impl_block(
        &mut self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
    ) -> Option<ImplBlock> {
        let mut type_name = None;
        let mut body_node = None;

        for capture in query_match.captures.iter() {
            let node = capture.node;
            let text = node.utf8_text(source_code.as_bytes()).ok()?;

            match capture.index {
                0 => type_name = Some(text.to_string()), // @type_name
                1 => body_node = Some(node),             // @body
                _ => {}
            }
        }

        let type_name = type_name?;
        let body_node = body_node?;

        let methods = self.extract_methods_from_body(body_node, source_code);
        let impl_id = self.next_impl_id;
        self.next_impl_id += 1;

        Some(ImplBlock {
            id: impl_id,
            target_type: Type::Concrete(type_name),
            trait_impl: None,
            methods,
            associated_types: Vec::new(), // TODO: Extract associated types
            generic_params: Vec::new(),   // TODO: Extract generic parameters
        })
    }

    fn parse_trait_impl(
        &mut self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
    ) -> Option<TraitImpl> {
        let mut trait_name = None;
        let mut type_name = None;
        let mut body_node = None;

        for capture in query_match.captures.iter() {
            let node = capture.node;
            let text = node.utf8_text(source_code.as_bytes()).ok()?;

            match capture.index {
                0 => trait_name = Some(text.to_string()), // @trait_name
                1 => type_name = Some(text.to_string()),  // @type_name
                2 => body_node = Some(node),              // @body
                _ => {}
            }
        }

        let _trait_name = trait_name?;
        let type_name = type_name?;
        let body_node = body_node?;

        let implemented_methods = self.extract_methods_from_body(body_node, source_code);
        let trait_impl_id = self.next_trait_impl_id;
        self.next_trait_impl_id += 1;

        Some(TraitImpl {
            id: trait_impl_id,
            trait_def: 0, // TODO: Link to actual trait definition
            target_type: Type::Concrete(type_name),
            implemented_methods,
        })
    }

    fn parse_trait_def(
        &mut self,
        query_match: &tree_sitter::QueryMatch,
        source_code: &str,
    ) -> Option<TraitDef> {
        let mut trait_name = None;
        let mut body_node = None;

        for capture in query_match.captures.iter() {
            let node = capture.node;
            let text = node.utf8_text(source_code.as_bytes()).ok()?;

            match capture.index {
                0 => trait_name = Some(text.to_string()), // @trait_name
                1 => body_node = Some(node),              // @body
                _ => {}
            }
        }

        let trait_name = trait_name?;
        let body_node = body_node?;

        let methods = self.extract_methods_from_body(body_node, source_code);
        let trait_id = self.next_trait_id;
        self.next_trait_id += 1;

        Some(TraitDef {
            id: trait_id,
            name: trait_name,
            methods,
            associated_types: Vec::new(), // TODO: Extract associated types
            super_traits: Vec::new(),     // TODO: Extract super traits
        })
    }

    fn extract_methods_from_body(&self, body_node: Node, source_code: &str) -> Vec<Definition> {
        let mut methods = Vec::new();
        let mut cursor = body_node.walk();

        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();

                if child.kind() == "function_item" || child.kind() == "function_signature_item" {
                    if let Some(method) = self.parse_method_definition(child, source_code) {
                        methods.push(method);
                    }
                }

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        methods
    }

    fn parse_method_definition(
        &self,
        function_node: Node,
        source_code: &str,
    ) -> Option<Definition> {
        // Get the function name
        let name_node = function_node.child_by_field_name("name")?;
        let name = name_node
            .utf8_text(source_code.as_bytes())
            .ok()?
            .to_string();

        // Check if it's a method (has self parameter) or associated function
        let definition_type = if self.has_self_parameter(function_node) {
            DefinitionType::MethodDefinition
        } else {
            DefinitionType::FunctionDefinition
        };

        Some(Definition {
            name,
            definition_type,
            position: Position::from_node(&function_node),
        })
    }

    fn has_self_parameter(&self, function_node: Node) -> bool {
        if let Some(params_node) = function_node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();

            if cursor.goto_first_child() {
                loop {
                    let param = cursor.node();

                    // Check for self parameter patterns
                    if param.kind() == "self_parameter" {
                        return true;
                    }

                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        false
    }
}

impl Default for RustImplCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impl_collection() {
        let source_code = r#"
            impl MyStruct {
                fn new() -> Self {
                    MyStruct {}
                }
                
                fn method(&self) -> i32 {
                    42
                }
            }
        "#;

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(source_code, None).unwrap();
        let root_node = tree.root_node();

        let mut collector = RustImplCollector::new();
        let impl_blocks = collector
            .collect_impl_blocks(source_code, root_node)
            .unwrap();

        assert_eq!(impl_blocks.len(), 1);
        assert_eq!(impl_blocks[0].target_type.name(), "MyStruct");
        assert_eq!(impl_blocks[0].methods.len(), 2);

        // Check method types
        let new_method = impl_blocks[0]
            .methods
            .iter()
            .find(|m| m.name == "new")
            .unwrap();
        assert_eq!(
            new_method.definition_type,
            DefinitionType::FunctionDefinition
        );

        let instance_method = impl_blocks[0]
            .methods
            .iter()
            .find(|m| m.name == "method")
            .unwrap();
        assert_eq!(
            instance_method.definition_type,
            DefinitionType::MethodDefinition
        );
    }

    #[test]
    fn test_trait_collection() {
        let source_code = r#"
            trait Display {
                fn display(&self) -> String;
                fn debug(&self) -> String {
                    format!("Debug: {}", self.display())
                }
            }
        "#;

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(source_code, None).unwrap();
        let root_node = tree.root_node();

        let mut collector = RustImplCollector::new();
        let traits = collector.collect_traits(source_code, root_node).unwrap();

        assert_eq!(traits.len(), 1);
        assert_eq!(traits[0].name, "Display");
        assert_eq!(traits[0].methods.len(), 2);
    }

    #[test]
    fn test_trait_impl_collection() {
        let source_code = r#"
            impl Display for MyStruct {
                fn display(&self) -> String {
                    "MyStruct".to_string()
                }
            }
        "#;

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&tree_sitter_rust::language()).unwrap();
        let tree = parser.parse(source_code, None).unwrap();
        let root_node = tree.root_node();

        let mut collector = RustImplCollector::new();
        let trait_impls = collector
            .collect_trait_impl_blocks(source_code, root_node)
            .unwrap();

        assert_eq!(trait_impls.len(), 1);
        assert_eq!(trait_impls[0].target_type.name(), "MyStruct");
        assert_eq!(trait_impls[0].implemented_methods.len(), 1);
        assert_eq!(trait_impls[0].implemented_methods[0].name, "display");
    }
}
