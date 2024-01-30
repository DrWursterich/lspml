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
    log::debug!("node: {node:?}");
    return Some(node);
}

pub(crate) fn attribute_name_of<'a>(attribute: Node<'_>, source: &'a str) -> &'a str {
    return attribute
        .child(0)
        .expect(
            format!(
                "attribute {:?} did not have a attribute-name child",
                attribute
            )
            .as_str(),
        )
        .utf8_text(source.as_bytes())
        .expect(
            format!(
                "attribute-name in {:?} did not have a contain text",
                attribute
            )
            .as_str(),
        );
}

pub(crate) fn attribute_value_of<'a>(attribute: Node<'_>, source: &'a str) -> &'a str {
    let value = attribute
        .child(2)
        .expect(
            format!(
                "attribute {:?} did not have a attribute-value child",
                attribute
            )
            .as_str(),
        )
        .utf8_text(source.as_bytes())
        .expect(format!("attribute-value in {:?} did not contain text", attribute).as_str());
    // value should be wrapped inside quotes
    if value.len() > 1 && value.starts_with("\"") && value.ends_with("\"") {
        return &value[1..value.len() - 1];
    }
    log::info!(
        "unquoted attribute-value found for {:?}: {}",
        attribute,
        value
    );
    return &value;
}
