use anyhow::{Error, Result};
use lsp_types::{Position, Url};
use std::fs;
use tree_sitter::{Node, Point, Tree};

pub(crate) fn get_text_document(uri: &Url) -> Result<String> {
    return match uri.to_file_path() {
        Ok(path) => fs::read_to_string(path.to_owned()).map_err(Error::from),
        Err(_) => Result::Err(anyhow::anyhow!("failed to read file path from uri {}", uri)),
    };
}

pub(crate) fn find_current_and_previous_nodes<'tree>(
    tree: &'tree Tree,
    position: Position,
) -> Option<(Node<'tree>, Option<Node<'tree>>)> {
    let root_node = tree.root_node();
    let trigger_point = Point::new(position.line as usize, position.character as usize);
    // let node = root_node.descendant_for_point_range(trigger_point, trigger_point)?;
    let mut cursor = root_node.walk();
    let mut node;
    let mut previous;
    loop {
        node = cursor.node();
        if node.end_position() <= trigger_point {
            previous = Some(node);
            if !cursor.goto_next_sibling() || cursor.node().start_position() > trigger_point {
                node = node.parent().unwrap();
                break;
            }
        } else if !cursor.goto_first_child() {
            previous = node.prev_sibling();
            break;
        }
    }
    if let Some(prev) = previous {
        if prev.kind() == "ERROR" {
            previous = prev.child(prev.child_count() - 1);
        }
    }
    eprintln!("node: {node:?}, previous: {previous:?}",);
    return Some((node, previous));
}
