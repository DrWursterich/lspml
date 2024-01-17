use super::parser;
use anyhow::{Error, Result};
use lsp_types::{/*Position,*/ TextDocumentPositionParams};
use tree_sitter::{/*Node,*/ Parser, Point, /*Query, QueryCursor, Tree*/};

// #[derive(Debug)]
// pub(crate) enum Symbol {
//     Element,
//     StartTag,
//     TagName,
//     Attribute,
//     AttributeName,
//     QuotedAttributeValue,
//     AttributeValue,
//     RawText,
//     EndTag,
//     SelfClosingTag,
//     Error,
//     ExpressionStatement,
//     MemberExpression,
//     Object,
//     Property,
//     Unknown,
// }

pub(crate) fn dump_current_node(text_params: &TextDocumentPositionParams) -> Result<()> {
    let text = parser::get_text_document(&text_params)?;
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_html::language())?;
    let tree = match parser.parse(&text, None) {
        Some(value) => Ok(value),
        None => Result::Err(anyhow::anyhow!("")),
    }?;
    let cursor = Point::new(
        text_params.position.line as usize,
        text_params.position.character as usize,
    );
    let node = tree
        .root_node()
        .descendant_for_point_range(cursor, cursor)
        .ok_or_else(|| anyhow::anyhow!(""))?;
    log::debug!("node: {node:?}");
    node.utf8_text(text.as_bytes())
        .map(|text| {
            log::debug!("kind: {}, text: {}", node.kind(), text);
            //     (
            //         match node.kind() {
            //             "element" => Symbol::Element,
            //             "start_tag" => Symbol::StartTag,
            //             "tag_name" => Symbol::TagName,
            //             "attribute" => Symbol::Attribute,
            //             "attribute_name" => Symbol::AttributeName,
            //             "quoted_attribute_value" => Symbol::QuotedAttributeValue,
            //             "attribute_value" => Symbol::AttributeValue,
            //             "raw_text" => Symbol::RawText,
            //             "end_tag" => Symbol::EndTag,
            //             "self_closing_tag" => Symbol::SelfClosingTag,
            //             "error" => Symbol::Error,
            //             "expression_statement" => Symbol::ExpressionStatement,
            //             "member_expression" => Symbol::MemberExpression,
            //             "object" => Symbol::Object,
            //             "property" => Symbol::Property,
            //             _ => Symbol::Unknown,
            //         },
            //         text.to_string(),
            //     )
        })
        .map_err(Error::from)?;
    return Ok(());
}

// fn get_node_at<'a>(tree: &'a Tree, position: Point) -> Result<Node<'a>> {
//     let mut tree_cursor = tree.walk();
//     loop {
//         let current_node = tree_cursor.node().to_owned();
//         if current_node.start_position() <= position && current_node.end_position() >= position {
//             if !tree_cursor.goto_first_child() {
//                 return Ok(current_node);
//             }
//         } else if !tree_cursor.goto_next_sibling() {
//             return Result::Err(anyhow::anyhow!(""));
//         }
//     }
// }

// fn get_position_from_lsp_completion(text_params: TextDocumentPositionParams) -> Option<Position> {
//     let text = parser::get_text_document(&text_params).unwrap();
//     let mut parser = Parser::new();
//     parser.set_language(tree_sitter_html::language());
//     let tree = parser.parse(&text, None)?;
//     let root_node = tree.root_node();
//     let cursor = Point::new(
//         text_params.position.line as usize,
//         text_params.position.character as usize,
//     );
//     return query_position(root_node, &text, trigger_point);
// }

// fn query_some_shit() {
//     let cursor = TreeCursor::new();
//     let query = Query::new(tree_sitter_html::language(), query_string)
//         .unwrap_or_else(|_| panic!("get_position_by_query invalid query {query_string}"));
//     let mut cursor_query = QueryCursor::new();
//     let capture_names = query.capture_names();
//     let matches = cursor_query.matches(&query, node, source.as_bytes());
//     // Only consider the captures that are within the range based on the
//     // trigger point (cursor position)
//     return matches
//         .into_iter()
//         .flat_map(|m| {
//             m.captures
//                 .iter()
//                 .filter(|capture| capture.node.start_position() <= trigger_point)
//         })
//         .fold(HashMap::new(), |mut acc, capture| {
//             let key = capture_names[capture.index as usize].to_owned();
//             let value = capture.node.utf8_text(source.as_bytes()).to_owned();
//             acc.insert(
//                 key,
//                 CaptureDetails {
//                     value,
//                     end_position: capture.node.end_position(),
//                 },
//             );
//             return acc;
//         });
//     // let props = query_props(query_string, node, source, trigger_point);
//     // let attr_name = props.get("attr_name")?;
//     // if props.get("unfinished_tag").is_some() {
//     //     return None;
//     // }
//     // return Some(Position::AttributeName(attr_name.value.to_owned()));
// }

