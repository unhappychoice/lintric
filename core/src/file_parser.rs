use std::fs;
use std::path::PathBuf;
use tree_sitter::{Parser as TreeSitterParser, Tree};

use crate::models::Language;

pub struct FileParser {
    file_content: String,
    language: Language,
}

impl FileParser {
    pub fn new(file_path: String) -> Result<Self, String> {
        let file_content = fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read file {file_path}: {e}"))?;
        let language = Language::from_extension(&PathBuf::from(&file_path))
            .ok_or_else(|| format!("Unsupported file type for analysis: {file_path}"))?;
        Ok(FileParser {
            file_content,
            language,
        })
    }

    pub fn parse(&self) -> Result<(String, Language, tree_sitter::Tree), String> {
        let tree = self.parse_file()?;
        Ok((self.file_content.clone(), self.language.clone(), tree))
    }

    pub fn parse_as_s_expression(&self) -> Result<String, String> {
        let tree = self.parse_file()?;
        Ok(self.format_s_expression(tree.root_node(), 0))
    }

    fn parse_file(&self) -> Result<Tree, String> {
        let mut parser = TreeSitterParser::new();

        let lang = &self.language.get_tree_sitter_language();

        parser
            .set_language(lang)
            .map_err(|e| format!("Error loading grammar: {e}"))?;

        let tree = parser
            .parse(&self.file_content, None)
            .ok_or_else(|| "Failed to parse the source code.".to_string())?;

        Ok(tree)
    }

    fn format_s_expression(&self, node: tree_sitter::Node, depth: usize) -> String {
        let indent = "  ".repeat(depth);
        let mut s_expr = String::new();

        let node_text = if node.kind() == "identifier" {
            format!(
                "identifier {}",
                node.utf8_text(self.file_content.as_bytes()).unwrap()
            )
        } else {
            node.kind().to_string()
        };

        s_expr.push_str(&format!("({node_text}"));

        let mut children_s_exprs = Vec::new();
        for i in 0..node.named_child_count() {
            if let Some(child) = node.named_child(i) {
                children_s_exprs.push(self.format_s_expression(child, depth + 1));
            }
        }

        if !children_s_exprs.is_empty() {
            for child_s_expr in children_s_exprs {
                s_expr.push_str(&format!("\n{}{}", "  ".repeat(depth + 1), child_s_expr));
            }
            s_expr.push_str(&format!("\n{indent})"));
        } else {
            s_expr.push(')');
        }

        s_expr
    }
}
