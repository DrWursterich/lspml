use anyhow::{Error, Result};
use lsp_types::Url;
use std::{
    collections::HashMap,
    fs,
    str::FromStr,
    sync::{Arc, Mutex, OnceLock},
};
use tree_sitter::{Node, Parser, Point, Tree};

use crate::{
    grammar::{TagAttributeType, TagAttributes, TagDefinition},
    parser,
    spel::{
        self,
        ast::{SpelAst, SpelResult},
    },
};

#[derive(Clone, Debug)]
pub(crate) struct Document {
    pub(crate) text: String,
    pub(crate) tree: Tree,
    pub(crate) spel: HashMap<Point, SpelAst>,
}

impl Document {
    pub(crate) fn new(text: String) -> Result<Document> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_spml::language())?;
        return match parser.parse(&text, None) {
            Some(tree) => {
                let spel = collect_spels(tree.root_node(), &text);
                Ok(Document { text, tree, spel })
            }
            None => return Result::Err(anyhow::anyhow!("failed to parse text: {}", text)),
        };
    }

    pub(crate) fn from_uri(uri: &Url) -> Result<Document> {
        return match uri.to_file_path() {
            Ok(path) if path.exists() => fs::read_to_string(path.to_owned())
                .map(|text| Document::new(text))
                .map_err(Error::from),
            Ok(path) => Err(anyhow::anyhow!("file {:?} does not exist", path)),
            Err(_) => Err(anyhow::anyhow!("failed to read file path from uri {}", uri)),
        }?;
    }
}

fn document_store() -> &'static Arc<Mutex<HashMap<Url, Document>>> {
    static DOCUMENT_STORE: OnceLock<Arc<Mutex<HashMap<Url, Document>>>> = OnceLock::new();
    return DOCUMENT_STORE.get_or_init(|| Arc::new(Mutex::new(HashMap::new())));
}

pub(crate) fn get(uri: &Url) -> Option<Document> {
    return document_store()
        .lock()
        .expect("document_store mutex poisoned")
        .get(&uri)
        .cloned();
}

pub(crate) fn put(uri: &Url, document: Document) -> Document {
    document_store()
        .lock()
        .expect("document_store mutex poisoned")
        .insert(uri.clone(), document.clone());
    return document;
}

fn collect_spels(root: Node, text: &String) -> HashMap<Point, SpelAst> {
    let mut spels = HashMap::new();
    for node in root.children(&mut root.walk()) {
        match node.kind() {
            "page_header" | "import_header" | "taglib_header" | "html_doctype" | "text"
            | "comment" | "xml_entity" => continue,
            "html_tag" | "html_option_tag" | "html_void_tag" | "xml_comment" | "java_tag"
            | "script_tag" | "style_tag" => collect_from_children(node, text, &mut spels),
            _ => match &TagDefinition::from_str(node.kind()) {
                Ok(tag) => collect_from_tag(tag, node, &text, &mut spels),
                Err(err) => log::info!(
                    "error while trying to interprete node \"{}\" as tag: {}",
                    node.kind(),
                    err
                ),
            },
        }
    }
    return spels;
}

fn collect_from_children(node: Node, text: &String, spels: &mut HashMap<Point, SpelAst>) {
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            "text" | "java_tag" | "html_void_tag" => {}
            "ERROR" | "html_tag" | "html_option_tag" | "script_tag" | "style_tag" => {
                collect_from_children(child, text, spels);
            }
            kind if kind.ends_with("_tag") => match &TagDefinition::from_str(kind) {
                Ok(child_tag) => collect_from_tag(child_tag, child, text, spels),
                Err(err) => {
                    log::info!("expected sp or spt tag: {}", err);
                }
            },
            _ => collect_from_children(child, text, spels),
        }
    }
}

fn collect_from_tag(
    tag: &TagDefinition,
    node: Node,
    text: &String,
    spel: &mut HashMap<Point, SpelAst>,
) {
    for child in node.children(&mut node.walk()) {
        match child.kind() {
            // may need to check on kind of missing child
            "html_void_tag" | "java_tag" | "script_tag" | "style_tag" => (),
            "ERROR" | "html_tag" | "html_option_tag" => collect_from_children(child, text, spel),
            kind if kind.ends_with("_attribute") => {
                match (parser::attribute_name_of(child, text), &tag.attributes) {
                    (Some(attribute), TagAttributes::These(definitions)) => {
                        if let Some(definition) = definitions
                            .iter()
                            .find(|definition| definition.name == attribute)
                        {
                            let value_node = match child.child(2).and_then(|child| child.child(1)) {
                                Some(node) => node,
                                _ => continue,
                            };
                            let position = value_node.start_position();
                            match spel_ast_of(value_node, text, &definition.r#type) {
                                Ok(ast) => {
                                    spel.insert(position, ast);
                                }
                                Err(err) => log::error!(
                                    "could not parse spel at ({}, {}) as {:?}: {}",
                                    position.row,
                                    position.column,
                                    definition.r#type,
                                    err
                                ),
                            };
                        };
                    }
                    _ => (),
                }
            }
            kind if kind.ends_with("_tag") => match &TagDefinition::from_str(kind) {
                Ok(child_tag) => collect_from_tag(child_tag, child, text, spel),
                Err(err) => {
                    log::info!("expected sp or spt tag: {}", err);
                }
            },
            _ => collect_from_children(child, text, spel),
        }
    }
}
fn spel_ast_of(node: Node, text: &str, r#type: &TagAttributeType) -> Result<SpelAst> {
    let parser = &mut spel::parser::Parser::new(node.utf8_text(&text.as_bytes())?);
    match r#type {
        TagAttributeType::Comparable => Ok(SpelAst::Comparable(match parser.parse_comparable() {
            Ok(result) => SpelResult::Valid(result),
            Err(err) => SpelResult::Invalid(err),
        })),
        TagAttributeType::Condition => Ok(SpelAst::Condition(match parser.parse_condition_ast() {
            Ok(result) => SpelResult::Valid(result.root),
            Err(err) => SpelResult::Invalid(err),
        })),
        TagAttributeType::Expression => {
            Ok(SpelAst::Expression(match parser.parse_expression_ast() {
                Ok(result) => SpelResult::Valid(result.root),
                Err(err) => SpelResult::Invalid(err),
            }))
        }
        TagAttributeType::Identifier => Ok(SpelAst::Identifier(match parser.parse_identifier() {
            Ok(result) => SpelResult::Valid(result),
            Err(err) => SpelResult::Invalid(err),
        })),
        TagAttributeType::Object => Ok(SpelAst::Object(match parser.parse_object_ast() {
            Ok(result) => SpelResult::Valid(result.root),
            Err(err) => SpelResult::Invalid(err),
        })),
        TagAttributeType::Regex => Ok(SpelAst::Regex(match parser.parse_regex() {
            Ok(result) => SpelResult::Valid(result),
            Err(err) => SpelResult::Invalid(err),
        })),
        TagAttributeType::String => Ok(SpelAst::String(match parser.parse_text() {
            Ok(result) => SpelResult::Valid(result),
            Err(err) => SpelResult::Invalid(err),
        })),
        TagAttributeType::Query => Ok(SpelAst::Query(match parser.parse_query() {
            Ok(result) => SpelResult::Valid(result),
            Err(err) => SpelResult::Invalid(err),
        })),
        TagAttributeType::Uri => Ok(SpelAst::Uri(match parser.parse_uri() {
            Ok(result) => SpelResult::Valid(result),
            Err(err) => SpelResult::Invalid(err),
        })),
    }
}
