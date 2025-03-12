use std::{
    cmp::Ordering,
    collections::HashMap,
    fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use anyhow::Result;
use ::grammar::AttributeRule;
use lsp_types::{
    CodeDescription, DiagnosticSeverity, DiagnosticTag, DocumentDiagnosticParams, NumberOrString,
    Position, Range, TextEdit, Uri as Url,
};

use capabilities::CodeActionImplementation;
use modules;
use parser::{
    AttributeError, DocumentNode, ErrorNode, Header, HtmlAttributeValueContent,
    HtmlAttributeValueFragment, HtmlNode, Node, ParsableTag, ParsedAttribute, ParsedHtml,
    ParsedLocation, ParsedNode, ParsedTag, RangedNode, SpelAttribute, SpelAttributeValue, SpmlTag,
    TagError, Tree,
};
use spel::{
    ast::{
        Argument, Comparable, Condition, Expression, Function, Identifier, Interpolation, Location,
        Object, Query, Regex, SpelAst, SpelResult, StringLiteral, Uri, Word, WordFragment,
    },
    grammar::{self, ArgumentNumber},
    parser::SyntaxError,
};


#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Severity,
    pub code: Option<NumberOrString>,
    pub code_description: Option<CodeDescription>,
    pub message: String,
    pub tags: Option<Vec<DiagnosticTag>>,
    pub data: Option<serde_json::Value>,
    pub fingerprint: Option<String>,
    pub r#type: Type,
}

impl Diagnostic {
    pub fn to_lsp_type(self) -> lsp_types::Diagnostic {
        return lsp_types::Diagnostic {
            range: self.range,
            severity: Some(match self.severity {
                Severity::Hint => DiagnosticSeverity::HINT,
                // Severity::Information => DiagnosticSeverity::INFORMATION,
                Severity::Warning => DiagnosticSeverity::WARNING,
                Severity::Error => DiagnosticSeverity::ERROR,
                Severity::Critical => DiagnosticSeverity::ERROR,
            }),
            code: self.code,
            code_description: None,
            source: Some(String::from("lspml")),
            message: self.message,
            related_information: None,
            tags: self.tags,
            data: self.data,
        };
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Default)]
pub enum Severity {
    Critical,
    Error,
    #[default]
    Warning,
    // Information,
    Hint,
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, Default)]
pub enum Type {
    MissingHeader,
    InvalidHeader,
    MissingValue,
    ConflictingValues,
    SuperfluousValue,
    InvalidLiteral,
    InvalidFunction,
    #[default]
    SyntaxError,
    Deprecation,
    MissingFile,
    UnknownModule,
}

impl clap::ValueEnum for Type {
    fn value_variants<'a>() -> &'a [Self] {
        static VARIANTS: [Type; 11] = [
            Type::MissingHeader,
            Type::InvalidHeader,
            Type::MissingValue,
            Type::ConflictingValues,
            Type::SuperfluousValue,
            Type::SyntaxError,
            Type::Deprecation,
            Type::InvalidLiteral,
            Type::InvalidFunction,
            Type::MissingFile,
            Type::UnknownModule,
        ];
        return &VARIANTS;
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        return Some(clap::builder::PossibleValue::new(match self {
            Type::MissingHeader => "MISSING_HEADER",
            Type::InvalidHeader => "INVALID_HEADER",
            Type::MissingValue => "MISSING_VALUE",
            Type::ConflictingValues => "CONFLICTING_VALUE",
            Type::SuperfluousValue => "SUPERFLUOUS_VALUE",
            Type::SyntaxError => "SYNTAX_ERROR",
            Type::Deprecation => "DEPRECATION",
            Type::InvalidLiteral => "INVALID_LITERAL",
            Type::InvalidFunction => "INVALID_FUNCTION",
            Type::MissingFile => "MISSING_FILE",
            Type::UnknownModule => "UNKNOWN_MODULE",
        }));
    }
}

