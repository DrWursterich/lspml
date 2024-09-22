use std::{path::Path, str::FromStr};

use anyhow::Result;
use lsp_server::ErrorCode;
use lsp_types::{
    Diagnostic, DiagnosticSeverity, DiagnosticTag, DocumentDiagnosticParams, NumberOrString,
    Position, Range, TextEdit, Uri as Url,
};

use crate::{
    capabilities::CodeActionImplementation,
    document_store,
    grammar::AttributeRule,
    modules,
    parser::{
        AttributeError, DocumentNode, ErrorNode, Header, HtmlNode, Node, ParsableTag,
        ParsedAttribute, ParsedLocation, ParsedNode, ParsedTag, RangedNode, SpelAttribute,
        SpelAttributeValue, SpmlTag, TagError, Tree,
    },
    spel::{
        ast::{
            Argument, Comparable, Condition, Expression, Function, Identifier, Interpolation,
            Location, Object, Query, Regex, SpelAst, SpelResult, StringLiteral, Uri, Word,
            WordFragment,
        },
        grammar::{self, ArgumentNumber},
        parser::SyntaxError,
    },
};

use super::LsError;

pub(crate) struct DiagnosticCollector {
    pub(crate) file: Url,
    pub(crate) diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollector {
    pub(crate) fn new(file: Url) -> DiagnosticCollector {
        return DiagnosticCollector {
            file,
            diagnostics: Vec::new(),
        };
    }

    pub(crate) fn validate_document(&mut self, tree: &Tree) -> Result<()> {
        self.validate_header(&tree.header)?;
        self.validate_nodes(&tree.nodes)?;
        return Ok(());
    }

    fn validate_header(&mut self, header: &Header) -> Result<()> {
        if header.java_headers.len() == 0 && header.taglib_imports.len() == 0 {
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
        for header in &header.java_headers {
            match header {
                ParsedNode::Valid(_header) => (),
                ParsedNode::Incomplete(header) => {
                    if let Some(range) = header.range() {
                        match &header.open_bracket {
                            ParsedLocation::Valid(_) => (),
                            ParsedLocation::Erroneous(location) => self.add_diagnostic(
                                "invalid java header opening bracket. should be '<%@'".to_string(),
                                DiagnosticSeverity::ERROR,
                                location.range(),
                            ),
                            ParsedLocation::Missing => self.add_diagnostic(
                                "invalid java header: missing '<%@'".to_string(),
                                DiagnosticSeverity::ERROR,
                                range,
                            ),
                        };
                        match &header.page {
                            ParsedLocation::Valid(_) => (),
                            ParsedLocation::Erroneous(location) => self.add_diagnostic(
                                "invalid java header 'page'".to_string(),
                                DiagnosticSeverity::ERROR,
                                location.range(),
                            ),
                            ParsedLocation::Missing => self.add_diagnostic(
                                "invalid java header: missing 'page'".to_string(),
                                DiagnosticSeverity::ERROR,
                                range,
                            ),
                        };
                        match &header.close_bracket {
                            ParsedLocation::Valid(_) => (),
                            ParsedLocation::Erroneous(location) => self.add_diagnostic(
                                "invalid java header closing bracket. should be '%>'".to_string(),
                                DiagnosticSeverity::ERROR,
                                location.range(),
                            ),
                            ParsedLocation::Missing => self.add_diagnostic(
                                "java header is unclosed".to_string(),
                                DiagnosticSeverity::ERROR,
                                range,
                            ),
                        }
                    }
                }
            }
        }
        for header in &header.taglib_imports {
            match header {
                ParsedNode::Valid(_header) => (),
                ParsedNode::Incomplete(header) => {
                    if let Some(range) = header.range() {
                        match &header.open_bracket {
                            ParsedLocation::Valid(_) => (),
                            ParsedLocation::Erroneous(location) => self.add_diagnostic(
                                "invalid taglib header opening bracket. should be '<%@'"
                                    .to_string(),
                                DiagnosticSeverity::ERROR,
                                location.range(),
                            ),
                            ParsedLocation::Missing => self.add_diagnostic(
                                "invalid taglib header: missing '<%@'".to_string(),
                                DiagnosticSeverity::ERROR,
                                range,
                            ),
                        };
                        match &header.taglib {
                            ParsedLocation::Valid(_) => (),
                            ParsedLocation::Erroneous(location) => self.add_diagnostic(
                                "invalid taglib header 'taglib'".to_string(),
                                DiagnosticSeverity::ERROR,
                                location.range(),
                            ),
                            ParsedLocation::Missing => self.add_diagnostic(
                                "invalid taglib header: missing 'taglib'".to_string(),
                                DiagnosticSeverity::ERROR,
                                range,
                            ),
                        };
                        match &header.origin {
                            Some(_) => (),
                            None => self.add_diagnostic(
                                "invalid taglib header: missing 'uri' or 'tagdir' attribute"
                                    .to_string(),
                                DiagnosticSeverity::ERROR,
                                range,
                            ),
                        };
                        match &header.prefix {
                            Some(_) => (),
                            None => self.add_diagnostic(
                                "invalid taglib header: missing 'prefix' attribute".to_string(),
                                DiagnosticSeverity::ERROR,
                                range,
                            ),
                        };
                        match &header.close_bracket {
                            ParsedLocation::Valid(_) => (),
                            ParsedLocation::Erroneous(location) => self.add_diagnostic(
                                "invalid taglib header closing bracket. should be '%>'".to_string(),
                                DiagnosticSeverity::ERROR,
                                location.range(),
                            ),
                            ParsedLocation::Missing => self.add_diagnostic(
                                "taglib header is unclosed".to_string(),
                                DiagnosticSeverity::ERROR,
                                range,
                            ),
                        }
                        for error in &header.errors {
                            self.add_diagnostic(
                                format!("syntax error: unexpected \"{}\"", error.content),
                                DiagnosticSeverity::ERROR,
                                error.range,
                            )
                        }
                    }
                }
            }
        }
        return Ok(());
    }

    fn validate_nodes(&mut self, nodes: &Vec<Node>) -> Result<()> {
        for node in nodes {
            match node {
                Node::Tag(ParsedTag::Valid(tag)) => self.validate_tag(tag)?,
                Node::Tag(ParsedTag::Erroneous(tag, errors)) => {
                    for error in errors {
                        match error {
                            TagError::Superfluous(text, location) => {
                                self.add_superfluous_diagnostic(text, location.range());
                            }
                            TagError::Missing(text, location) => {
                                self.add_diagnostic_with_code(
                                    format!("\"{}\" is missing", text),
                                    DiagnosticSeverity::ERROR,
                                    node.range(),
                                    CodeActionImplementation::ADD_MISSING_CODE,
                                    serde_json::to_value(TextEdit {
                                        range: location.range(),
                                        new_text: text.to_string(),
                                    })
                                    .ok(),
                                );
                            }
                        }
                    }
                    self.validate_tag(tag)?;
                }
                Node::Tag(ParsedTag::Unparsable(message, location)) => {
                    self.add_diagnostic(
                        message.to_string(),
                        DiagnosticSeverity::ERROR,
                        location.range(),
                    );
                }
                Node::Html(html) => self.validate_html(html)?,
                Node::Text(_) => (),
                Node::Error(ErrorNode { content, range }) => {
                    self.add_diagnostic(
                        format!("syntax error: unexpected \"{}\"", content),
                        DiagnosticSeverity::ERROR,
                        *range,
                    );
                }
            }
        }
        return Ok(());
    }

    fn validate_tag(&mut self, tag: &SpmlTag) -> Result<()> {
        if tag.definition().deprecated {
            self.add_diagnostic_with_tag(
                format!("{} tag is deprecated", tag.definition().name),
                DiagnosticSeverity::INFORMATION,
                tag.range(),
                DiagnosticTag::DEPRECATED,
            );
        }
        for (_, attribute) in tag.spel_attributes() {
            let attribute = match attribute {
                ParsedAttribute::Valid(attribute) => attribute,
                ParsedAttribute::Erroneous(attribute, errors) => {
                    for error in errors {
                        match error {
                            AttributeError::Superfluous(text, location) => {
                                self.add_superfluous_diagnostic(text, location.range());
                            }
                        }
                    }
                    attribute
                }
                ParsedAttribute::Unparsable(message, location) => {
                    self.add_diagnostic(
                        message.to_string(),
                        DiagnosticSeverity::ERROR,
                        location.range(),
                    );
                    continue;
                }
            };
            SpelValidator::validate(self, &attribute)?;
        }
        for rule in tag.definition().attribute_rules {
            match rule {
                AttributeRule::Deprecated(name) if tag.spel_attribute(*name).is_some() => {
                    self.add_diagnostic_with_tag(
                        format!("attribute {} is deprecated", name),
                        DiagnosticSeverity::INFORMATION,
                        tag.range(),
                        DiagnosticTag::DEPRECATED,
                    );
                }
                AttributeRule::AtleastOneOf(names)
                    if !names.iter().any(|name| tag.spel_attribute(*name).is_some()) =>
                {
                    self.add_diagnostic(
                        format!(
                            "requires atleast one of these attributes: {}",
                            names.join(", ")
                        ),
                        DiagnosticSeverity::ERROR,
                        tag.range(),
                    );
                }
                AttributeRule::ExactlyOneOf(names) => {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| tag.spel_attribute(*name).is_some())
                        .collect();
                    match present.len() {
                        0 => self.add_diagnostic(
                            format!("requires one of these attributes: {}", names.join(", ")),
                            DiagnosticSeverity::ERROR,
                            tag.range(),
                        ),
                        1 => {}
                        _ => self.add_diagnostic(
                            format!(
                                "requires only one of these attributes: {}",
                                present.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            tag.range(),
                        ),
                    }
                }
                AttributeRule::ExactlyOneOfOrBody(names) => {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| tag.spel_attribute(*name).is_some())
                        .collect();
                    match (present.len(), tag.body().is_some()) {
                        (0, false) => self.add_diagnostic(
                            format!(
                                "requires either a tag-body or one of these attributes: {}",
                                names.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            tag.range(),
                        ),
                        (0, true) | (1, false) => {}
                        _ => self.add_diagnostic(
                            format!(
                                "requires either a tag-body or only one of these attributes: {}",
                                present.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            tag.range(),
                        ),
                    }
                }
                AttributeRule::ExactlyOrBody(name)
                    if tag.spel_attribute(*name).is_some() == tag.body().is_some() =>
                {
                    self.add_diagnostic(
                        format!("requires either a tag-body or the attribute {}", name,),
                        DiagnosticSeverity::ERROR,
                        tag.range(),
                    );
                }
                AttributeRule::OnlyOneOf(names) => {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| tag.spel_attribute(name).is_some())
                        .collect();
                    if present.len() > 1 {
                        self.add_diagnostic(
                            format!(
                                "can only have one of these attributes: {}",
                                present.join(", ")
                            ),
                            DiagnosticSeverity::WARNING,
                            tag.range(),
                        );
                    }
                }
                AttributeRule::OnlyOneOfOrBody(names) => {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| tag.spel_attribute(name).is_some())
                        .collect();
                    match (present.len(), tag.body().is_some()) {
                        (len, true) if len > 0 => self.add_diagnostic(
                            format!(
                                "can only have either a tag-body or one of these attributes: {}",
                                present.join(", ")
                            ),
                            DiagnosticSeverity::WARNING,
                            tag.range(),
                        ),
                        (len, false) if len > 1 => self.add_diagnostic(
                            format!(
                                "can only have one of these attributes: {}",
                                present.join(", ")
                            ),
                            DiagnosticSeverity::WARNING,
                            tag.range(),
                        ),
                        _ => {}
                    }
                }
                AttributeRule::OnlyOrBody(name)
                    if tag.spel_attribute(*name).is_some() && tag.body().is_some() =>
                {
                    self.add_diagnostic(
                        format!("can only have either a tag-body or the {} attribute", name),
                        DiagnosticSeverity::WARNING,
                        tag.range(),
                    );
                }
                AttributeRule::OnlyWith(name1, name2)
                    if tag.spel_attribute(*name1).is_some()
                        && !tag.spel_attribute(*name2).is_some() =>
                {
                    self.add_diagnostic(
                        format!("attribute {} is useless without attribute {}", name1, name2),
                        DiagnosticSeverity::WARNING,
                        tag.range(),
                    );
                }
                AttributeRule::OnlyWithEither(name, names)
                    if tag.spel_attribute(*name).is_some()
                        && !names.iter().any(|name| tag.spel_attribute(*name).is_some()) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is useless without one of these attributes: {}",
                            name,
                            names.join(", ")
                        ),
                        DiagnosticSeverity::WARNING,
                        tag.range(),
                    );
                }
                AttributeRule::OnlyWithEitherOrBody(name, names)
                    if tag.spel_attribute(*name).is_some()
                        && !names.iter().any(|name| tag.spel_attribute(*name).is_some())
                        && tag.body().is_none() =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is useless without either a tag-body or one of these attributes: {}",
                            name,
                            names.join(", ")
                        ),
                        DiagnosticSeverity::WARNING,
                        tag.range(),
                    );
                }
                AttributeRule::Required(name) if !tag.spel_attribute(*name).is_some() => {
                    self.add_diagnostic(
                        format!("missing required attribute {}", name),
                        DiagnosticSeverity::ERROR,
                        tag.range(),
                    );
                }
                AttributeRule::UriExists(uri_name, module_name) => {
                    let uri_attribute = match tag.spel_attribute(*uri_name) {
                        Some(ParsedAttribute::Valid(attribute)) => Some(attribute),
                        Some(ParsedAttribute::Erroneous(attribute, _)) => Some(attribute),
                        _ => None,
                    };
                    if let Some(SpelAttribute {
                        value:
                            SpelAttributeValue {
                                spel: SpelAst::Uri(SpelResult::Valid(Uri::Literal(uri))),
                                ..
                            },
                        ..
                    }) = uri_attribute
                    {
                        let module_attribute = match tag.spel_attribute(*module_name) {
                            Some(ParsedAttribute::Valid(attribute)) => Some(attribute),
                            Some(ParsedAttribute::Erroneous(attribute, _)) => Some(attribute),
                            _ => None,
                        };
                        let module_value = match module_attribute {
                            Some(SpelAttribute {
                                value:
                                    SpelAttributeValue {
                                        spel: SpelAst::String(SpelResult::Valid(Word { fragments })),
                                        ..
                                    },
                                ..
                            }) if fragments.len() == 1 => Some(fragments[0].clone()),
                            Some(_) => continue,
                            None => None,
                        };
                        let module = match module_value {
                            Some(WordFragment::Interpolation(_)) | None => {
                                modules::find_module_for_file(Path::new(self.file.path().as_str()))
                            }
                            Some(WordFragment::String(StringLiteral { ref content, .. })) => {
                                modules::find_module_by_name(&content)
                            }
                        };
                        match module {
                            Some(module) => {
                                let file = format!("{}{}", module.path, uri);
                                if !Path::new(&file).exists() {
                                    self.add_diagnostic(
                                        format!("included file {} does not exist", file),
                                        DiagnosticSeverity::ERROR,
                                        tag.range(),
                                    );
                                }
                            }
                            None => self.add_diagnostic(
                                match module_value {
                                    Some(WordFragment::Interpolation(i)) => format!(
                                        "interpolation \"{}\" is interpreted as the current module, which is not listed in the module-file",
                                        i
                                    ),
                                    Some(WordFragment::String(StringLiteral {
                                        content, ..
                                    })) => {
                                        format!("module \"{}\" not listed in module-file", content)
                                    },
                                    None => "current module not listed in module-file".to_string(),
                                },
                                DiagnosticSeverity::HINT,
                                tag.range(),
                            ),
                        }
                    }
                }
                AttributeRule::ValueOneOf(name, values)
                    if string_literal_attribute_value(tag.spel_attribute(*name))
                        .is_some_and(|value| !values.contains(&value.as_str())) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} should be one of these values: [{}]",
                            name,
                            values.join(", ")
                        ),
                        DiagnosticSeverity::ERROR,
                        tag.range(),
                    );
                }
                AttributeRule::ValueOneOfCaseInsensitive(name, values)
                    if string_literal_attribute_value(tag.spel_attribute(*name))
                        .is_some_and(|value| !values.contains(&value.to_uppercase().as_str())) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} should be one of these (caseinsensitive) values: [{}]",
                            name,
                            values.join(", ")
                        ),
                        DiagnosticSeverity::ERROR,
                        tag.range(),
                    );
                }
                AttributeRule::OnlyWithValue(name, attribute, value)
                    if tag.spel_attribute(*name).is_some()
                        && !string_literal_attribute_value(tag.spel_attribute(*attribute))
                            .is_some_and(|v| &v.as_str() == value) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is useless without attribute {} containing the value {}",
                            name, attribute, value
                        ),
                        DiagnosticSeverity::WARNING,
                        tag.range(),
                    );
                }
                AttributeRule::OnlyWithEitherValue(name, attribute, values)
                    if tag.spel_attribute(*name).is_some()
                        && !string_literal_attribute_value(tag.spel_attribute(*attribute))
                            .is_some_and(|value| values.contains(&value.as_str())) =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is useless without attribute {} containing one of these values: [{}]",
                            name, attribute, values.join(", ")
                        ),
                        DiagnosticSeverity::WARNING,
                        tag.range(),
                    );
                }
                AttributeRule::BodyOnlyWithEitherValue(attribute, values)
                    if tag.body().is_some()
                        && !string_literal_attribute_value(tag.spel_attribute(*attribute))
                            .is_some_and(|value| values.contains(&value.as_str())) =>
                {
                    self.add_diagnostic(
                        format!(
                            "tag-body is useless without attribute {} containing one of these values: [{}]",
                            attribute, values.join(", ")
                        ),
                        DiagnosticSeverity::WARNING,
                        tag.range(),
                    );
                }
                AttributeRule::RequiredWithValue(name, attribute, value)
                    if string_literal_attribute_value(tag.spel_attribute(*attribute))
                        .is_some_and(|v| &v.as_str() == value)
                        && !tag.spel_attribute(*name).is_some() =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is required when attribute {} is {}",
                            name, attribute, value
                        ),
                        DiagnosticSeverity::ERROR,
                        tag.range(),
                    );
                }
                AttributeRule::RequiredOrBodyWithValue(name, attribute, value)
                    if string_literal_attribute_value(tag.spel_attribute(*attribute))
                        .is_some_and(|v| &v.as_str() == value) =>
                {
                    let has_attribute = tag.spel_attribute(*name).is_some();
                    let has_body = tag.body().is_some();
                    match (has_attribute, has_body) {
                        (false, false) => self.add_diagnostic(
                            format!(
                                "either attribute {} or a tag-body is required when attribute {} is \"{}\"",
                                name,
                                attribute,
                                value
                            ),
                            DiagnosticSeverity::ERROR,
                            tag.range(),
                        ),
                        (true, true) => self.add_diagnostic(
                            format!(
                                "exactly one of attribute {} or a tag-body is required when attribute {} is \"{}\"",
                                name,
                                attribute,
                                value
                            ),
                            DiagnosticSeverity::ERROR,
                            tag.range(),
                        ),
                        _ => {}
                    }
                }
                AttributeRule::RequiredWithEitherValue(name, attribute, values)
                    if string_literal_attribute_value(tag.spel_attribute(*attribute))
                        .is_some_and(|value| values.contains(&value.as_str()))
                        && !tag.spel_attribute(*name).is_some() =>
                {
                    self.add_diagnostic(
                        format!(
                            "attribute {} is required when attribute {} is either of [{}]",
                            name,
                            attribute,
                            values.join(", ")
                        ),
                        DiagnosticSeverity::ERROR,
                        tag.range(),
                    );
                }
                AttributeRule::ExactlyOneOfOrBodyWithValue(names, attribute, value)
                    if string_literal_attribute_value(tag.spel_attribute(*attribute))
                        .is_some_and(|v| &v.as_str() == value) =>
                {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| tag.spel_attribute(*name).is_some())
                        .collect();
                    let has_body = tag.body().is_some();
                    match (present.len(), has_body) {
                        (0, false) => {
                            self.add_diagnostic(
                                format!(
                                    "when attribute {} is \"{}\" either a tag-body or exactly one of these attributes is required: [{}]",
                                    attribute, value, names.join(", ")
                                ),
                                DiagnosticSeverity::ERROR,
                                tag.range(),
                            );
                        }
                        (0, true) | (1, false) => {}
                        _ => self.add_diagnostic(
                            format!(
                                "when attribute {} is \"{}\" only one of a tag-body and these attributes is required: [{}]",
                                attribute, value, names.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            tag.range(),
                        ),
                    }
                }
                AttributeRule::ExactlyOneOfOrBodyWithEitherValue(names, attribute, values)
                    if string_literal_attribute_value(tag.spel_attribute(*attribute))
                        .is_some_and(|value| values.contains(&value.as_str())) =>
                {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| tag.spel_attribute(*name).is_some())
                        .collect();
                    let has_body = tag.body().is_some();
                    match (present.len(), has_body) {
                        (0, false) => self.add_diagnostic(
                            format!(
                                "when attribute {} is either of [{}] either a tag-body or exactly one of these attributes is required: [{}]",
                                attribute, values.join(", "), names.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            tag.range(),
                        ),
                        (0, true) | (1, false) => {}
                        _ => self.add_diagnostic(
                            format!(
                                "when attribute {} is either of [{}] only one of a tag-body and these attributes is required: [{}]",
                                attribute, values.join(", "), names.join(", ")
                            ),
                            DiagnosticSeverity::ERROR,
                            tag.range(),
                        )
                    }
                }
                _ => {}
            }
        }
        return match tag.body() {
            Some(body) => self.validate_nodes(&body.nodes),
            None => Ok(()),
        };
    }

    fn validate_html(&mut self, html: &HtmlNode) -> Result<()> {
        // TODO: html attributes can contain spml tags
        for attribute in &html.attributes {
            match attribute {
                ParsedAttribute::Valid(_) => (),
                ParsedAttribute::Erroneous(_, errors) => {
                    for error in errors {
                        match error {
                            AttributeError::Superfluous(text, location) => {
                                self.add_superfluous_diagnostic(text, location.range());
                            }
                        }
                    }
                }
                ParsedAttribute::Unparsable(message, location) => {
                    self.add_diagnostic(
                        message.to_string(),
                        DiagnosticSeverity::ERROR,
                        location.range(),
                    );
                }
            }
        }
        return match html.body() {
            Some(body) => self.validate_nodes(&body.nodes),
            None => Ok(()),
        };
    }

    fn add_diagnostic(&mut self, message: String, severity: DiagnosticSeverity, range: Range) {
        self.diagnostics.push(Diagnostic {
            message,
            severity: Some(severity),
            range,
            source: Some(String::from("lspml")),
            ..Default::default()
        });
    }

    fn add_diagnostic_with_tag(
        &mut self,
        message: String,
        severity: DiagnosticSeverity,
        range: Range,
        tag: DiagnosticTag,
    ) {
        self.diagnostics.push(Diagnostic {
            message,
            severity: Some(severity),
            range,
            source: Some(String::from("lspml")),
            tags: Some(vec![tag]),
            ..Default::default()
        });
    }

    fn add_diagnostic_with_code(
        &mut self,
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

    fn add_superfluous_diagnostic(&mut self, text: &String, range: Range) {
        self.diagnostics.push(Diagnostic {
            message: format!("\"{}\" is superfluous", text),
            severity: Some(DiagnosticSeverity::ERROR),
            range,
            source: Some(String::from("lspml")),
            tags: Some(vec![DiagnosticTag::UNNECESSARY]),
            code: Some(CodeActionImplementation::REMOVE_SUPERFLUOUS_CODE),
            ..Default::default()
        });
    }
}

