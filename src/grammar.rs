use anyhow::{Error, Result};
use std::{slice::Iter, str::FromStr};

#[derive(Debug)]
pub(crate) struct TagProperties {
    pub(crate) name: &'static str,
    pub(crate) detail: Option<&'static str>,
    pub(crate) documentation: Option<&'static str>,
    pub(crate) children: TagChildren,
    pub(crate) attribute_rules: &'static [AttributeRule],
}

#[derive(Debug)]
pub(crate) enum AttributeRule {
    Deprecated(&'static str),
    ExactlyOneOf(&'static [&'static str]),
    OnlyOneOf(&'static [&'static str]),
    AtleastOneOf(&'static [&'static str]),
    OnlyWith(&'static str, &'static str),
    OnlyWithEither(&'static str, &'static [&'static str]),
    Required(&'static str),
    // TODO:
    // OnlyIfAttributeHasValue
    // Body?!?
}

#[derive(Debug)]
pub(crate) enum TagChildren {
    Any,
    None,
    Scalar(SpTag),
    Vector(&'static [SpTag]),
}

#[derive(Debug, PartialEq)]
pub(crate) enum SpTag {
    Argument,
    Attribute,
    Barcode,
    Break,
    Calendarsheet,
    Checkbox,
    Code,
    Collection,
    Condition,
    Diff,
    Else,
    Elseif,
    Error,
    Expire,
    Filter,
    For,
    Form,
    Hidden,
    If,
    Include,
    Io,
    Iterator,
    Json,
    Linktree,
    LinkedInformation,
    Livetree,
    Log,
    Login,
    Loop,
    Map,
    Option,
    Password,
    Print,
    Querytree,
    Radio,
    Range,
    Return,
    Sass,
    Scaleimage,
    Scope,
    Search,
    Select,
    Set,
    Sort,
    Subinformation,
    Tagbody,
    Text,
    Textarea,
    Textimage,
    Throw,
    Toggle,
    Upload,
    Url,
    Warning,
    Worklist,
    Zip,
}

const ARGUMENT: TagProperties = TagProperties {
    name: "sp:argument",
    detail: None,
    documentation: Some(
        r#"
Setzt ein Argument für ein sp:include"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::ExactlyOneOf(&["value", "expression", "condition", "object"]), // or body
        AttributeRule::OnlyWithEither("default", &["object", "expression"]),
    ],
};

const ATTRIBUTE: TagProperties = TagProperties {
    name: "sp:attribute",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Deprecated("name"),
        AttributeRule::ExactlyOneOf(&["name", "text", "object", "dynamics"]),
    ],
};

const BARCODE: TagProperties = TagProperties {
    name: "sp:barcode",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("text"),
        AttributeRule::Required("type"),
    ],
};

const BREAK: TagProperties = TagProperties {
    name: "sp:break",
    detail: None,
    documentation: Some(
        r#"
Beendet FOR- und ITERATE-Schleifen."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[],
};

const CALENDARSHEET: TagProperties = TagProperties {
    name: "sp:calendarsheet",
    detail: None,
    documentation: Some(
        r#"
CalendarSheet manage dates and objects"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("action"),
        AttributeRule::Required("from"),
        AttributeRule::Required("to"),
        AttributeRule::ExactlyOneOf(&["value", "object", "date"]),
    ],
};

const CHECKBOX: TagProperties = TagProperties {
    name: "sp:checkbox",
    detail: None,
    documentation: Some(
        r#"
Check-Box-Tag, erzeugt eine checkBox."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[AttributeRule::Required("name")],
};

const CODE: TagProperties = TagProperties {
    name: "sp:code",
    detail: None,
    documentation: Some(
        r#"
Schreibt den bodyContent ohne dass dieser ausgeführt wird in die Ergebnis-Datei."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[],
};

const COLLECTION: TagProperties = TagProperties {
    name: "sp:collection",
    detail: None,
    documentation: Some(
        r#"
Collection tag offers certain operation that deal with a common collection. For further description see the javadoc of the class com.sitepark.ies.taglib.core.CollectionTag."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::ExactlyOneOf(&["action", "query"]),
        AttributeRule::ExactlyOneOf(&["value", "object", "expression", "condition"]), // or body
                                                                                      // index is required if "value" is "remove" or "replace"
    ],
};

const CONDITION: TagProperties = TagProperties {
    name: "sp:condition",
    detail: None,
    documentation: Some(
        r#"
Umklammert einen if-else Konstrukt."#,
    ),
    children: TagChildren::Vector(&[SpTag::If, SpTag::Else, SpTag::Elseif]),
    attribute_rules: &[],
};

const DIFF: TagProperties = TagProperties {
    name: "sp:diff",
    detail: None,
    documentation: Some(
        r#"
Vergleicht ein Attribute von zwei Versionen einer Information"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("from"),
        AttributeRule::Required("to"),
        AttributeRule::OnlyWith("lookup", "locale"), // is that correct?
    ],
};

const ELSE: TagProperties = TagProperties {
    name: "sp:else",
    detail: None,
    documentation: Some(
        r#"
passendes else zu einem If innerhalb eines contitionTag."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[],
};

const ELSEIF: TagProperties = TagProperties {
    name: "sp:elseif",
    detail: None,
    documentation: Some(
        r#"
ElseIf-Tag, schreibt Body wenn Bedingung ok ist und vorheriges if fehl schlug."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
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
        AttributeRule::OnlyWithEither("ic", &["eq", "neq", "gt", "gte", "lt", "lte", "contains"]),
    ],
};

const ERROR: TagProperties = TagProperties {
    name: "sp:error",
    detail: None,
    documentation: Some(
        r#"
Prüft ein Fehler aufgetreten ist, markiert ihn gegebenenfals als gefangen und führt den innhalt des Tags aus."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[AttributeRule::Required("code")],
};

const EXPIRE: TagProperties = TagProperties {
    name: "sp:expire",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[AttributeRule::Required("date")],
};

const FILTER: TagProperties = TagProperties {
    name: "sp:filter",
    detail: None,
    documentation: Some(
        r#"
Filtert eine Liste"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("collection"),
        AttributeRule::OnlyWithEither("ic", &["filter"]),
        AttributeRule::OnlyWithEither("type", &["from", "to"]),
        AttributeRule::OnlyWithEither("format", &["from", "to"]),
    ],
};

const FOR: TagProperties = TagProperties {
    name: "sp:for",
    detail: None,
    documentation: Some(
        r#"
For-Tag, wiederholt solange wie angegeben."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("index"),
        AttributeRule::Required("from"),
        AttributeRule::ExactlyOneOf(&["to", "condition"]),
    ],
};

const FORM: TagProperties = TagProperties {
    name: "sp:form",
    detail: None,
    documentation: Some(
        r#"
Erzeugt ein HTML-Form-Tag mit einem angepassten Kommando"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Deprecated("command"),
        AttributeRule::OnlyOneOf(&["uri", "template"]),
        AttributeRule::OnlyWith("module", "uri"),
    ],
};

const HIDDEN: TagProperties = TagProperties {
    name: "sp:hidden",
    detail: None,
    documentation: Some(
        r#"
Hidden-Tag, erzeugt ein Hiddenfeld."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::ExactlyOneOf(&["value", "fixvalue"]),
    ],
};

const IF: TagProperties = TagProperties {
    name: "sp:if",
    detail: None,
    documentation: Some(
        r#"
If-Tag, schreibt Body wenn Bedingung ok ist."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
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
        AttributeRule::OnlyWithEither("ic", &["eq", "neq", "gt", "gte", "lt", "lte", "contains"]),
    ],
};

const INCLUDE: TagProperties = TagProperties {
    name: "sp:include",
    detail: None,
    documentation: Some(
        r#"
includiert ein anderes bereits im System gespeichertes Template."#,
    ),
    children: TagChildren::Scalar(SpTag::Argument),
    attribute_rules: &[
        AttributeRule::ExactlyOneOf(&["template", "anchor", "uri"]),
        AttributeRule::OnlyOneOf(&["context", "module"]),
        AttributeRule::OnlyWith("context", "uri"),
        AttributeRule::OnlyWith("module", "uri"),
    ],
};

const IO: TagProperties = TagProperties {
    name: "sp:io",
    detail: None,
    documentation: Some(
        r#"
IO-Tag"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("type"),
    ],
};

const ITERATOR: TagProperties = TagProperties {
    name: "sp:iterator",
    detail: None,
    documentation: Some(
        r#"
Wird für den Aufbau von Wiederholfeldern verwendet."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("collection"),
    ],
};

const JSON: TagProperties = TagProperties {
    name: "sp:json",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[],
};

const LINKEDINFORMATION: TagProperties = TagProperties {
    name: "sp:linkedInformation",
    detail: None,
    documentation: Some(
        r#"
Diese Tag definiert einen Link eines Artikels auf einen Anderen Artikel. Das Besondere ist, dass der Artikel auf dem Verlinkt wird erst innerhalb dieses tags definiert wird. Dazu müssen alle Paramter wie parent, filename, usw. vorhanden sein. Mit dem Reques können dann schliesslich beide Artikel ubgedatet werden(oder auch erstellt)."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[], // not documented
};

const LINKTREE: TagProperties = TagProperties {
    name: "sp:linktree",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Deprecated("attributes"),
        AttributeRule::Required("name"),
        AttributeRule::OnlyWith("sortsequences", "sortkeys"),
        AttributeRule::OnlyWith("sortkeys", "sortsequences"), // OnlyBoth?
        AttributeRule::OnlyWith("sorttypes", "sortkeys"),
    ],
};

const LIVETREE: TagProperties = TagProperties {
    name: "sp:livetree",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("rootElement"),
        AttributeRule::Required("publisher"),
        AttributeRule::Required("parentlink"),
        AttributeRule::OnlyWith("sortsequences", "sortkeys"),
        AttributeRule::OnlyWith("sortkeys", "sortsequences"), // OnlyBoth?
        AttributeRule::OnlyWith("sorttypes", "sortkeys"),
    ],
};

const LOG: TagProperties = TagProperties {
    name: "sp:log",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("level"),
    ],
};

const LOGIN: TagProperties = TagProperties {
    name: "sp:login",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::ExactlyOneOf(&["session", "login", "password", "client"]),
    ],
};

const LOOP: TagProperties = TagProperties {
    name: "sp:loop",
    detail: None,
    documentation: Some(
        r#"
Dient zur Ausgabe eines oder mehrerer Elemente."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::ExactlyOneOf(&["collection", "list"]),
        AttributeRule::OnlyWith("separator", "list"),
    ],
};

const MAP: TagProperties = TagProperties {
    name: "sp:map",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("action"),
        // depending on the action, key or the value/expression/.. may or may not be required
        AttributeRule::OnlyOneOf(&["value", "expression", "condition", "object"]), // or body
        AttributeRule::OnlyWithEither("default", &["object", "expression"]),
    ],
};

const OPTION: TagProperties = TagProperties {
    name: "sp:option",
    detail: None,
    documentation: Some(
        r#"
Option-Tag, für das Select Tag."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[],
};

const PASSWORD: TagProperties = TagProperties {
    name: "sp:password",
    detail: None,
    documentation: Some(
        r#"
Password-Tag, erzeugt ein Passwordfeld."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[], // not documented
};

const PRINT: TagProperties = TagProperties {
    name: "sp:print",
    detail: None,
    documentation: Some(
        r#"
Dient zur Ausgabe eines Attributes"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Deprecated("arg"),
        AttributeRule::ExactlyOneOf(&["name", "text", "expression", "condition"]),
        AttributeRule::OnlyWithEither("default", &["name", "expression"]),
        AttributeRule::OnlyOneOf(&["convert", "encoding", "decoding", "encrypt", "decrypt"]),
        AttributeRule::OnlyWithEither("cryptkey", &["encrypt", "decrypt"]),
        AttributeRule::OnlyOneOf(&["dateformat", "decimalformat"]),
        AttributeRule::OnlyWith("arg", "text"),
    ],
};

const QUERYTREE: TagProperties = TagProperties {
    name: "sp:querytree",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[], // not documented
};

const RADIO: TagProperties = TagProperties {
    name: "sp:radio",
    detail: None,
    documentation: Some(
        r#"
Radio Button-Tag, erzeugt einen RadioButton."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
    ],
};

const RANGE: TagProperties = TagProperties {
    name: "sp:range",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("collection"),
        AttributeRule::Required("range"),
    ],
};

const RETURN: TagProperties = TagProperties {
    name: "sp:return",
    detail: None,
    documentation: Some(
        r#"
Verlässt die SPML-Seite und setzt ggf. einen Rückgabewert für sp:include"#,
    ),
    children: TagChildren::None,
    attribute_rules: &[
        AttributeRule::ExactlyOneOf(&["value", "expression", "condition", "object"]), // or body
        AttributeRule::OnlyWithEither("default", &["object", "expression"]),
    ],
};

const SASS: TagProperties = TagProperties {
    name: "sp:sass",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("source"),
        AttributeRule::Required("options"),
    ],
};

const SCALEIMAGE: TagProperties = TagProperties {
    name: "sp:scaleimage",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::AtleastOneOf(&["height", "width"]),
        AttributeRule::Deprecated("scalesteps"),
    ],
};

const SCOPE: TagProperties = TagProperties {
    name: "sp:scope",
    detail: None,
    documentation: Some(
        r#"
Setzt bereichsweise oder global den Scope für die folgenden Tags"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("scope"),
    ],
};

const SEARCH: TagProperties = TagProperties {
    name: "sp:search",
    detail: None,
    documentation: Some(
        r#"
Findet die gewünschte Suche"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[], // not documented
};

const SELECT: TagProperties = TagProperties {
    name: "sp:select",
    detail: None,
    documentation: Some(
        r#"
Select-Tag, erzeugt den Rahmen einen Auswahlliste."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
    ],
};

const SET: TagProperties = TagProperties {
    name: "sp:set",
    detail: None,
    documentation: Some(
        r#"
Setzt ein Attribute"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::ExactlyOneOf(&["value", "expression", "condition", "object"]), // or body
        AttributeRule::OnlyWithEither("default", &["object", "expression"]),
        AttributeRule::OnlyOneOf(&["overwrite", "insert"]),
    ],
};

const SORT: TagProperties = TagProperties {
    name: "sp:sort",
    detail: None,
    documentation: Some(
        r#"
Sortiert eine Liste"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("collection"),
    ],
};

const SUBINFORMATION: TagProperties = TagProperties {
    name: "sp:subinformation",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
    ],
};

const TAGBODY: TagProperties = TagProperties {
    name: "sp:tagbody",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[],
};

const TEXT: TagProperties = TagProperties {
    name: "sp:text",
    detail: None,
    documentation: Some(
        r#"
Text-Tag, erzeugt ein Eingabefeld."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const TEXTAREA: TagProperties = TagProperties {
    name: "sp:textarea",
    detail: None,
    documentation: Some(
        r#"
Textarea-Tag, erzeugt einen Einabebereich."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const TEXTIMAGE: TagProperties = TagProperties {
    name: "sp:textimage",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("text"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const THROW: TagProperties = TagProperties {
    name: "sp:throw",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[], // not documented
};

const TOGGLE: TagProperties = TagProperties {
    name: "sp:toggle",
    detail: None,
    documentation: Some(
        r#"
Toggle-Tag erzeugt einen toggle der einen einzigen boolischen Wert speichert"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const UPLOAD: TagProperties = TagProperties {
    name: "sp:upload",
    detail: None,
    documentation: Some(
        r#"
Das Tag, erzeugt ein Eingabefeld zum Herunderladen von Dateien."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
    ],
};

const URL: TagProperties = TagProperties {
    name: "sp:url",
    detail: None,
    documentation: Some(
        r#"
Fügt den ContextPath vor die angegebene URL und hängt, falls nötig die Session ID an die URL."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Deprecated("command"),
        AttributeRule::Deprecated("information"),
        AttributeRule::Deprecated("publisher"),
        AttributeRule::Deprecated("absolute"),
        AttributeRule::Deprecated("gui"),
        AttributeRule::ExactlyOneOf(&["uri", "template", "command", "information"]),
        AttributeRule::OnlyOneOf(&["context", "module"]),
        AttributeRule::OnlyWith("context", "uri"),
        AttributeRule::OnlyWith("module", "uri"),
    ],
};

const WARNING: TagProperties = TagProperties {
    name: "sp:warning",
    detail: None,
    documentation: Some(
        r#"
Prüft, ob eine Warnung aufgetreten ist, markiert sie gegebenenfalls als gefangen und führt den innhalt des Tags aus."#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("code"),
    ],
};

const WORKLIST: TagProperties = TagProperties {
    name: "sp:worklist",
    detail: None,
    documentation: Some(
        r#"
Findet die gewünschte Workliste"#,
    ),
    children: TagChildren::Any,
    attribute_rules: &[
        AttributeRule::Required("name"),
    ],
};

const ZIP: TagProperties = TagProperties {
    name: "sp:zip",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attribute_rules: &[], // not documented
};

impl FromStr for SpTag {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        return match string {
            "argument_tag" => Ok(SpTag::Argument),
            "attribute_tag" => Ok(SpTag::Attribute),
            "barcode_tag" => Ok(SpTag::Barcode),
            "break_tag" => Ok(SpTag::Break),
            "calendarsheet_tag" => Ok(SpTag::Calendarsheet),
            "checkbox_tag" => Ok(SpTag::Checkbox),
            "code_tag" => Ok(SpTag::Code),
            "collection_tag" => Ok(SpTag::Collection),
            "condition_tag" => Ok(SpTag::Condition),
            "diff_tag" => Ok(SpTag::Diff),
            "else_tag" => Ok(SpTag::Else),
            "elseif_tag" => Ok(SpTag::Elseif),
            "error_tag" => Ok(SpTag::Error),
            "expire_tag" => Ok(SpTag::Expire),
            "filter_tag" => Ok(SpTag::Filter),
            "for_tag" => Ok(SpTag::For),
            "form_tag" => Ok(SpTag::Form),
            "hidden_tag" => Ok(SpTag::Hidden),
            "if_tag" => Ok(SpTag::If),
            "include_tag" => Ok(SpTag::Include),
            "io_tag" => Ok(SpTag::Io),
            "iterator_tag" => Ok(SpTag::Iterator),
            "json_tag" => Ok(SpTag::Json),
            "linktree_tag" => Ok(SpTag::Linktree),
            "linkedInformation_tag" => Ok(SpTag::LinkedInformation),
            "livetree_tag" => Ok(SpTag::Livetree),
            "log_tag" => Ok(SpTag::Log),
            "login_tag" => Ok(SpTag::Login),
            "loop_tag" => Ok(SpTag::Loop),
            "map_tag" => Ok(SpTag::Map),
            "option_tag" => Ok(SpTag::Option),
            "password_tag" => Ok(SpTag::Password),
            "print_tag" => Ok(SpTag::Print),
            "querytree_tag" => Ok(SpTag::Querytree),
            "radio_tag" => Ok(SpTag::Radio),
            "range_tag" => Ok(SpTag::Range),
            "return_tag" => Ok(SpTag::Return),
            "sass_tag" => Ok(SpTag::Sass),
            "scaleimage_tag" => Ok(SpTag::Scaleimage),
            "scope_tag" => Ok(SpTag::Scope),
            "search_tag" => Ok(SpTag::Search),
            "select_tag" => Ok(SpTag::Select),
            "set_tag" => Ok(SpTag::Set),
            "sort_tag" => Ok(SpTag::Sort),
            "subinformation_tag" => Ok(SpTag::Subinformation),
            "tagbody_tag" => Ok(SpTag::Tagbody),
            "text_tag" => Ok(SpTag::Text),
            "textarea_tag" => Ok(SpTag::Textarea),
            "textimage_tag" => Ok(SpTag::Textimage),
            "throw_tag" => Ok(SpTag::Throw),
            "toggle_tag" => Ok(SpTag::Toggle),
            "upload_tag" => Ok(SpTag::Upload),
            "url_tag" => Ok(SpTag::Url),
            "warning_tag" => Ok(SpTag::Warning),
            "worklist_tag" => Ok(SpTag::Worklist),
            "zip_tag" => Ok(SpTag::Zip),
            tag => Err(anyhow::anyhow!("not a valid tag: \"{}\"", tag)),
        };
    }
}

impl SpTag {
    pub fn properties(&self) -> TagProperties {
        return match self {
            SpTag::Argument => ARGUMENT,
            SpTag::Attribute => ATTRIBUTE,
            SpTag::Barcode => BARCODE,
            SpTag::Break => BREAK,
            SpTag::Calendarsheet => CALENDARSHEET,
            SpTag::Checkbox => CHECKBOX,
            SpTag::Code => CODE,
            SpTag::Collection => COLLECTION,
            SpTag::Condition => CONDITION,
            SpTag::Diff => DIFF,
            SpTag::Else => ELSE,
            SpTag::Elseif => ELSEIF,
            SpTag::Error => ERROR,
            SpTag::Expire => EXPIRE,
            SpTag::Filter => FILTER,
            SpTag::For => FOR,
            SpTag::Form => FORM,
            SpTag::Hidden => HIDDEN,
            SpTag::If => IF,
            SpTag::Include => INCLUDE,
            SpTag::Io => IO,
            SpTag::Iterator => ITERATOR,
            SpTag::Json => JSON,
            SpTag::Linktree => LINKTREE,
            SpTag::LinkedInformation => LINKEDINFORMATION,
            SpTag::Livetree => LIVETREE,
            SpTag::Log => LOG,
            SpTag::Login => LOGIN,
            SpTag::Loop => LOOP,
            SpTag::Map => MAP,
            SpTag::Option => OPTION,
            SpTag::Password => PASSWORD,
            SpTag::Print => PRINT,
            SpTag::Querytree => QUERYTREE,
            SpTag::Radio => RADIO,
            SpTag::Range => RANGE,
            SpTag::Return => RETURN,
            SpTag::Sass => SASS,
            SpTag::Scaleimage => SCALEIMAGE,
            SpTag::Scope => SCOPE,
            SpTag::Search => SEARCH,
            SpTag::Select => SELECT,
            SpTag::Set => SET,
            SpTag::Sort => SORT,
            SpTag::Subinformation => SUBINFORMATION,
            SpTag::Tagbody => TAGBODY,
            SpTag::Text => TEXT,
            SpTag::Textarea => TEXTAREA,
            SpTag::Textimage => TEXTIMAGE,
            SpTag::Throw => THROW,
            SpTag::Toggle => TOGGLE,
            SpTag::Upload => UPLOAD,
            SpTag::Url => URL,
            SpTag::Warning => WARNING,
            SpTag::Worklist => WORKLIST,
            SpTag::Zip => ZIP,
        };
    }

    pub fn iter() -> Iter<'static, SpTag> {
        static SP_TAGS: [SpTag; 56] = [
            SpTag::Argument,
            SpTag::Attribute,
            SpTag::Barcode,
            SpTag::Break,
            SpTag::Calendarsheet,
            SpTag::Checkbox,
            SpTag::Code,
            SpTag::Collection,
            SpTag::Condition,
            SpTag::Diff,
            SpTag::Else,
            SpTag::Elseif,
            SpTag::Error,
            SpTag::Expire,
            SpTag::Filter,
            SpTag::For,
            SpTag::Form,
            SpTag::Hidden,
            SpTag::If,
            SpTag::Include,
            SpTag::Io,
            SpTag::Iterator,
            SpTag::Json,
            SpTag::Linktree,
            SpTag::LinkedInformation,
            SpTag::Livetree,
            SpTag::Log,
            SpTag::Login,
            SpTag::Loop,
            SpTag::Map,
            SpTag::Option,
            SpTag::Password,
            SpTag::Print,
            SpTag::Querytree,
            SpTag::Radio,
            SpTag::Range,
            SpTag::Return,
            SpTag::Sass,
            SpTag::Scaleimage,
            SpTag::Scope,
            SpTag::Search,
            SpTag::Select,
            SpTag::Set,
            SpTag::Sort,
            SpTag::Subinformation,
            SpTag::Tagbody,
            SpTag::Text,
            SpTag::Textarea,
            SpTag::Textimage,
            SpTag::Throw,
            SpTag::Toggle,
            SpTag::Upload,
            SpTag::Url,
            SpTag::Warning,
            SpTag::Worklist,
            SpTag::Zip,
        ];
        return SP_TAGS.iter();
    }
}
