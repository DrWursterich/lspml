use std::{collections::HashMap, path::Path, str::FromStr};

use anyhow::Result;
use lsp_server::ErrorCode;
use lsp_types::{
    Diagnostic, DiagnosticSeverity, DiagnosticTag, DocumentDiagnosticParams, NumberOrString,
    Position, Range, TextEdit, Url,
};
use tree_sitter::{Node, Point};

use crate::{
    document_store,
    grammar::{self, TagChildren, TagDefinition},
    modules, parser,
    spel::{
        self,
        ast::{self, SpelAst, SpelResult},
        grammar::ArgumentNumber,
        parser::SyntaxError,
    },
    CodeActionImplementation,
};

use super::LsError;

pub(crate) struct DiagnosticCollector {
    pub(crate) file: Url,
    pub(crate) text: String,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    pub(crate) fn new(file: Url, text: String) -> DiagnosticCollector {
        return DiagnosticCollector {
            file,
            text,
            diagnostics: Vec::new(),
        };
    }

    pub(crate) fn validate_document(
        self: &mut Self,
        root: &Node,
        spel: &HashMap<Point, SpelAst>,
    ) -> Result<()> {
        self.validate_header(root)?;
        for node in root.children(&mut root.walk()) {
            match node.kind() {
                "page_header" | "import_header" | "taglib_header" | "html_doctype" | "text"
                | "comment" | "xml_entity" => continue,
                "ERROR" => self.add_diagnostic(
                    format!("unexpected \"{}\"", node.utf8_text(self.text.as_bytes())?),
                    DiagnosticSeverity::ERROR,
                    self.node_range(&node),
                ),
                "html_tag" | "html_option_tag" | "html_void_tag" | "xml_comment" | "java_tag"
                | "script_tag" | "style_tag" => self.validate_children(&node, spel)?,
                _ => match &TagDefinition::from_str(node.kind()) {
                    Ok(tag) => self.validate_tag(tag, &node, spel),
                    Err(err) => {
                        log::info!(
                            "error while trying to interprete node \"{}\" as tag: {}",
                            node.kind(),
                            err
                        );
                        continue;
                    }
                }?,
            }
        }
        return Ok(());
    }

    fn validate_header(self: &mut Self, root: &Node) -> Result<()> {
        if root.kind() != "document" {
            let document_start = Position {
                line: 0,
                character: 0,
            };
            self.add_diagnostic_with_code(
                format!(
                    "missing atleast one header. Try generating one with the \"{}\" code-action",
                    CodeActionImplementation::GenerateDefaultHeaders
                ),
                DiagnosticSeverity::ERROR,
                Range {
                    start: document_start,
                    end: document_start,
                },
                CodeActionImplementation::GENERATE_DEFAULT_HEADER_CODE,
                None,
            );
        }
        return Ok(());
    }

