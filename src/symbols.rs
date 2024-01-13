use lsp_types::{Documentation, MarkupContent, MarkupKind};
use std::slice::Iter;

#[derive(Debug)]
pub(crate) struct TagProperties {
    pub name: String,
    pub detail: Option<String>,
    pub documentation: Option<Documentation>,
    // pub children: TagChildren,
}

// #[derive(Debug)]
// pub(crate) enum TagChildren {
//     Any,
//     None,
//     Scalar(SpTag),
//     Vector(Vec<SpTag>),
// }

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
    Upload,
    Url,
    Warning,
    Worklist,
    Zip,
}

impl SpTag {
    pub fn properties(&self) -> TagProperties {
        return match self {
            SpTag::Argument => TagProperties {
                name: String::from("<sp:argument"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Setzt ein Argument für ein sp:include"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Attribute => TagProperties {
                name: String::from("<sp:attribute"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Barcode => TagProperties {
                name: String::from("<sp:barcode"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Break => TagProperties {
                name: String::from("<sp:break"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Beendet FOR- und ITERATE-Schleifen."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Calendarsheet => TagProperties {
                name: String::from("<sp:calendarsheet"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
CalendarSheet manage dates and objects"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Checkbox => TagProperties {
                name: String::from("<sp:checkbox"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Check-Box-Tag, erzeugt eine checkBox."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Code => TagProperties {
                name: String::from("<sp:code"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Schreibt den bodyContent ohne dass dieser ausgeführt wird in die Ergebnis-Datei."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Collection => TagProperties {
                name: String::from("<sp:collection"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Collection tag offers certain operation that deal with a common collection. For further description see the javadoc of the class com.sitepark.ies.taglib.core.CollectionTag."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Condition => TagProperties {
                name: String::from("<sp:condition"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Umklammert einen if-else Konstrukt."#,
                    ),
                })),
                // children: TagChildren::Vector(vec![SpTag::If, SpTag::Else, SpTag::Elseif]),
            },
            SpTag::Diff => TagProperties {
                name: String::from("<sp:diff"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Vergleicht ein Attribute von zwei Versionen einer Information"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Else => TagProperties {
                name: String::from("<sp:else"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
passendes else zu einem If innerhalb eines contitionTag."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Elseif => TagProperties {
                name: String::from("<sp:elseif"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
ElseIf-Tag, schreibt Body wenn Bedingung ok ist und vorheriges if fehl schlug."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Error => TagProperties {
                name: String::from("<sp:error"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Prüft ein Fehler aufgetreten ist, markiert ihn gegebenenfals als gefangen und führt den innhalt des Tags aus."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Expire => TagProperties {
                name: String::from("<sp:expire"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Filter => TagProperties {
                name: String::from("<sp:filter"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Filtert eine Liste"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::For => TagProperties {
                name: String::from("<sp:for"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
For-Tag, wiederholt solange wie angegeben."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Form => TagProperties {
                name: String::from("<sp:form"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Erzeugt ein HTML-Form-Tag mit einem angepassten Kommando"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Hidden => TagProperties {
                name: String::from("<sp:hidden"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Hidden-Tag, erzeugt ein Hiddenfeld."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::If => TagProperties {
                name: String::from("<sp:if"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
If-Tag, schreibt Body wenn Bedingung ok ist."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Include => TagProperties {
                name: String::from("<sp:include"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
includiert ein anderes bereits im System gespeichertes Template."#,
                    ),
                })),
                // children: TagChildren::Scalar(SpTag::Argument),
            },
            SpTag::Io => TagProperties {
                name: String::from("<sp:io"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
IO-Tag"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Iterator => TagProperties {
                name: String::from("<sp:iterator"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Wird für den Aufbau von Wiederholfeldern verwendet."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Json => TagProperties {
                name: String::from("<sp:json"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::LinkedInformation => TagProperties {
                name: String::from("<sp:linkedInformation"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Diese Tag definiert einen Link eines Artikels auf einen Anderen Artikel. Das Besondere ist, dass der Artikel auf dem Verlinkt wird erst innerhalb dieses tags definiert wird. Dazu müssen alle Paramter wie parent, filename, usw. vorhanden sein. Mit dem Reques können dann schliesslich beide Artikel ubgedatet werden(oder auch erstellt).
"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Linktree => TagProperties {
                name: String::from("<sp:linktree"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Livetree => TagProperties {
                name: String::from("<sp:livetree"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Log => TagProperties {
                name: String::from("<sp:log"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Login => TagProperties {
                name: String::from("<sp:login"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Loop => TagProperties {
                name: String::from("<sp:loop"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Dient zur Ausgabe eines oder mehrerer Elemente."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Map => TagProperties {
                name: String::from("<sp:map"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Option => TagProperties {
                name: String::from("<sp:option"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Option-Tag, für das Select Tag."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Password => TagProperties {
                name: String::from("<sp:password"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Password-Tag, erzeugt ein Passwordfeld."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Print => TagProperties {
                name: String::from("<sp:print"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Dient zur Ausgabe eines Attributes"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Querytree => TagProperties {
                name: String::from("<sp:querytree"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Radio => TagProperties {
                name: String::from("<sp:radio"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Radio Button-Tag, erzeugt einen RadioButton."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Range => TagProperties {
                name: String::from("<sp:range"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Return => TagProperties {
                name: String::from("<sp:return"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Verlässt die SPML-Seite und setzt ggf. einen Rückgabewert für sp:include"#,
                    ),
                })),
                // children: TagChildren::None,
            },
            SpTag::Sass => TagProperties {
                name: String::from("<sp:sass"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Scaleimage => TagProperties {
                name: String::from("<sp:scaleimage"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Scope => TagProperties {
                name: String::from("<sp:scope"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Setzt bereichsweise oder global den Scope für die folgenden Tags"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Search => TagProperties {
                name: String::from("<sp:search"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Findet die gewünschte Suche"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Select => TagProperties {
                name: String::from("<sp:select"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Select-Tag, erzeugt den Rahmen einen Auswahlliste."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Set => TagProperties {
                name: String::from("<sp:set"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Setzt ein Attribute"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Sort => TagProperties {
                name: String::from("<sp:sort"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Sortiert eine Liste"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Subinformation => TagProperties {
                name: String::from("<sp:subinformation"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Tagbody => TagProperties {
                name: String::from("<sp:tagbody"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Text => TagProperties {
                name: String::from("<sp:text"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Text-Tag, erzeugt ein Eingabefeld."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Textarea => TagProperties {
                name: String::from("<sp:textarea"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Textarea-Tag, erzeugt einen Einabebereich."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Textimage => TagProperties {
                name: String::from("<sp:textimage"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Throw => TagProperties {
                name: String::from("<sp:querytree"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
            SpTag::Upload => TagProperties {
                name: String::from("<sp:upload"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Das Tag, erzeugt ein Eingabefeld zum Herunderladen von Dateien."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Url => TagProperties {
                name: String::from("<sp:url"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Fügt den ContextPath vor die angegebene URL und hüngt, falls nötig die Session ID an die URL."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Warning => TagProperties {
                name: String::from("<sp:warning"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Prüft, ob eine Warnung aufgetreten ist, markiert sie gegebenenfalls als gefangen und führt den innhalt des Tags aus."#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Worklist => TagProperties {
                name: String::from("<sp:worklist"),
                detail: None,
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: String::from(
                        r#"
Findet die gewünschte Workliste"#,
                    ),
                })),
                // children: TagChildren::Any,
            },
            SpTag::Zip => TagProperties {
                name: String::from("<sp:zip"),
                detail: None,
                documentation: None,
                // children: TagChildren::Any,
            },
        };
    }

    // pub fn from_treesitter_tag_name(tag_name: &str) -> Option<&'static SpTag> {
    //     return SpTag::iter().find(|tag| tag.properties().name == tag_name);
    // }

    pub fn iter() -> Iter<'static, SpTag> {
        static SP_TAGS: [SpTag; 55] = [
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
            SpTag::Upload,
            SpTag::Url,
            SpTag::Warning,
            SpTag::Worklist,
            SpTag::Zip,
        ];
        return SP_TAGS.iter();
    }
}
