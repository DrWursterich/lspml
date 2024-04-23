use lsp_types::Position;
use tree_sitter::{Node, Point, Tree};

pub(crate) fn find_current_node<'tree>(
    tree: &'tree Tree,
    position: Position,
) -> Option<Node<'tree>> {
    let root_node = tree.root_node();
    let trigger_point = Point::new(position.line as usize, position.character as usize);
    let mut cursor = root_node.walk();
    let mut node;
    loop {
        node = cursor.node();
        if node.end_position() <= trigger_point {
            if !cursor.goto_next_sibling() || cursor.node().start_position() > trigger_point {
                node = node.parent().unwrap();
                break;
            }
        } else if !cursor.goto_first_child() {
            break;
        }
    }
    log::debug!("current node: {:?}", node);
    return Some(node);
}

pub(crate) fn attribute_name_of<'a>(attribute: Node<'_>, source: &'a str) -> Option<&'a str> {
    return attribute
        .child(0)
        .and_then(|node| node.utf8_text(source.as_bytes()).ok());
}

pub(crate) fn attribute_value_of<'a>(attribute: Node<'_>, source: &'a str) -> Option<&'a str> {
    return attribute
        .child(2)
        .and_then(|node| node.child(1))
        .and_then(|node| node.utf8_text(source.as_bytes()).ok());
}

pub(crate) fn attribute_name_and_value_of<'a>(
    attribute: Node<'_>,
    source: &'a str,
) -> Option<(&'a str, &'a str)> {
    return attribute
        .child(0)
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(|name| {
            (
                name,
                attribute
                    .child(2)
                    .and_then(|node| node.child(1))
                    .filter(|node| node.kind() == "string_content")
                    .and_then(|node| node.utf8_text(source.as_bytes()).ok())
                    .unwrap_or(""),
            )
        });
}