    fn validate_tag(
        self: &mut Self,
        tag: &TagDefinition,
        node: &Node,
        spel: &HashMap<Point, SpelAst>,
    ) -> Result<()> {
        if tag.deprecated {
            self.add_diagnostic_with_tag(
                format!("{} tag is deprecated", tag.name),
                DiagnosticSeverity::INFORMATION,
                self.node_tag_range(node),
                DiagnosticTag::DEPRECATED,
            );
        }
        let mut attributes: HashMap<String, String> = HashMap::new();
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                // may need to check on kind of missing child
                _ if child.is_missing() => self.add_diagnostic(
                    format!("{} is never closed", node.kind()),
                    DiagnosticSeverity::ERROR,
                    self.node_tag_range(node),
                ),
                _ if child.is_error() => self.add_diagnostic(
                    format!("unexpected \"{}\"", child.utf8_text(self.text.as_bytes())?),
                    DiagnosticSeverity::ERROR,
                    self.node_range(&child),
                ),
                "html_void_tag" | "java_tag" | "script_tag" | "style_tag" => {}
                "html_tag" | "html_option_tag" => self.validate_children(&child, spel)?,
                kind if kind.ends_with("_attribute") => {
                    let (attribute, value) =
                        match parser::attribute_name_and_value_of(child, self.text.as_str()) {
                            Some((attribute, value)) => (attribute.to_string(), value.to_string()),
                            _ => continue,
                        };
                    if let Some(value_node) = child.child(2).and_then(|child| child.child(1)) {
                        SpelValidator::validate(self, &value_node, spel)?;
                    };
                    if attributes.contains_key(&attribute) {
                        self.add_diagnostic(
                            format!("duplicate {} attribute", attribute),
                            DiagnosticSeverity::WARNING,
                            self.node_tag_range(node),
                        );
                    } else {
                        attributes.insert(attribute, value);
                    }
                }
                kind if kind.ends_with("_tag") => match &TagDefinition::from_str(kind) {
                    Ok(child_tag) if self.can_have_child(&tag, child_tag) => {
                        self.validate_tag(child_tag, &child, spel)?;
                    }
                    Ok(_) => self.add_diagnostic(
                        format!("unexpected {} tag", &kind[..kind.find("_tag").unwrap()]),
                        DiagnosticSeverity::WARNING,
                        self.node_range(&child),
                    ),
                    Err(err) => log::info!("expected sp or spt tag: {}", err),
                },
                _ => self.validate_children(&child, spel)?,
            }
        }
        for rule in tag.attribute_rules {
            match rule {
                grammar::AttributeRule::Deprecated(name) if attributes.contains_key(*name) => {
                    self.add_diagnostic_with_tag(
                        format!("attribute {} is deprecated", name),
                        DiagnosticSeverity::INFORMATION,
                        self.node_tag_range(node),
                        DiagnosticTag::DEPRECATED,
                    );
                }
                grammar::AttributeRule::AtleastOneOf(names)
                    if !names.iter().any(|name| attributes.contains_key(*name)) =>
                {
                    self.add_diagnostic(
                        format!(
                            "requires atleast one of these attributes: {}",
                            names.join(", ")
                        ),
                        DiagnosticSeverity::ERROR,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::ExactlyOneOf(names) => {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| attributes.contains_key(*name))
                        .collect();
                    match present.len() {
                        0 => self.add_diagnostic(
                            format!("requires one of these attributes: {}", names.join(", ")),
                            DiagnosticSeverity::ERROR,
                            self.node_tag_range(node),
                        ),
                        1 => {}
                        _ => self.add_diagnostic(
                            format!(
                                "requires only one of these attributes: {}",
                                present.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            self.node_tag_range(node),
                        ),
                    }
                }
                grammar::AttributeRule::ExactlyOneOfOrBody(names) => {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| attributes.contains_key(*name))
                        .collect();
                    let has_body = node
                        .child(node.child_count() - 1)
                        .is_some_and(|tag| tag.kind().ends_with("_tag_close"));
                    match (present.len(), has_body) {
                        (0, false) => self.add_diagnostic(
                            format!(
                                "requires either a tag-body or one of these attributes: {}",
                                names.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            self.node_tag_range(node),
                        ),
                        (0, true) | (1, false) => {}
                        _ => self.add_diagnostic(
                            format!(
                                "requires either a tag-body or only one of these attributes: {}",
                                present.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            self.node_tag_range(node),
                        ),
                    }
                }
                grammar::AttributeRule::ExactlyOrBody(name)
                    if attributes.contains_key(*name)
                        != node
                            .child(node.child_count() - 1)
                            .is_some_and(|tag| tag.kind().ends_with("_tag_close")) =>
                {
                    self.add_diagnostic(
                        format!("requires either a tag-body or the attribute {}", name,),
                        DiagnosticSeverity::ERROR,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::OnlyOneOf(names) => {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| attributes.contains_key(*name))
                        .collect();
                    if present.len() > 1 {
                        self.add_diagnostic(
                            format!(
                                "can only have one of these attributes: {}",
                                present.join(", ")
                            ),
                            DiagnosticSeverity::WARNING,
                            self.node_tag_range(node),
                        );
                    }
                }
                grammar::AttributeRule::OnlyOneOfOrBody(names) => {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| attributes.contains_key(*name))
                        .collect();
                    let has_body = node
                        .child(node.child_count() - 1)
                        .is_some_and(|tag| tag.kind().ends_with("_tag_close"));
                    match (present.len(), has_body) {
                        (len, true) if len > 0 => self.add_diagnostic(
                            format!(
                                "can only have either a tag-body or one of these attributes: {}",
                                present.join(", ")
                            ),
                            DiagnosticSeverity::WARNING,
                            self.node_tag_range(node),
                        ),
                        (len, false) if len > 1 => self.add_diagnostic(
                            format!(
                                "can only have one of these attributes: {}",
                                present.join(", ")
                            ),
                            DiagnosticSeverity::WARNING,
                            self.node_tag_range(node),
                        ),
                        _ => {}
                    }
                }
                grammar::AttributeRule::OnlyOrBody(name)
                    if attributes.contains_key(*name)
                        && node
                            .child(node.child_count() - 1)
                            .is_some_and(|tag| tag.kind().ends_with("_tag_close")) =>
                {
                    self.add_diagnostic(
                        format!("can only have either a tag-body or the {} attribute", name),
                        DiagnosticSeverity::WARNING,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::OnlyWith(name1, name2)
                    if attributes.contains_key(*name1) && !attributes.contains_key(*name2) =>
                {
                    self.add_diagnostic(
                        format!("attribute {} is useless without attribute {}", name1, name2),
                        DiagnosticSeverity::WARNING,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::OnlyWithEither(name, names)
                    if attributes.contains_key(*name)
                        && !names.iter().any(|name| attributes.contains_key(*name)) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is useless without one of these attributes: {}",
                            name,
                            names.join(", ")
                        ),
                        DiagnosticSeverity::WARNING,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::OnlyWithEitherOrBody(name, names)
                    if attributes.contains_key(*name)
                        && !names.iter().any(|name| attributes.contains_key(*name))
                        && !node
                            .child(node.child_count() - 1)
                            .is_some_and(|tag| tag.kind().ends_with("_tag_close")) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is useless without either a tag-body or one of these attributes: {}",
                            name,
                            names.join(", ")
                        ),
                        DiagnosticSeverity::WARNING,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::Required(name) if !attributes.contains_key(*name) => {
                    self.add_diagnostic(
                        format!("missing required attribute {}", name),
                        DiagnosticSeverity::ERROR,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::UriExists(uri_name, module_name) => {
                    if let Some(uri) = attributes.get(*uri_name) {
                        if uri.contains("${") {
                            continue;
                        }
                        let module_value = attributes.get(*module_name).map(|str| str.as_str());
                        let module = match module_value {
                            Some("${module.id}") | None => self
                                .file
                                .to_file_path()
                                .ok()
                                .and_then(|file| modules::find_module_for_file(file.as_path())),
                            Some(module) => modules::find_module_by_name(module),
                        };
                        match module {
                            Some(module) => {
                                let file = format!("{}{}", module.path, uri);
                                if !Path::new(&file).exists() {
                                    self.add_diagnostic(
                                        format!("included file {} does not exist", file),
                                        DiagnosticSeverity::ERROR,
                                        self.node_tag_range(node),
                                    );
                                }
                            }
                            None => self.add_diagnostic(
                                match module_value {
                                    Some("${module.id}") | None => {
                                        "current module not listed in module-file".to_string()
                                    }
                                    Some(name) => {
                                        format!("module \"{}\" not listed in module-file", name)
                                    }
                                },
                                DiagnosticSeverity::HINT,
                                self.node_tag_range(node),
                            ),
                        }
                    }
                }
                grammar::AttributeRule::ValueOneOf(name, values)
                    if attributes
                        .get(*name)
                        .is_some_and(|v| !v.contains("${") && !values.contains(&v.as_str())) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} should be one of these values: [{}]",
                            name,
                            values.join(", ")
                        ),
                        DiagnosticSeverity::ERROR,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::ValueOneOfCaseInsensitive(name, values)
                    if attributes.get(*name).is_some_and(|v| {
                        !v.contains("${") && !values.contains(&v.to_uppercase().as_str())
                    }) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} should be one of these values: [{}]",
                            name,
                            values.join(", ")
                        ),
                        DiagnosticSeverity::ERROR,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::OnlyWithValue(name, attribute, value)
                    if attributes.contains_key(*name)
                        && !attributes.get(*attribute).is_some_and(|v| v == value) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is useless without attribute {} containing the value {}",
                            name, attribute, value
                        ),
                        DiagnosticSeverity::WARNING,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::OnlyWithEitherValue(name, attribute, values)
                    if attributes.contains_key(*name)
                        && !attributes
                            .get(*attribute)
                            .is_some_and(|v| v.contains("${") || values.contains(&v.as_str())) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is useless without attribute {} containing one of these values: [{}]",
                            name, attribute, values.join(", ")
                        ),
                        DiagnosticSeverity::WARNING,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::BodyOnlyWithEitherValue(attribute, values)
                    if node
                        .child(node.child_count() - 1)
                        .is_some_and(|tag| tag.kind().ends_with("_tag_close"))
                        && !attributes
                            .get(*attribute)
                            .is_some_and(|v| values.contains(&v.as_str())) =>
                {
                    self.add_diagnostic(
                        format!(
                            "tag-body is useless without attribute {} containing one of these values: [{}]",
                            attribute, values.join(", ")
                        ),
                        DiagnosticSeverity::WARNING,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::RequiredWithValue(name, attribute, value)
                    if attributes.get(*attribute).is_some_and(|v| v == value)
                        && !attributes.contains_key(*name) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is required when attribute {} is {}",
                            name, attribute, value
                        ),
                        DiagnosticSeverity::ERROR,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::RequiredOrBodyWithValue(name, attribute, value)
                    if attributes.get(*attribute).is_some_and(|v| v == value) =>
                {
                    let has_attribute = attributes.contains_key(*name);
                    let has_body = node
                        .child(node.child_count() - 1)
                        .is_some_and(|tag| tag.kind().ends_with("_tag_close"));
                    match (has_attribute, has_body) {
                        (false, false) => self.add_diagnostic(
                            format!(
                                "either attribute {} or a tag-body is required when attribute {} is \"{}\"",
                                name,
                                attribute,
                                value
                            ),
                            DiagnosticSeverity::ERROR,
                            self.node_tag_range(node),
                        ),
                        (true, true) => self.add_diagnostic(
                            format!(
                                "exactly one of attribute {} or a tag-body is required when attribute {} is \"{}\"",
                                name,
                                attribute,
                                value
                            ),
                            DiagnosticSeverity::ERROR,
                            self.node_range(node),
                        ),
                        _ => {}
                    }
                }
                grammar::AttributeRule::RequiredWithEitherValue(name, attribute, values)
                    if attributes
                        .get(*attribute)
                        .is_some_and(|v| values.contains(&v.as_str()))
                        && !attributes.contains_key(*name) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is required when attribute {} is either of [{}]",
                            name,
                            attribute,
                            values.join(", ")
                        ),
                        DiagnosticSeverity::ERROR,
                        self.node_tag_range(node),
                    );
                }
                grammar::AttributeRule::ExactlyOneOfOrBodyWithValue(names, attribute, value)
                    if attributes.get(*attribute).is_some_and(|v| v == value) =>
                {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| attributes.contains_key(*name))
                        .collect();
                    let has_body = node
                        .child(node.child_count() - 1)
                        .is_some_and(|tag| tag.kind().ends_with("_tag_close"));
                    match (present.len(), has_body) {
                        (0, false) => {
                            self.add_diagnostic(
                                format!(
                                    "when attribute {} is \"{}\" either a tag-body or exactly one of these attributes is required: [{}]",
                                    attribute, value, names.join(", ")
                                ),
                                DiagnosticSeverity::ERROR,
                                self.node_tag_range(node),
                            );
                        }
                        (0, true) | (1, false) => {}
                        _ => self.add_diagnostic(
                            format!(
                                "when attribute {} is \"{}\" only one of a tag-body and these attributes is required: [{}]",
                                attribute, value, names.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            self.node_tag_range(node),
                        ),
                    }
                }
                grammar::AttributeRule::ExactlyOneOfOrBodyWithEitherValue(
                    names,
                    attribute,
                    values,
                ) if attributes
                    .get(*attribute)
                    .is_some_and(|v| values.contains(&v.as_str())) =>
                {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| attributes.contains_key(*name))
                        .collect();
                    let has_body = node
                        .child(node.child_count() - 1)
                        .is_some_and(|tag| tag.kind().ends_with("_tag_close"));
                    match (present.len(), has_body) {
                        (0, false) => self.add_diagnostic(
                            format!(
                                "when attribute {} is either of [{}] either a tag-body or exactly one of these attributes is required: [{}]",
                                attribute, values.join(", "), names.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            self.node_tag_range(node),
                        ),
                        (0, true) | (1, false) => {}
                        _ => self.add_diagnostic(
                            format!(
                                "when attribute {} is either of [{}] only one of a tag-body and these attributes is required: [{}]",
                                attribute, values.join(", "), names.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            self.node_tag_range(node),
                        )
                    }
                }
                _ => {}
            }
        }
        return Ok(());
    }

    fn can_have_child(self: &Self, tag: &TagDefinition, child: &TagDefinition) -> bool {
        return match &tag.children {
            TagChildren::Any => true,
            TagChildren::None => false,
            TagChildren::Scalar(tag) => *child == **tag,
            TagChildren::Vector(tags) => tags.iter().any(|tag| child == tag),
        };
    }

    fn validate_children(
        self: &mut Self,
        node: &Node,
        spel: &HashMap<Point, SpelAst>,
    ) -> Result<()> {
        for child in node.children(&mut node.walk()) {
            match child.kind() {
                "ERROR" => self.add_diagnostic(
                    format!("unexpected \"{}\"", child.utf8_text(self.text.as_bytes())?),
                    DiagnosticSeverity::ERROR,
                    self.node_range(&child),
                ),
                "text" => {
                    // TODO: what tags can/cannot have text?
                }
                "html_tag" | "html_option_tag" | "html_void_tag" | "java_tag" | "script_tag"
                | "style_tag" => {
                    self.validate_children(&child, spel)?;
                }
                kind if kind.ends_with("_tag") => match &TagDefinition::from_str(kind) {
                    Ok(child_tag) => self.validate_tag(child_tag, &child, spel)?,
                    Err(err) => {
                        log::info!("expected sp or spt tag: {}", err);
                    }
                },
                _ => self.validate_children(&child, spel)?,
            }
        }
        return Ok(());
    }

    fn add_diagnostic(
        self: &mut Self,
        message: String,
        severity: DiagnosticSeverity,
        range: Range,
    ) {
        self.diagnostics.push(Diagnostic {
            message,
            severity: Some(severity),
            range,
            source: Some(String::from("lspml")),
            ..Default::default()
        });
    }

    fn add_diagnostic_with_tag(
        self: &mut Self,
        message: String,
        severity: DiagnosticSeverity,
        range: Range,
        tags: DiagnosticTag,
    ) {
        self.diagnostics.push(Diagnostic {
            message,
            severity: Some(severity),
            range,
            source: Some(String::from("lspml")),
            tags: Some(vec![tags]),
            ..Default::default()
        });
    }

    fn add_diagnostic_with_code(
        self: &mut Self,
        message: String,
        severity: DiagnosticSeverity,
        range: Range,
        code: NumberOrString,
        data: Option<serde_json::Value>,
    ) {
        self.diagnostics.push(Diagnostic {
            message,
            severity: Some(severity),
            range,
            source: Some(String::from("lspml")),
            code: Some(code),
            data,
            ..Default::default()
        });
    }

    fn node_range(self: &Self, node: &Node) -> Range {
        return Range {
            start: Position {
                line: node.start_position().row as u32,
                character: node.start_position().column as u32,
            },
            end: Position {
                line: node.end_position().row as u32,
                character: node.end_position().column as u32,
            },
        };
    }

    fn node_tag_range(self: &Self, node: &Node) -> Range {
        let mut closing = None;
        for child in node.children(&mut node.walk()) {
            if child.kind() == ">" {
                closing = Some(child);
                break;
            }
        }
        return Range {
            start: Position {
                line: node.start_position().row as u32,
                character: node.start_position().column as u32,
            },
            end: Position {
                line: closing.as_ref().unwrap_or(node).end_position().row as u32,
                character: closing.as_ref().unwrap_or(node).end_position().column as u32,
            },
        };
    }
}