fn string_literal_attribute_value(
    attribute: Option<&ParsedAttribute<SpelAttribute>>,
) -> Option<&String> {
    let attribute = match attribute {
        Some(ParsedAttribute::Valid(attribute)) => attribute,
        Some(ParsedAttribute::Erroneous(attribute, _)) => attribute,
        _ => return None,
    };
    return match &attribute.value.spel {
        SpelAst::String(SpelResult::Valid(Word { fragments })) if fragments.len() == 1 => {
            match &fragments[0] {
                WordFragment::String(StringLiteral { content, .. }) => Some(content),
                _ => None,
            }
        }
        _ => None,
    };
}

struct SpelValidator<'a> {
    collector: &'a mut DiagnosticCollector,
    offset: Position,
}

impl SpelValidator<'_> {
    fn new<'a>(collector: &'a mut DiagnosticCollector, offset: Position) -> SpelValidator<'a> {
        return SpelValidator { collector, offset };
    }

    fn validate_identifier(&mut self, identifier: &Identifier) -> Result<()> {
        match identifier {
            Identifier::Name(name) => {
                self.validate_interpolations_in_word(&name)?;
            }
            Identifier::FieldAccess {
                identifier, field, ..
            } => {
                self.validate_identifier(identifier)?;
                self.validate_interpolations_in_word(&field)?;
            }
        };
        return Ok(());
    }

    fn validate_object(&mut self, object: &Object) -> Result<()> {
        match object {
            Object::Anchor(anchor) => {
                self.validate_interpolations_in_word(&anchor.name)?;
            }
            Object::Function(function) => self.validate_global_function(function)?,
            Object::Name(name) => {
                self.validate_interpolations_in_word(name)?;
            }
            // Object::Null(null) => todo!(),
            // Object::String(string) => todo!(),
            Object::FieldAccess {
                object, /* field, */
                ..
            } => {
                self.validate_object(object)?;
            }
            Object::MethodAccess {
                object, /* function, */
                ..
            } => {
                self.validate_object(object)?;
                // validate_method(*object, diagnostics, offset)?;
            }
            Object::ArrayAccess { object, index, .. } => {
                self.validate_object(object)?;
                self.validate_expression(index)?;
            }
            _ => {}
        };
        return Ok(());
    }

    fn validate_expression(&mut self, expression: &Expression) -> Result<()> {
        match expression {
            // Expression::Number(number) => todo!(),
            // Expression::Null(null) => todo!(),
            Expression::Function(function) => self.validate_global_function(function)?,
            Expression::Object(interpolation) => {
                self.validate_object(&interpolation.content)?;
            }
            Expression::SignedExpression { expression, .. } => {
                self.validate_expression(expression)?
            }
            Expression::BracketedExpression { expression, .. } => {
                self.validate_expression(expression)?
            }
            Expression::BinaryOperation { left, right, .. } => {
                self.validate_expression(left)?;
                self.validate_expression(right)?;
            }
            Expression::Ternary {
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

    fn validate_condition(&mut self, condition: &Condition) -> Result<()> {
        match condition {
            Condition::Object(Interpolation { content, .. }) => {
                self.validate_object(content)?;
            }
            Condition::Function(function) => self.validate_global_function(function)?,
            Condition::BracketedCondition { condition, .. } => {
                self.validate_condition(condition)?
            }
            Condition::NegatedCondition { condition, .. } => self.validate_condition(condition)?,
            Condition::BinaryOperation { left, right, .. } => {
                self.validate_condition(left)?;
                self.validate_condition(right)?;
            }
            Condition::Comparisson { left, right, .. } => {
                self.validate_comparable(left)?;
                self.validate_comparable(right)?;
            }
            _ => {}
        };
        return Ok(());
    }

    fn validate_comparable(&mut self, comparable: &Comparable) -> Result<()> {
        match comparable {
            Comparable::Condition(condition) => self.validate_condition(condition),
            Comparable::Expression(expression) => self.validate_expression(expression),
            Comparable::Function(function) => self.validate_global_function(function),
            Comparable::Object(interpolation) => self.validate_object(&interpolation.content),
            // Comparable::String(string) => todo!(),
            // Comparable::Null(null) => todo!(),
            _ => Ok(()),
        }
    }

    fn validate_global_function(&mut self, function: &Function) -> Result<()> {
        let argument_count = function.arguments.len();
        match grammar::Function::from_str(function.name.as_str()) {
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
                Argument::Anchor(anchor) => {
                    self.validate_interpolations_in_word(&anchor.name)?;
                }
                Argument::Function(function) => self.validate_global_function(&function)?,
                // Argument::Null(_) => todo!(),
                // Argument::Number(_) => todo!(),
                Argument::Object(interpolation) => self.validate_object(&interpolation.content)?,
                // Argument::SignedNumber(_) => todo!(),
                // Argument::String(_) => todo!(),
                _ => {}
            };
        }
        return Ok(());
    }

    fn validate_query(&mut self, _query: &Query) -> Result<()> {
        // TODO!
        return Ok(());
    }

    fn validate_regex(&mut self, _regex: &Regex) -> Result<()> {
        // TODO!
        return Ok(());
    }

    fn validate_uri(&mut self, _uri: &Uri) -> Result<()> {
        // TODO!
        return Ok(());
    }

    fn validate_interpolations_in_word(&mut self, word: &Word) -> Result<()> {
        for fragment in &word.fragments {
            if let WordFragment::Interpolation(interpolation) = fragment {
                self.validate_object(&interpolation.content)?;
            }
        }
        return Ok(());
    }

    fn locations_range(&self, left: &Location, right: &Location) -> Range {
        return Range {
            start: Position {
                line: left.line() as u32 + self.offset.line,
                character: left.char() as u32 + self.offset.character,
            },
            end: Position {
                line: right.line() as u32 + self.offset.line,
                character: right.char() as u32 + right.len() as u32 + self.offset.character,
            },
        };
    }

    fn validate<'a>(
        collector: &'a mut DiagnosticCollector,
        attribute: &SpelAttribute,
    ) -> Result<()> {
        let offset = Position {
            line: attribute.value.opening_quote_location.line as u32,
            character: attribute.value.opening_quote_location.char as u32 + 1,
        };
        let mut validator = SpelValidator::new(collector, offset);
        match &attribute.value.spel {
            SpelAst::Comparable(result) => match result {
                SpelResult::Valid(comparable) => validator.validate_comparable(comparable)?,
                SpelResult::Invalid(err) => validator.parse_failed(attribute, err, "comparable"),
            },
            SpelAst::Condition(result) => match result {
                SpelResult::Valid(result) => validator.validate_condition(result)?,
                SpelResult::Invalid(err) => validator.parse_failed(attribute, err, "condition"),
            },
            SpelAst::Expression(result) => match result {
                SpelResult::Valid(result) => validator.validate_expression(result)?,
                SpelResult::Invalid(err) => validator.parse_failed(attribute, err, "expression"),
            },
            SpelAst::Identifier(result) => match result {
                SpelResult::Valid(result) => validator.validate_identifier(result)?,
                SpelResult::Invalid(err) => validator.parse_failed(attribute, err, "identifier"),
            },
            SpelAst::Object(result) => match result {
                SpelResult::Valid(result) => validator.validate_object(result)?,
                SpelResult::Invalid(err) => validator.parse_failed(attribute, err, "object"),
            },
            SpelAst::Query(result) => match result {
                SpelResult::Valid(query) => validator.validate_query(query)?,
                SpelResult::Invalid(err) => validator.parse_failed(attribute, err, "query"),
            },
            SpelAst::Regex(result) => match result {
                SpelResult::Valid(regex) => validator.validate_regex(regex)?,
                SpelResult::Invalid(err) => validator.parse_failed(attribute, err, "regex"),
            },
            SpelAst::String(result) => match result {
                SpelResult::Valid(word) => validator.validate_interpolations_in_word(word)?,
                SpelResult::Invalid(err) => validator.parse_failed(attribute, err, "text"),
            },
            SpelAst::Uri(result) => match result {
                SpelResult::Valid(uri) => validator.validate_uri(uri)?,
                SpelResult::Invalid(err) => validator.parse_failed(attribute, err, "uri"),
            },
        };
        return Ok(());
    }

    fn parse_failed(&mut self, attribute: &SpelAttribute, err: &SyntaxError, r#type: &str) -> () {
        let range = Range {
            start: Position {
                line: attribute.value.opening_quote_location.line as u32,
                character: attribute.value.opening_quote_location.char as u32,
            },
            end: Position {
                line: attribute.value.closing_quote_location.line as u32,
                character: attribute.value.closing_quote_location.char as u32,
            },
        };
        match err.proposed_fixes.len() {
            0 => self.collector.add_diagnostic(
                format!("invalid {}: {}", r#type, err.message),
                DiagnosticSeverity::ERROR,
                range,
            ),
            _ => {
                let offset = Position {
                    line: attribute.value.opening_quote_location.line as u32,
                    character: attribute.value.opening_quote_location.char as u32 + 1,
                };
                self.collector.add_diagnostic_with_code(
                    format!("invalid {}: {}", r#type, err.message),
                    DiagnosticSeverity::ERROR,
                    range,
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
                log::error!("failed to read {:?}: {}", uri, err);
                return LsError {
                    message: format!("cannot read file {:?}", uri),
                    code: ErrorCode::RequestFailed,
                };
            }),
    }?;
    let mut collector = DiagnosticCollector::new(uri);
    collector
        .validate_document(&document.tree)
        .map_err(|err| LsError {
            message: format!("failed to validate document: {}", err),
            code: ErrorCode::RequestFailed,
        })?;
    return Ok(collector.diagnostics);
}