impl ToString for Type {
    fn to_string(&self) -> String {
        return String::from(match self {
            Type::MissingHeader => "MISSING_HEADER",
            Type::InvalidHeader => "INVALID_HEADER",
            Type::MissingValue => "MISSING_VALUE",
            Type::ConflictingValues => "CONFLICTING_VALUE",
            Type::SuperfluousValue => "SUPERFLUOUS_VALUE",
            Type::SyntaxError => "SYNTAX_ERROR",
            Type::Deprecation => "DEPRECATION",
            Type::InvalidLiteral => "INVALID_LITERAL",
            Type::InvalidFunction => "INVALID_FUNCTION",
            Type::MissingFile => "MISSING_FILE",
            Type::UnknownModule => "UNKNOWN_MODULE",
        });
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct DiagnosticBuilder {
    range: Range,
    severity: Severity,
    code: Option<NumberOrString>,
    code_description: Option<CodeDescription>,
    message: String,
    tags: Option<Vec<DiagnosticTag>>,
    data: Option<serde_json::Value>,
    fingerprint: Option<String>,
    r#type: Type,
}

impl DiagnosticBuilder {
    pub fn new(message: impl ToString, range: Range, r#type: Type, severity: Severity) -> Self {
        let message: String = message.to_string();
        let mut hasher = DefaultHasher::new();
        message.hash(&mut hasher);
        let fingerprint = Some(hasher.finish().to_string());
        return DiagnosticBuilder {
            range,
            severity,
            message,
            r#type,
            fingerprint,
            ..Default::default()
        };
    }

    pub fn new_invalid_header(message: impl ToString, range: Range) -> Self {
        return Self::new(message, range, Type::InvalidHeader, Severity::Critical);
    }

    pub fn new_missing_value(message: impl ToString, range: Range) -> Self {
        return Self::new(message, range, Type::MissingValue, Severity::Critical);
    }

    pub fn new_conflicting_values(message: impl ToString, range: Range) -> Self {
        return Self::new(message, range, Type::ConflictingValues, Severity::Error);
    }

    pub fn new_superfluous_value(message: impl ToString, range: Range) -> Self {
        return Self::new(message, range, Type::SuperfluousValue, Severity::Warning)
            .with_tag(DiagnosticTag::UNNECESSARY)
            .with_code(CodeActionImplementation::REMOVE_SUPERFLUOUS_CODE);
    }

    pub fn new_syntax_error(message: impl ToString, range: Range) -> Self {
        return Self::new(message, range, Type::SyntaxError, Severity::Critical);
    }

    pub fn new_invalid_literal(message: impl ToString, range: Range) -> Self {
        return Self::new(message, range, Type::InvalidLiteral, Severity::Critical);
    }

    pub fn new_invalid_function(message: impl ToString, range: Range) -> Self {
        return Self::new(message, range, Type::InvalidFunction, Severity::Critical);
    }

    pub fn new_deprecation(message: impl ToString, range: Range) -> Self {
        return Self::new(message, range, Type::Deprecation, Severity::Warning)
            .with_tag(DiagnosticTag::DEPRECATED);
    }

    pub fn new_missing_file(message: impl ToString, range: Range) -> Self {
        return Self::new(message, range, Type::MissingFile, Severity::Error);
    }

    pub fn new_unknown_module(message: impl ToString, range: Range) -> Self {
        return Self::new(message, range, Type::UnknownModule, Severity::Hint);
    }

    pub fn with_code(mut self, code: NumberOrString) -> Self {
        self.code = Some(code);
        return self;
    }

    pub fn with_tag(mut self, tag: DiagnosticTag) -> Self {
        match self.tags {
            Some(ref mut tags) => tags.push(tag),
            None => self.tags = Some(vec![tag]),
        }
        return self;
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        return self;
    }

    // pub fn with_fingerprint(mut self, fingerprint: impl Into<String>) -> Self {
    //     self.fingerprint = Some(fingerprint.into());
    //     return self;
    // }

    pub fn build(self) -> Diagnostic {
        return Diagnostic {
            range: self.range,
            severity: self.severity,
            code: self.code,
            code_description: self.code_description,
            message: self.message,
            tags: self.tags,
            data: self.data,
            fingerprint: self.fingerprint,
            r#type: self.r#type,
        };
    }
}

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
            self.add(
                DiagnosticBuilder::new(
                    format!(
                        "missing atleast one header. Try generating one with the \"{}\" code-action",
                        CodeActionImplementation::GenerateDefaultHeaders
                    ),
                    Range::new(document_start, document_start),
                    Type::MissingHeader,
                    Severity::Critical,
                )
                .with_code(CodeActionImplementation::GENERATE_DEFAULT_HEADER_CODE),
            );
        }
        for header in &header.java_headers {
            match header {
                ParsedNode::Valid(_header) => (),
                ParsedNode::Incomplete(header) => {
                    if let Some(range) = header.range() {
                        match &header.open_bracket {
                            ParsedLocation::Valid(_) => (),
                            ParsedLocation::Erroneous(location) => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "invalid java header opening bracket. should be '<%@'",
                                    location.range(),
                                ))
                            }
                            ParsedLocation::Missing => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "invalid java header: missing '<%@'",
                                    range,
                                ))
                            }
                        };
                        match &header.page {
                            ParsedLocation::Valid(_) => (),
                            ParsedLocation::Erroneous(location) => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "invalid java header 'page'",
                                    location.range(),
                                ))
                            }
                            ParsedLocation::Missing => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "invalid java header: missing 'page'",
                                    range,
                                ))
                            }
                        };
                        match &header.close_bracket {
                            ParsedLocation::Valid(_) => (),
                            ParsedLocation::Erroneous(location) => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "invalid java header closing bracket. should be '%>'",
                                    location.range(),
                                ))
                            }
                            ParsedLocation::Missing => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "java header is unclosed",
                                    range,
                                ))
                            }
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
                            ParsedLocation::Erroneous(location) => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "invalid taglib header opening bracket. should be '<%@'",
                                    location.range(),
                                ))
                            }
                            ParsedLocation::Missing => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "invalid taglib header: missing '<%@'",
                                    range,
                                ))
                            }
                        };
                        match &header.taglib {
                            ParsedLocation::Valid(_) => (),
                            ParsedLocation::Erroneous(location) => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "invalid taglib header 'taglib'",
                                    location.range(),
                                ))
                            }
                            ParsedLocation::Missing => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "invalid taglib header: missing 'taglib'",
                                    range,
                                ))
                            }
                        };
                        match &header.origin {
                            Some(_) => (),
                            None => self.add(DiagnosticBuilder::new_invalid_header(
                                "invalid taglib header: missing 'uri' or 'tagdir' attribute",
                                range,
                            )),
                        };
                        match &header.prefix {
                            Some(_) => (),
                            None => self.add(DiagnosticBuilder::new_invalid_header(
                                "invalid taglib header: missing 'prefix' attribute",
                                range,
                            )),
                        };
                        match &header.close_bracket {
                            ParsedLocation::Valid(_) => (),
                            ParsedLocation::Erroneous(location) => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "invalid taglib header closing bracket. should be '%>'",
                                    location.range(),
                                ))
                            }
                            ParsedLocation::Missing => {
                                self.add(DiagnosticBuilder::new_invalid_header(
                                    "taglib header is unclosed",
                                    range,
                                ))
                            }
                        }
                        for error in &header.errors {
                            self.add(DiagnosticBuilder::new_invalid_header(
                                format!("syntax error: unexpected \"{}\"", error.content),
                                error.range,
                            ))
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
                Node::Tag(tag) => self.validate_parsed_spml_tag(tag)?,
                Node::Html(html) => self.validate_parsed_html(html)?,
                Node::Text(_) => (),
                Node::Error(ErrorNode { content, range }) => {
                    self.add(DiagnosticBuilder::new_syntax_error(
                        format!("syntax error: unexpected \"{}\"", content),
                        *range,
                    ));
                }
            }
        }
        return Ok(());
    }

    fn validate_parsed_spml_tag(&mut self, tag: &ParsedTag<SpmlTag>) -> Result<()> {
        match tag {
            ParsedTag::Valid(tag) => self.validate_tag(tag)?,
            ParsedTag::Erroneous(tag, errors) => {
                for error in errors {
                    match error {
                        TagError::Superfluous(text, location) => {
                            self.add(
                                DiagnosticBuilder::new_syntax_error(text, location.range())
                                    .with_tag(DiagnosticTag::UNNECESSARY)
                                    .with_code(CodeActionImplementation::REMOVE_SUPERFLUOUS_CODE),
                            );
                        }
                        TagError::Missing(text, location) => {
                            self.add(
                                DiagnosticBuilder::new_missing_value(
                                    format!("\"{}\" is missing", text),
                                    tag.range(),
                                )
                                .with_code(CodeActionImplementation::ADD_MISSING_CODE)
                                .with_data(serde_json::to_value(TextEdit {
                                    range: location.range(),
                                    new_text: text.to_string(),
                                })?),
                            );
                        }
                    }
                }
                self.validate_tag(tag)?;
            }
            ParsedTag::Unparsable(message, location) => {
                self.add(DiagnosticBuilder::new_syntax_error(
                    message,
                    location.range(),
                ));
            }
        };
        return Ok(());
    }

    fn validate_parsed_html(&mut self, tag: &ParsedHtml) -> Result<()> {
        match tag {
            ParsedHtml::Valid(html) => self.validate_html(html)?,
            ParsedHtml::Erroneous(html, errors) => {
                for error in errors {
                    match error {
                        TagError::Superfluous(text, location) => {
                            self.add(
                                DiagnosticBuilder::new_syntax_error(text, location.range())
                                    .with_tag(DiagnosticTag::UNNECESSARY)
                                    .with_code(CodeActionImplementation::REMOVE_SUPERFLUOUS_CODE),
                            );
                        }
                        TagError::Missing(text, location) => {
                            self.add(
                                DiagnosticBuilder::new_missing_value(
                                    format!("\"{}\" is missing", text),
                                    html.range(),
                                )
                                .with_code(CodeActionImplementation::ADD_MISSING_CODE)
                                .with_data(serde_json::to_value(TextEdit {
                                    range: location.range(),
                                    new_text: text.to_string(),
                                })?),
                            );
                        }
                    }
                }
                self.validate_html(html)?;
            }
            ParsedHtml::Unparsable(message, location) => {
                self.add(DiagnosticBuilder::new_syntax_error(
                    message,
                    location.range(),
                ));
            }
        };
        return Ok(());
    }

    fn validate_tag(&mut self, tag: &SpmlTag) -> Result<()> {
        if tag.definition().deprecated {
            self.add(DiagnosticBuilder::new_deprecation(
                format!("{} tag is deprecated", tag.definition().name),
                tag.range(),
            ));
        }
        for (_, attribute) in tag.spel_attributes() {
            let attribute = match attribute {
                ParsedAttribute::Valid(attribute) => attribute,
                ParsedAttribute::Erroneous(attribute, errors) => {
                    for error in errors {
                        match error {
                            AttributeError::Superfluous(text, location) => {
                                self.add(
                                    DiagnosticBuilder::new_syntax_error(text, location.range())
                                        .with_tag(DiagnosticTag::UNNECESSARY)
                                        .with_code(
                                            CodeActionImplementation::REMOVE_SUPERFLUOUS_CODE,
                                        ),
                                );
                            }
                        }
                    }
                    attribute
                }
                ParsedAttribute::Unparsable(message, location) => {
                    self.add(DiagnosticBuilder::new_syntax_error(
                        message,
                        location.range(),
                    ));
                    continue;
                }
            };
            SpelValidator::validate(self, &attribute)?;
        }
        for rule in tag.definition().attribute_rules {
            match rule {
                AttributeRule::Deprecated(name) if tag.spel_attribute(*name).is_some() => {
                    self.add(
                        DiagnosticBuilder::new_deprecation(
                            format!("attribute \"{}\" is deprecated", name),
                            tag.range(),
                        ),
                    );
                }
                AttributeRule::AtleastOneOf(names)
                    if !names.iter().any(|name| tag.spel_attribute(*name).is_some()) =>
                {
                    self.add(
                        DiagnosticBuilder::new_missing_value(
                            format!(
                                "requires atleast one of these attributes: {}",
                                names.join(", ")
                            ),
                            tag.range(),
                        ),
                    );
                }
                AttributeRule::ExactlyOneOf(names) => {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| tag.spel_attribute(*name).is_some())
                        .collect();
                    match present.len() {
                        0 => self.add(
                            DiagnosticBuilder::new_missing_value(
                                format!("requires one of these attributes: {}", names.join(", ")),
                                tag.range(),
                            ),
                        ),
                        1 => {}
                        _ => self.add(
                            DiagnosticBuilder::new_conflicting_values(
                                format!(
                                    "requires only one of these attributes: {}",
                                    present.join(", ")
                                ),
                                tag.range(),
                            ),
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
                        (0, false) => self.add(
                            DiagnosticBuilder::new_missing_value(
                                format!(
                                    "requires either a tag-body or one of these attributes: {}",
                                    names.join(", ")
                                ),
                                tag.range(),
                            ),
                        ),
                        (0, true) | (1, false) => {}
                        _ => self.add(
                            DiagnosticBuilder::new_conflicting_values(
                                format!(
                                    "requires either a tag-body or only one of these attributes: {}",
                                    present.join(", ")
                                ),
                                tag.range(),
                            ),
                        ),
                    }
                }
                AttributeRule::ExactlyOrBody(name) => match (tag.spel_attribute(*name), tag.body()) {
                    (None, None) => self.add(
                        DiagnosticBuilder::new_missing_value(
                            format!("requires either a tag-body or the attribute \"{}\"", name),
                            tag.range(),
                        ),
                    ),
                    (Some(_), Some(_)) => self.add(
                        DiagnosticBuilder::new_conflicting_values(
                            format!("requires either a tag-body or the attribute \"{}\"", name),
                            tag.range(),
                        ),
                    ),
                    _ => {}
                }
                AttributeRule::OnlyOneOf(names) => {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| tag.spel_attribute(name).is_some())
                        .collect();
                    if present.len() > 1 {
                        self.add(
                            DiagnosticBuilder::new_conflicting_values(
                                format!(
                                    "can only have one of these attributes: {}",
                                    present.join(", ")
                                ),
                                tag.range(),
                            ),
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
                        (len, true) if len > 0 => self.add(
                            DiagnosticBuilder::new_conflicting_values(
                                format!(
                                    "can only have either a tag-body or one of these attributes: {}",
                                    present.join(", ")
                                ),
                                tag.range(),
                            ),
                        ),
                        (len, false) if len > 1 => self.add(
                            DiagnosticBuilder::new_conflicting_values(
                                format!(
                                    "can only have one of these attributes: {}",
                                    present.join(", ")
                                ),
                                tag.range(),
                            ),
                        ),
                        _ => {}
                    }
                }
                AttributeRule::OnlyOrBody(name)
                    if tag.spel_attribute(*name).is_some() && tag.body().is_some() =>
                {
                    self.add(
                        DiagnosticBuilder::new_conflicting_values(
                            format!("can only have either a tag-body or the \"{}\" attribute", name),
                            tag.range(),
                        ),
                    );
                }
                AttributeRule::OnlyWith(name1, name2)
                    if !tag.spel_attribute(*name2).is_some() => match tag.spel_attribute(*name1) {
                    Some(attribute) => self.add(
                        DiagnosticBuilder::new_superfluous_value(
                            format!("attribute \"{}\" is useless without attribute \"{}\"", name1, name2),
                            attribute.range(),
                        ),
                    ),
                    None => {}
                }
                AttributeRule::OnlyWithEither(name, names)
                    if !names.iter().any(|name| tag.spel_attribute(*name).is_some()) =>
                        match tag.spel_attribute(*name) {
                    Some(attribute) => self.add(
                        DiagnosticBuilder::new_superfluous_value(
                            format!(
                                "attribute \"{}\" is useless without one of these attributes: {}",
                                name,
                                names.join(", ")
                            ),
                            attribute.range(),
                        ),
                    ),
                    None => {}
                }
                AttributeRule::OnlyWithEitherOrBody(name, names)
                    if !names.iter().any(|name| tag.spel_attribute(*name).is_some())
                        && tag.body().is_none() => match tag.spel_attribute(*name) {
                    Some(attribute) => self.add(
                        DiagnosticBuilder::new_superfluous_value(
                            format!(
                                "attribute \"{}\" is useless without either a tag-body or one of these attributes: {}",
                                name,
                                names.join(", ")
                            ),
                            attribute.range(),
                        ),
                    ),
                    None => {}
                }
                AttributeRule::Required(name) if !tag.spel_attribute(*name).is_some() => {
                    self.add(
                        DiagnosticBuilder::new_missing_value(
                            format!("missing required attribute \"{}\"", name),
                            tag.range(),
                        ),
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
                            }) if fragments.len() == 1 => fragments.first(),
                            Some(_) => continue,
                            None => None,
                        };
                        let module = match module_value {
                            Some(WordFragment::Interpolation(_)) | None => {
                                modules::find_module_for_file(Path::new(self.file.path().as_str()))
                            }
                            Some(WordFragment::String(StringLiteral { content, .. })) => {
                                modules::find_module_by_name(&content)
                            }
                        };
                        match module {
                            Some(module) => {
                                let file = format!("{}{}", module.path, uri);
                                if !Path::new(&file).exists() {
                                    self.add(DiagnosticBuilder::new_missing_file(
                                        format!("included file \"{}\" does not exist", uri),
                                        tag.range(),
                                    ));
                                }
                            }
                            None => self.add(DiagnosticBuilder::new_unknown_module(
                                match module_value {
                                    Some(WordFragment::Interpolation(i)) => format!(
                                        concat!(
                                            "interpolation \"{}\" is interpreted as the current ",
                                            "module, which is not listed in the module-file"
                                        ),
                                        i
                                    ),
                                    Some(WordFragment::String(literal)) => format!(
                                        "module \"{}\" not listed in module-file",
                                        literal.content
                                    ),
                                    None => "current module not listed in module-file".to_string(),
                                },
                                tag.range(),
                            )),
                        }
                    }
                }
                AttributeRule::ValueOneOf(name, values)
                    if string_literal_attribute_value(tag.spel_attribute(*name))
                        .is_some_and(|value| !values.contains(&&*value)) =>
                {
                    self.add(DiagnosticBuilder::new_invalid_literal(
                        format!(
                            "attribute \"{}\" should be one of these values: [{}]",
                            name,
                            values.join(", ")
                        ),
                        tag.range(),
                    ));
                }
                AttributeRule::ValueOneOfCaseInsensitive(name, values)
                    if string_literal_attribute_value(tag.spel_attribute(*name))
                        .is_some_and(|value| !values.contains(&value.to_uppercase().as_str())) =>
                {
                    self.add(DiagnosticBuilder::new_invalid_literal(
                        format!(
                            "attribute \"{}\" should be one of these (caseinsensitive) values: [{}]",
                            name,
                            values.join(", ")
                        ),
                        tag.range(),
                    ));
                }
                AttributeRule::OnlyWithValue(name, attribute_name, value)
                    if !string_literal_attribute_value(tag.spel_attribute(*attribute_name))
                        .is_some_and(|v| *v == **value) => match tag.spel_attribute(*name) {
                    Some(attribute) => self.add(DiagnosticBuilder::new_superfluous_value(
                        format!(
                            "attribute \"{}\" is useless without attribute \"{}\" containing the value {}",
                            name, attribute_name, value
                        ),
                        attribute.range(),
                    )),
                    None => {}
                }
                AttributeRule::OnlyWithEitherValue(name, attribute_name, values)
                    if !string_literal_attribute_value(tag.spel_attribute(*attribute_name))
                        .is_some_and(|value| values.contains(&&*value)) => match tag.spel_attribute(*name) {
                    Some(attribute) => self.add(DiagnosticBuilder::new_superfluous_value(
                        format!(
                            "attribute \"{}\" is useless without attribute \"{}\" containing one of these values: [{}]",
                            name, attribute_name, values.join(", ")
                        ),
                        attribute.range(),
                    )),
                    None => {}
                }
                AttributeRule::BodyOnlyWithEitherValue(attribute_name, values)
                    if !string_literal_attribute_value(tag.spel_attribute(*attribute_name))
                        .is_some_and(|value| values.contains(&&*value)) => match tag.body() {
                    Some(body) => self.add(DiagnosticBuilder::new_superfluous_value(
                        format!(
                            "tag-body is useless without attribute \"{}\" containing one of these values: [{}]",
                            attribute_name, values.join(", ")
                        ),
                        body.open_location.range(), // TODO: why does body have no close_location?
                    )),
                    None => {}
                }
                AttributeRule::RequiredWithValue(name, attribute, value)
                    if string_literal_attribute_value(tag.spel_attribute(*attribute))
                        .is_some_and(|v| *v == **value)
                        && !tag.spel_attribute(*name).is_some() => {
                    self.add(DiagnosticBuilder::new_missing_value(
                        format!(
                            "attribute \"{}\" is required when attribute \"{}\" is \"{}\"",
                            name, attribute, value
                        ),
                        tag.range(),
                    ));
                }
                AttributeRule::RequiredOrBodyWithValue(name, attribute, value)
                    if string_literal_attribute_value(tag.spel_attribute(*attribute))
                        .is_some_and(|v| *v == **value) => {
                    let has_attribute = tag.spel_attribute(*name).is_some();
                    let has_body = tag.body().is_some();
                    match (has_attribute, has_body) {
                        (false, false) => self.add(DiagnosticBuilder::new_missing_value(
                            format!(
                                "either attribute \"{}\" or a tag-body is required when attribute \"{}\" is \"{}\"",
                                name,
                                attribute,
                                value
                            ),
                            tag.range(),
                        )),
                        (true, true) => self.add(DiagnosticBuilder::new_conflicting_values(
                            format!(
                                "exactly one of attribute \"{}\" or a tag-body is required when attribute \"{}\" is \"{}\"",
                                name,
                                attribute,
                                value
                            ),
                            tag.range(),
                        )),
                        _ => {}
                    }
                }
                AttributeRule::RequiredWithEitherValue(name, attribute, values)
                    if string_literal_attribute_value(tag.spel_attribute(*attribute))
                        .is_some_and(|value| values.contains(&&*value))
                        && !tag.spel_attribute(*name).is_some() =>
                {
                    self.add(DiagnosticBuilder::new_missing_value(
                        format!(
                            "attribute \"{}\" is required when attribute \"{}\" is either of [{}]",
                            name,
                            attribute,
                            values.join(", ")
                        ),
                        tag.range(),
                    ));
                }
                AttributeRule::ExactlyOneOfOrBodyWithValue(names, attribute, value)
                    if string_literal_attribute_value(tag.spel_attribute(*attribute))
                        .is_some_and(|v| *v == **value) =>
                {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| tag.spel_attribute(*name).is_some())
                        .collect();
                    let has_body = tag.body().is_some();
                    match (present.len(), has_body) {
                        (0, false) => {
                            self.add(DiagnosticBuilder::new_missing_value(
                                format!(
                                    concat!(
                                        "when attribute \"{}\" is \"{}\" either a tag-body or ",
                                        "exactly one of these attributes is required: [{}]",
                                    ),
                                    attribute, value, names.join(", ")
                                ),
                                tag.range(),
                            ));
                        }
                        (0, true) | (1, false) => {}
                        _ => self.add(DiagnosticBuilder::new_conflicting_values(
                            format!(
                                concat!(
                                    "when attribute \"{}\" is \"{}\" only one of a tag-body and ",
                                    "these attributes is required: [{}]",
                                ),
                                attribute, value, names.join(", ")
                            ),
                            tag.range(),
                        )),
                    }
                }
                AttributeRule::ExactlyOneOfOrBodyWithEitherValue(names, attribute, values)
                    if string_literal_attribute_value(tag.spel_attribute(*attribute))
                        .is_some_and(|value| values.contains(&&*value)) =>
                {
                    let present: Vec<&str> = names
                        .iter()
                        .map(|name| *name)
                        .filter(|name| tag.spel_attribute(*name).is_some())
                        .collect();
                    let has_body = tag.body().is_some();
                    match (present.len(), has_body) {
                        (0, false) => self.add(DiagnosticBuilder::new_missing_value(
                            format!(
                                concat!(
                                    "when attribute \"{}\" is either of [{}] either a tag-body or ",
                                    "exactly one of these attributes is required: [{}]",
                                ),
                                attribute, values.join(", "), names.join(", ")
                            ),
                            tag.range(),
                        )),
                        (0, true) | (1, false) => {}
                        _ => self.add(DiagnosticBuilder::new_conflicting_values(
                            format!(
                                concat!(
                                    "when attribute \"{}\" is either of [{}] only one of a ",
                                    "tag-body and these attributes is required: [{}]",
                                ),
                                attribute, values.join(", "), names.join(", ")
                            ),
                            tag.range(),
                        ))
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
        for attribute in &html.attributes {
            match attribute {
                ParsedAttribute::Valid(tag) => {
                    if let Some(value) = &tag.value {
                        match &value.content {
                            HtmlAttributeValueContent::Tag(tag) => {
                                self.validate_parsed_spml_tag(tag)?
                            }
                            HtmlAttributeValueContent::Fragmented(fragments) => {
                                for fragment in fragments {
                                    if let HtmlAttributeValueFragment::Tag(tag) = fragment {
                                        self.validate_parsed_spml_tag(tag)?
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                }
                ParsedAttribute::Erroneous(_, errors) => {
                    for error in errors {
                        match error {
                            AttributeError::Superfluous(text, location) => self.add(
                                DiagnosticBuilder::new_syntax_error(text, location.range())
                                    .with_tag(DiagnosticTag::UNNECESSARY)
                                    .with_code(CodeActionImplementation::REMOVE_SUPERFLUOUS_CODE),
                            ),
                        }
                    }
                }
                ParsedAttribute::Unparsable(message, location) => self.add(
                    DiagnosticBuilder::new_syntax_error(message, location.range()),
                ),
            }
        }
        return match html.body() {
            Some(body) => self.validate_nodes(&body.nodes),
            None => Ok(()),
        };
    }

    fn add(&mut self, builder: DiagnosticBuilder) {
        self.diagnostics.push(builder.build());
    }
}

fn string_literal_attribute_value(
    attribute: Option<&ParsedAttribute<SpelAttribute>>,
) -> Option<Arc<str>> {
    let attribute = match attribute {
        Some(ParsedAttribute::Valid(attribute)) => attribute,
        Some(ParsedAttribute::Erroneous(attribute, _)) => attribute,
        _ => return None,
    };
    return match &attribute.value.spel {
        SpelAst::String(SpelResult::Valid(Word { fragments })) if fragments.len() == 1 => {
            match &fragments[0] {
                WordFragment::String(StringLiteral { content, .. }) => Some(content.clone()),
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
        match grammar::FunctionDefinition::from_str(&*function.name) {
            Ok(definition) => match definition.argument_number {
                ArgumentNumber::AtLeast(number) if argument_count < number => {
                    self.collector.add(DiagnosticBuilder::new_invalid_function(
                        format!(
                            "invalid arguments number to \"{}\", expected {} or more but got {}",
                            definition.name, number, argument_count,
                        ),
                        self.locations_range(
                            &function.name_location,
                            &function.closing_bracket_location,
                        ),
                    ))
                }
                ArgumentNumber::Exactly(number) if argument_count != number => {
                    self.collector.add(DiagnosticBuilder::new_invalid_function(
                        format!(
                            "invalid arguments number to \"{}\", expected {} but got {}",
                            definition.name, number, argument_count,
                        ),
                        self.locations_range(
                            &function.name_location,
                            &function.closing_bracket_location,
                        ),
                    ));
                }
                ArgumentNumber::None if argument_count != 0 => {
                    self.collector.add(DiagnosticBuilder::new_invalid_function(
                        format!(
                            "invalid arguments number to \"{}\", expected 0 but got {}",
                            definition.name, argument_count,
                        ),
                        self.locations_range(
                            &function.name_location,
                            &function.closing_bracket_location,
                        ),
                    ))
                }
                _ => {}
            },
            Err(err) => self.collector.add(DiagnosticBuilder::new_invalid_function(
                err,
                self.locations_range(&function.name_location, &function.closing_bracket_location),
            )),
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
            0 => self.collector.add(DiagnosticBuilder::new_syntax_error(
                format!("invalid {}: {}", r#type, err.message),
                range,
            )),
            _ => {
                let offset = Position {
                    line: attribute.value.opening_quote_location.line as u32,
                    character: attribute.value.opening_quote_location.char as u32 + 1,
                };
                self.collector.add(
                    DiagnosticBuilder::new_syntax_error(
                        format!("invalid {}: {}", r#type, err.message),
                        range,
                    )
                    .with_code(CodeActionImplementation::FIX_SPEL_SYNTAX_CODE)
                    .with_data(
                        serde_json::to_value(
                            err.proposed_fixes
                                .iter()
                                .map(|fix| fix.to_text_edit(&offset))
                                .collect::<Vec<TextEdit>>(),
                        )
                        .unwrap(),
                    ),
                );
            }
        }
    }
}

pub fn diagnostic(params: DocumentDiagnosticParams) -> Result<Vec<Diagnostic>> {
    return diagnose_uri(params.text_document.uri);
}

pub fn diagnose_all(
    dir: &Path,
    result: &mut HashMap<PathBuf, Vec<Diagnostic>>,
) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            diagnose_all(&path, result)?;
        } else {
            let path = path.canonicalize()?;
            let file = path.to_string_lossy().to_string();
            if !file.ends_with(".spml") {
                continue;
            }
            let uri = Url::from_str(&format!("file://{}", file).as_str())
                .expect("path should be a parsable uri");
            let mut diagnostics = diagnose_uri(uri)?;
            if !diagnostics.is_empty() {
                diagnostics.sort_by(|a, b| match a.severity.cmp(&b.severity) {
                    Ordering::Equal => a.r#type.cmp(&b.r#type),
                    cmp => cmp,
                });
                result.insert(path, diagnostics);
            }
        }
    }
    Ok(())
}

fn diagnose_uri(uri: Url) -> Result<Vec<Diagnostic>> {
    let document = match document_store::get(&uri) {
        Some(document) => Ok(document),
        None => document_store::Document::from_uri(&uri)
            .map(|document| document_store::put(&uri, document))
            .map_err(|err| anyhow::anyhow!("cannot read file {}: {}", uri.path(), err)),
    }?;
    let mut collector = DiagnosticCollector::new(uri.clone());
    collector
        .validate_document(&document.tree)
        .map_err(|err|  anyhow::anyhow!("failed to validate {}: {}", uri.path(), err))?;
    return Ok(collector.diagnostics);
}