struct SpelValidator<'a> {
    collector: &'a mut DiagnosticCollector,
    offset: Point,
}

impl SpelValidator<'_> {
    fn new<'a>(collector: &'a mut DiagnosticCollector, offset: Point) -> SpelValidator<'a> {
        return SpelValidator { collector, offset };
    }

    fn validate_identifier(self: &mut Self, identifier: &ast::Identifier) -> Result<()> {
        match identifier {
            ast::Identifier::Name(name) => {
                self.validate_interpolations_in_word(&name)?;
            }
            ast::Identifier::FieldAccess {
                identifier, field, ..
            } => {
                self.validate_identifier(identifier)?;
                self.validate_interpolations_in_word(&field)?;
            }
        };
        return Ok(());
    }

    fn validate_object(self: &mut Self, object: &ast::Object) -> Result<()> {
        match object {
            ast::Object::Anchor(anchor) => {
                self.validate_interpolations_in_word(&anchor.name)?;
            }
            ast::Object::Function(function) => self.validate_global_function(function)?,
            ast::Object::Name(name) => {
                self.validate_interpolations_in_word(name)?;
            }
            // ast::Object::Null(null) => todo!(),
            // ast::Object::String(string) => todo!(),
            ast::Object::FieldAccess {
                object, /* field, */
                ..
            } => {
                self.validate_object(object)?;
            }
            ast::Object::MethodAccess {
                object, /* function, */
                ..
            } => {
                self.validate_object(object)?;
                // validate_method(*object, diagnostics, offset)?;
            }
            ast::Object::ArrayAccess { object, index, .. } => {
                self.validate_object(object)?;
                self.validate_expression(index)?;
            }
            _ => {}
        };
        return Ok(());
    }

    fn validate_expression(self: &mut Self, expression: &ast::Expression) -> Result<()> {
        match expression {
            // ast::Expression::Number(number) => todo!(),
            // ast::Expression::Null(null) => todo!(),
            ast::Expression::Function(function) => self.validate_global_function(function)?,
            ast::Expression::Object(interpolation) => {
                self.validate_object(&interpolation.content)?;
            }
            ast::Expression::SignedExpression { expression, .. } => {
                self.validate_expression(expression)?
            }
            ast::Expression::BracketedExpression { expression, .. } => {
                self.validate_expression(expression)?
            }
            ast::Expression::BinaryOperation { left, right, .. } => {
                self.validate_expression(left)?;
                self.validate_expression(right)?;
            }
            ast::Expression::Ternary {
                condition,
                left,
                right,
                ..
            } => {
                self.validate_condition(condition)?;
                self.validate_expression(left)?;
                self.validate_expression(right)?;
            }
            _ => {}
        };
        return Ok(());
    }

    fn validate_condition(self: &mut Self, condition: &ast::Condition) -> Result<()> {
        match condition {
            ast::Condition::Object(ast::Interpolation { content, .. }) => {
                self.validate_object(content)?;
            }
            ast::Condition::Function(function) => self.validate_global_function(function)?,
            ast::Condition::BracketedCondition { condition, .. } => {
                self.validate_condition(condition)?
            }
            ast::Condition::NegatedCondition { condition, .. } => {
                self.validate_condition(condition)?
            }
            ast::Condition::BinaryOperation { left, right, .. } => {
                self.validate_condition(left)?;
                self.validate_condition(right)?;
            }
            ast::Condition::Comparisson { left, right, .. } => {
                self.validate_comparable(left)?;
                self.validate_comparable(right)?;
            }
            _ => {}
        };
        return Ok(());
    }

    fn validate_comparable(self: &mut Self, comparable: &ast::Comparable) -> Result<()> {
        match comparable {
            ast::Comparable::Condition(condition) => self.validate_condition(condition),
            ast::Comparable::Expression(expression) => self.validate_expression(expression),
            ast::Comparable::Function(function) => self.validate_global_function(function),
            ast::Comparable::Object(interpolation) => self.validate_object(&interpolation.content),
            // ast::Comparable::String(string) => todo!(),
            // ast::Comparable::Null(null) => todo!(),
            _ => Ok(()),
        }
    }

    fn validate_global_function(self: &mut Self, function: &ast::Function) -> Result<()> {
        let argument_count = function.arguments.len();
        match spel::grammar::Function::from_str(function.name.as_str()) {
            Ok(definition) => match definition.argument_number {
                ArgumentNumber::AtLeast(number) if argument_count < number => {
                    self.collector.add_diagnostic(
                        format!(
                            "invalid arguments number to \"{}\", expected {} or more but got {}",
                            definition.name, number, argument_count,
                        ),
                        DiagnosticSeverity::ERROR,
                        self.locations_range(
                            &function.name_location,
                            &function.closing_bracket_location,
                        ),
                    )
                }
                ArgumentNumber::Exactly(number) if argument_count != number => {
                    self.collector.add_diagnostic(
                        format!(
                            "invalid arguments number to \"{}\", expected {} but got {}",
                            definition.name, number, argument_count,
                        ),
                        DiagnosticSeverity::ERROR,
                        self.locations_range(
                            &function.name_location,
                            &function.closing_bracket_location,
                        ),
                    );
                }
                ArgumentNumber::None if argument_count != 0 => self.collector.add_diagnostic(
                    format!(
                        "invalid arguments number to \"{}\", expected 0 but got {}",
                        definition.name, argument_count,
                    ),
                    DiagnosticSeverity::ERROR,
                    self.locations_range(
                        &function.name_location,
                        &function.closing_bracket_location,
                    ),
                ),
                _ => {}
            },
            Err(err) => self.collector.add_diagnostic(
                err.to_string(),
                DiagnosticSeverity::ERROR,
                self.locations_range(&function.name_location, &function.closing_bracket_location),
            ),
        }
        for argument in &function.arguments {
            match &argument.argument {
                ast::Argument::Anchor(anchor) => {
                    self.validate_interpolations_in_word(&anchor.name)?;
                }
                ast::Argument::Function(function) => self.validate_global_function(&function)?,
                // ast::Argument::Null(_) => todo!(),
                // ast::Argument::Number(_) => todo!(),
                ast::Argument::Object(interpolation) => {
                    self.validate_object(&interpolation.content)?
                }
                // ast::Argument::SignedNumber(_) => todo!(),
                // ast::Argument::String(_) => todo!(),
                _ => {}
            };
        }
        return Ok(());
    }

    fn validate_query(self: &mut Self, _query: &ast::Query) -> Result<()> {
        // TODO!
        return Ok(());
    }

    fn validate_regex(self: &mut Self, _regex: &ast::Regex) -> Result<()> {
        // TODO!
        return Ok(());
    }

    fn validate_uri(self: &mut Self, _uri: &ast::Uri) -> Result<()> {
        // TODO!
        return Ok(());
    }

    fn validate_interpolations_in_word(self: &mut Self, word: &ast::Word) -> Result<()> {
        for fragment in &word.fragments {
            if let ast::WordFragment::Interpolation(interpolation) = fragment {
                self.validate_object(&interpolation.content)?;
            }
        }
        return Ok(());
    }

    fn locations_range(self: &Self, left: &ast::Location, right: &ast::Location) -> Range {
        return Range {
            start: Position {
                line: left.line() as u32 + self.offset.row as u32,
                character: left.char() as u32 + self.offset.column as u32,
            },
            end: Position {
                line: right.line() as u32 + self.offset.row as u32,
                character: right.char() as u32 + right.len() as u32 + self.offset.column as u32,
            },
        };
    }

    fn validate<'a>(
        collector: &'a mut DiagnosticCollector,
        node: &'a Node<'a>,
        spel: &HashMap<Point, SpelAst>,
    ) -> Result<()> {
        let offset = node.start_position();
        let mut validator = SpelValidator::new(collector, offset);
        match spel.get(&offset) {
            Some(SpelAst::Comparable(result)) => match result {
                SpelResult::Valid(comparable) => validator.validate_comparable(comparable)?,
                SpelResult::Invalid(err) => validator.parse_failed(node, err, "comparable"),
            },
            Some(SpelAst::Condition(result)) => match result {
                SpelResult::Valid(result) => validator.validate_condition(result)?,
                SpelResult::Invalid(err) => validator.parse_failed(node, err, "condition"),
            },
            Some(SpelAst::Expression(result)) => match result {
                SpelResult::Valid(result) => validator.validate_expression(result)?,
                SpelResult::Invalid(err) => validator.parse_failed(node, err, "expression"),
            },
            Some(SpelAst::Identifier(result)) => match result {
                SpelResult::Valid(result) => validator.validate_identifier(result)?,
                SpelResult::Invalid(err) => validator.parse_failed(node, err, "identifier"),
            },
            Some(SpelAst::Object(result)) => match result {
                SpelResult::Valid(result) => validator.validate_object(result)?,
                SpelResult::Invalid(err) => validator.parse_failed(node, err, "object"),
            },
            Some(SpelAst::Query(result)) => match result {
                SpelResult::Valid(query) => validator.validate_query(query)?,
                SpelResult::Invalid(err) => validator.parse_failed(node, err, "query"),
            },
            Some(SpelAst::Regex(result)) => match result {
                SpelResult::Valid(regex) => validator.validate_regex(regex)?,
                SpelResult::Invalid(err) => validator.parse_failed(node, err, "regex"),
            },
            Some(SpelAst::String(result)) => match result {
                SpelResult::Valid(word) => validator.validate_interpolations_in_word(word)?,
                SpelResult::Invalid(err) => validator.parse_failed(node, err, "text"),
            },
            Some(SpelAst::Uri(result)) => match result {
                SpelResult::Valid(uri) => validator.validate_uri(uri)?,
                SpelResult::Invalid(err) => validator.parse_failed(node, err, "uri"),
            },
            _ => (),
        };
        return Ok(());
    }

    fn parse_failed(self: &mut Self, node: &Node<'_>, err: &SyntaxError, r#type: &str) -> () {
        match err.proposed_fixes.len() {
            0 => self.collector.add_diagnostic(
                format!("invalid {}: {}", r#type, err.message),
                DiagnosticSeverity::ERROR,
                self.collector.node_range(node),
            ),
            _ => {
                let offset = node.start_position();
                self.collector.add_diagnostic_with_code(
                    format!("invalid {}: {}", r#type, err.message),
                    DiagnosticSeverity::ERROR,
                    self.collector.node_range(node),
                    CodeActionImplementation::FIX_SPEL_SYNTAX_CODE,
                    serde_json::to_value(
                        err.proposed_fixes
                            .iter()
                            .map(|fix| fix.to_text_edit(&offset))
                            .collect::<Vec<TextEdit>>(),
                    )
                    .ok(),
                );
            }
        }
    }
}

pub(crate) fn diagnostic(params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>, LsError> {
    let uri = params.text_document.uri;
    let document = match document_store::get(&uri) {
        Some(document) => Ok(document),
        None => document_store::Document::from_uri(&uri)
            .map(|document| document_store::put(&uri, document))
            .map_err(|err| {
                log::error!("failed to read {}: {}", uri, err);
                return LsError {
                    message: format!("cannot read file {}", uri),
                    code: ErrorCode::RequestFailed,
                };
            }),
    }?;
    let mut collector = DiagnosticCollector::new(uri, document.text);
    collector
        .validate_document(&document.tree.root_node(), &document.spel)
        .map_err(|err| LsError {
            message: format!("failed to validate document: {}", err),
            code: ErrorCode::RequestFailed,
        })?;
    return Ok(collector.diagnostics);
}
