use anyhow::{Error, Result};
use std::str::FromStr;

#[derive(Debug)]
pub(crate) struct TagDefinition {
    pub(crate) name: &'static str,
    pub(crate) detail: Option<&'static str>,
    pub(crate) documentation: Option<&'static str>,
    pub(crate) deprecated: bool,
    pub(crate) children: TagChildren,
    pub(crate) attributes: TagAttributes,
    pub(crate) attribute_rules: &'static [AttributeRule],
}

impl PartialEq for TagDefinition {
    fn eq(&self, other: &Self) -> bool {
        return self.name == other.name;
    }
}

#[derive(Debug)]
pub(crate) enum TagAttributes {
    None,
    These(&'static [TagAttribute]),
}

#[derive(Debug)]
pub(crate) struct TagAttribute {
    pub(crate) name: &'static str,
    pub(crate) r#type: TagAttributeType,
    pub(crate) detail: Option<&'static str>,
    pub(crate) documentation: Option<&'static str>,
}

#[derive(Debug)]
pub(crate) enum TagAttributeType {
    Comparable,
    Condition,
    // Enum(Vec<String>), e.g. for action="put|putAll|remove|..."
    Expression,
    Identifier, // TODO: dotted field access allowed?
    Object,
    Query,
    Regex,
    String,
    Uri { module_attribute: &'static str },
    Module,
}

#[derive(Debug)]
pub(crate) enum AttributeRule {
    Deprecated(&'static str),
    ExactlyOneOf(&'static [&'static str]),
    ExactlyOrBody(&'static str),
    OnlyOneOf(&'static [&'static str]),
    AtleastOneOf(&'static [&'static str]),
    OnlyWith(&'static str, &'static str),
    OnlyWithEither(&'static str, &'static [&'static str]),
    OnlyWithEitherOrBody(&'static str, &'static [&'static str]),
    Required(&'static str),
    UriExists(&'static str, &'static str),
    ValueOneOf(&'static str, &'static [&'static str]),
    ValueOneOfCaseInsensitive(&'static str, &'static [&'static str]),
    OnlyWithValue(&'static str, &'static str, &'static str),
    OnlyWithEitherValue(&'static str, &'static str, &'static [&'static str]),
    RequiredWithValue(&'static str, &'static str, &'static str),
    RequiredOrBodyWithValue(&'static str, &'static str, &'static str),
    RequiredWithEitherValue(&'static str, &'static str, &'static [&'static str]),
    ExactlyOneOfOrBody(&'static [&'static str]),
    OnlyOrBody(&'static str),
    OnlyOneOfOrBody(&'static [&'static str]),
    BodyOnlyWithEitherValue(&'static str, &'static [&'static str]),
    ExactlyOneOfOrBodyWithValue(&'static [&'static str], &'static str, &'static str),
    ExactlyOneOfOrBodyWithEitherValue(
        &'static [&'static str],
        &'static str,
        &'static [&'static str],
    ),
    // Object(&'static str),
    // Expression(&'static str),
    // Condition(&'static str),
    // TODO: Renamed?
}

#[derive(Debug)]
pub(crate) enum TagChildren {
    Any,
    None,
    Scalar(&'static TagDefinition),
    Vector(&'static [TagDefinition]),
}

macro_rules! tag_definition {
    (
        type $tag_type:expr,
        name $tag_name:expr,
        deprecated $deprecated:expr,
        children $children:expr,
        rules $attribute_rules:expr
    ) => {
        TagDefinition {
            name: concat!($tag_type, ":", $tag_name),
            detail: None,
            documentation: Some(
                include_str!(concat!("../doc/", $tag_type, "_", $tag_name, "/tag.md"))
            ),
            deprecated: $deprecated,
            children: $children,
            attributes: TagAttributes::None,
            attribute_rules: $attribute_rules,
        }
    };

    (
        type $tag_type:expr,
        name $tag_name:expr,
        deprecated $deprecated:expr,
        children $children:expr,
        attributes $(($attribute_name:expr, $attribute_type:expr)),+,
        rules $attribute_rules:expr
    ) => {
        TagDefinition {
            name: concat!($tag_type, ":", $tag_name),
            detail: None,
            documentation: match include_str!(concat!(
                "../doc/",
                $tag_type,
                "_",
                $tag_name,
                "/tag.md"
            )) {
                doc if doc.len() == 0 => None,
                doc => Some(doc),
            },
            deprecated: $deprecated,
            children: $children,
            attributes: TagAttributes::These(
                &[
                    $(TagAttribute {
                        name: $attribute_name,
                        r#type: $attribute_type,
                        detail: None,
                        documentation: Some(
                            include_str!(
                                concat!(
                                    "../doc/",
                                    $tag_type,
                                    "_",
                                    $tag_name,
                                    "/",
                                    $attribute_name,
                                    "_attribute.md"
                                )
                            )
                        ),
                    }),+
                ]
            ),
            attribute_rules: $attribute_rules,
        }
    };
}

impl TagDefinition {
    const SP_ARGUMENT: TagDefinition = tag_definition!(
        type "sp",
        name "argument",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("condition", TagAttributeType::Condition),
            ("default", TagAttributeType::String),
            ("expression", TagAttributeType::Expression),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::ExactlyOneOfOrBody(&["value", "expression", "condition", "object"]),
            AttributeRule::OnlyWithEither("default", &["object", "expression"]),
        ]
    );

    const SP_ATTRIBUTE: TagDefinition = tag_definition!(
        type "sp",
        name "attribute",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("dynamics", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("text", TagAttributeType::String),
        rules &[
            AttributeRule::Deprecated("name"),
            AttributeRule::ExactlyOneOf(&["name", "text", "object", "dynamics"]),
        ]
    );

    const SP_BARCODE: TagDefinition = tag_definition!(
        type "sp",
        name "barcode",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("height", TagAttributeType::Expression),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("scope", TagAttributeType::String),
            ("text", TagAttributeType::String),
            ("type", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("text"),
            AttributeRule::Required("type"),
            AttributeRule::ValueOneOf("type", &["qrcode"]),
            AttributeRule::ValueOneOf("scope", &["page", "request"]),
        ]
    );

    const SP_BREAK: TagDefinition = tag_definition!(
        type "sp",
        name "break",
        deprecated false,
        children TagChildren::Any,
        rules &[]
    );

    const SP_CALENDARSHEET: TagDefinition = tag_definition!(
        type "sp",
        name "calendarsheet",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("action", TagAttributeType::String),
            ("date", TagAttributeType::Object),
            ("from", TagAttributeType::Object),
            ("mode", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("scope", TagAttributeType::String),
            ("to", TagAttributeType::Object),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("action"),
            AttributeRule::ValueOneOf("action", &["add", "clear", "new"]),
            AttributeRule::ValueOneOf("mode", &["allDays", "startDays", "firstDays"]),
            AttributeRule::ValueOneOf("scope", &["page", "request"]),
            AttributeRule::OnlyWithValue("from", "action", "new"),
            AttributeRule::OnlyWithValue("to", "action", "new"),
            AttributeRule::RequiredWithValue("from", "action", "new"),
            AttributeRule::RequiredWithValue("to", "action", "new"),
            AttributeRule::OnlyWithValue("value", "action", "add"),
            AttributeRule::OnlyWithValue("object", "action", "add"),
            AttributeRule::OnlyWithValue("date", "action", "add"),
            AttributeRule::OnlyOneOf(&["value", "object", "date"]),
        ]
    );

    const SP_CHECKBOX: TagDefinition = tag_definition!(
        type "sp",
        name "checkbox",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("checked", TagAttributeType::Condition),
            ("disabled", TagAttributeType::Condition),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("readonly", TagAttributeType::Condition),
            ("type", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::OnlyOrBody("value"),
        ]
    );

    const SP_CODE: TagDefinition = tag_definition!(
        type "sp",
        name "code",
        deprecated false,
        children TagChildren::Any,
        rules &[]
    );

    const SP_COLLECTION: TagDefinition = tag_definition!(
        type "sp",
        name "collection",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("action", TagAttributeType::String),
            ("condition", TagAttributeType::Condition),
            ("default", TagAttributeType::String),
            ("expression", TagAttributeType::Expression),
            ("index", TagAttributeType::Expression),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("publisher", TagAttributeType::String),
            ("query", TagAttributeType::Query),
            ("scope", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::AtleastOneOf(&["action", "query"]),
            AttributeRule::ValueOneOf(
                "action",
                &[
                    "add",
                    "addNotEmpty",
                    "addAll",
                    "remove",
                    "clear",
                    "insert",
                    "new",
                    "replace",
                    "removeFirst",
                    "removeLast",
                    "unique",
                ],
            ),
            AttributeRule::ValueOneOf("publisher", &["current", "ignore", "all", "auto"]),
            AttributeRule::ValueOneOf("scope", &["page", "request"]),
            AttributeRule::ExactlyOneOfOrBodyWithEitherValue(
                &["value", "object", "expression", "condition"],
                "action",
                &["add", "addNotEmpty", "insert"],
            ),
            AttributeRule::ExactlyOneOfOrBodyWithEitherValue(
                &["index", "value", "object"],
                "action",
                &["remove", "replace"],
            ),
            AttributeRule::ExactlyOneOfOrBodyWithValue(&["object", "query"], "action", "addAll"),
            AttributeRule::BodyOnlyWithEitherValue(
                "action",
                &[
                    "add",
                    "addAll",
                    "addNotEmpty",
                    "insert",
                    "remove",
                    "replace",
                ],
            ),
            AttributeRule::RequiredWithValue("index", "action", "insert"),
            AttributeRule::OnlyWithEitherValue(
                "value",
                "action",
                &["add", "addNotEmpty", "insert", "remove", "replace"],
            ),
            AttributeRule::OnlyWithEitherValue(
                "expression",
                "action",
                &["add", "addNotEmpty", "insert"],
            ),
            AttributeRule::OnlyWithEitherValue(
                "condition",
                "action",
                &["add", "addNotEmpty", "insert"],
            ),
            AttributeRule::OnlyWithEitherValue(
                "object",
                "action",
                &[
                    "add",
                    "addNotEmpty",
                    "addAll",
                    "insert",
                    "remove",
                    "replace",
                ],
            ),
            AttributeRule::OnlyWithEitherValue("index", "action", &["insert", "remove", "replace"]),
            AttributeRule::OnlyWithEither("default", &["object", "expression"]),
            AttributeRule::OnlyWithEither("publisher", &["query", "object"]),
        ]
    );

    const SP_CONDITION: TagDefinition = tag_definition!(
        type "sp",
        name "condition",
        deprecated false,
        children TagChildren::Vector(&[
            TagDefinition::SP_IF,
            TagDefinition::SP_ELSE,
            TagDefinition::SP_ELSEIF,
        ]),
        rules &[]
    );

    const SP_DIFF: TagDefinition = tag_definition!(
        type "sp",
        name "diff",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("from", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("lookup", TagAttributeType::Condition),
            ("name", TagAttributeType::Identifier),
            ("to", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("from"),
            AttributeRule::Required("to"),
            AttributeRule::OnlyWith("lookup", "locale"), // is that correct?
        ]
    );

    const SP_ELSE: TagDefinition = tag_definition!(
        type "sp",
        name "else",
        deprecated false,
        children TagChildren::Any,
        rules &[]
    );

    const SP_ELSEIF: TagDefinition = tag_definition!(
        type "sp",
        name "elseif",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("condition", TagAttributeType::Condition),
            ("eq", TagAttributeType::Comparable),
            ("gt", TagAttributeType::Comparable),
            ("gte", TagAttributeType::Comparable),
            ("ic", TagAttributeType::Condition),
            ("isNull", TagAttributeType::Condition),
            ("locale", TagAttributeType::Object),
            ("lt", TagAttributeType::Comparable),
            ("lte", TagAttributeType::Comparable),
            ("match", TagAttributeType::Regex),
            ("name", TagAttributeType::Identifier),
            ("neq", TagAttributeType::Comparable),
        rules &[
            AttributeRule::ExactlyOneOf(&["name", "condition"]),
            AttributeRule::OnlyOneOf(&[
                "isNull", "contains", "match", "eq", "neq", "gt", "gte", "lt", "lte",
            ]),
            AttributeRule::OnlyWith("isNull", "name"),
            AttributeRule::OnlyWith("contains", "name"),
            AttributeRule::OnlyWith("match", "name"),
            AttributeRule::OnlyWith("eq", "name"),
            AttributeRule::OnlyWith("neq", "name"),
            AttributeRule::OnlyWith("gt", "name"),
            AttributeRule::OnlyWith("gte", "name"),
            AttributeRule::OnlyWith("lt", "name"),
            AttributeRule::OnlyWith("lte", "name"),
            AttributeRule::OnlyWithEither(
                "ic",
                &["eq", "neq", "gt", "gte", "lt", "lte", "contains"],
            ),
        ]
    );

    const SP_ERROR: TagDefinition = tag_definition!(
        type "sp",
        name "error",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("code", TagAttributeType::String),
        rules &[AttributeRule::Required("code")]
    );

    const SP_EXPIRE: TagDefinition = tag_definition!(
        type "sp",
        name "expire",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("date", TagAttributeType::Expression),
        rules &[AttributeRule::Required("date")]
    );

    const SP_FILTER: TagDefinition = tag_definition!(
        type "sp",
        name "filter",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("attribute", TagAttributeType::String),
            ("collection", TagAttributeType::Object),
            ("filter", TagAttributeType::Regex), // only if mode = "regex", otherwise String
            ("format", TagAttributeType::String),
            ("from", TagAttributeType::String),
            ("ic", TagAttributeType::Condition),
            ("invert", TagAttributeType::Condition),
            ("locale", TagAttributeType::String),
            ("mode", TagAttributeType::String),
            ("name", TagAttributeType::Identifier),
            ("scope", TagAttributeType::String),
            ("to", TagAttributeType::String),
            ("type", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("collection"),
            AttributeRule::OnlyWith("ic", "filter"),
            AttributeRule::OnlyWithEither("type", &["from", "to"]),
            AttributeRule::ValueOneOf("mode", &["simple", "regex"]),
            AttributeRule::ValueOneOf("type", &["number", "text", "date"]),
            AttributeRule::ValueOneOf("scope", &["page", "request", "session"]),
            AttributeRule::OnlyWithValue("format", "type", "date"),
        ]
    );

    const SP_FOR: TagDefinition = tag_definition!(
        type "sp",
        name "for",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("condition", TagAttributeType::Condition),
            ("from", TagAttributeType::Expression),
            ("index", TagAttributeType::Identifier),
            ("locale", TagAttributeType::Object),
            ("step", TagAttributeType::Expression),
            ("to", TagAttributeType::Expression),
        rules &[
            AttributeRule::Required("index"),
            AttributeRule::Required("from"),
            AttributeRule::ExactlyOneOf(&["to", "condition"]),
        ]
    );

    const SP_FORM: TagDefinition = tag_definition!(
        type "sp",
        name "form",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("command", TagAttributeType::String),
            ("context", TagAttributeType::String),
            ("enctype", TagAttributeType::String),
            ("handler", TagAttributeType::String),
            ("id", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("method", TagAttributeType::String),
            ("module", TagAttributeType::Module),
            ("name", TagAttributeType::Identifier),
            ("nameencoding", TagAttributeType::String),
            ("template", TagAttributeType::String),
            ("uri", TagAttributeType::Uri { module_attribute: "module" }),
        rules &[
            AttributeRule::Deprecated("command"),
            AttributeRule::OnlyOneOf(&["uri", "template"]),
            AttributeRule::OnlyWith("module", "uri"),
            AttributeRule::ValueOneOf("nameencoding", &["escff", "hex"]),
            AttributeRule::ValueOneOf("enctype", &["text/plain", "multipart/form-data"]),
            AttributeRule::ValueOneOf("method", &["get", "post"]),
            AttributeRule::UriExists("uri", "module"),
        ]
    );

    const SP_HIDDEN: TagDefinition = tag_definition!(
        type "sp",
        name "hidden",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("fixvalue", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("type", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::ExactlyOneOf(&["value", "fixvalue"]),
        ]
    );

    const SP_IF: TagDefinition = tag_definition!(
        type "sp",
        name "if",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("condition", TagAttributeType::Condition),
            ("eq", TagAttributeType::Comparable),
            ("gt", TagAttributeType::Comparable),
            ("gte", TagAttributeType::Comparable),
            ("ic", TagAttributeType::Condition),
            ("isNull", TagAttributeType::Condition),
            ("locale", TagAttributeType::Object),
            ("lt", TagAttributeType::Comparable),
            ("lte", TagAttributeType::Comparable),
            ("match", TagAttributeType::Regex),
            ("name", TagAttributeType::Object),
            ("neq", TagAttributeType::Comparable),
        rules &[
            AttributeRule::ExactlyOneOf(&["name", "condition"]),
            AttributeRule::OnlyOneOf(&[
                "isNull", "contains", "match", "eq", "neq", "gt", "gte", "lt", "lte",
            ]),
            AttributeRule::OnlyWith("isNull", "name"),
            AttributeRule::OnlyWith("contains", "name"),
            AttributeRule::OnlyWith("match", "name"),
            AttributeRule::OnlyWith("eq", "name"),
            AttributeRule::OnlyWith("neq", "name"),
            AttributeRule::OnlyWith("gt", "name"),
            AttributeRule::OnlyWith("gte", "name"),
            AttributeRule::OnlyWith("lt", "name"),
            AttributeRule::OnlyWith("lte", "name"),
            AttributeRule::OnlyWithEither(
                "ic",
                &["eq", "neq", "gt", "gte", "lt", "lte", "contains"],
            ),
        ]
    );

    const SP_INCLUDE: TagDefinition = tag_definition!(
        type "sp",
        name "include",
        deprecated false,
        children TagChildren::Scalar(&TagDefinition::SP_ARGUMENT),
        attributes
            ("anchor", TagAttributeType::String),
            ("arguments", TagAttributeType::Object),
            ("context", TagAttributeType::String),
            ("mode", TagAttributeType::String),
            ("module", TagAttributeType::Module),
            ("return", TagAttributeType::Identifier),
            ("template", TagAttributeType::String),
            ("uri", TagAttributeType::Uri { module_attribute: "module" }),
        rules &[
            AttributeRule::ExactlyOneOf(&["template", "anchor", "uri"]),
            AttributeRule::OnlyOneOf(&["context", "module"]),
            AttributeRule::OnlyWith("context", "uri"),
            AttributeRule::OnlyWith("module", "uri"),
            AttributeRule::ValueOneOf("mode", &["in", "out"]),
            AttributeRule::UriExists("uri", "module"),
        ]
    );

    const SP_IO: TagDefinition = tag_definition!(
        type "sp",
        name "io",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("contenttype", TagAttributeType::String),
            ("type", TagAttributeType::String),
        rules &[
            AttributeRule::Required("type"),
            AttributeRule::ValueOneOf("type", &["in", "out"]),
        ]
    );

    const SP_ITERATOR: TagDefinition = tag_definition!(
        type "sp",
        name "iterator",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("collection", TagAttributeType::Object),
            ("item", TagAttributeType::Identifier),
            ("max", TagAttributeType::Expression),
            ("min", TagAttributeType::Expression),
        rules &[AttributeRule::Required("collection")]
    );

    const SP_JSON: TagDefinition = tag_definition!(
        type "sp",
        name "json",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("indent", TagAttributeType::Expression),
            ("locale", TagAttributeType::String),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("overwrite", TagAttributeType::Condition),
            ("scope", TagAttributeType::String),
        rules &[
            AttributeRule::ValueOneOf("scope", &["page", "request"]),
            AttributeRule::ExactlyOrBody("object"),
            AttributeRule::OnlyWith("indent", "object"),
            AttributeRule::OnlyWith("overwrite", "object"),
        ]
    );

    const SP_LINKEDINFORMATION: TagDefinition = tag_definition!(
        type "sp",
        name "linkedInformation",
        deprecated false,
        children TagChildren::Any,
        rules &[]
    );

    const SP_LINKTREE: TagDefinition = tag_definition!(
        type "sp",
        name "linktree",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("attributes", TagAttributeType::String),
            ("locale", TagAttributeType::String),
            ("localelink", TagAttributeType::Condition),
            ("name", TagAttributeType::Identifier),
            ("parentlink", TagAttributeType::String),
            ("rootelement", TagAttributeType::Object),
            ("sortkeys", TagAttributeType::String),
            ("sortsequences", TagAttributeType::String),
            ("sorttypes", TagAttributeType::String),
        rules &[
            AttributeRule::Deprecated("attributes"),
            AttributeRule::Required("name"),
            AttributeRule::OnlyWith("sortsequences", "sortkeys"),
            AttributeRule::OnlyWith("sortkeys", "sortsequences"), // OnlyBoth?
            AttributeRule::OnlyWith("sorttypes", "sortkeys"),
        ]
    );

    const SP_LIVETREE: TagDefinition = tag_definition!(
        type "sp",
        name "livetree",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("action", TagAttributeType::String),
            ("leaflink", TagAttributeType::String),
            ("locale", TagAttributeType::String),
            ("name", TagAttributeType::Identifier),
            ("node", TagAttributeType::Identifier),
            ("parentlink", TagAttributeType::String),
            ("publisher", TagAttributeType::Object),
            ("rootElement", TagAttributeType::Object),
            ("sortkeys", TagAttributeType::String),
            ("sortsequences", TagAttributeType::String),
            ("sorttypes", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("rootElement"),
            AttributeRule::Required("publisher"),
            AttributeRule::Required("parentlink"),
            AttributeRule::OnlyWith("sortsequences", "sortkeys"),
            AttributeRule::OnlyWith("sortkeys", "sortsequences"),
            AttributeRule::OnlyWith("sorttypes", "sortkeys"),
            AttributeRule::ValueOneOf("action", &["flip", "open", "close", "expand", "none"]),
        ]
    );

    const SP_LOG: TagDefinition = tag_definition!(
        type "sp",
        name "log",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("level", TagAttributeType::String),
        rules &[
            AttributeRule::Required("level"),
            AttributeRule::ValueOneOfCaseInsensitive(
                "level",
                &["TRACE", "DEBUG", "INFO", "WARN", "ERROR", "FATAL"],
            ),
        ]
    );

    const SP_LOGIN: TagDefinition = tag_definition!(
        type "sp",
        name "login",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("captcharequired", TagAttributeType::Condition),
            ("client", TagAttributeType::Object),
            ("locale", TagAttributeType::Object),
            ("login", TagAttributeType::String),
            ("password", TagAttributeType::String),
            ("scope", TagAttributeType::String),
            ("session", TagAttributeType::String),
        rules &[
            AttributeRule::ExactlyOneOf(&["session", "login", "password", "client"]),
            AttributeRule::ValueOneOf("scope", &["windowSession", "browserSession", "application"]),
        ]
    );

    const SP_LOOP: TagDefinition = tag_definition!(
        type "sp",
        name "loop",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("collection", TagAttributeType::Object),
            ("item", TagAttributeType::Identifier),
            ("list", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("separator", TagAttributeType::String),
        rules &[
            AttributeRule::ExactlyOneOf(&["collection", "list"]),
            AttributeRule::OnlyWith("separator", "list"),
        ]
    );

    const SP_MAP: TagDefinition = tag_definition!(
        type "sp",
        name "map",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("action", TagAttributeType::String),
            ("condition", TagAttributeType::Condition),
            ("default", TagAttributeType::String),
            ("expression", TagAttributeType::Expression),
            ("key", TagAttributeType::String),
            ("locale", TagAttributeType::String),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("overwrite", TagAttributeType::Condition),
            ("scope", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("action"),
            AttributeRule::ValueOneOf(
                "action",
                &[
                    "put",
                    "putNotEmpty",
                    "putAll",
                    "merge",
                    "remove",
                    "new",
                    "clear",
                ],
            ),
            AttributeRule::ValueOneOf("scope", &["page", "request"]),
            AttributeRule::ExactlyOneOfOrBodyWithEitherValue(
                &["value", "expression", "condition", "object"],
                "action",
                &["put", "putNotEmpty"],
            ),
            AttributeRule::RequiredWithValue("object", "action", "merge"),
            AttributeRule::RequiredOrBodyWithValue("object", "action", "putAll"),
            AttributeRule::RequiredWithEitherValue(
                "key",
                "action",
                &["put", "putNotEmpty", "remove"],
            ),
            AttributeRule::OnlyOneOfOrBody(&["value", "expression", "condition", "object"]),
            AttributeRule::BodyOnlyWithEitherValue("action", &["put", "putAll", "putNotEmpty"]),
            AttributeRule::OnlyWithEitherValue("value", "action", &["put", "putNotEmpty"]),
            AttributeRule::OnlyWithEitherValue("expression", "action", &["put", "putNotEmpty"]),
            AttributeRule::OnlyWithEitherValue("condition", "action", &["put", "putNotEmpty"]),
            AttributeRule::OnlyWithEitherValue(
                "object",
                "action",
                &["put", "putNotEmpty", "putAll", "merge"],
            ),
            AttributeRule::OnlyWithEitherValue("key", "action", &["put", "putNotEmpty", "remove"]),
            AttributeRule::OnlyWithEither("default", &["object", "expression"]),
            AttributeRule::OnlyWithEitherValue(
                "overwrite",
                "action",
                &["put", "putNotEmpty", "putAll", "merge"],
            ),
        ]
    );

    const SP_OPTION: TagDefinition = tag_definition!(
        type "sp",
        name "option",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("disabled", TagAttributeType::Condition),
            ("locale", TagAttributeType::String),
            ("selected", TagAttributeType::Condition),
            ("value", TagAttributeType::String),
        rules &[
            // multiple separated by commas possible for all of these:
            AttributeRule::ValueOneOf("convert", &["html2text", "wiki2html", "html2wiki"]),
            AttributeRule::ValueOneOf(
                "encoding",
                &[
                    "none",
                    "html",
                    "xml",
                    "script",
                    "php",
                    // "php<;ignore=[backslash|singleQuote|doubleQuote|carriageReturn|newLine|backspace|tab|dollar] ...>
                    "url",
                    "url; charset=latin1",
                    "entity",
                    "plain",
                    "ascii",
                    "path",
                    "filename",
                    "wikitext",
                    "base64",
                    "base64NotChunked",
                    "hex",
                    "escff",
                ],
            ),
            AttributeRule::ValueOneOf("decoding", &["none", "xml", "url", "base64", "escff"]),
            AttributeRule::ValueOneOf("encrypt", &["3des", "aes", "unixcrypt", "md5", "sha"]),
            AttributeRule::ValueOneOf("decrypt", &["3des", "aes"]),
            AttributeRule::OnlyWithEither("cryptkey", &["encrypt", "decrypt"]),
        ]
    );

    const SP_PASSWORD: TagDefinition = tag_definition!(
        type "sp",
        name "password",
        deprecated false,
        children TagChildren::Any,
        rules &[]
    );

    const SP_PRINT: TagDefinition = tag_definition!(
        type "sp",
        name "print",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("arg", TagAttributeType::Object),
            ("condition", TagAttributeType::Condition),
            ("convert", TagAttributeType::String),
            ("cryptkey", TagAttributeType::String),
            ("dateformat", TagAttributeType::String),
            ("decimalformat", TagAttributeType::String),
            ("decoding", TagAttributeType::String),
            ("decrypt", TagAttributeType::String),
            ("default", TagAttributeType::String),
            ("encoding", TagAttributeType::String),
            ("encrypt", TagAttributeType::String),
            ("expression", TagAttributeType::Expression),
            ("locale", TagAttributeType::String),
            ("name", TagAttributeType::Object),
            ("text", TagAttributeType::String),
        rules &[
            AttributeRule::Deprecated("arg"),
            AttributeRule::ExactlyOneOfOrBody(&["name", "text", "expression", "condition"]),
            AttributeRule::OnlyWithEitherOrBody("default", &["name", "expression"]),
            AttributeRule::OnlyOneOf(&["convert", "encoding", "decoding", "encrypt", "decrypt"]),
            AttributeRule::OnlyWithEither("cryptkey", &["encrypt", "decrypt"]),
            AttributeRule::OnlyOneOf(&["dateformat", "decimalformat"]),
            AttributeRule::OnlyWith("arg", "text"),
        ]
    );

    const SP_QUERYTREE: TagDefinition = tag_definition!(
        type "sp",
        name "querytree",
        deprecated false,
        children TagChildren::Any,
        rules &[]
    );

    const SP_RADIO: TagDefinition = tag_definition!(
        type "sp",
        name "radio",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("checked", TagAttributeType::Condition),
            ("disabled", TagAttributeType::Condition),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("readonly", TagAttributeType::Condition),
            ("type", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[AttributeRule::Required("name")]
    );

    const SP_RANGE: TagDefinition = tag_definition!(
        type "sp",
        name "range",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("collection", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("range", TagAttributeType::String),
            ("scope", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("collection"),
            AttributeRule::Required("range"),
            AttributeRule::ValueOneOf("scope", &["page", "request"]),
        ]
    );

    const SP_RETURN: TagDefinition = tag_definition!(
        type "sp",
        name "return",
        deprecated false,
        children TagChildren::None,
        attributes
            ("condition", TagAttributeType::Condition),
            ("default", TagAttributeType::String),
            ("expression", TagAttributeType::Expression),
            ("locale", TagAttributeType::Object),
            ("object", TagAttributeType::Object),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::ExactlyOneOfOrBody(&["value", "expression", "condition", "object"]),
            AttributeRule::OnlyWithEitherOrBody("default", &["object", "expression"]),
        ]
    );

    const SP_SASS: TagDefinition = tag_definition!(
        type "sp",
        name "sass",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("name", TagAttributeType::Identifier),
            ("options", TagAttributeType::Object),
            ("source", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("source"),
            AttributeRule::Required("options"),
        ]
    );

    const SP_SCALEIMAGE: TagDefinition = tag_definition!(
        type "sp",
        name "scaleimage",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("background", TagAttributeType::String),
            ("height", TagAttributeType::Expression),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("options", TagAttributeType::Object),
            ("padding", TagAttributeType::String),
            ("quality", TagAttributeType::String),
            ("scalesteps", TagAttributeType::Condition),
            ("scope", TagAttributeType::String),
            ("width", TagAttributeType::Expression),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::AtleastOneOf(&["height", "width"]),
            AttributeRule::Deprecated("scalesteps"),
            AttributeRule::ValueOneOf("padding", &["on", "off", "fit", "fit/no"]),
            AttributeRule::ValueOneOf("scope", &["page", "request"]),
        ]
    );

    const SP_SCOPE: TagDefinition = tag_definition!(
        type "sp",
        name "scope",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("scope", TagAttributeType::String),
        rules &[
            AttributeRule::Required("scope"),
            AttributeRule::ValueOneOf("scope", &["page", "request"]),
        ]
    );

    const SP_SEARCH: TagDefinition = tag_definition!(
        type "sp",
        name "search",
        deprecated false,
        children TagChildren::Any,
        rules &[]
    );

    const SP_SELECT: TagDefinition = tag_definition!(
        type "sp",
        name "select",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("disabled", TagAttributeType::Condition),
            ("locale", TagAttributeType::Object),
            ("multiple", TagAttributeType::Condition),
            ("name", TagAttributeType::Identifier),
            ("type", TagAttributeType::String),
        rules &[AttributeRule::Required("name")]
    );

    const SP_SET: TagDefinition = tag_definition!(
        type "sp",
        name "set",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("condition", TagAttributeType::Condition),
            ("contentType", TagAttributeType::String),
            ("default", TagAttributeType::String),
            ("expression", TagAttributeType::Expression),
            ("insert", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("overwrite", TagAttributeType::Condition),
            ("scope", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::ExactlyOneOfOrBody(&["value", "expression", "condition", "object"]),
            AttributeRule::OnlyWithEitherOrBody("default", &["object", "expression"]),
            AttributeRule::OnlyOneOf(&["overwrite", "insert"]),
            AttributeRule::ValueOneOf("scope", &["page", "request"]),
            AttributeRule::ValueOneOf("insert", &["replace", "append", "prepend"]),
            AttributeRule::ValueOneOf("contentType", &["json"]),
        ]
    );

    const SP_SORT: TagDefinition = tag_definition!(
        type "sp",
        name "sort",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("collection", TagAttributeType::Object),
            ("keys", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("scope", TagAttributeType::String),
            ("sequences", TagAttributeType::String),
            ("types", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("collection"),
            AttributeRule::ValueOneOf("scope", &["page", "request", "session"]),
        ]
    );

    const SP_SUBINFORMATION: TagDefinition = tag_definition!(
        type "sp",
        name "subinformation",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("name", TagAttributeType::Identifier),
            ("type", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::ValueOneOf("type", &["calendar"]),
        ]
    );

    const SP_TAGBODY: TagDefinition = tag_definition!(
        type "sp",
        name "tagbody",
        deprecated false,
        children TagChildren::Any,
        rules &[]
    );

    const SP_TEXT: TagDefinition = tag_definition!(
        type "sp",
        name "text",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("disabled", TagAttributeType::Condition),
            ("fixvalue", TagAttributeType::String),
            ("format", TagAttributeType::String),
            ("inputType", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("readonly", TagAttributeType::Condition),
            ("type", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
            AttributeRule::ValueOneOf("type", &["date", "email", "number", "text", "url"]),
            AttributeRule::OnlyWithEitherValue("format", "type", &["date", "number"]),
        ]
    );

    const SP_TEXTAREA: TagDefinition = tag_definition!(
        type "sp",
        name "textarea",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("disabled", TagAttributeType::Condition),
            ("fixvalue", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("readonly", TagAttributeType::Condition),
            ("type", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
        ]
    );

    const SP_TEXTIMAGE: TagDefinition = tag_definition!(
        type "sp",
        name "textimage",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("background", TagAttributeType::String),
            ("fontcolor", TagAttributeType::String),
            ("fontname", TagAttributeType::String),
            ("fontsize", TagAttributeType::Expression),
            ("fontstyle", TagAttributeType::String),
            ("gravity", TagAttributeType::String),
            ("height", TagAttributeType::Expression),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("offset", TagAttributeType::String),
            ("scope", TagAttributeType::String),
            ("text", TagAttributeType::String),
            ("width", TagAttributeType::Expression),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("text"),
            AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
            AttributeRule::ValueOneOf("fontstyle", &["plain", "bold", "italic"]),
            AttributeRule::ValueOneOf(
                "gravity",
                &[
                    "c",
                    "center",
                    "n",
                    "north",
                    "ne",
                    "northeast",
                    "e",
                    "east",
                    "se",
                    "southeast",
                    "s",
                    "south",
                    "sw",
                    "southwest",
                    "w",
                    "west",
                    "nw",
                    "northwest",
                ],
            ),
            AttributeRule::ValueOneOf("scope", &["page", "request"]),
        ]
    );

    const SP_THROW: TagDefinition = tag_definition!(
        type "sp",
        name "throw",
        deprecated false,
        children TagChildren::Any,
        rules &[]
    );

    const SP_TOGGLE: TagDefinition = tag_definition!(
        type "sp",
        name "toggle",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("disabled", TagAttributeType::Condition),
            ("fixvalue", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("offValue", TagAttributeType::Condition),
            ("onValue", TagAttributeType::Condition),
            ("readonly", TagAttributeType::Condition),
            ("type", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
        ]
    );

    const SP_UPLOAD: TagDefinition = tag_definition!(
        type "sp",
        name "upload",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
        rules &[AttributeRule::Required("name")]
    );

    const SP_URL: TagDefinition = tag_definition!(
        type "sp",
        name "url",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("absolute", TagAttributeType::Condition),
            ("command", TagAttributeType::String),
            ("context", TagAttributeType::String),
            ("gui", TagAttributeType::Condition),
            ("handler", TagAttributeType::String),
            ("information", TagAttributeType::Object),
            ("locale", TagAttributeType::Object),
            ("module", TagAttributeType::Module),
            ("publisher", TagAttributeType::Object),
            ("template", TagAttributeType::String),
            ("uri", TagAttributeType::Uri { module_attribute: "module" }),
            ("window", TagAttributeType::Condition),
        rules &[
            AttributeRule::Deprecated("command"),
            AttributeRule::Deprecated("information"),
            AttributeRule::Deprecated("publisher"),
            AttributeRule::Deprecated("absolute"),
            AttributeRule::Deprecated("gui"),
            AttributeRule::ExactlyOneOf(&["uri", "template", "command", "information"]),
            AttributeRule::OnlyOneOf(&["context", "module"]),
            AttributeRule::OnlyWith("context", "uri"),
            AttributeRule::OnlyWith("module", "uri"),
            AttributeRule::UriExists("uri", "module"),
        ]
    );

    const SP_WARNING: TagDefinition = tag_definition!(
        type "sp",
        name "warning",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("code", TagAttributeType::String),
        rules &[AttributeRule::Required("code")]
    );

    const SP_WORKLIST: TagDefinition = tag_definition!(
        type "sp",
        name "worklist",
        deprecated true,
        children TagChildren::Any,
        attributes
            ("element", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("role", TagAttributeType::Object),
            ("user", TagAttributeType::Object),
        rules &[AttributeRule::Required("name")]
    );

    const SP_ZIP: TagDefinition = tag_definition!(
        type "sp",
        name "zip",
        deprecated false,
        children TagChildren::Any,
        rules &[]
    );

    // SPTTAGS:

    const SPT_COUNTER: TagDefinition = tag_definition!(
        type "spt",
        name "counter",
        deprecated true,
        children TagChildren::None,
        attributes
            ("language", TagAttributeType::String),
            ("mode", TagAttributeType::String),
            ("name", TagAttributeType::Identifier),
            ("varName", TagAttributeType::Identifier),
            ("varname", TagAttributeType::Identifier),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Deprecated("varName"),
            AttributeRule::ValueOneOf("mode", &["read", "write"]),
            AttributeRule::ValueOneOf("language", &["javascript", "php"]),
        ]
    );

    const SPT_DATE: TagDefinition = tag_definition!(
        type "spt",
        name "date",
        deprecated false,
        children TagChildren::None,
        attributes
            ("disabled", TagAttributeType::Condition),
            ("fixvalue", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("nowButton", TagAttributeType::Condition),
            ("placeholder", TagAttributeType::String),
            ("readonly", TagAttributeType::Condition),
            ("size", TagAttributeType::Expression),
            ("type", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
            AttributeRule::ValueOneOf("type", &["date", "datetime"]),
        ]
    );

    const SPT_DIFF: TagDefinition = tag_definition!(
        type "spt",
        name "diff",
        deprecated false,
        children TagChildren::None,
        attributes
            ("from", TagAttributeType::String),
            ("style", TagAttributeType::String),
            ("to", TagAttributeType::String),
        rules &[
            AttributeRule::Required("from"),
            AttributeRule::Required("to"),
            AttributeRule::Required("style"),
        ]
    );

    const SPT_EMAIL2IMG: TagDefinition = tag_definition!(
        type "spt",
        name "email2img",
        deprecated true,
        children TagChildren::None,
        attributes
            ("alt", TagAttributeType::String),
            ("bgcolor", TagAttributeType::String),
            ("bgcolor2", TagAttributeType::String),
            ("color", TagAttributeType::String),
            ("color2", TagAttributeType::String),
            ("font", TagAttributeType::String),
            ("font2", TagAttributeType::String),
            ("fontsize", TagAttributeType::Expression),
            ("fontsize2", TagAttributeType::Expression),
            ("fontweight", TagAttributeType::String),
            ("fontweight2", TagAttributeType::String),
            ("form", TagAttributeType::Object),
            ("linkcolor", TagAttributeType::String),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("onclick", TagAttributeType::String),
            ("popupheight", TagAttributeType::Expression),
            ("popupwidth", TagAttributeType::Expression),
            ("title", TagAttributeType::String),
            ("urlparam", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("object"),
            AttributeRule::ValueOneOf("font", &["Arial", "Lucida", "Verdana", "Futura"]),
            AttributeRule::ValueOneOf("fontweight", &["plain", "bold", "italic"]),
            AttributeRule::ValueOneOf("font2", &["Arial", "Lucida", "Verdana", "Futura"]),
            AttributeRule::ValueOneOf("fontweight2", &["plain", "bold", "italic"]),
        ]
    );

    const SPT_ENCRYPTEMAIL: TagDefinition = tag_definition!(
        type "spt",
        name "encryptemail",
        deprecated false,
        children TagChildren::None,
        attributes
            ("form", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("popupheight", TagAttributeType::Expression),
            ("popupwidth", TagAttributeType::Expression),
            ("urlparam", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("object"),
        ]
    );

    const SPT_ESCAPEEMAIL: TagDefinition = tag_definition!(
        type "spt",
        name "escapeemail",
        deprecated true,
        children TagChildren::None,
        attributes
            ("alt", TagAttributeType::String),
            ("bgcolor", TagAttributeType::String),
            ("color", TagAttributeType::String),
            ("font", TagAttributeType::String),
            ("fontsize", TagAttributeType::Expression),
            ("fontweight", TagAttributeType::Expression),
            ("form", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("onclick", TagAttributeType::String),
            ("popupheight", TagAttributeType::Expression),
            ("popupwidth", TagAttributeType::Expression),
            ("title", TagAttributeType::String),
        rules &[
            AttributeRule::Required("object"),
            AttributeRule::ValueOneOf("font", &["Arial", "Lucida", "Verdana", "Futura"]),
            AttributeRule::ValueOneOf("fontweight", &["plain", "bold", "italic"]),
        ]
    );

    const SPT_FORMSOLUTIONS: TagDefinition = tag_definition!(
        type "spt",
        name "formsolutions",
        deprecated false,
        children TagChildren::None,
        attributes
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
        rules &[AttributeRule::Required("name")]
    );

    const SPT_ID2URL: TagDefinition = tag_definition!(
        type "spt",
        name "id2url",
        deprecated false,
        children TagChildren::None,
        attributes
            ("classname", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("objekt", TagAttributeType::Object),
            ("querystring", TagAttributeType::String),
            ("url", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("object"),
            AttributeRule::Required("querystring"),
            AttributeRule::ValueOneOf("url", &["relative", "absolute"]),
        ]
    );

    const SPT_ILINK: TagDefinition = tag_definition!(
        type "spt",
        name "ilink",
        deprecated false,
        children TagChildren::None,
        attributes
            ("action", TagAttributeType::String),
            ("information", TagAttributeType::Object),
            ("step", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[AttributeRule::ValueOneOf("action", &["edit", "list"])]
    );

    const SPT_IMAGEEDITOR: TagDefinition = tag_definition!(
        type "spt",
        name "imageeditor",
        deprecated false,
        children TagChildren::None,
        attributes
            ("delete", TagAttributeType::Condition),
            ("focalpoint", TagAttributeType::Condition),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
            ("width", TagAttributeType::Expression),
        rules &[]
    );

    const SPT_IMP: TagDefinition = tag_definition!(
        type "spt",
        name "imp",
        deprecated false,
        children TagChildren::None,
        attributes
            ("alt", TagAttributeType::String),
            ("background", TagAttributeType::String),
            ("color", TagAttributeType::String),
            ("excerpt", TagAttributeType::String),
            ("font", TagAttributeType::String),
            ("font-size", TagAttributeType::Expression),
            ("font-weight", TagAttributeType::String),
            ("fontcolor", TagAttributeType::String),
            ("fontname", TagAttributeType::String),
            ("fontsize", TagAttributeType::Expression),
            ("fontweight", TagAttributeType::String),
            ("format", TagAttributeType::String),
            ("gravity", TagAttributeType::String),
            ("height", TagAttributeType::Expression),
            ("image", TagAttributeType::Object),
            ("manipulate", TagAttributeType::String),
            ("offset", TagAttributeType::String),
            ("padding", TagAttributeType::String),
            ("paddingcolor", TagAttributeType::String),
            ("scalesteps", TagAttributeType::Condition),
            ("text", TagAttributeType::String),
            ("text-transform", TagAttributeType::String),
            ("transform", TagAttributeType::String),
            ("urlonly", TagAttributeType::Condition),
            ("width", TagAttributeType::Expression),
        rules &[
            AttributeRule::Required("image"),
            AttributeRule::AtleastOneOf(&["height", "width"]),
            AttributeRule::Deprecated("color"),
            AttributeRule::Deprecated("excerpt"),
            AttributeRule::Deprecated("font"),
            AttributeRule::Deprecated("font_size"),
            AttributeRule::Deprecated("font_weight"),
            AttributeRule::Deprecated("manipulate"),
            AttributeRule::Deprecated("paddingcolor"),
            AttributeRule::Deprecated("text_transform"),
            AttributeRule::Deprecated("transform"),
            AttributeRule::Deprecated("urlonly"),
            AttributeRule::ValueOneOf("format", &["png", "jpeg"]),
            AttributeRule::ValueOneOf("padding", &["on", "off", "fit", "yes", "no"]),
            AttributeRule::ValueOneOf(
                "manipulate",
                &[
                    "sharp1", "sharp2", "sharp3", "sharp4", "laplace1", "laplace2", "box",
                    "lowpass", "neon", "emboss", "bw",
                ],
            ),
            AttributeRule::ValueOneOf(
                "gravity",
                &[
                    "c",
                    "Center",
                    "n",
                    "North",
                    "ne",
                    "NorthEast",
                    "e",
                    "East",
                    "se",
                    "SouthEast",
                    "s",
                    "South",
                    "sw",
                    "SouthWest",
                    "w",
                    "West",
                    "nw",
                    "NorthWest",
                ],
            ),
            AttributeRule::ValueOneOf("transform", &["uppercase", "lowercase"]),
            AttributeRule::ValueOneOf("text_transform", &["uppercase", "lowercase"]),
        ]
    );

    const SPT_ITERATOR: TagDefinition = tag_definition!(
        type "spt",
        name "iterator",
        deprecated false,
        children TagChildren::Any,
        attributes
            ("disabled", TagAttributeType::Condition),
            ("invert", TagAttributeType::Condition),
            ("item", TagAttributeType::Identifier),
            ("itemtext", TagAttributeType::String),
            ("layout", TagAttributeType::String),
            ("max", TagAttributeType::Expression),
            ("min", TagAttributeType::Expression),
            ("name", TagAttributeType::Identifier),
            ("readonly", TagAttributeType::Condition),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::ValueOneOf("layout", &["standard", "plain"]),
        ]
    );

    const SPT_LINK: TagDefinition = tag_definition!(
        type "spt",
        name "link",
        deprecated false,
        children TagChildren::None,
        attributes
            ("filter", TagAttributeType::String),
            ("filterattribute", TagAttributeType::String),
            ("filteric", TagAttributeType::Condition),
            ("filterinvert", TagAttributeType::Condition),
            ("filtermode", TagAttributeType::String),
            ("filterquery", TagAttributeType::Query),
            ("fixvalue", TagAttributeType::String),
            ("height", TagAttributeType::Expression),
            ("hidden", TagAttributeType::Condition),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("pools", TagAttributeType::String),
            ("previewimage", TagAttributeType::Condition),
            ("showtree", TagAttributeType::Condition),
            ("size", TagAttributeType::Expression),
            ("type", TagAttributeType::String),
            ("value", TagAttributeType::String),
            ("width", TagAttributeType::Expression),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
            AttributeRule::OnlyWith("filterattribute", "filter"),
            AttributeRule::OnlyWith("filteric", "filter"),
            AttributeRule::OnlyWith("filterinvert", "filter"),
            AttributeRule::OnlyWith("filtermode", "filter"),
            AttributeRule::ValueOneOf(
                "type",
                &["systemlink", "navlink", "resultlink", "link", "image"],
            ),
            AttributeRule::ValueOneOf("filtermode", &["simple", "regex"]),
            AttributeRule::OnlyWithValue("height", "type", "image"),
            AttributeRule::OnlyWithValue("width", "type", "image"),
        ]
    );

    const SPT_NUMBER: TagDefinition = tag_definition!(
        type "spt",
        name "number",
        deprecated false,
        children TagChildren::None,
        attributes
            ("align", TagAttributeType::String),
            ("disabled", TagAttributeType::Condition),
            ("fixvalue", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("readonly", TagAttributeType::Condition),
            ("size", TagAttributeType::Expression),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
        ]
    );

    const SPT_PERSONALIZATION: TagDefinition = tag_definition!(
        type "spt",
        name "personalization",
        deprecated false,
        children TagChildren::None,
        attributes
            ("information", TagAttributeType::Object),
            ("mode", TagAttributeType::String),
            ("name", TagAttributeType::Identifier),
            ("publisher", TagAttributeType::Object),
        rules &[AttributeRule::ValueOneOf("mode", &["php"])]
    );

    const SPT_PREHTML: TagDefinition = tag_definition!(
        type "spt",
        name "prehtml",
        deprecated false,
        children TagChildren::None,
        attributes
            ("name", TagAttributeType::Identifier),
            ("object", TagAttributeType::Object),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::Required("object"),
        ]
    );

    const SPT_SMARTEDITOR: TagDefinition = tag_definition!(
        type "spt",
        name "smarteditor",
        deprecated true,
        children TagChildren::None,
        attributes
            ("cols", TagAttributeType::Expression),
            ("hide", TagAttributeType::Condition),
            ("name", TagAttributeType::Identifier),
            ("options", TagAttributeType::Object),
            ("rows", TagAttributeType::Expression),
            ("textlabel", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[AttributeRule::Required("name")]
    );

    const SPT_SPML: TagDefinition = tag_definition!(
        type "spt",
        name "spml",
        deprecated false,
        children TagChildren::None,
        attributes
            ("api", TagAttributeType::String),
        rules &[AttributeRule::ValueOneOf("api", &["log4j", "jdom", "mail"])]
    );

    const SPT_TEXT: TagDefinition = tag_definition!(
        type "spt",
        name "text",
        deprecated false,
        children TagChildren::None,
        attributes
            ("disabled", TagAttributeType::Condition),
            ("editablePlaceholder", TagAttributeType::Condition),
            ("fixvalue", TagAttributeType::String),
            ("format", TagAttributeType::String),
            ("hyphenEditor", TagAttributeType::Condition),
            ("inputType", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("readonly", TagAttributeType::Condition),
            ("size", TagAttributeType::Expression),
            ("type", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
            AttributeRule::ValueOneOf("type", &["date", "number", "text"]),
            AttributeRule::OnlyWithEitherValue("format", "type", &["date", "number"]),
        ]
    );

    const SPT_TEXTAREA: TagDefinition = tag_definition!(
        type "spt",
        name "textarea",
        deprecated false,
        children TagChildren::None,
        attributes
            ("disabled", TagAttributeType::Condition),
            ("editablePlaceholder", TagAttributeType::Condition),
            ("fixvalue", TagAttributeType::String),
            ("format", TagAttributeType::String),
            ("hyphenEditor", TagAttributeType::Condition),
            ("inputType", TagAttributeType::String),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("readonly", TagAttributeType::Condition),
            ("size", TagAttributeType::Expression),
            ("type", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
        ]
    );

    const SPT_TIMESTAMP: TagDefinition = tag_definition!(
        type "spt",
        name "timestamp",
        deprecated false,
        children TagChildren::None,
        attributes
            ("connect", TagAttributeType::Identifier),
        rules &[AttributeRule::Required("connect")]
    );

    const SPT_TINYMCE: TagDefinition = tag_definition!(
        type "spt",
        name "tinymce",
        deprecated false,
        children TagChildren::None,
        attributes
            ("cols", TagAttributeType::Expression),
            ("config", TagAttributeType::Object),
            ("configextension", TagAttributeType::Object),
            ("configvalues", TagAttributeType::Object),
            ("disabled", TagAttributeType::Condition),
            ("fixvalue", TagAttributeType::String),
            ("name", TagAttributeType::Identifier),
            ("pools", TagAttributeType::String),
            ("readonly", TagAttributeType::Condition),
            ("rows", TagAttributeType::Expression),
            ("theme", TagAttributeType::String),
            ("toggle", TagAttributeType::String),
            ("type", TagAttributeType::String),
            ("value", TagAttributeType::String),
        rules &[
            AttributeRule::Required("name"),
            AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
            AttributeRule::ValueOneOf("theme", &["simple", "advanced"]),
            AttributeRule::ValueOneOf("toggle", &["true", "false", "auto"]),
        ]
    );

    const SPT_UPDOWN: TagDefinition = tag_definition!(
        type "spt",
        name "updown",
        deprecated false,
        children TagChildren::None,
        attributes
            ("from", TagAttributeType::Expression),
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("to", TagAttributeType::Expression),
            ("value", TagAttributeType::String),
        rules &[AttributeRule::Required("name")]
    );

    const SPT_UPLOAD: TagDefinition = tag_definition!(
        type "spt",
        name "upload",
        deprecated false,
        children TagChildren::None,
        attributes
            ("locale", TagAttributeType::Object),
            ("name", TagAttributeType::Identifier),
            ("previewimage", TagAttributeType::Condition),
        rules &[AttributeRule::Required("name")]
    );

    const SPT_WORKLIST: TagDefinition = tag_definition!(
        type "spt",
        name "worklist",
        deprecated true,
        children TagChildren::None,
        attributes
            ("command", TagAttributeType::String),
            ("informationID", TagAttributeType::Object),
            ("poolID", TagAttributeType::Object),
            ("worklistID", TagAttributeType::Object),
        rules &[
            AttributeRule::Required("command"),
            AttributeRule::ValueOneOf("command", &["create", "update"]),
        ]
    );
}

pub(crate) const TOP_LEVEL_TAGS: [TagDefinition; 78] = [
    TagDefinition::SP_ATTRIBUTE,
    TagDefinition::SP_BARCODE,
    TagDefinition::SP_BREAK,
    TagDefinition::SP_CALENDARSHEET,
    TagDefinition::SP_CHECKBOX,
    TagDefinition::SP_CODE,
    TagDefinition::SP_COLLECTION,
    TagDefinition::SP_CONDITION,
    TagDefinition::SP_DIFF,
    TagDefinition::SP_ERROR,
    TagDefinition::SP_EXPIRE,
    TagDefinition::SP_FILTER,
    TagDefinition::SP_FOR,
    TagDefinition::SP_FORM,
    TagDefinition::SP_HIDDEN,
    TagDefinition::SP_IF,
    TagDefinition::SP_INCLUDE,
    TagDefinition::SP_IO,
    TagDefinition::SP_ITERATOR,
    TagDefinition::SP_JSON,
    TagDefinition::SP_LINKEDINFORMATION,
    TagDefinition::SP_LINKTREE,
    TagDefinition::SP_LIVETREE,
    TagDefinition::SP_LOG,
    TagDefinition::SP_LOGIN,
    TagDefinition::SP_LOOP,
    TagDefinition::SP_MAP,
    TagDefinition::SP_OPTION,
    TagDefinition::SP_PASSWORD,
    TagDefinition::SP_PRINT,
    TagDefinition::SP_QUERYTREE,
    TagDefinition::SP_RADIO,
    TagDefinition::SP_RANGE,
    TagDefinition::SP_RETURN,
    TagDefinition::SP_SASS,
    TagDefinition::SP_SCALEIMAGE,
    TagDefinition::SP_SCOPE,
    TagDefinition::SP_SEARCH,
    TagDefinition::SP_SELECT,
    TagDefinition::SP_SET,
    TagDefinition::SP_SORT,
    TagDefinition::SP_SUBINFORMATION,
    TagDefinition::SP_TAGBODY,
    TagDefinition::SP_TEXT,
    TagDefinition::SP_TEXTAREA,
    TagDefinition::SP_TEXTIMAGE,
    TagDefinition::SP_THROW,
    TagDefinition::SP_TOGGLE,
    TagDefinition::SP_UPLOAD,
    TagDefinition::SP_URL,
    TagDefinition::SP_WARNING,
    TagDefinition::SP_WORKLIST,
    TagDefinition::SP_ZIP,
    TagDefinition::SPT_COUNTER,
    TagDefinition::SPT_DATE,
    TagDefinition::SPT_DIFF,
    TagDefinition::SPT_EMAIL2IMG,
    TagDefinition::SPT_ENCRYPTEMAIL,
    TagDefinition::SPT_ESCAPEEMAIL,
    TagDefinition::SPT_FORMSOLUTIONS,
    TagDefinition::SPT_ID2URL,
    TagDefinition::SPT_ILINK,
    TagDefinition::SPT_IMAGEEDITOR,
    TagDefinition::SPT_IMP,
    TagDefinition::SPT_ITERATOR,
    TagDefinition::SPT_LINK,
    TagDefinition::SPT_NUMBER,
    TagDefinition::SPT_PERSONALIZATION,
    TagDefinition::SPT_PREHTML,
    TagDefinition::SPT_SMARTEDITOR,
    TagDefinition::SPT_SPML,
    TagDefinition::SPT_TEXT,
    TagDefinition::SPT_TEXTAREA,
    TagDefinition::SPT_TIMESTAMP,
    TagDefinition::SPT_TINYMCE,
    TagDefinition::SPT_UPDOWN,
    TagDefinition::SPT_UPLOAD,
    TagDefinition::SPT_WORKLIST,
];

impl FromStr for TagDefinition {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        return match string {
            "argument_tag" => Ok(TagDefinition::SP_ARGUMENT),
            "attribute_tag" => Ok(TagDefinition::SP_ATTRIBUTE),
            "barcode_tag" => Ok(TagDefinition::SP_BARCODE),
            "break_tag" => Ok(TagDefinition::SP_BREAK),
            "calendarsheet_tag" => Ok(TagDefinition::SP_CALENDARSHEET),
            "checkbox_tag" => Ok(TagDefinition::SP_CHECKBOX),
            "code_tag" => Ok(TagDefinition::SP_CODE),
            "collection_tag" => Ok(TagDefinition::SP_COLLECTION),
            "condition_tag" => Ok(TagDefinition::SP_CONDITION),
            "diff_tag" => Ok(TagDefinition::SP_DIFF),
            "else_tag" => Ok(TagDefinition::SP_ELSE),
            "elseif_tag" => Ok(TagDefinition::SP_ELSEIF),
            "error_tag" => Ok(TagDefinition::SP_ERROR),
            "expire_tag" => Ok(TagDefinition::SP_EXPIRE),
            "filter_tag" => Ok(TagDefinition::SP_FILTER),
            "for_tag" => Ok(TagDefinition::SP_FOR),
            "form_tag" => Ok(TagDefinition::SP_FORM),
            "hidden_tag" => Ok(TagDefinition::SP_HIDDEN),
            "if_tag" => Ok(TagDefinition::SP_IF),
            "include_tag" => Ok(TagDefinition::SP_INCLUDE),
            "io_tag" => Ok(TagDefinition::SP_IO),
            "iterator_tag" => Ok(TagDefinition::SP_ITERATOR),
            "json_tag" => Ok(TagDefinition::SP_JSON),
            "linktree_tag" => Ok(TagDefinition::SP_LINKTREE),
            "linkedInformation_tag" => Ok(TagDefinition::SP_LINKEDINFORMATION),
            "livetree_tag" => Ok(TagDefinition::SP_LIVETREE),
            "log_tag" => Ok(TagDefinition::SP_LOG),
            "login_tag" => Ok(TagDefinition::SP_LOGIN),
            "loop_tag" => Ok(TagDefinition::SP_LOOP),
            "map_tag" => Ok(TagDefinition::SP_MAP),
            "option_tag" => Ok(TagDefinition::SP_OPTION),
            "password_tag" => Ok(TagDefinition::SP_PASSWORD),
            "print_tag" => Ok(TagDefinition::SP_PRINT),
            "querytree_tag" => Ok(TagDefinition::SP_QUERYTREE),
            "radio_tag" => Ok(TagDefinition::SP_RADIO),
            "range_tag" => Ok(TagDefinition::SP_RANGE),
            "return_tag" => Ok(TagDefinition::SP_RETURN),
            "sass_tag" => Ok(TagDefinition::SP_SASS),
            "scaleimage_tag" => Ok(TagDefinition::SP_SCALEIMAGE),
            "scope_tag" => Ok(TagDefinition::SP_SCOPE),
            "search_tag" => Ok(TagDefinition::SP_SEARCH),
            "select_tag" => Ok(TagDefinition::SP_SELECT),
            "set_tag" => Ok(TagDefinition::SP_SET),
            "sort_tag" => Ok(TagDefinition::SP_SORT),
            "subinformation_tag" => Ok(TagDefinition::SP_SUBINFORMATION),
            "tagbody_tag" => Ok(TagDefinition::SP_TAGBODY),
            "text_tag" => Ok(TagDefinition::SP_TEXT),
            "textarea_tag" => Ok(TagDefinition::SP_TEXTAREA),
            "textimage_tag" => Ok(TagDefinition::SP_TEXTIMAGE),
            "throw_tag" => Ok(TagDefinition::SP_THROW),
            "toggle_tag" => Ok(TagDefinition::SP_TOGGLE),
            "upload_tag" => Ok(TagDefinition::SP_UPLOAD),
            "url_tag" => Ok(TagDefinition::SP_URL),
            "warning_tag" => Ok(TagDefinition::SP_WARNING),
            "worklist_tag" => Ok(TagDefinition::SP_WORKLIST),
            "zip_tag" => Ok(TagDefinition::SP_ZIP),
            "spt_counter_tag" => Ok(TagDefinition::SPT_COUNTER),
            "spt_date_tag" => Ok(TagDefinition::SPT_DATE),
            "spt_diff_tag" => Ok(TagDefinition::SPT_DIFF),
            "spt_email2img_tag" => Ok(TagDefinition::SPT_EMAIL2IMG),
            "spt_encryptemail_tag" => Ok(TagDefinition::SPT_ENCRYPTEMAIL),
            "spt_escapeemail_tag" => Ok(TagDefinition::SPT_ESCAPEEMAIL),
            "spt_formsolutions_tag" => Ok(TagDefinition::SPT_FORMSOLUTIONS),
            "spt_id2url_tag" => Ok(TagDefinition::SPT_ID2URL),
            "spt_ilink_tag" => Ok(TagDefinition::SPT_ILINK),
            "spt_imageeditor_tag" => Ok(TagDefinition::SPT_IMAGEEDITOR),
            "spt_imp_tag" => Ok(TagDefinition::SPT_IMP),
            "spt_iterator_tag" => Ok(TagDefinition::SPT_ITERATOR),
            "spt_link_tag" => Ok(TagDefinition::SPT_LINK),
            "spt_number_tag" => Ok(TagDefinition::SPT_NUMBER),
            "spt_personalization_tag" => Ok(TagDefinition::SPT_PERSONALIZATION),
            "spt_prehtml_tag" => Ok(TagDefinition::SPT_PREHTML),
            "spt_smarteditor_tag" => Ok(TagDefinition::SPT_SMARTEDITOR),
            "spt_spml_tag" => Ok(TagDefinition::SPT_SPML),
            "spt_text_tag" => Ok(TagDefinition::SPT_TEXT),
            "spt_textarea_tag" => Ok(TagDefinition::SPT_TEXTAREA),
            "spt_timestamp_tag" => Ok(TagDefinition::SPT_TIMESTAMP),
            "spt_tinymce_tag" => Ok(TagDefinition::SPT_TINYMCE),
            "spt_updown_tag" => Ok(TagDefinition::SPT_UPDOWN),
            "spt_upload_tag" => Ok(TagDefinition::SPT_UPLOAD),
            "spt_worklist_tag" => Ok(TagDefinition::SPT_WORKLIST),
            tag => Err(anyhow::anyhow!("not a valid tag: \"{}\"", tag)),
        };
    }
}