// fn query_position(root: Node<'_>, source: &str, trigger_point: Point) -> Option<Position> {
//     let closest_node = root.descendant_for_point_range(trigger_point, trigger_point)?;
//     let element = find_element_referent_to_current_node(closest_node)?;
//     let attr_completion = query_attr_keys_for_completion(element, source, trigger_point);
//     if attr_completion.is_some() {
//         return attr_completion;
//     }
//     return query_attr_values_for_completion(element, source, trigger_point);
// }

// fn find_element_referent_to_current_node(node: Node<'_>) -> Option<Node<'_>> {
//     if node.kind() == "element" || node.kind() == "fragment" {
//         return Some(node);
//     }
//     return None;
// }

// pub fn query_attr_keys_for_completion(
//     node: Node<'_>,
//     source: &str,
//     trigger_point: Point,
// ) -> Option<Position> {
//     // [ ] means match any of the following
//     let query_string = r#"
//     (
//         [
//             (_
//                 (tag_name)
//                 (_)*
//                 (attribute (attribute_name) @attr_name) @complete_match
//                 (#eq? @attr_name @complete_match)
//             )
//             (_
//               (tag_name)
//               (attribute (attribute_name))
//               (ERROR)
//             ) @unfinished_tag
//         ]
//         (#match? @attr_name "hx-.*")
//     )"#;
//     let props = query_props(query_string, node, source, trigger_point);
//     let attr_name = props.get("attr_name")?;
//     if props.get("unfinished_tag").is_some() {
//         return None;
//     }
//     return Some(Position::AttributeName(attr_name.value.to_owned()));
// }

// pub fn query_attr_values_for_completion(
//     node: Node<'_>,
//     source: &str,
//     trigger_point: Point,
// ) -> Option<Position> {
//     // [ ] means match any of the following
//     let query_string = r#"(
//         [
//           (ERROR
//             (tag_name)
//             (attribute_name) @attr_name
//             (_)
//           ) @open_quote_error
//           (_
//             (tag_name)
//             (attribute
//               (attribute_name) @attr_name
//               (_)
//             ) @last_item
//             (ERROR) @error_char
//           )
//           (_
//             (tag_name)
//             (attribute
//               (attribute_name) @attr_name
//               (quoted_attribute_value) @quoted_attr_value
//               (#eq? @quoted_attr_value "\"\"")
//             ) @empty_attribute
//           )
//           (_
//             (tag_name)
//             (attribute
//               (attribute_name) @attr_name
//               (quoted_attribute_value (attribute_value) @attr_value)
//               ) @non_empty_attribute
//           )
//         ]
//         (#match? @attr_name "hx-.*")
//     )"#;
//     let props = query_props(query_string, node, source, trigger_point);
//     let attr_name = props.get("attr_name")?;
//     if props.get("open_quote_error").is_some() || props.get("empty_attribute").is_some() {
//         return Some(Position::AttributeValue {
//             name: attr_name.value.to_owned(),
//             value: "".to_string(),
//         });
//     }
//     if let Some(error_char) = props.get("error_char") {
//         if error_char.value == KEY_VALUE_SEPARATOR {
//             return None;
//         }
//     };
//     if let Some(capture) = props.get("non_empty_attribute") {
//         if trigger_point >= capture.end_position {
//             return None;
//         }
//     }
//     return Some(Position::AttributeValue {
//         name: attr_name.value.to_owned(),
//         value: "".to_string(),
//     });
// }

// fn query_props(
//     query_string: &str,
//     node: Node<'_>,
//     source: &str,
//     trigger_point: Point,
// ) -> HashMap<String, CaptureDetails> {
//     let query = Query::new(tree_sitter_html::language(), query_string)
//         .unwrap_or_else(|_| panic!("get_position_by_query invalid query {query_string}"));
//     let mut cursor_qry = QueryCursor::new();
//     let capture_names = query.capture_names();
//     let matches = cursor_qry.matches(&query, node, source.as_bytes());
//     // Only consider the captures that are within the range based on the
//     // trigger point (cursor position)
//     return matches
//         .into_iter()
//         .flat_map(|m| {
//             m.captures
//                 .iter()
//                 .filter(|capture| capture.node.start_position() <= trigger_point)
//         })
//         .fold(HashMap::new(), |mut acc, capture| {
//             let key = capture_names[capture.index as usize].to_owned();
//             let value = capture.node.utf8_text(source.as_bytes()).to_owned();
//             acc.insert(
//                 key,
//                 CaptureDetails {
//                     value,
//                     end_position: capture.node.end_position(),
//                 },
//             );
//             return acc;
//         });
// }
