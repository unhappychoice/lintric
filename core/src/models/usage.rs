use tree_sitter::Node;

#[derive(Debug, Clone, PartialEq)]
pub enum UsageKind {
    Identifier,
    CallExpression,
    FieldExpression,
    StructExpression,
    Metavariable,
}

#[derive(Debug, Clone)]
pub struct Usage<'a> {
    pub node: Node<'a>,
    pub kind: UsageKind,
    pub scope: Option<String>,
}
