//! Tree-sitter helper functions (adapted from veriplan).

/// Get text from a tree-sitter node.
pub fn node_text<'a>(
    node: &tree_sitter::Node,
    source: &'a [u8],
) -> Result<&'a str, std::str::Utf8Error> {
    std::str::from_utf8(&source[node.byte_range()])
}

/// Find child node by kind.
pub fn find_child<'a>(node: &tree_sitter::Node<'a>, kind: &str) -> Option<tree_sitter::Node<'a>> {
    let mut cursor = node.walk();
    node.children(&mut cursor)
        .find(|&child| child.kind() == kind)
}
