use anyhow::{Error, Result};
use std::{slice::Iter, str::FromStr};

#[derive(Debug)]
pub(crate) struct TagProperties {
    pub(crate) name: &'static str,
    pub(crate) detail: Option<&'static str>,
    pub(crate) documentation: Option<&'static str>,
    // pub(crate) deprecated: bool,
    pub(crate) children: TagChildren,
    pub(crate) attributes: TagAttributes,
    pub(crate) attribute_rules: &'static [AttributeRule],
}

#[derive(Debug)]
pub(crate) enum TagAttributes {
    None,
    #[allow(dead_code)]
    OnlyDynamic,
    These(&'static [TagAttribute]),
    #[allow(dead_code)]
    TheseAndDynamic(&'static [TagAttribute]),
}

#[derive(Debug)]
pub(crate) struct TagAttribute {
    pub(crate) name: &'static str,
    pub(crate) detail: Option<&'static str>,
    pub(crate) documentation: Option<&'static str>,
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
    UriExists(&'static str, &'static str),
    // TODO:
    // OnlyIfAttributeHasValue
    // Renamed
    // Body?!?
}

#[derive(Debug)]
pub(crate) enum TagChildren {
    Any,
    None,
    Scalar(Tag),
    Vector(&'static [Tag]),
}

#[derive(Debug, PartialEq)]
pub(crate) enum Tag {
    SpArgument,
    SpAttribute,
    SpBarcode,
    SpBreak,
    SpCalendarsheet,
    SpCheckbox,
    SpCode,
    SpCollection,
    SpCondition,
    SpDiff,
    SpElse,
    SpElseif,
    SpError,
    SpExpire,
    SpFilter,
    SpFor,
    SpForm,
    SpHidden,
    SpIf,
    SpInclude,
    SpIo,
    SpIterator,
    SpJson,
    SpLinktree,
    SpLinkedInformation,
    SpLivetree,
    SpLog,
    SpLogin,
    SpLoop,
    SpMap,
    SpOption,
    SpPassword,
    SpPrint,
    SpQuerytree,
    SpRadio,
    SpRange,
    SpReturn,
    SpSass,
    SpScaleimage,
    SpScope,
    SpSearch,
    SpSelect,
    SpSet,
    SpSort,
    SpSubinformation,
    SpTagbody,
    SpText,
    SpTextarea,
    SpTextimage,
    SpThrow,
    SpToggle,
    SpUpload,
    SpUrl,
    SpWarning,
    SpWorklist,
    SpZip,
    SptCounter,
    SptDate,
    SptDiff,
    SptEmail2img,
    SptEncryptemail,
    SptEscapeemail,
    SptFormsolutions,
    SptId2url,
    SptIlink,
    SptImageeditor,
    SptImp,
    SptIterator,
    SptLink,
    SptNumber,
    SptPersonalization,
    SptPrehtml,
    SptSmarteditor,
    SptSpml,
    SptText,
    SptTextarea,
    SptTimestamp,
    SptTinymce,
    SptUpdown,
    SptUpload,
    SptWorklist,
}

const SP_ARGUMENT: TagProperties = TagProperties {
    name: "sp:argument",
    detail: None,
    documentation: Some(
        r#"
Setzt ein Argument für ein sp:include"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "condition",
            detail: None,
            documentation: Some(
                r#"
Die Condition wird ausgewertet und als Bedingung in das Argument geschrieben."#,
            ),
        },
        TagAttribute {
            name: "default",
            detail: None,
            documentation: Some(
                r#"
Der Text, der verwendet wird, wenn die Inhalte von `value`, `expression` und body leer sind."#,
            ),
        },
        TagAttribute {
            name: "expression",
            detail: None,
            documentation: Some(
                r#"
Die Expression wird ausgewertet und als Wert in das Argument geschrieben."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendenden Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name des Arguments."#,
            ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Evaluiert das Attribut und setzt den evaluierten Wert. Im Gegensatz zu `value` wird hier das Object gespeichert und nicht der Text."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Zu setzender Wert. Dieser wird immer als Zeichenkette ausgewertet."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::ExactlyOneOf(&["value", "expression", "condition", "object"]), // or body
        AttributeRule::OnlyWithEither("default", &["object", "expression"]),
    ],
};

const SP_ATTRIBUTE: TagProperties = TagProperties {
    name: "sp:attribute",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "dynamics",
            detail: None,
            documentation: Some(
                r#"
Evaluierung aller dynamischen Attribute."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name des Attributes, das als Objekt evaluiert werden soll."#,
            ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Name des Attributes, das als Objekt evaluiert werden soll."#,
            ),
        },
        TagAttribute {
            name: "text",
            detail: None,
            documentation: Some(
                r#"
Text der evaluiert werden soll. Dies ist funktional identisch mit `name`"#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Deprecated("name"),
        AttributeRule::ExactlyOneOf(&["name", "text", "object", "dynamics"]),
    ],
};

const SP_BARCODE: TagProperties = TagProperties {
    name: "sp:barcode",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "height",
            detail: None,
            documentation: Some(
                r#"
Höhe des Bildes."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Variable für den Zugriff auf das Ergebnis-Object. Je nach Angegebenen Typ. Mögliche Objekte sind: `QRCode`."#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Gültigkeitsbereich, in dem die Variable definiert ist. Möglich sind `page` und `request`."#,
            ),
        },
        TagAttribute {
            name: "text",
            detail: None,
            documentation: Some(
                r#"
Text aus dem der Barcode generiert werden soll."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Zu erzeugender Barcode-Typ. Unterstütz wird z.Z. nur `qrcode`"#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("text"),
        AttributeRule::Required("type"),
    ],
};

const SP_BREAK: TagProperties = TagProperties {
    name: "sp:break",
    detail: None,
    documentation: Some(
        r#"
Beendet FOR- und ITERATE-Schleifen."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::None,
    attribute_rules: &[],
};

const SP_CALENDARSHEET: TagProperties = TagProperties {
    name: "sp:calendarsheet",
    detail: None,
    documentation: Some(
        r#"
CalendarSheet manage dates and objects"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "action",
            detail: None,
            documentation: Some(
                r#"
Aktion, die ausgeführt werden soll. Es existieren die Aktionen `add`, `clear` und `new`."#,
            ),
        },
        TagAttribute {
            name: "date",
            detail: None,
            documentation: Some(
                r#"
Zu setzender Wert. Hiermit kann direkt ein einzelnes Datum angegeben werden. Über die Attribute `value` bzw. `object` kann die zugehörige Referenz angegeben werden."#,
            ),
        },
        TagAttribute {
            name: "from",
            detail: None,
            documentation: Some(
                r#"
Über diesen Parameter wird der zeitliche Rahmen des CalendarSheets festgelegt."#,
            ),
        },
        TagAttribute {
            name: "mode",
            detail: None,
            documentation: Some(
                r#"
Über diesen Parameter wird Modus angegeben, mit dem ein Termin eingefügt werden soll. Gültige Werte sind: `allDays` (alle Tage einfügen), `startDays` (nur den Start-Tag, sofern dieser innerhalb des angegebenen Zeitraums liegt einfügen) und `firstDays` (den ersten gültigen Tag einfügen)"#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name über den das `CalendarSheet` angesprochen werden kann."#,
            ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Evaluiert das Attribut und setzt das entsprechende `CalendarInformation`-Objekt. Im Gegensatz zu `value` wird hier das Objekt direkt erwartet und nicht der Text. Das Element, zu dem das CalendarInformation gehört (`root`) wird automatisch als Referenz verwendet."#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Namensraum, in dem die Variable definiert ist. Für diesen Tag ist der Page- und Request-Scope möglich (`page`, `request`)."#,
            ),
        },
        TagAttribute {
            name: "to",
            detail: None,
            documentation: Some(
                r#"
Über diesen Parameter wird der zeitliche Rahmen des `CalendarSheets` festgelegt."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Zu setzendes `CalendarInformation`-Objekt. Dieser wird immer als Zeichenkette ausgewertet. Das Element, zu dem das `CalendarInformation` gehört (`root`) wird automatisch als Referenz verwendet."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("action"),
        AttributeRule::Required("from"),
        AttributeRule::Required("to"),
        AttributeRule::ExactlyOneOf(&["value", "object", "date"]),
    ],
};

const SP_CHECKBOX: TagProperties = TagProperties {
    name: "sp:checkbox",
    detail: None,
    documentation: Some(
        r#"
Check-Box-Tag, erzeugt eine checkBox."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "checked",
            detail: None,
            documentation: Some(
                r#"
Gibt an, ob der Radio-Button per default gechecked werden soll. Diese Einstellung gibt es nur so lange, bis eine Auswahl durch den Bearbeiter vorgenommen und gespeichert wurde."#,
            ),
        },
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "readonly",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Der Typ des Eingabefeldes."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Setzt einen zu übertragenen Wert dieser Checkbox"#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("name")],
};

const SP_CODE: TagProperties = TagProperties {
    name: "sp:code",
    detail: None,
    documentation: Some(
        r#"
Schreibt den bodyContent ohne dass dieser ausgeführt wird in die Ergebnis-Datei."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::None,
    attribute_rules: &[],
};

const SP_COLLECTION: TagProperties = TagProperties {
    name: "sp:collection",
    detail: None,
    documentation: Some(
        r#"
Collection tag offers certain operation that deal with a common collection. For further description see the javadoc of the class com.sitepark.ies.taglib.core.CollectionTag."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "action",
            detail: None,
            documentation: Some(
                r#"
Aktion, die ausgeführt werden soll. Es existieren die Aktionen `add`, `addAll`, `remove`, `clear`, `new`, `replace`, `removeFirst`, `removeLast` und `unique`.
- `add` Fügt ein Element am Ende der Liste ein. Ist ein `index` angegeben, so wird das Element an dieser Position eingefügt. Das ursprüngliche Elemente und alle nachfolgenden Elemente werden eine Position weiter geschoben.
- `addNotEmpty` Fügt ein Element am Ende der Liste ein, wenn der Wert nicht `null` oder ein Leerstring ist. Ist ein `index` angegeben, so wird das Element an dieser Position eingefügt. Das ursprüngliche Elemente und alle nachfolgenden Elemente werden eine Position weiter geschoben.
- `addAll` Mit dieser Aktion können mehrere Elemente der Liste hinzugefügt werden. Dazu muss `object` vom Typ `Collection` sein.
- `remove` Löscht ein Element aus der Liste. Ist `index` angegeben, wird das Element an der Index-Position gelöscht und alle nachfolgenden Elemente rutschen eine Position nach oben. Ist `object` bzw. `value` angegeben, wird das Element in der Liste gesucht und gelöscht.
- `clear` Löscht alle Elemente aus der Liste.
- `new` Erzeugt eine neue leere Liste.
- `replace` Ersetzt ein Element der Liste. `index` gibt hierbei die Position des Elements an, das durch `object` bzw. `value` ersetzt werden soll.
- `removeFirst` Löscht das erste Element der Liste.
- `removeLast` Löscht das letzte Element der Liste.
- `unique` Entfernt alle mehrfach vorkommenden Elemente aus der Liste.
- `insert` Fügt ein Element ein und verschiebt alle nachfolgenden Elemente um eine Position. Wenn in eine Position eingefügt wird, die noch nicht belegt ist, wird das delta mit `null` aufgefüllt."#,
            ),
        },
        TagAttribute {
            name: "condition",
            detail: None,
            documentation: Some(
                r#"
Die `Condition` wird ausgewertet und als Bedingung in die Variable geschrieben."#,
            ),
        },
        TagAttribute {
            name: "default",
            detail: None,
            documentation: Some(
                r#"
Der Text, der verwendet wird, wenn die Inhalte von `value`, `expression` und body leer sind."#,
            ),
        },
        TagAttribute {
            name: "expression",
            detail: None,
            documentation: Some(
                r#"
Die `Expression` wird ausgewertet und als Wert in die Variable geschrieben."#,
            ),
        },
        TagAttribute {
            name: "index",
            detail: None,
            documentation: Some(
                r#"
Listen-Position mit der eine Aktion ausgeführt werden soll."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Liste."#,
            ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Ein `Objekt` das mit der Liste verarbeitet werden soll. Ist `object` vom Typ `QueryInformation`, so gilt das gleiche wie beim Attribut `query`."#,
            ),
        },
        TagAttribute {
            name: "publisher",
            detail: None,
            documentation: Some(
                r#"
Wird der Collection-Tag in Verbindung mit Suchabfragen verwendet (durch `query` oder `object`), ist ein Publikationsbereich erforderlich, mit der die Suchabfrage ausgeführt werden soll. Mit diesem Attribut können ein oder mehrere Publikationsbereiche angegeben werden (durch Kommata getrennt). Entweder werden die Publikationsbereiche durch ihren Anchor angegeben, oder folgende Schlüsselwörter verwendet:
- `current` Der aktuelle Publikationsbereich. Dieser steht im `out`- und `preview`-Modus als default-Wert zur Verfügung.
- `ignore` Ignoriert die Publikationsbereiche und liefert die Treffer unabhängig davon, ob sie publiziert sind oder nicht.
- `all` Liefert die Treffer, wenn sie in irgendeinem der dem Mandanten zugewiesenen Publikationsbereiche publiziert sind.
- `auto` Entspricht im `out`- und `preview`-Modus dem Schlüsselwort `current` und im `in`-Modus `ignore`."#,
            ),
        },
        TagAttribute {
            name: "query",
            detail: None,
            documentation: Some(
                r#"
Fügt in die Collection die Ergebnisse der übergebenen Suchabfrage ein. Ist dieses Attibut gesetzt, ist kein `action` nötig. Die Aktion entspricht einem `addAll`. Es kann jedoch eine andere Aktion angegeben werden."#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Namensraum, in dem die Variable definiert ist. Für diesen Tag ist der Page- und Request-Scope möglich (`page`, `request`)."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Ein Text, der mit der Liste verarbeitet werden soll."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::ExactlyOneOf(&["action", "query"]),
        AttributeRule::ExactlyOneOf(&["value", "object", "expression", "condition"]), // or body
                                                                                      // index is required if "value" is "remove" or "replace"
    ],
};

const SP_CONDITION: TagProperties = TagProperties {
    name: "sp:condition",
    detail: None,
    documentation: Some(
        r#"
Umklammert einen if-else Konstrukt."#,
    ),
    children: TagChildren::Vector(&[Tag::SpIf, Tag::SpElse, Tag::SpElseif]),
    attributes: TagAttributes::None,
    attribute_rules: &[],
};

const SP_DIFF: TagProperties = TagProperties {
    name: "sp:diff",
    detail: None,
    documentation: Some(
        r#"
Vergleicht ein Attribute von zwei Versionen einer Information"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "from",
            detail: None,
            documentation: Some(
                r#"
Text der Verglichen werden soll. Wörter die hier enthalten und in to nicht mehr enthalten sind, werden als 'gelöscht' markiert."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "lookup",
            detail: None,
            documentation: Some(
                r#"
Gibt an, ob bei der der Auflösung von mehrsprachigen Variablen mit der, durch locale angegebenen Sprache auch ein Lookup ausgeführt werden soll."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Liste, in die das Ergebnis gespeichert wird. Die Liste enthält `DiffChunk`-Objekte."#,
            ),
        },
        TagAttribute {
            name: "to",
            detail: None,
            documentation: Some(
                r#"
Text der Verglichen werden soll. Wörter die hier enthalten und in from nicht enthalten sind werden als 'neu' markiert."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("from"),
        AttributeRule::Required("to"),
        AttributeRule::OnlyWith("lookup", "locale"), // is that correct?
    ],
};

const SP_ELSE: TagProperties = TagProperties {
    name: "sp:else",
    detail: None,
    documentation: Some(
        r#"
passendes else zu einem If innerhalb eines contitionTag."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::None,
    attribute_rules: &[],
};

const SP_ELSEIF: TagProperties = TagProperties {
    name: "sp:elseif",
    detail: None,
    documentation: Some(
        r#"
ElseIf-Tag, schreibt Body wenn Bedingung ok ist und vorheriges if fehl schlug."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "condition",
            detail: None,
            documentation: Some(
                r#"
Es wird eine Condition erwartet, die den Wert `true` oder `false` zurückliefert."#,
            ),
        },
        TagAttribute {
            name: "eq",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` gleich der Variable in `eq` ist."#,
            ),
        },
        TagAttribute {
            name: "gt",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` größer als der Variable in `gt` ist."#,
            ),
        },
        TagAttribute {
            name: "gte",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` größer oder gleich der Variable in `gte` ist."#,
            ),
        },
        TagAttribute {
            name: "ic",
            detail: None,
            documentation: Some(
                r#"
Die Auswertung soll "ignore case" durchgeführt werden. Bezieht sich auf `eq`, `neq`, `gt`, `gte`, `lt`, `lte`, und `contains`."#,
            ),
        },
        TagAttribute {
            name: "iNull",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` leer oder nicht vorhanden ist und `isNull` den Wert `true` hat. Wenn `isNull` den Wert `false` hat, ist die Bedingungen erfüllt, wenn die Variable in `name` nicht leer ist."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendenden Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "lt",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` kleiner als in der Variable in `lte` ist."#,
            ),
        },
        TagAttribute {
            name: "lte",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` kleiner oder gleich der Variable in `lte` ist."#,
            ),
        },
        TagAttribute {
            name: "match",
            detail: None,
            documentation: Some(
                r#"
Regulärer Ausdruck, der in der Variablen enthalten sein soll."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Variablenname eines Objektes das verglichen werden soll."#,
            ),
        },
        TagAttribute {
            name: "neq",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` ungleich der Variable in `neq` ist."#,
            ),
        },
    ]),
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

const SP_ERROR: TagProperties = TagProperties {
    name: "sp:error",
    detail: None,
    documentation: Some(
        r#"
Prüft ein Fehler aufgetreten ist, markiert ihn gegebenenfals als gefangen und führt den innhalt des Tags aus."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[TagAttribute {
        name: "code",
        detail: None,
        documentation: Some(
            r#"
Zu prüfender Error-Code."#,
        ),
    }]),
    attribute_rules: &[AttributeRule::Required("code")],
};

const SP_EXPIRE: TagProperties = TagProperties {
    name: "sp:expire",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[TagAttribute {
        name: "date",
        detail: None,
        documentation: Some(
            r#"
Long-Wert mit dem Unix-Timestamp des gewünschten Datums"#,
        ),
    }]),
    attribute_rules: &[AttributeRule::Required("date")],
};

const SP_FILTER: TagProperties = TagProperties {
    name: "sp:filter",
    detail: None,
    documentation: Some(
        r#"
Filtert eine Liste"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "attribute",
            detail: None,
            documentation: Some(
                r#"
Attribut, auf das der Filter angewendet werden soll."#,
            ),
        },
        TagAttribute {
            name: "collection",
            detail: None,
            documentation: Some(
                r#"
Name der zu filternden Liste."#,
            ),
        },
        TagAttribute {
            name: "filter",
            detail: None,
            documentation: Some(
                r#"
Die Filterdefinition für die Filtertypen Wildcard und regulärer Ausdruck. Der mit dem Attribut `mode` angegebene Modus wird verwendet. Ohne Angabe eines Modus wird `simple` verwendet."#,
            ),
        },
        TagAttribute {
            name: "format",
            detail: None,
            documentation: Some(
                r#"
Das Format des Datums, wenn die `from` und `to` Werte als Datum interpretiert werden sollen."#,
            ),
        },
        TagAttribute {
            name: "from",
            detail: None,
            documentation: Some(
                r#"
Der Wert für den Beginn eines Bereiches, z.B. "Aa" oder "100". Ob der Wert als Text, Zahl oder Datum interpretiert wird, kann mit dem Attribut `type` angegeben werden."#,
            ),
        },
        TagAttribute {
            name: "ic",
            detail: None,
            documentation: Some(
                r#"
Ist Ignore-Case auf `true` gesetzt, wird eine Groß- und Kleinschreibung nicht berücksichtigt. Dieses Attribut gilt nur für die Filtertypen Wildcard und regulärer Ausdruck."#,
            ),
        },
        TagAttribute {
            name: "invert",
            detail: None,
            documentation: Some(
                r#"
Invertiert die Logik des Filters. Alle Elemente die normalerweise herausgefiltert würden, bilden die Filterergebnisse."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendenden Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "mode",
            detail: None,
            documentation: Some(
                r#"
Auswahl des Filter-Mechanismus.
__simple (Wildcard-Filter)__
Der Filter kann die Wildcards `*` für beliebige Zeichen und `?` für ein beliebiges Zeichen enthalten. So würde eine wie folgt gefilterte Liste nur Elemente enthalten, die mit a beginnen.
```regex
a*
```
__regex (Reguläre Ausdrücke)__
Für komplexe Filter stehen Reguläre Ausdrücke (POSIX) zur Verfügung. So würde im regex-Filtermode eine mit
```regex
[a-dA-D].*
```
gefilterte Liste nur Elemente enthalten, die mit dem Buchstaben A, a, B, b, C, c, d oder D beginnen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der gefilterten Liste."#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Gültigkeitsbereich, in dem die Variable definiert ist. Möglich sind: `page`|`request`|`session`."#,
            ),
        },
        TagAttribute {
            name: "to",
            detail: None,
            documentation: Some(
                r#"
Der Wert für das Ende eines Bereiches, z.B. "Zz" oder "999". Ob der Wert als Text, Zahl oder Datum interpretiert wird, kann mit dem Attribut `type` angegeben werden."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Der Typ für die from und to Attribute: `number`, `text`, `date`.
- `number` Das Element oder Attribut wird als Zahl interpretiert. Es wird nicht herausgefiltert wenn es innerhalb des Zahlenbereiches liegt, der mit `from` und `to` definiert wurde.
- `text` Das Element oder Attribut wird als Text interpretiert. Es wird nicht herausgefiltert wenn der Text mit den Zeichen beginnt, die in dem mit `from` und `to` definierten Bereich liegen.
- `date` Das Element oder Attribut wird als Datum interpretiert. Es wird nicht herausgefiltert wenn es innerhalb des Datumbereiches liegt, der mit `from` und `to` definiert wurde."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("collection"),
        AttributeRule::OnlyWith("ic", "filter"),
        AttributeRule::OnlyWithEither("type", &["from", "to"]),
        AttributeRule::OnlyWithEither("format", &["from", "to"]),
    ],
};

const SP_FOR: TagProperties = TagProperties {
    name: "sp:for",
    detail: None,
    documentation: Some(
        r#"
For-Tag, wiederholt solange wie angegeben."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "condition",
            detail: None,
            documentation: Some(
                r#"
Die For-Schleife wird solange durchlaufen, bis die Bedingung `false` ergibt"#,
            ),
        },
        TagAttribute {
            name: "from",
            detail: None,
            documentation: Some(
                r#"
Startwert des Zählers"#,
            ),
        },
        TagAttribute {
            name: "index",
            detail: None,
            documentation: Some(
                r#"
Name der Variable, die den aktuellen Zählerstand enthält"#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachigen Variablen "#,
            ),
        },
        TagAttribute {
            name: "step",
            detail: None,
            documentation: Some(
                r#"
Schrittweite, in der gezählt werden soll. `step` kann für einen Rückwärtszähler negativ sein"#,
            ),
        },
        TagAttribute {
            name: "to",
            detail: None,
            documentation: Some(
                r#"
Endwert des Zählers"#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("index"),
        AttributeRule::Required("from"),
        AttributeRule::ExactlyOneOf(&["to", "condition"]),
    ],
};

const SP_FORM: TagProperties = TagProperties {
    name: "sp:form",
    detail: None,
    documentation: Some(
        r#"
Erzeugt ein HTML-Form-Tag mit einem angepassten Kommando"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "command",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut ist veraltet und wird in zukünftigen Versionen nicht mehr unterstüzt werden. Kommandos wurden in der Version 2.0 zugunsten einer flexibleren Lösung abgeschafft. Ein Kommando bestand aus einem Template mit einem optionalen Handler. Für jede Template-Handler-Kombination musste ein eigenes Kommando angelegt werden. Diese Verbindung wurde aufgebrochen und durch zwei neue Attribute `template` und `handler` ersetzt. Um einen Handler aufzurufen und anschließend ein Template auszuführen, ist nun die Definition eines Kommandos nicht mehr nötig. Um einen Handler aufzurufen und anschließend ein Template auszuführen, verwenden Sie die beiden Attribute `handler` und `template`. Um einen Handler aufzurufen und anschließend eine SPML-Seite auszuführen, verwenden Sie die Attribute `handler` und `uri`.*
Existierendes Command. Muss im GUI definiert worden sein."#,
            ),
        },
        TagAttribute {
            name: "context",
            detail: None,
            documentation: Some(
                r#"
SPML-Seiten sind immer Teil einer Webapplikation. Jede Webapplikation besitzt einen Context-Pfad mit dem die URL beginnt (Es existert auch ein ROOT-Context-Pfad (`/`)). Soll die URL einer Seite herausgeschrieben werden, die in einer anderen Webapplikation liegt, so wird mit diesem Attribut der Context-Pfad angegeben. Context-Pfade von Webapplikationen können sich ändern. Damit bei solchen Änderungen auch die URL richtig generiert wird, sollte in den meisten Fällen das Attribut `module` verwendet werden."#,
            ),
        },
        TagAttribute {
            name: "enctype",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Typ der zu übertragenden Daten:
- `text/plain` `text/plain` ist die default Einstellung.
- `multipart/form-data` Für Datei-Uploads muss `multipart/form-data` angegeben werden."#,
            ),
        },
        TagAttribute {
            name: "handler",
            detail: None,
            documentation: Some(
                r#"
Handler, an den das Formular gesendet werden soll. Handler werden vor der, mit `uri` oder `template` angegebenen Seite ausgeführt."#,
            ),
        },
        TagAttribute {
            name: "id",
            detail: None,
            documentation: Some(
                r#"
Optionale id für den erzeugten HTML-Form-Tag. Ist dieses Attribut nicht gesetzt, wird automatisch eine ID generiert"#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachiger Variablen."#,
            ),
        },
        TagAttribute {
            name: "method",
            detail: None,
            documentation: Some(
                r#"
Bestimmt die Übertragungsmethode: get oder post. Bei get werden die Parameter offen über die aufzurufende URL übermittelt, bei post verborgen im HTTP-Protokoll. Für Datei-Uploads ist post Pflicht."#,
            ),
        },
        TagAttribute {
            name: "module",
            detail: None,
            documentation: Some(
                r#"
SPML-Seiten sind immer Teil einer Webapplikation. Jede Webapplikation besitzt einen Context-Pfad mit dem die URL beginnt (Es existert auch ein ROOT-Context-Pfad (`/`)). Soll das Formular an eine Seite gesendet werden, die in einer anderen Webapplikation liegt, so wird mit diesem Attribut die ID dieser Webapplikation angegeben."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Optionaler Name für das erzeugte Formular."#,
            ),
        },
        TagAttribute {
            name: "nameencoding",
            detail: None,
            documentation: Some(
                r#"
Die innerhalb von sp:form liegenden Input-Tags (`sp:text`, `spt:text`, `sp:checkbox`, ...) erhalten vom IES generierte Feldnamen, die unter Umständen (wenn sie z.B. innerhalb von `sp:iterator` liegen) Sonderzeichen wie eckige Klammern (`[`, `]`) enthalten können. Beim Aufbau von Live-Seiten, die in PHP eingebettet sind, wird das Formular an PHP-Seiten gesendet. Da Request-Parameternamen mit Sonderzeichen von PHP nicht richtig ausgewertet werden, ist es mit diesem Attribut möglich, die Formularfeldnamen zu encoden, damit keine Sonderzeichen mehr enthalten sind. Vom IES unterstüzte Encodings für Feldnamen sind:
- `escff` *(default)* Wandelt nur die Zeichen des Feldnamens um, die zu Fehlern führen können z.B. Eckige Klammern (`[]`). Beispiel: Aus `sp_iterator[1].sp_body` wird `escff:sp_iterator:5b:1:5d::2e:sp_body.` Dieses Encoding ist kein Standard-Encoding, sondern eine proprietäre Entwicklung von Sitepark.
- `hex` Wandelt jedes Zeichen des Feldnamens in den entsprechenden Hex-Wert um. Beispiel: Aus `sp_body` wird "hex:73705f626f6479""#,
            ),
        },
        TagAttribute {
            name: "template",
            detail: None,
            documentation: Some(
                r#"
Template, an das das Formular gesendet werden soll. Dieses Attribut sollte nur für Umstellungen von Live-Seiten verwendet werden, die sich durch den Wegfall der Kommandos ergeben. Prinzipiell sollten Live-Seiten und Webapplikationen gemeinhin nicht mit Templates, sondern mit SPML-Seite realisiert werden."#,
            ),
        },
        TagAttribute {
            name: "uri",
            detail: None,
            documentation: Some(
                r#"
Dies kann ein beliebiger Pfad zu einer Seite sein. sp:form sorgt dafür, dass alle Session-Informationen mitgesendet werden, sodass die Session nicht verloren geht. Wird bei SPML-Seiten weder das Attribut `uri` noch `template` angegeben, so wird die aktuelle URL angesprochen."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Deprecated("command"),
        AttributeRule::OnlyOneOf(&["uri", "template"]),
        AttributeRule::OnlyWith("module", "uri"),
    ],
};

const SP_HIDDEN: TagProperties = TagProperties {
    name: "sp:hidden",
    detail: None,
    documentation: Some(
        r#"
Hidden-Tag, erzeugt ein Hiddenfeld."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "fixvalue",
            detail: None,
            documentation: Some(
                r#"
Überschreibt jeden vorhandenen Inhalt der mit `name` bestimmten Variablen mit dem durch `fixvalue` angegebenen Wert."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Der Typ des Eingabefeldes."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Setzt einen Default-Wert für die mit `name` angegebenen Variable, wenn sie leer ist."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::ExactlyOneOf(&["value", "fixvalue"]),
    ],
};

const SP_IF: TagProperties = TagProperties {
    name: "sp:if",
    detail: None,
    documentation: Some(
        r#"
If-Tag, schreibt Body wenn Bedingung ok ist."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "condition",
            detail: None,
            documentation: Some(
                r#"
Es wird eine Condition erwartet, die den Wert `true` oder `false` zurückliefert."#,
            ),
        },
        TagAttribute {
            name: "eq",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` gleich der Variable in `eq` ist."#,
            ),
        },
        TagAttribute {
            name: "gt",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` größer als der Variable in `gt` ist."#,
            ),
        },
        TagAttribute {
            name: "gte",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` größer oder gleich der Variable in `gte` ist."#,
            ),
        },
        TagAttribute {
            name: "ic",
            detail: None,
            documentation: Some(
                r#"
Die Auswertung soll "ignore case" durchgeführt werden. Bezieht sich auf `eq`, `neq`, `gt`, `gte`, `lt`, `lte`, und `contains`."#,
            ),
        },
        TagAttribute {
            name: "iNull",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` leer oder nicht vorhanden ist und `isNull` den Wert `true` hat. Wenn `isNull` den Wert `false` hat, ist die Bedingungen erfüllt, wenn die Variable in `name` nicht leer ist."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendenden Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "lt",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` kleiner als in der Variable in `lte` ist."#,
            ),
        },
        TagAttribute {
            name: "lte",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` kleiner oder gleich der Variable in `lte` ist."#,
            ),
        },
        TagAttribute {
            name: "match",
            detail: None,
            documentation: Some(
                r#"
Regulärer Ausdruck, der in der Variablen enthalten sein soll."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Variablenname eines Objektes das verglichen werden soll."#,
            ),
        },
        TagAttribute {
            name: "neq",
            detail: None,
            documentation: Some(
                r#"
Die Bedingung ist erfüllt, wenn die Variable in `name` ungleich der Variable in `neq` ist."#,
            ),
        },
    ]),
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

const SP_INCLUDE: TagProperties = TagProperties {
    name: "sp:include",
    detail: None,
    documentation: Some(
        r#"
includiert ein anderes bereits im System gespeichertes Template."#,
    ),
    children: TagChildren::Scalar(Tag::SpArgument),
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "anchor",
            detail: None,
            documentation: Some(
                r#"
Anchor-Name des zu includenden Templates."#,
            ),
        },
        TagAttribute {
            name: "arguments",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut können Argumente in Form einer Map übergeben, die mit `system.arguments` in der includierten SPML-Datei wieder ausgelesen werden können. Zusätzlich kann noch `sp:argument` verwendet werden. Mit diesem Tag werden ggf. Argumente der Map überschrieben."#,
            ),
        },
        TagAttribute {
            name: "context",
            detail: None,
            documentation: Some(
                r#"
SPML-Seiten sind immer Teil einer Webapplikation. Die mit dem Attribut `uri` angegebene SPML-Seite bezieht sich immer auf die aktuelle Webapplikation. Soll eine Seite einer anderen Webapplikation eingebunden werden, so wird mit diesem Attribut der Context der Webapplikation angegeben. Da sich der Context einer Webapplikation ändern kann, ist in den meisten Fällen die Verwendung des Attributes `module` zu empfehlen, da hier die ID der Webapplikation angegeben wird."#,
            ),
        },
        TagAttribute {
            name: "mode",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut kann angegeben werden, in welchem Modus die includete SPML-Seite oder das includete Template ausgeführt werden soll.
- `in` Führt das Template oder die SPML-Seite im In-Modus aus.
- `out` Führt das Template oder die SPML-Seite im Out-Modus aus."#,
            ),
        },
        TagAttribute {
            name: "module",
            detail: None,
            documentation: Some(
                r#"
SPML-Seiten sind immer Teil einer Webapplikation. Die mit dem Attribut `uri` angegebenen SPML-Seite bezieht sich immer auf die aktuelle Webapplikation. Soll eine Seite einer anderen Webapplikation eingebunden werden, so wird mit diesem Attribut die ID der Webapplikation angegeben. Dieses Attribut ist dem Attribut `context` vorzuziehen, da sich der Context einer Webapplikation ändern kann."#,
            ),
        },
        TagAttribute {
            name: "return",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut wird der Name der Variable definiert, in der der Rückgabe-Wert des inkludierten Templates abgelegt wird. Inkludierte Templates können sp:return verwenden um Rückgabe-Werte zu definieren. (Siehe auch Eigene Funktionen)"#,
            ),
        },
        TagAttribute {
            name: "template",
            detail: None,
            documentation: Some(
                r#"
Zu includendes Template (Variable mit einer Template-ID)."#,
            ),
        },
        TagAttribute {
            name: "uri",
            detail: None,
            documentation: Some(
                r#"
URI einer Seite die includiert werden soll. Dieser muss in der gleichen Webapplikation liegen. Weiterhin kann mit dem `context`-Attribut oder dem module-Attribut eine andere Webapplikation angegeben werden, deren Seite includiert werden soll."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::ExactlyOneOf(&["template", "anchor", "uri"]),
        AttributeRule::OnlyOneOf(&["context", "module"]),
        AttributeRule::OnlyWith("context", "uri"),
        AttributeRule::OnlyWith("module", "uri"),
        AttributeRule::UriExists("uri", "module"),
    ],
};

const SP_IO: TagProperties = TagProperties {
    name: "sp:io",
    detail: None,
    documentation: Some(
        r#"
IO-Tag"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "contenttype",
            detail: None,
            documentation: Some(
                r#"
Mit Hilfe dieses Attributes kann der Content-Typ für einen bestimmten Bereich neu gesetzt werden. Der Content-Typ des Dokumentes bzw. des aktuellen Dokument-Teils kann über das `System`-Object abgefragt werden. Wird der Content-Type auf den Wert `text/xhtml` gesetzt, werden alle vom System erzeugten HTML-Tag's XHTML-konform generiert"#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Bestimmt ob der Bereich für die Ein- oder Ausgabe ist. Gültig sind `in` und `out`."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("type")],
};

const SP_ITERATOR: TagProperties = TagProperties {
    name: "sp:iterator",
    detail: None,
    documentation: Some(
        r#"
Wird für den Aufbau von Wiederholfeldern verwendet."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "collection",
            detail: None,
            documentation: Some(
                r#"
Die zu iterierende Liste. Dieses Attribut entspricht dem `name`-Attribut des `spt:iterator-Tags`."#,
            ),
        },
        TagAttribute {
            name: "item",
            detail: None,
            documentation: Some(
                r#"
Die in `collection` angegebene Liste wird Element für Element durchlaufen. Mit dem in diesem Attribut angegebenen Namen kann auf das aktuelle Element der Liste zugegriffen werden. Für das aktuelle Element können noch zusätzliche Informationen die den Schleifendurchlauf betreffen abgefragt werden (siehe `IteratorItem`)."#,
            ),
        },
        TagAttribute {
            name: "max",
            detail: None,
            documentation: Some(
                r#"
Die Anzahl der maximal zu iterierenden Elemente. Enthält die zu iterierende Liste mehr Elemente als in `max` angegeben, so wird die Anzahl der Elemente auf die Anzahl `max` gekürzt."#,
            ),
        },
        TagAttribute {
            name: "min",
            detail: None,
            documentation: Some(
                r#"
Die Anzahl der mindestens zu iterierenden Elemente. Enthält die zu iterierende Liste weniger Elemente als in `min` angegeben, werden so viele leere Elemente hinzugefügt, bis mindestens die in `min` angegebene Anzahl von Elementen vorhanden ist."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("collection")],
};

const SP_JSON: TagProperties = TagProperties {
    name: "sp:json",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "indent",
            detail: None,
            documentation: Some(
                r#"
Initiale Einrückung für eine formatierte Ausgabe."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendenden Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der neuen Variable."#,
            ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Objekt, das als `JSONObject` in die Variable gespeichert werden soll oder bodyContent"#,
            ),
        },
        TagAttribute {
            name: "overwrite",
            detail: None,
            documentation: Some(
                r#"
Bestimmt, ob eine evtl. vorhandene Variable überschrieben werden soll. `true` bzw. `false`."#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Gültigkeitsbereich, in dem die Variable definiert ist. Möglich sind `page` und `request`."#,
            ),
        },
    ]),
    attribute_rules: &[],
};

const SP_LINKEDINFORMATION: TagProperties = TagProperties {
    name: "sp:linkedInformation",
    detail: None,
    documentation: Some(
        r#"
Diese Tag definiert einen Link eines Artikels auf einen Anderen Artikel. Das Besondere ist, dass der Artikel auf dem Verlinkt wird erst innerhalb dieses tags definiert wird. Dazu müssen alle Paramter wie parent, filename, usw. vorhanden sein. Mit dem Reques können dann schliesslich beide Artikel ubgedatet werden(oder auch erstellt)."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::None,
    attribute_rules: &[], // not documented
};

const SP_LINKTREE: TagProperties = TagProperties {
    name: "sp:linktree",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "attributes",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut wird nicht mehr benötigt. Die Abhängigkeiten werden automatisch erkannt (siehe `Dependencies-Service`, `LinkTree-Service`)*
Eine Kommaseparierte Liste von Attributen, die der Artikel enthalten und auf dessen Änderungen er reagieren soll."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "localelink",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut kann angegeben werden, ob ein Linktree sprachabhängig aufgebaut werden soll. Ist `localelink` auf `true` gesetzt, wird die Sprache des Publikationsbereichs für den Tree verwendet. Die `parentlink`s, die den Baum ergeben, müssen dann mit einer Sprache definiert werden."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Variable für das LinkTree-Objekt."#,
            ),
        },
        TagAttribute {
            name: "parentlink",
            detail: None,
            documentation: Some(
                r#"
Name des Links, der auf einen, in der zu erstellenden Struktur, übergeordneten Artikel verweist."#,
            ),
        },
        TagAttribute {
            name: "rootelement",
            detail: None,
            documentation: Some(
                r#"
Das Root-Element des Baums. Ist kein Root-Element angegeben, wird der dazugehörige Artikel als Root-Element verwendet."#,
            ),
        },
        TagAttribute {
            name: "sortkeys",
            detail: None,
            documentation: Some(
                r#"
Attribute des Artikels, nach denen der Baum sortiert werden soll. Jede Ebene des Baums wird für sich sortiert."#,
            ),
        },
        TagAttribute {
            name: "sortsequences",
            detail: None,
            documentation: Some(
                r#"
Für jedes Sortierkriterium muss eine Sortierreihenfolge festgelegt werden, mit der bestimmt wird, ob mit dem Sortierkriterium aufsteigend (´desc´), absteigend (´asc´) oder zufällig (´random´) sortiert wird."#,
            ),
        },
        TagAttribute {
            name: "sorttypes",
            detail: None,
            documentation: Some(
                r#"
Für jedes Sortierkriterium kann ein Sortiertyp festgelegt werden, der bestimmt, wie sortiert wird. Dabei ist eine Sortierung von Zeichenketten (`text`) oder eine Sortierung von Zahlen (`number`) möglich."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Deprecated("attributes"),
        AttributeRule::Required("name"),
        AttributeRule::OnlyWith("sortsequences", "sortkeys"),
        AttributeRule::OnlyWith("sortkeys", "sortsequences"), // OnlyBoth?
        AttributeRule::OnlyWith("sorttypes", "sortkeys"),
    ],
};

const SP_LIVETREE: TagProperties = TagProperties {
    name: "sp:livetree",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "action",
            detail: None,
            documentation: Some(
                r#"
Das Kommando, das auf das Element in `node` angewendet werden soll.
- `flip` Offenen Node schliessen / Geschlossenen Node öffnen.
- `open` Node öffnen.
- `close` Node schliessen.
- `expand` Node und den gesamten Pfad öffnen.
- `none` Es wird keine Aktion ausgeführt."#,
            ),
        },
        TagAttribute {
            name: "leaflink",
            detail: None,
            documentation: Some(
                r#"
Name des Links, der Kinder, die auf Artikel in dem Baum verweisen, selber aber nicht in dem Baum enthalten sein sollen."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Variable für die `Collection`, die den Baum in Form von `ElementNodes` enthält."#,
            ),
        },
        TagAttribute {
            name: "node",
            detail: None,
            documentation: Some(
                r#"
Der Name der Variablen, Element dessen Wert eine Element-ID des Elementes sein muss, auf die sich `action` bezieht. Solange der gleiche Variablenname verwendet wird, bleiben die geöffneten Elemente offen, auch wenn `sp:livetree` erneut aufgerufen wird (innerhalb einer Session)."#,
            ),
        },
        TagAttribute {
            name: "parentlink",
            detail: None,
            documentation: Some(
                r#"
Name des Links, der auf einen, in der zu erstellenden Struktur, übergeordneten Artikel verweist."#,
            ),
        },
        TagAttribute {
            name: "publisher",
            detail: None,
            documentation: Some(
                r#"
`ID` des Publishers, in dem die Artikel des Baumes publiziert sein müssen."#,
            ),
        },
        TagAttribute {
            name: "rootElement",
            detail: None,
            documentation: Some(
                r#"
Das Root-Element des Baumes."#,
            ),
        },
        TagAttribute {
            name: "sortkeys",
            detail: None,
            documentation: Some(
                r#"
Attribute des Artikels, nach denen der Baum sortiert werden soll. Jede Ebene des Baums wird für sich sortiert."#,
            ),
        },
        TagAttribute {
            name: "sortsequences",
            detail: None,
            documentation: Some(
                r#"
                Für jedes Sortierkriterium muss eine Sortierreihenfolge festgelegt werden, mit der bestimmt wird, ob mit dem Sortierkriterium aufsteigend (´desc´), absteigend (´asc´) oder zufällig (´random´) sortiert wird."#,
            ),
        },
        TagAttribute {
            name: "sorttypes",
            detail: None,
            documentation: Some(
                r#"
Für jedes Sortierkriterium kann ein Sortiertyp festgelegt werden, der bestimmt, wie sortiert wird. Dabei ist eine Sortierung von Zeichenketten (`text`) oder eine Sortierung von Zahlen (`number`) möglich."#,
            ),
        },
    ]),
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

const SP_LOG: TagProperties = TagProperties {
    name: "sp:log",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[TagAttribute {
        name: "level",
        detail: None,
        documentation: Some(
            r#"
Der Log-Level (`TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR`, `FATAL`)"#,
        ),
    }]),
    attribute_rules: &[AttributeRule::Required("level")],
};

const SP_LOGIN: TagProperties = TagProperties {
    name: "sp:login",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "captcharequired",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut wird verwendet um eine Session zu erzeugen, die Handler-Aufrufe nur zusammen mit der Eingabe eines Captchas ermöglicht. Dadurch können Live-Applikationen (z.B. Anmeldung zu einem Newsletter) vor maschinellen Zugriffen geschützt werden."#,
            ),
        },
        TagAttribute {
            name: "client",
            detail: None,
            documentation: Some(
                r#"
`Anchor` des Clients."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "login",
            detail: None,
            documentation: Some(
                r#"
Nutzer-Login."#,
            ),
        },
        TagAttribute {
            name: "password",
            detail: None,
            documentation: Some(
                r#"
Nutzer-Passwort."#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Bereich in dem die erzeugte Verbindung zum IES gespeichert werden soll.
- `windowSession` Verbindung wird nur für ein Browser-Fenster/Browser-Tab verwendet (siehe `Window` Scope).
- `browserSession` Verbindung gilt für die komplette Browser-Instanz (siehe `Session` Scope).
- `application` Verbindung gilt für das gesamte IES-Modul (Web-Applikation). Bei Verwendung von `sp:login` in Live-Seiten ist dieser Scope zu empfehlen, wenn immer der gleiche Nutzer verwendet wird (siehe `Application` Scope).
"#,
            ),
        },
        TagAttribute {
            name: "session",
            detail: None,
            documentation: Some(
                r#"
Verwendet eine aktive Session für die Authentifizierung."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::ExactlyOneOf(&[
        "session", "login", "password", "client",
    ])],
};

const SP_LOOP: TagProperties = TagProperties {
    name: "sp:loop",
    detail: None,
    documentation: Some(
        r#"
Dient zur Ausgabe eines oder mehrerer Elemente."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "collection",
            detail: None,
            documentation: Some(
                r#"
Die zu iterierende Liste."#,
            ),
        },
        TagAttribute {
            name: "item",
            detail: None,
            documentation: Some(
                r#"
Die in `collection` angegebene Liste wird Element für Element durchlaufen. Mit dem in diesem Attribut angegebenen Namen, kann auf das aktuelle Element der Liste zugegriffen werden. Für das aktuelle Element können noch zusätzliche Informationen, die den Schleifendurchlauf betreffen abgefragt werden (siehe `IteratorItem`)."#,
            ),
        },
        TagAttribute {
            name: "list",
            detail: None,
            documentation: Some(
                r#"
Eine Zeichenkette mit dem in `separator` angegebenen Trennzeichen."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendenden Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "separator",
            detail: None,
            documentation: Some(
                r#"
Das Trennzeichen der übergebenen Liste."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::ExactlyOneOf(&["collection", "list"]),
        AttributeRule::OnlyWith("separator", "list"),
    ],
};

const SP_MAP: TagProperties = TagProperties {
    name: "sp:map",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "action",
            detail: None,
            documentation: Some(
                r#"
Aktion, die ausgeführt werden soll. Die folgenden Aktionen sind möglich: `put`, `remove`, `new` und `clear`.
- `put` Träg ein neues Schlüssel-Werte-Paar in die Map ein. Existiert schon ein Eintrag mit dem angegebenen Schlüssel, wird der alter Wert überschrieben.
- `putNotEmpty` Träg ein neues Schlüssel-Werte-Paar in die Map ein, wenn der Wert nicht null oder ein Leerstring ist. Existiert schon ein Eintrag mit dem angegebenen Schlüssel, wird der alter Wert überschrieben.
- `putAll` Bei dieser Aktion muss eine weitere Map übergeben werden. Alle Einträge werden in die Map übernommen.
- `merge` Bei dieser Aktion muss eine weitere Map übergeben werden. Alle Einträge werden in die Map übernommen. Enthält die Map aber weitere Map-Strukturen, werden diese zusammengeführt. Bei der Merge-Aktion werden immer Kopien der Daten in die Map übernommen. Bei putAll sind es immer Referenzen. Wie bei putAll werden alle Eintäge in die Map übernommen.
- `remove` Löscht das Schlüssel-Werte-Paar mit dem in `key` angegebenen Schlüssel aus der Map.
- `new` Erzeugt eine neue Map
- `clear` Löscht den Inhalt der Map
"#,
            ),
        },
        TagAttribute {
            name: "condition",
            detail: None,
            documentation: Some(
                r#"
Die Condition wird ausgewertet und als Bedingung in die Variable geschrieben."#,
            ),
        },
        TagAttribute {
            name: "default",
            detail: None,
            documentation: Some(
                r#"
Der Text, der verwendet wird, wenn die Inhalte von value, expression und body leer sind."#,
            ),
        },
        TagAttribute {
            name: "expression",
            detail: None,
            documentation: Some(
                r#"
Die Expression wird ausgewertet und als Wert in die Variable geschrieben."#,
            ),
        },
        TagAttribute {
            name: "key",
            detail: None,
            documentation: Some(
                r#"
Schlüssel, über den auf die Werte der Map zugegriffen werden soll."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendenden Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Map. Ein Punkt trennt die Namen für verschachtelte Maps."#,
            ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Kennzeichnet das Objekt, das eingefügt, ersetzt oder gelöscht werden soll."#,
            ),
        },
        TagAttribute {
            name: "overwrite",
            detail: None,
            documentation: Some(
                r#"
Bestimmt, ob eine evtl. vorhandene Variable überschrieben werden soll. `true` bzw. `false`."#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Namensraum, in dem die Variable definiert ist. Für diesen Tag ist der Page- und Request-Scope möglich (`page`, `request`)."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Kennzeichnet den Wert, der eingefügt, ersetzt oder gelöscht werden soll."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("action"),
        // depending on the action, key or the value/expression/.. may or may not be required
        AttributeRule::OnlyOneOf(&["value", "expression", "condition", "object"]), // or body
        AttributeRule::OnlyWithEither("default", &["object", "expression"]),
    ],
};

const SP_OPTION: TagProperties = TagProperties {
    name: "sp:option",
    detail: None,
    documentation: Some(
        r#"
Option-Tag, für das Select Tag."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
Gibt an, ob die Option deaktiviert werden soll."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "selected",
            detail: None,
            documentation: Some(
                r#"
Gibt an, ob die Option per default ausgewählt (selected) sein soll. Diese Einstellung gilt nur so lange, bis eine Auswahl durch den Bearbeiter vorgenommen und gespeichert wurde."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Gibt den Wert der Option an."#,
            ),
        },
    ]),
    attribute_rules: &[],
};

const SP_PASSWORD: TagProperties = TagProperties {
    name: "sp:password",
    detail: None,
    documentation: Some(
        r#"
Password-Tag, erzeugt ein Passwordfeld."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::None,
    attribute_rules: &[], // not documented
};

const SP_PRINT: TagProperties = TagProperties {
    name: "sp:print",
    detail: None,
    documentation: Some(
        r#"
Dient zur Ausgabe eines Attributes"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "arg",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut werden Werte für eine Formatierung im StringFormat angegeben. Für dieses Attribut gilt der Sonderfall, dass mehrere Werte in einzelnen `arg`-Attributen angegeben werden. Es ist also möglich mehrere Attribute `arg` in diesem Tag anzugeben. Diese Formatierung wird durchgeführt, wenn mindestens ein `arg`-Attribut angegeben wurde. Diese Formatierung wird nach allen anderen Formatierungen (deciamlformat, numberformat), de- und encodings und de- und encrypting durchgeführt. Die ermittelte Zeichenkette wird zusammen mit den übergebenen Argumenten in den `arg`-Attributen nach den Regeln des StringFormats formatiert. Zu beachten gilt, dass die `arg`-Argumente eine Expression erwartet. Zahlen können direkt übergeben werden. Zeichenketten müssen in ' gefasst werden
```spml
<sp:print text="a number: %d" arg="3"/>
<sp:print text="a word: %s" arg="'word'"/>
```"#,
            ),
        },
        TagAttribute {
            name: "condition",
            detail: None,
            documentation: Some(
                r#"
Alternative zu name (Siehe „Condition“)."#,
            ),
        },
        TagAttribute {
            name: "convert",
            detail: None,
            documentation: Some(
                r#"
Konvertiert die auszugebende Zeichenkette mit dem angegebenen Konverter. Es ist möglich eine kommaseparierte Liste von Konvertern anzugeben, die nacheinander ausgeführt werden. Gültige Werte sind:
- `html2text` Wandelt HTML in reinen Text um und versucht das Erscheinungsbild des Textes so gut wie möglich beizubehalten (Z.B. bei Tabellen)
- `wiki2html` Erzeugt aus einer Wiki-Text Syntax HTML. Weitere Informationen über Wiki-Text finden sie [hier](http://de.wikipedia.org/wiki/Hilfe:Textgestaltung)
- `html2wiki` Erzeugt aus HTML-Daten entsprechenden Wiki-Text. Weitere Informationen über Wiki-Text finden sie [hier](http://de.wikipedia.org/wiki/Hilfe:Textgestaltung)"#,
            ),
        },
        TagAttribute {
            name: "cryptkey",
            detail: None,
            documentation: Some(
                r#"
Der bei 3DES bzw. AES zu verwendene Schlüssel. Wird keiner angegeben, wird der IES-Default-Key verwendet."#,
            ),
        },
        TagAttribute {
            name: "dateformat",
            detail: None,
            documentation: Some(
                r#"
Angaben zur Datumsformatierung. Um für die Formatierung die gewünschte Sprache zu erhalten, bestehen folgende Möglichkeiten:
- Die Angabe einer Sprache über das `locale`-Attribut dieses Tags. Dies hat aber auch Einfluss auf die in `name` angegebenen Variablen.
- Übername des Locals des aktiven Publishers. Wird das `locale`-Attribut nicht verwendet, wird das Locale des aktiven Publishers verwendet. Ist kein Publisher aktiv (`in`-Modus) oder wurde im Publisher kein Locale angegeben, wird das default-Locale des Systems verwendet (im Regelfall `de_DE`).
- Angabe eines Locale in der Formatdefinition. In der Formatdefinition kann unabhängig von allen sonst definierten Formaten nur für dieses Format ein Locale angegeben werden. Dazu muss nach der Formatdefinition, mit einem Pipe-Zeichen (`|`) getrennt, das Locale angegeben werden:
```
dd.MM.yyyy HH:mm|en
```"#,
            ),
        },
        TagAttribute {
            name: "decimalformat",
            detail: None,
            documentation: Some(
                r#"
Angaben zur Dezimalformatierung. Um für die Formatierung die gewünschte Sprache zu erhalten, bestehen folgende Möglichkeiten:
- Die Angabe einer Sprache über das `locale`-Attribut dieses Tags. Dies hat aber auch Einfluss auf die in `name` angegebenen Variablen.
- Übername des Locals des aktiven Publishers. Wird das `locale`-Attribut nicht verwendet, wird das Locale des aktiven Publishers verwendet. Ist kein Publisher aktiv (`in`-Modus) oder wurde im Publisher kein Locale angegeben, wird das default-Locale des Systems verwendet (im Regelfall `de_DE`).
- Angabe eines Locale in der Formatdefinition. In der Formatdefinition kann unabhängig von allen sonst definierten Formaten nur für dieses Format ein Locale angegeben werden. Dazu muss nach der Formatdefinition mit einem Pipe-Zeichen (`|`) getrennt, das Locale angegeben werden:
```
##.00|en
```

__Hinweis__: *Bis Version 2.0.2 wurde der Doppelpunkt als Trennzeichen verwendet. Da dateformat diese Funktion ab Version 2.0.3 auch besitzt konnte der Doppelpunkt nicht mehr verwendet werden, da dieser Teil der Format-Definition sein kann. Aus diesem Grund wurde der Doppelpunkt als Locale-Trennzeichen als deprecated deklariert.*"#,
            ),
        },
        TagAttribute {
            name: "decoding",
            detail: None,
            documentation: Some(
                r#"
Decodiert die auszugebende Zeichenkette mit dem angegebenen Encoding. Es ist möglich eine kommaseparierte Liste von Encodings anzugeben, die nacheinander ausgeführt werden. Gültige Werte sind:
- `none` kein decoding
- `xml` decoded XML-Text:
    `&lt;` zu `<`
    `&gt;` zu `>`
    `&apos;` zu `'`
    `&quot;` zu `"`
    `&amp;` zu `&`
- `url` decoded eine URL (entsprechend dem Charset des Publishers)
- `base64` decoded eine BASE64 encodete Zeichenkette
- `escff (ab Version 2.0.3.26)` decodet die mit dem `escff`-encoding encodierten Zeichenketten."#,
            ),
        },
        TagAttribute {
            name: "decrypt",
            detail: None,
            documentation: Some(
                r#"
Decryptet die auszugebende Zeichenkette mit dem angegebenen Crypt-Algorithmus. Es ist möglich eine kommaseparierte Liste von Crypt-Algorithmen anzugeben, die nacheinander ausgeführt werden. Gültige Werte sind
- `3des` Triple DES Crypting Algorithmus
- `aes` AES Algorithmus
"#,
            ),
        },
        TagAttribute {
            name: "default",
            detail: None,
            documentation: Some(
                r#"
Auszugebender Default-Wert, wenn das Ergebnis von name bzw. `text` bzw. `expression` leer ist."#,
            ),
        },
        TagAttribute {
            name: "encoding",
            detail: None,
            documentation: Some(
                r#"
Encodiert die auszugebende Zeichenkette mit dem angegebenen Encoding. Es ist möglich eine kommaseparierte Liste von Encodings anzugeben, die nacheinander ausgeführt werden. Gültige Werte sind:
- `none` kein encoding
- `html` encoded HTML-Text
    `<` zu `&lt;`
    `>` zu `&gt;`
    `'` zu `&#039;`
    `"` zu `&#034;`
    `&` zu `&amp;`
    wird z.B. verwendet um value-Attribute in Formularen zu füllen
- `xml` encoded XML-Text
    `<` zu `&lt;`
    `>` zu `&gt;`
    `'` zu `&apos;`
    `"` zu `&quot;`
    `&` zu `&amp;`
    und alle Zeichen außerhalb des 7-Bit ASCII-Zeichensatzes
- `script` encoded für JavaScript, JSP, o.ä (escaped `\n`, `\r`, `"` und `'`)
    `\` zu `\\` *(Ab Version 2.0.3)*
    `'` zu `\'`
    `"` zu `\"`
    `\n` zu `\\n`
    `\r` zu `\\r`
- `php` *(ab Version 2.1.0.44)* encoded für PHP (escaped `\n`, `\r`, `$`, `"` und `'`)
    `\` zu `\\`
    `'` zu `\'`
    `"` zu `\"`
    `$` zu `\$`
    `\n` zu `\\n`
    `\r` zu `\\r`
- `php;[KEY=VALUE;KEY=VALUE;...]` *(ab Version 2.12.22)* Derzeit wird nur der KEY `'ignore'` aktzeptiert, um zu definieren, welche Werte NICHT encodiert werden sollen! Mögliche Werte für den `KEY` '`ignore'` sind:
    - `backslash`
    - `singleQuote`
    - `doubleQuote`
    - `carriageReturn`
    - `newLine`
    - `backspace`
    - `tab`
    - `dollar`
    Beispiel:
    ```
    php;ignore=singleQuote;ignore=newLine
    ```
- `url` encoded eine URL (entsprechend dem Charset des Publishers)
- `url; charset=latin1` encoded eine URL (mit dem übergebenen Charset)
- `entity` encoded alle Entitäten (jedes Zeichen wird zu seinem Entitäts-Pendant)
    z.B.
    `A` zu `&#65;`
    `[SPACE]` zu `&#32;`
- `plain` encoded `<`, `>` und Zeilenenden (`\n`, `\r`, `\r\n`)
    `<` zu `&lt;`
    `>` zu `&gt;`
    `\n` zu `<br>` oder `<br/>\n`
    `\r\n` zu `<br>` oder `<br/>\r\n`
- `ascii` encoded Windows-Sonderzeichen nach ASCII
- `path` encoded einen Verzeichnisnamen
- `filename` encoded einen Dateinamen
- `wikitext` *(ab Version 2.0.3)* Erzeugt ein Wiki-Text Syntax HTML. Weitere Informationen über Wiki-Text finden sie [hier](http://de.wikipedia.org/wiki/Hilfe:Textgestaltung)
    __Deprecated (ab Version 2.1.0)__ *wikitext ist kein encoding, sondern eine Konvertierung und sollte jetzt über das Attribut convert und dem Wert wiki2html verwendet werden*
- `base64 (ab Version 2.0.1)` encoded nach BASE64
- `base64NotChunked` (ab Version 2.8)* encoded nach BASE64, fügt aber keine Zeilenumbrüche hinzu
- `hex` *(ab Version 2.0.1)* encoded nach HEX. Hierbei wird jedes Zeichen in eine Zahl umgewandelt und dessen Hex-Wert ausgegeben
- `escff` *(ab Version 2.0.3.26)* encodet alle Zeichen mit einem Byte-Wert kleiner als 128 in einen Hex-Wert, beginnend mit einem Doppelpunkt (`:`). Dieses Encoding wird dazu verwendet, von `sp:form` erzeugte Formularfelder zu encoden, wenn das Formular an eine PHP-Seite gesendet wird. Dieses Encoding ist kein Standardencoding, sondern eine proprietäre Entwicklung von Sitepark."#,
            ),
        },
        TagAttribute {
            name: "encrypt",
            detail: None,
            documentation: Some(
                r#"
Encryptet die auszugebende Zeichenkette mit dem angegebenen Crypt-Algorithmus. Es ist möglich eine kommaseparierte Liste von Crypt-Algorithmen anzugeben, die nacheinander ausgeführt werden. Gültige Werte sind
- `3des` Triple DES Crypting Algorithmus
- `aes` AES Algorithmus
- `unixcrypt` UNIX-Crypt Algorithmus
- `md5` MD5 Algorithmus
- `sha` SHA Algorithmus"#,
            ),
        },
        TagAttribute {
            name: "expression",
            detail: None,
            documentation: Some(
                r#"
Alternative zu name (Siehe „Expression“)."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachiger Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Attribut das ausgegeben werden soll (Siehe „Attribute“)."#,
            ),
        },
        TagAttribute {
            name: "text",
            detail: None,
            documentation: Some(
                r#"
Alternative zu name (Siehe „Text“)."#,
            ),
        },
    ]),
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

const SP_QUERYTREE: TagProperties = TagProperties {
    name: "sp:querytree",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::None,
    attribute_rules: &[], // not documented
};

const SP_RADIO: TagProperties = TagProperties {
    name: "sp:radio",
    detail: None,
    documentation: Some(
        r#"
Radio Button-Tag, erzeugt einen RadioButton."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "checked",
            detail: None,
            documentation: Some(
                r#"
Gibt an, ob der Radio-Button per default gechecked werden soll. Diese Einstellung gibt es nur so lange, bis eine Auswahl durch den Bearbeiter vorgenommen und gespeichert wurde."#,
            ),
        },
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "readonly",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Der Typ des Eingabefeldes."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Setzt einen Default-Wert für die mit `name` angegebenen Variable, wenn sie leer ist."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("name")],
};

const SP_RANGE: TagProperties = TagProperties {
    name: "sp:range",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "collection",
            detail: None,
            documentation: Some(
                r#"
Name der Liste die verarbeitet werden soll."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Liste die aus der Auswahl erstellt wird."#,
            ),
        },
        TagAttribute {
            name: "range",
            detail: None,
            documentation: Some(
                r#"
Bereichsdefinition."#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Namensraum, in dem die Variable definiert ist. Für diesen Tag ist der Page- und Request-Scope möglich (`page`, `request`)."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("collection"),
        AttributeRule::Required("range"),
    ],
};

const SP_RETURN: TagProperties = TagProperties {
    name: "sp:return",
    detail: None,
    documentation: Some(
        r#"
Verlässt die SPML-Seite und setzt ggf. einen Rückgabewert für sp:include"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "condition",
            detail: None,
            documentation: Some(
                r#"
Die Condition wird ausgewertet und als Bedingung in den Rückgabe-Wert geschrieben."#,
            ),
        },
        TagAttribute {
            name: "default",
            detail: None,
            documentation: Some(
                r#"
Der Text, der verwendet wird, wenn die Inhalte von value, expression und body leer sind."#,
            ),
        },
        TagAttribute {
            name: "expression",
            detail: None,
            documentation: Some(
                r#"
Die Expression wird ausgewertet und als Rückgabe-Wert geschrieben."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendenden Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Evaluiert das Attribut und setzt den evaluierten Wert. Im Gegensatz zu `value` wird hier das Object zurück gegeben und nicht der Text."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Zu setzender Wert. Dieser wird immer als Zeichenkette ausgewertet."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::ExactlyOneOf(&["value", "expression", "condition", "object"]), // or body
        AttributeRule::OnlyWithEither("default", &["object", "expression"]),
    ],
};

const SP_SASS: TagProperties = TagProperties {
    name: "sp:sass",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Variable für den Zugriff auf das erzeugte CSS."#,
            ),
        },
        TagAttribute {
            name: "options",
            detail: None,
            documentation: Some(
                r#"
Die Optionen sind
- `outputStyle` `nested`, `compact`, `expanded`, `compressed`
- `includePath` Liste von Pfaden in denen nach SCSS-Scripten gesucht werden soll
- `precision` Genauigkeit von Mathematischen Rundungen"#,
            ),
        },
        TagAttribute {
            name: "source",
            detail: None,
            documentation: Some(
                r#"
Text der das Sass-Script enthält."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("source"),
        AttributeRule::Required("options"),
    ],
};

const SP_SCALEIMAGE: TagProperties = TagProperties {
    name: "sp:scaleimage",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "background",
            detail: None,
            documentation: Some(
                r#"
Hintergrundfabe für das Padding als HEX-Wert im RGB oder RGBA-Format.
Transparente Farben funktionieren nur bei PNG-Bildern.
`ffffff` = weiß
`ffffff00` = transparent bei PNG, ansonsten weiß
`00000000` = transparent bei PNG, ansonsten schwarz"#,
            ),
        },
        TagAttribute {
            name: "height",
            detail: None,
            documentation: Some(
                r#"
Höhe des zu berechnenden Bildes."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Variable für den Zugriff auf das `ScaleImage`-Objekt."#,
            ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Original-Bild."#,
            ),
        },
        TagAttribute {
            name: "options",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut können Bild-Optionen für die Berechnung des Bildes übergeben werden. Z.Z. ist nur die Übergabe eines Focus-Point möglich:
- `focuspoint` Der Focus-Point definiert den Bereich eines Bildes, der als Mittelpunkt des Bildes angenommen werden soll, wenn über den `padding`-Modus `'fit'` das Bild beim Verkleinern beschnitten wird.
```json
{ "focuspoint": { "x":0.062, "y":0.527 } }
```"#,
            ),
        },
        TagAttribute {
            name: "padding",
            detail: None,
            documentation: Some(
                r#"
Der Wert `"on"` erzeugt Rahmen zur Auffüllung der Flächen um das Bild. Damit ist das resultierende Bild immer so groß. wie durch die Auflösung gefordert.
Der Wert `"off"` erzeugt keinen Rahmen zur Auffüllung der Flächen um das Bild. Damit ist das resultierende Bild unter Umständen kleiner als die geforderte Auflösung.
Mit `"fit"` wird der größtmögliche Ausschnitt aus dem Originalbild bzw. aus dem durch excerpt gewählten Ausschnitt gesucht, bei dem das Seitenverhältnis der geforderten Auflösung entspricht. Es wird kein Rahmen erzeugt, sondern das Bild in einer Dimension gegebenenfalls gekürzt. Ist das gewünschte Bild größer als das Original wird das Bild wie bei `padding="on"` aufgefüllt.
Mit `"fit/no"` wird der größtmögliche Ausschnitt aus dem Originalbild bzw. aus dem durch excerpt gewählten Ausschnitt gesucht, bei dem das Seitenverhältnis der geforderten Auflösung entspricht. Es wird kein Rahmen erzeugt, sondern das Bild in einer Dimension gegebenenfalls gekürzt. Ist das gewünschte Bild größer als das Original wird das Bild nicht aufgefüllt."#,
            ),
        },
        TagAttribute {
            name: "quality",
            detail: None,
            documentation: Some(
                r#"
Rate mit der das Bild komprimiert wird. Die Werte liegen zwischen 1 und 100. Wobei 1 einer niedrige Qualität bzw. hohen Kompression und 100 einer hohen Qualität bzw. niedrige Kompression entspricht. Der angegeben Wert hat je nach Bildformat (gif, png, jpg) unterschiedlich interpretiert (siehe [hier](https://www.imagemagick.org/script/command-line-options.php#quality%7Chier)). Um für die unterschiedlichen Bildformate differenzierte Qualitätsstufen angeben zu können werden diese Kommasepariert Wertepaare mit Doppelpunkt-Trenner angegeben.
__Einfache Angabe__
```
60
```
__Spezifische Angabe__
```
gif:70,png:50,jpg:62
```"#,
            ),
        },
        TagAttribute {
            name: "scalesteps",
            detail: None,
            documentation: Some(
                r#"
Schalter um das Optimierungsverhalten im In-Modus auszuschalten. *(deprecated ab Version 2.22)*"#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Gültigkeitsbereich, in dem die Variable definiert ist. Möglich sind `page` und `request`."#,
            ),
        },
        TagAttribute {
            name: "width",
            detail: None,
            documentation: Some(
                r#"
Breite des zu berechnenden Bildes."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::AtleastOneOf(&["height", "width"]),
        AttributeRule::Deprecated("scalesteps"),
    ],
};

const SP_SCOPE: TagProperties = TagProperties {
    name: "sp:scope",
    detail: None,
    documentation: Some(
        r#"
Setzt bereichsweise oder global den Scope für die folgenden Tags"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[TagAttribute {
        name: "scope",
        detail: None,
        documentation: Some(
            r#"
Gültigkeitsbereich der als Standard-Scope im Tagbody definiert werden soll. Möglich sind `page` und `request`."#,
        ),
    }]),
    attribute_rules: &[AttributeRule::Required("scope")],
};

const SP_SEARCH: TagProperties = TagProperties {
    name: "sp:search",
    detail: None,
    documentation: Some(
        r#"
Findet die gewünschte Suche"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::None,
    attribute_rules: &[], // not documented
};

const SP_SELECT: TagProperties = TagProperties {
    name: "sp:select",
    detail: None,
    documentation: Some(
        r#"
Select-Tag, erzeugt den Rahmen einen Auswahlliste."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "multiple",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Der Typ des Eingabefeldes."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("name")],
};

const SP_SET: TagProperties = TagProperties {
    name: "sp:set",
    detail: None,
    documentation: Some(
        r#"
Setzt ein Attribute"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "condition",
            detail: None,
            documentation: Some(
                r#"
Die Condition wird ausgewertet und als Bedingung in die Variable geschrieben."#,
            ),
        },
        TagAttribute {
            name: "contentType",
            detail: None,
            documentation: Some(
                r#"
Hier kann zZ nur der Wert `json` gesetzt werden, um den angegebenen Content als JSON Objekt zu sichern."#,
            ),
        },
        TagAttribute {
            name: "default",
            detail: None,
            documentation: Some(
                r#"
Der Text, der verwendet wird, wenn die Inhalte von value, expression und body leer sind."#,
            ),
        },
        TagAttribute {
            name: "expression",
            detail: None,
            documentation: Some(
                r#"
Die Expression wird ausgewertet und als Wert in die Variable geschrieben."#,
            ),
        },
        TagAttribute {
            name: "insert",
            detail: None,
            documentation: Some(
                r#"
Definiert wie der Wert gesetzt werden soll. Die folgenden Werte sind möglich: `replace`, `append`, `prepend`
- `replace` Ersetzt den Wert einer eventuell bereits vorhandenen Variable
- `append` Hängt den Wert an eine eventuell bereits vorhandenen Variable hinten an
- `prepend` Hängt den Wert an eine eventuell bereits vorhandenen Variable vorne an"#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendenden Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der neuen Variable."#,
            ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Evaluiert das Attribut und setzt den evaluierten Wert. Im Gegensatz zu `value` wird hier das Object gespeichert und nicht der Text."#,
            ),
        },
        TagAttribute {
            name: "overwrite",
            detail: None,
            documentation: Some(
                r#"
Bestimmt, ob eine evtl. vorhandene Variable überschrieben werden soll. `true` bzw. `false`."#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Gültigkeitsbereich, in dem die Variable definiert ist. Möglich sind `page` und `request`."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Zu setzender Wert. Dieser wird immer als Zeichenkette ausgewertet."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::ExactlyOneOf(&["value", "expression", "condition", "object"]), // or body
        AttributeRule::OnlyWithEither("default", &["object", "expression"]),
        AttributeRule::OnlyOneOf(&["overwrite", "insert"]),
    ],
};

const SP_SORT: TagProperties = TagProperties {
    name: "sp:sort",
    detail: None,
    documentation: Some(
        r#"
Sortiert eine Liste"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "collection",
            detail: None,
            documentation: Some(
                r#"
Name der zu sortierenden Liste."#,
            ),
        },
        TagAttribute {
            name: "keys",
            detail: None,
            documentation: Some(
                r#"
Die Sortierkriterien nach denen die Elemente der Liste sortiert werden sollen. Diese Parameter ist optional. Ist kein Sortierkriterium angegeben, so wird das Element selber für die Sortierung verwendet. Dies ist beispielsweise der Fall, wenn die Liste nicht aus Elementen sondern aus einfachen Zeichenketten besteht. Die Objekte der Liste haben keine Attribute und es soll nach den Zeichenketten selbst sortiert werden."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendenden Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der sortierten Liste."#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Gültigkeitsbereich, in dem die Variable definiert ist. Möglich sind: `page`|`request`|`session`."#,
            ),
        },
        TagAttribute {
            name: "sequences",
            detail: None,
            documentation: Some(
                r#"
Für jedes Sortierkriterium muss eine Sortierreihenfolge festgelegt werden, mit der bestimmt wird, ob mit dem Sortierkriterium aufsteigend (´desc´), absteigend (´asc´) oder zufällig (´random´) sortiert wird. Ist kein Sortierkriterium angegeben, muss genau eine Sortierreihenfolge angegeben werden."#,
            ),
        },
        TagAttribute {
            name: "types",
            detail: None,
            documentation: Some(
                r#"
Für jedes Sortierkriterium muss ein Sortiertyp festgelegt werden, der bestimmt, wie sortiert wird. Dabei ist eine Sortierung von Zeichenketten (`text`) oder eine Sortierung von Zahlen (`number`) möglich. Ist kein Sortierkriterium angegeben, muss genau ein Sortiertyp angegeben werden."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("collection"),
    ],
};

const SP_SUBINFORMATION: TagProperties = TagProperties {
    name: "sp:subinformation",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Subinformation."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Optionale Angabe eines Typs. Dieser Tag erzeugt standardmäßig `Subinformation`-Objekte, kann aber auch bestimmte andere Datentypen erstellen. So kann über den Typ `calendar` ein `CalendarInformation`-Object angelegt werden."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("name")],
};

const SP_TAGBODY: TagProperties = TagProperties {
    name: "sp:tagbody",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::None,
    attribute_rules: &[],
};

const SP_TEXT: TagProperties = TagProperties {
    name: "sp:text",
    detail: None,
    documentation: Some(
        r#"
Text-Tag, erzeugt ein Eingabefeld."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "fixvalue",
            detail: None,
            documentation: Some(
                r#"
Überschreibt jeden vorhandenen Inhalt der mit `name` bestimmten Variablen mit dem durch `fixvalue` angegebenen Wert."#,
            ),
        },
        TagAttribute {
            name: "format",
            detail: None,
            documentation: Some(
                r#"
Wenn bei `type` beispielsweise `date` oder `number` angegeben wurde, kann `format` entsprechend des Types die Formatierung bestimmen (zB. `dd.MM.yyyy` oder `#0.00`)."#,
            ),
        },
        TagAttribute {
            name: "inputType",
            detail: None,
            documentation: Some(
                r#"
Setzt den [Typ](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#Form_%3Cinput%3E_types) des Eingeabefelds"#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "readonly",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Der Typ des Eingabefeldes."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Setzt einen Default-Wert für die mit `name` angegebenen Variable, wenn sie leer ist."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const SP_TEXTAREA: TagProperties = TagProperties {
    name: "sp:textarea",
    detail: None,
    documentation: Some(
        r#"
Textarea-Tag, erzeugt einen Einabebereich."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "fixvalue",
            detail: None,
            documentation: Some(
                r#"
Überschreibt jeden vorhandenen Inhalt der mit `name` bestimmten Variablen mit dem durch `fixvalue` angegebenen Wert."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "readonly",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Der Typ des Eingabefeldes."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Setzt einen Default-Wert für die mit `name` angegebenen Variable, wenn sie leer ist."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const SP_TEXTIMAGE: TagProperties = TagProperties {
    name: "sp:textimage",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "background",
            detail: None,
            documentation: Some(
                r#"
Hintergrundfarbe."#,
            ),
        },
        TagAttribute {
            name: "fontcolor",
            detail: None,
            documentation: Some(
                r#"
Schriftfarbe."#,
            ),
        },
        TagAttribute {
            name: "fontname",
            detail: None,
            documentation: Some(
                r#"
Name des zu verwendenden Zeichensatzes - muss unter dem angegebenem Namen auf dem IES-Server verfügbar sein."#,
            ),
        },
        TagAttribute {
            name: "fontsize",
            detail: None,
            documentation: Some(
                r#"
Schriftgröße."#,
            ),
        },
        TagAttribute {
            name: "fontstyle",
            detail: None,
            documentation: Some(
                r#"
Schriftstil. Mögliche Werte sind `plain`, `bold` und `italic`."#,
            ),
        },
        TagAttribute {
            name: "gravity",
            detail: None,
            documentation: Some(
                r#"
Ausrichtung der Schrift auf dem Bild.
- `c`, `center` Schrift zentrieren
- `n`, `north` Am oberen Rand ausrichten
- `ne`, `northeast` Am oberen-rechten Rand ausrichten
- `e`, `east` Am rechten Rand ausrichten
- `se`, `southeast` Am unteren-rechten Rand ausrichten
- `s`, `south` Am unteren Rand ausrichten
- `sw`, `southwest` Am unteren-linken Rand ausrichten
- `w`, `west` Am linken Rand ausrichten
- `nw`, `northwest` Am oberen-linken Rand aurichten"#,
            ),
        },
        TagAttribute {
            name: "height",
            detail: None,
            documentation: Some(
                r#"
Höhe des Bildes."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Variable für den Zugriff auf das `TextImage`-Objekt."#,
            ),
        },
        TagAttribute {
            name: "offset",
            detail: None,
            documentation: Some(
                r#"
Der Offset wird mit zwei Kommata getrennten Zahlen angegeben. Der erste Wert gibt den x-offset (horizontale Verschiebung), der zweite den y-offset (vertikale Verschiebung) an."#,
            ),
        },
        TagAttribute {
            name: "scope",
            detail: None,
            documentation: Some(
                r#"
Gültigkeitsbereich, in dem die Variable definiert ist. Möglich sind `page` und `request`."#,
            ),
        },
        TagAttribute {
            name: "text",
            detail: None,
            documentation: Some(
                r#"
Text der in ein Bild umgewandelt werden soll."#,
            ),
        },
        TagAttribute {
            name: "width",
            detail: None,
            documentation: Some(
                r#"
Breite des Bildes."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("text"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const SP_THROW: TagProperties = TagProperties {
    name: "sp:throw",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::None,
    attribute_rules: &[], // not documented
};

const SP_TOGGLE: TagProperties = TagProperties {
    name: "sp:toggle",
    detail: None,
    documentation: Some(
        r#"
Toggle-Tag erzeugt einen toggle der einen einzigen boolischen Wert speichert"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "fixvalue",
            detail: None,
            documentation: Some(
                r#"
Überschreibt jeden vorhandenen Inhalt der mit `name` bestimmten Variablen mit dem durch `fixvalue` angegebenen Wert."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "offValue",
            detail: None,
            documentation: Some(
                r#"
Wert der gesetzt wird, wenn die Checkbox nicht gechecked ist"#,
            ),
        },
        TagAttribute {
            name: "onValue",
            detail: None,
            documentation: Some(
                r#"
Wert der gesetzt wird, wenn die Checkbox gechecked ist"#,
            ),
        },
        TagAttribute {
            name: "readonly",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Der Typ des Eingabefeldes."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Setzt einen Default-Wert für die mit `name` angegebenen Variable, wenn sie leer ist."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const SP_UPLOAD: TagProperties = TagProperties {
    name: "sp:upload",
    detail: None,
    documentation: Some(
        r#"
Das Tag, erzeugt ein Eingabefeld zum Herunderladen von Dateien."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("name")],
};

const SP_URL: TagProperties = TagProperties {
    name: "sp:url",
    detail: None,
    documentation: Some(
        r#"
Fügt den ContextPath vor die angegebene URL und hängt, falls nötig die Session ID an die URL."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "absolute",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Da dieses Attribut von dem Attribut information abhängt, ist auch dieses Attribut veraltet. (Siehe Attribut information).*
Gibt an, ob die URL die durch das Attribut information ermittelt wurde mit absolutem Pfad ausgegeben werden soll."#,
            ),
        },
        TagAttribute {
            name: "command",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut ist veraltet und wird in zukünftigen Versionen nicht mehr unterstüzt werden. Kommandos wurden in der Version 2.0 zugunsten einer flexibleren Lösung abgeschafft. Ein Kommando bestand aus einem Template mit einem optionalen Handler. Für jede Template-Handler-Kombination musste ein eigenes Kommando angelegt werden. Diese Verbindung wurde aufgebrochen und durch die zwei neuen Attribute `template` und `handler` ersetzt. Um einen Handler aufzurufen und anschließend ein Template auszuführen, ist die Definition eines Kommandos nicht mehr nötig. Um einen Handler aufzurufen und anschließend ein Template auszuführen verwenden Sie die beiden Attribute `handler` und `template`. Um einen Handler aufzurufen und anschließend eine SPML-Seite auszuführen verwenden Sie die Attribute `handler` und `uri`.*
Existierendes Command. Muss im GUI definiert worden sein."#,
            ),
        },
        TagAttribute {
            name: "context",
            detail: None,
            documentation: Some(
                r#"
SPML-Seiten sind immer Teil einer Webapplikation. Jede Webapplikation besitzt eine Context-Pfad mit dem die URL beginnt (Es existert auch ein ROOT-Context-Pfad (`/`)). Soll die URL einer Seite herausgeschrieben werden, die in einer anderen Webapplikation liegt, so wird mit diesem Attribut der Context-Pfad angegeben. Context-Pfade von Webapplikationen können sich ändern. Damit auch bei solchen Änderungen die URL richtig generiert wird, sollte in den meisten Fällen eher das Attribut `module` verwendet werden."#,
            ),
        },
        TagAttribute {
            name: "gui",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Da dieses Attribut von dem Attribut `command` abhängt ist auch dieses Attribut veraltet. (Siehe Attribut `command`). Ein GUI war eine `List` von Kommandos um Live-Seiten zu realisieren. GUIs wurde durch Webapplikationen ersetzt.*
Steuert, ob das aktuelle GUI an die URL angefügt werden soll (nur in Verbindung mit `command` notwendig). Wird ab IES Version 2 nicht mehr ausgewertet, da keine GUIs mehr existieren. Sie werden durch Live-Seiten und Webapplikationen abgelöst."#,
            ),
        },
        TagAttribute {
            name: "handler",
            detail: None,
            documentation: Some(
                r#"
Handler der vor dem Aufruf, der mit `uri` oder `template` angegebenen Seite, ausgeführt werden soll."#,
            ),
        },
        TagAttribute {
            name: "information",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Dieses Attribute ist veraltet und wird in zukünftigen Versionen nicht mehr unterstützt. `sp:url` wurde in in früheren Versionen auch dazu verwendet die URL eines generierten Artikels zu ermitteln. Welche URL herausgeschieben werden sollte, wurde auch noch über die Attribute publisher und absolute gesteuert. Für diesen Zweck sollte `sp:url` nicht mehr verwendet werden. Statt dessen sollten die Attribute `url`, `relativeUrl`, `absoluteUrl` und die Methoden `url()`, `relativeUrl()` oder `absoluteUrl()` der Objecte `Article`, `Media` und `Resource` verwendet werden.*
Artikel dessen URL geschrieben werden soll."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendende Sprache bei mehrsprachiger Variablen."#,
            ),
        },
        TagAttribute {
            name: "module",
            detail: None,
            documentation: Some(
                r#"
SPML-Seiten sind immer Teil einer Webapplikation. Jede Webapplikation besitzt einen Context-Pfad mit dem die URL beginnt (Es existert auch ein ROOT-Context-Pfad (`/`)). Soll die URL einer Seite herausgeschrieben werden, die in einer anderen Webapplikation liegt, so wird mit diesem Attribut die ID dieser Webapplikation angegeben. Somit wird die URL auch dann richtig erzeugt, wenn sich der Context der Ziel-Webapplikation ändert."#,
            ),
        },
        TagAttribute {
            name: "publisher",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Da dieses Attribut von dem Attribut `information` abhängt, ist auch dieses Attribut veraltet. (Siehe Attribut `information`).*
Wird in Verbindung mit information verwendet, um zu bestimmen, aus welchem Publikationsbereich die URL erzeugt werden soll."#,
            ),
        },
        TagAttribute {
            name: "template",
            detail: None,
            documentation: Some(
                r#"
Template aus dem eine URL generiert werden soll. Alle Templates des IES liegen als SPML-Seiten im System. `sp:url` ermittelt die SPML-Seite des Templates und gibt sie aus. Dieses Attribut sollte nur für Umstellungen von Live-Seiten verwendet werden, die sich durch den Wegfall der Kommandos ergeben. Prinzipiell sollten Live-Seiten und Webapplikationen insgesamt, nicht mit Templates, sondern mit SPML-Seite realisiert werden."#,
            ),
        },
        TagAttribute {
            name: "uri",
            detail: None,
            documentation: Some(
                r#"
Dies kann ein beliebiger Pfad zu einer Seite sein. sp:url sorgt dafür, dass alle Session-Informationen an die URL gehängt werden, so dass die Session nicht verloren geht."#,
            ),
        },
        TagAttribute {
            name: "window",
            detail: None,
            documentation: Some(
                r#"
Innerhalb einer (`Session`) können für jedes Browserfenster weitere `Windowsessions` existieren. Dies ist sinnvoll, wenn die Session über ein Cookie gehalten wird und dennoch unterschiedliche Sessions in einem Browser benötigt werden. Existiert so eine Windowsession wird die `ID` dieser Session mit an die URL gehangen. Um dies zu verhindern, muss dieses Attribut auf `false` gesetzt werden."#,
            ),
        },
    ]),
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

const SP_WARNING: TagProperties = TagProperties {
    name: "sp:warning",
    detail: None,
    documentation: Some(
        r#"
Prüft, ob eine Warnung aufgetreten ist, markiert sie gegebenenfalls als gefangen und führt den innhalt des Tags aus."#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[TagAttribute {
        name: "code",
        detail: None,
        documentation: Some(
            r#"
Zu prüfender Error-Code."#,
        ),
    }]),
    attribute_rules: &[AttributeRule::Required("code")],
};

const SP_WORKLIST: TagProperties = TagProperties {
    name: "sp:worklist",
    detail: None,
    documentation: Some(
        r#"
Findet die gewünschte Workliste"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "element",
            detail: None,
            documentation: Some(
                r#"
Elemente, für die alle Worklist-Items geladen werden sollen. Mit diesem Parameter lassen sich alle offenen Tasks eines Elementes anzeigen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Der Name, über den auf die Collection zugegriffen werden kann."#,
            ),
        },
        TagAttribute {
            name: "role",
            detail: None,
            documentation: Some(
                r#"
Rolle, für die die Worklist-Items geladen werden sollen."#,
            ),
        },
        TagAttribute {
            name: "user",
            detail: None,
            documentation: Some(
                r#"
Nutzer, für die die Worklist geladen werden soll."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("name")],
};

const SP_ZIP: TagProperties = TagProperties {
    name: "sp:zip",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::None,
    attribute_rules: &[], // not documented
};

// SPTTAGS:

const SPT_COUNTER: TagProperties = TagProperties {
    name: "spt:counter",
    detail: None,
    documentation: Some(
        r#"
Zählt Zugriffe auf publizierte Informationen"#,
    ),
    // deprecated: true,
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "language",
            detail: None,
            documentation: Some(
                r#"
Gibt an in welcher Programmiersprache der Code generiert werden soll. Mögliche Werte sind:
- `javascript` Es wird JavaScript-Code generiert.
- `php` Es wird PHP-Code generiert."#,
            ),
        },
        TagAttribute {
            name: "mode",
            detail: None,
            documentation: Some(
                r#"
Der Zähler kann in verschiedenen Modi betrieben werden. Gültige Modi sind:
- `read` Counter wird nicht hochgezählt, sondern es wird nur der aktuelle Zählerstand als Variable ausgegeben.
- `write` Counter wird hochgezählt, aber es wird keine Variable gesetzt.
- *keine Angabe* Counter wird hochgezählt und der aktuelle Zählerstand wird als Variable ausgegeben."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Variable in der der Zugriffswert gespeichert wird."#,
            ),
        },
        TagAttribute {
                name: "varName",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__ *(ab Version 2.0.3) Das Attribut ist veraltet, und sollte nicht mehr verwendet werden. Stattdessen sollte varname verwendet werden.*
der Name der Variable, in die der aktuelle Counterwert ausgegeben wird [default=`sp_counter`]."#,
            ),
        },
        TagAttribute {
            name: "varname",
            detail: None,
            documentation: Some(
                r#"
der Name der Variable, in die der aktuelle Counterwert ausgegeben wird [default=`sp_counter`]."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Deprecated("varName"),
    ],
};

const SPT_DATE: TagProperties = TagProperties {
    name: "spt:date",
    detail: None,
    documentation: Some(
        r#"
Datums- und Uhrzeiteingabe mit Prüfung auf Gültigkeit"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
                name: "disabled",
                detail: None,
                documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
                name: "fixvalue",
                detail: None,
                documentation: Some(
                r#"
Überschreibt jeden vorhandenen Inhalt der mit `name` bestimmten Variablen mit dem durch `fixvalue` angegebenen Wert."#,
            ),
        },
        TagAttribute {
                name: "locale",
                detail: None,
                documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
                name: "name",
                detail: None,
                documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
                name: "nowButton",
                detail: None,
                documentation: Some(
                r#"
Zeigt bei true eine Schaltfläche zum setzen des aktuellen Zeitpunkts an"#,
            ),
        },
        TagAttribute {
                name: "placeholder",
                detail: None,
                documentation: Some(
                r#"
Muss ein Datum sind und wird als Placeholder eingesetzt"#,
            ),
        },
        TagAttribute {
                name: "readonly",
                detail: None,
                documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
                name: "size",
                detail: None,
                documentation: Some(
                r#"
`'size'`-Wert des generierten input-Tags."#,
            ),
        },
        TagAttribute {
            name: "type",
                detail: None,
                documentation: Some(
                r#"
Der Typ des Eingabefeldes.
- `date` Einfaches Eingabefeld im Format `TT.MM.JJJJ`
- `datetime` Zweifaches Eingabefeld für separate Eingabe von Datum und Uhrzeit im Format `TT.MM.JJJJ` und `HH:MM`"#,
            ),
        },
        TagAttribute {
                name: "value",
                detail: None,
                documentation: Some(
                r#"
Vorgabetext für das erzeugte Eingabefeld. Ohne bzw. mit einem leeren `value`-Attribut wird in der Eingabe das aktuelle Datum angezeigt. Soll das Eingabefeld leer bleiben, muss als `value` ein Leerzeichen (`" "`) angegeben werden."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const SPT_DIFF: TagProperties = TagProperties {
    name: "spt:diff",
    detail: None,
    documentation: Some(
        r#"
Vergleicht zwei Zeichenketten und zeigt die Unterschiede an"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "from",
            detail: None,
            documentation: Some(
                r#"
Text der verglichen werden soll. Wörter die hier enthalten und in `to` nicht mehr enthalten sind, werden als 'gelöscht' markiert."#,
        ),
        },
        TagAttribute {
            name: "style",
            detail: None,
            documentation: Some(
                r#"
CSS Styleangaben, die noch in den umschließenden div-Tag eingetragen werden."#,
        ),
        },
        TagAttribute {
            name: "to",
            detail: None,
            documentation: Some(
                r#"
Text der verglichen werden soll. Wörter die hier enthalten und in `from` nicht enthalten sind, werden als 'neu' markiert."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("from"),
        AttributeRule::Required("to"),
        AttributeRule::Required("style"),
    ],
};

const SPT_EMAIL2IMG: TagProperties = TagProperties {
    name: "spt:email2img",
    detail: None,
    documentation: Some(
        r#"
Ersetzt E-Mail-Adressen durch Bilder"#,
    ),
    // deprecated: true,
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
                name: "alt",
                detail: None,
                documentation: Some(
                r#"
Alternativtext der in die `alt`-Attribute der `<img>`-Tags eingetragen wird."#,
            ),
        },
        TagAttribute {
                name: "bgcolor",
                detail: None,
                documentation: Some(
                r#"
Hintergrundfarbe, die für den E-Mail-Text in dem generierten Bild verwendet werden soll."#,
            ),
        },
        TagAttribute {
                name: "bgcolor2",
                detail: None,
                documentation: Some(
                r#"
Hintergrundfarbe, die für den E-Mail-Text in dem generierten Bild für das Mailformular verwendet werden soll."#,
            ),
        },
        TagAttribute {
                name: "color",
                detail: None,
                documentation: Some(
                r#"
Schriftfarbe, die für den E-Mail-Text in dem generierten Bild verwendet werden soll."#,
            ),
        },
        TagAttribute {
                name: "color2",
                detail: None,
                documentation: Some(
                r#"
Schriftfarbe, die für den E-Mail-Text in dem generierten Bild für das Mailformular verwendet werden soll."#,
            ),
        },
        TagAttribute {
                name: "font",
                detail: None,
                documentation: Some(
                r#"
Schriftart, die für den E-Mail-Text in dem generierten Bild verwendet werden soll.
In der Standardinstallalation enthaltene Fonts sind: `Arial` `Lucida` `Verdana` `Futura`"#,
            ),
        },
        TagAttribute {
                name: "font2",
                detail: None,
                documentation: Some(
                r#"
Schriftart, die für den E-Mail-Text in dem generierten Bild für das Mailformular verwendet werden soll.
In der Standardinstallalation enthaltene Fonts sind: `Arial` `Lucida` `Verdana` `Futura`"#,
            ),
        },
        TagAttribute {
                name: "fontsize",
                detail: None,
                documentation: Some(
                r#"
Schriftgröße, die für den E-Mail-Text in dem generierten Bild verwendet werden soll."#,
            ),
        },
        TagAttribute {
                name: "fontsize2",
                detail: None,
                documentation: Some(
                r#"
Schriftgröße, die für den E-Mail-Text in dem generierten Bild für das Mailformular verwendet werden soll."#,
            ),
        },
        TagAttribute {
                name: "fontweight",
                detail: None,
                documentation: Some(
                r#"
Schriftstyle, die für den E-Mail-Text in dem generierten Bild verwendet werden soll.
Mögliche Werte sind: `plain` `bold` `italic`"#,
            ),
        },
        TagAttribute {
                name: "fontweight2",
                detail: None,
                documentation: Some(
                r#"
Schriftstyle, die für den E-Mail-Text in dem generierten Bild für das Mailformular verwendet werden soll.
Mögliche Werte sind: `plain` `bold` `italic`"#,
            ),
        },
        TagAttribute {
                name: "form",
                detail: None,
                documentation: Some(
                r#"
Artikel, der das Kontaktformular bereitstellt."#,
            ),
        },
        TagAttribute {
                name: "linkcolor",
                detail: None,
                documentation: Some(
                r#"
Schriftfarbe, die für den E-Mail-Text in dem generierten und verlinkten Bild verwendet werden soll."#,
            ),
        },
        TagAttribute {
                name: "name",
                detail: None,
                documentation: Some(
                r#"
Variable, in der der ersetzte Text abgelegt wird."#,
            ),
        },
        TagAttribute {
                name: "object",
                detail: None,
                documentation: Some(
                r#"
Objekt das den zu ersetzenden Text enhält."#,
            ),
        },
        TagAttribute {
                name: "onclick",
                detail: None,
                documentation: Some(
                r#"
JavaScript-Funktion die nach dem Klick auf eine E-Mail-Adresse ausgeführt werden soll."#,
            ),
        },
        TagAttribute {
                name: "popupheight",
                detail: None,
                documentation: Some(
                r#"
Höhe des Popup-Fensters für das Kontaktformular."#,
            ),
        },
        TagAttribute {
                name: "popupwidth",
                detail: None,
                documentation: Some(
                r#"
Breite des Popup-Fensters für das Kontaktformular."#,
            ),
        },
        TagAttribute {
            name: "title",
            detail: None,
            documentation: Some(
                r#"
Alternativtext der in die `title`-Attribute der `<img>`-Tags eingetragen wird."#,
            ),
        },
        TagAttribute {
            name: "urlparam",
            detail: None,
            documentation: Some(
                r#"
Übergabe weiterer Parameter an das Kontaktformular. Mehrere Parameter werden über `&amp;` getrennt (Beispiel: `"peter=pan&amp;donald=duck"`)"#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("object"),
    ],
};

const SPT_ENCRYPTEMAIL: TagProperties = TagProperties {
    name: "spt:encryptemail",
    detail: None,
    documentation: Some(
        r#"
Verschlüsselt Email-Adressen so, dass sie auch für Responsive-Design-Anforderungen verwendet werden können"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "form",
            detail: None,
            documentation: Some(
                r#"
Artikel, der das Kontaktformular bereitstellt."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Variable, in der der ersetzte Text abgelegt wird."#,
            ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Objekt das den zu ersetzenden Text enhält."#,
            ),
        },
        TagAttribute {
            name: "popupheight",
            detail: None,
            documentation: Some(
                r#"
Höhe des Popup-Fensters für das Kontaktformular."#,
        ),
        },
        TagAttribute {
            name: "popupwidth",
            detail: None,
            documentation: Some(
                r#"
Breite des Popup-Fensters für das Kontaktformular."#,
        ),
        },
        TagAttribute {
            name: "urlparam",
            detail: None,
            documentation: Some(
                r#"
Übergabe weiterer Parameter an das Kontaktformular. Mehrere Parameter werden über `&amp;` getrennt (Beispiel: `"peter=pan&amp;donald=duck"`)"#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("object"),
    ],
};

const SPT_ESCAPEEMAIL: TagProperties = TagProperties {
    name: "spt:escapeemail",
    detail: None,
    documentation: Some(
        r#"
Ersetzt Email-Adressen durch Bilder"#,
    ),
    // deprecated: true,
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
                name: "alt",
                detail: None,
                documentation: Some(
                r#"
Alternativtext der in die `alt`-Attribute der `<img>`-Tags eingetragen wird."#,
            ),
        },
        TagAttribute {
                name: "bgcolor",
                detail: None,
                documentation: Some(
                r#"
Hintergrundfarbe, die für den E-Mail-Text in dem generierten Bild verwendet werden soll."#,
            ),
        },
        TagAttribute {
                name: "color",
                detail: None,
                documentation: Some(
                r#"
Schriftfarbe, die für den E-Mail-Text in dem generierten Bild verwendet werden soll."#,
            ),
        },
        TagAttribute {
                name: "font",
                detail: None,
                documentation: Some(
                r#"
Schriftart, die für den E-Mail-Text in dem generierten Bild verwendet werden soll.
In der Standardinstallalation enthaltene Fonts sind: `Arial` `Lucida` `Verdana` `Futura`"#,
            ),
        },
        TagAttribute {
                name: "fontsize",
                detail: None,
                documentation: Some(
                r#"
Schriftgröße, die für den E-Mail-Text in dem generierten Bild verwendet werden soll."#,
            ),
        },
        TagAttribute {
                name: "fontweight",
                detail: None,
                documentation: Some(
                r#"
Schriftstyle, die für den E-Mail-Text in dem generierten Bild verwendet werden soll.
Mögliche Werte sind: `plain` `bold` `italic`"#,
            ),
        },
        TagAttribute {
                name: "form",
                detail: None,
                documentation: Some(
                r#"
Artikel, der das Kontaktformular bereitstellt."#,
            ),
        },
        TagAttribute {
                name: "name",
                detail: None,
                documentation: Some(
                r#"
Variable, in der der ersetzte Text abgelegt wird."#,
            ),
        },
        TagAttribute {
                name: "object",
                detail: None,
                documentation: Some(
                r#"
Objekt das den zu ersetzenden Text enhält."#,
            ),
        },
        TagAttribute {
                name: "onclick",
                detail: None,
                documentation: Some(
                r#"
JavaScript-Funktion die nach dem Klick auf eine E-Mail-Adresse ausgeführt werden soll."#,
            ),
        },
        TagAttribute {
                name: "popupheight",
                detail: None,
                documentation: Some(
                r#"
Höhe des Popup-Fensters für das Kontaktformular."#,
            ),
        },
        TagAttribute {
                name: "popupwidth",
                detail: None,
                documentation: Some(
                r#"
Breite des Popup-Fensters für das Kontaktformular."#,
            ),
        },
        TagAttribute {
            name: "title",
            detail: None,
            documentation: Some(
                r#"
Alternativtext der in die `title`-Attribute der `<img>`-Tags eingetragen wird."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("object")],
};

const SPT_FORMSOLUTIONS: TagProperties = TagProperties {
    name: "spt:formsolutions",
    detail: None,
    documentation: Some(
        r#"
Erzeugt eine eindeutige Url auf PDF-Dokumente des Form-Solutions Formular Servers."#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Diese Attribut bestimmt die Mehrsprachigkeit der Variable."#,
            ),
        },
        TagAttribute {
                name: "name",
                detail: None,
                documentation: Some(
                r#"
Name der Variable, unter der die Verknüpfung in die Datenbank geschrieben wird."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("name")],
};

const SPT_ID2URL: TagProperties = TagProperties {
    name: "spt:id2url",
    detail: None,
    documentation: Some(
        r#"
Durchsucht einen Text nach ID-Signaturen von Artikeln und ersetzt die IDs durch die URL des aktuellen Publikationsbereichs."#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "classname",
            detail: None,
            documentation: Some(
                r#"
Setzt oder ergänzt das class-Attribut des `<a>`-Tags für die Links, bei denen die ID durch die URL ersetzt wird."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut dient zur Auswahl der zu verwendenden Sprache bei mehrsprachigen Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Variablen, unter der die ersetzte Zeichenkette gespeichert werden soll."#,
            ),
        },
        TagAttribute {
            name: "objekt",
            detail: None,
            documentation: Some(
                r#"
Variablenname des Objektes, das die Zeichenkette enthält."#,
            ),
        },
        TagAttribute {
            name: "querystring",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut kann für die eingesetzten URL noch ein Querystring (Parameter nach einem `?`) angehängt werden."#,
            ),
        },
        TagAttribute {
            name: "url",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut kann benutzt werden um relative oder absolute URL zu generieren.
Erlaubte Werte: `relative` | `absolute`"#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("object"),
        AttributeRule::Required("querystring"),
    ],
};

const SPT_ILINK: TagProperties = TagProperties {
    name: "spt:ilink",
    detail: None,
    documentation: Some(
        r#"
Erzeugt einen Link auf das CMS"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "action",
            detail: None,
            documentation: Some(
                r#"
Gibt an ob das Objekt in der Bearbeitungsansicht (`edit`) oder in der Listen-Ansicht (`list`) geöffnet werden soll"#,
            ),
        },
        TagAttribute {
            name: "information",
            detail: None,
            documentation: Some(
                r#"
Optionale Angabe eines Artikels, auf den der Link zeigen soll (z.B. für Listen)."#,
            ),
        },
        TagAttribute {
            name: "step",
            detail: None,
            documentation: Some(
                r#"
Bei Templates, die mit mehreren Steps aufgebaut sind ist hiermit der Sprung an eine definierte Stelle möglich. Die Angabe erfolgt relativ zum Step "Verwaltung"."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Code für den generierten i-Link."#,
            ),
        },
    ]),
    attribute_rules: &[],
};

const SPT_IMAGEEDITOR: TagProperties = TagProperties {
    name: "spt:imageeditor",
    detail: None,
    documentation: Some(
        r#"
Erzeugt eine Bearbeitungsoberfläche für Bilder"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "delete",
            detail: None,
            documentation: Some(
                r#"
Aktiviert die Möglichkeit das Bild im Editor löschen zu können"#,
            ),
        },
        TagAttribute {
            name: "focalpoint",
            detail: None,
            documentation: Some(
                r#"
Aktiviert die Bearbeitung des Fokus-Punktes"#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
        ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Referenz zu einem Bild. Wenn eine Referenz zu einem Bild übergeben wird, ist der ImageEditor im Read-Only Modus."#,
        ),
        },
        TagAttribute {
            name: "width",
            detail: None,
            documentation: Some(
                r#"
Setzt die Breite des ImageEditors. Die Höhe wird dynamisch im Seitenverhältnis von 3/2 ermittelt."#,
            ),
        },
    ]),
    attribute_rules: &[],
};

const SPT_IMP: TagProperties = TagProperties {
    name: "spt:imp",
    detail: None,
    documentation: Some(
        r#"
Erzeugt einen <img src="...">-Tag für kleingerechnete, sowie aus Texten generierte Bilder"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
                name: "alt",
                detail: None,
                documentation: Some(
                r#"
Der Alternativtext für Bilder. Die Ausgabe erfolgt automatisch mit `encoding=ascii,html`, so dass Anführungszeichen im Alternativtext nicht zu Fehlern führen."#,
            ),
        },
        TagAttribute {
                name: "background",
                detail: None,
                documentation: Some(
                r#"
Die Farbe des Hintergrunds kann durch Hexadezimalwerte gesetzt werden (z.B. `e3a383`). Für Thumbnails wird hiermit die Farbe des `padding`-Rahmens bestimmt. Für Textbilder wird hiermit die Hindergrundfarbe des Bildes gesetzt."#,
            ),
        },
        TagAttribute {
                name: "color",
                detail: None,
                documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut ist veraltet und es sollte das Attribut fontcolor verwendet werden.*
Die Farbe der Schrift. Beispielsweise `AA00DD` oder `ff77ff`"#,
            ),
        },
        TagAttribute {
                name: "excerpt",
                detail: None,
                documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut wird nicht mehr unterstützt.*
Diese Option schneidet einen Ausschnitt eines größeren Bildes aus. Die ersten beiden Zahlen geben die linke obere Ecke des Ausschnittes an, die letzteren beiden die untere rechte Ecke. Mögliche Werte sind x0,y0,x1,y1 z.B. 100,100,300,200. Dieser Ausschnitt wird entsprechend der Optionen `height` und `width` noch verkleinert oder vergrößert. Hierbei wird gegebenenfalls ein Rand erzeugt, sprich die Option `padding=yes` ist automatisch gesetzt, falls nicht `padding=fit` gesetzt ist.
Alle 4 Zahlen können auch negativ sein. In diesem Fall wird der Wert als Differenz zum hinteren oder unteren Rand des Bildes berechnet. Also bedeutet -10% dasselbe wie 90% und -100 bei einem 300 Pixel breiten (oder hohen) Bild dasselbe wie 200. Ist `x0 > x1`, wird das Bild an der `x`-Achse gespiegelt.
Ist `y0 > y1`, wird das Bild an der `y`-Achse gespiegelt. Mit Angabe der Werte `x0,y0` z.B. 100,50 wird der Ausschnitt in der exakten Größe der mittels `height` und `width` geforderten Auflösung gewählt. Es ist dann keine Verkleinerung oder Vergrößerung mehr notwendig und man erhält einen 1:1 Ausschnitt des Orignals. Hierbei ist immer `padding=no` gesetzt.
Mit den Variablen `north`, `west`, `east` oder `south` wird ein in der jeweiligen Himmelsrichtung gelegener Ausschnitt in der mittels `height` und `width` geforderten Auflösung gewählt. Also wird mit `excerpt=south` ein Ausschnitt auf der Mitte der Bildbreite ganz unten gewählt, mit `excerpt=east` dagegen ein Ausschnitt aus der Mitte der Bildhöhe ganz rechts. Es ist dann keine Verkleinerung oder Vergrößerung mehr notwendig und man erhält einen 1:1 Ausschnitt des Orignals. Hierbei ist immer `padding=no` gesetzt.
Mit northwest, northeast, southwest oder southeast wird ein in der jeweiligen Himmelsrichtung gelegener Ausschnitt in der mittels `height` und `width` geforderten Auflösung gewählt. Also wird mit `excerpt=southeast` die äußerste untere, rechte Ecke des Originalbildes gewählt, mit `excerpt=northwest` dagegen die obere, linke Ecke. Es ist dann keine Verkleinerung oder Vergrößerung mehr notwendig und man erhält einen 1:1 Ausschnitt des Orignals. Hierbei is immer `padding=no` gesetzt."#,
            ),
        },
        TagAttribute {
                name: "font",
                detail: None,
                documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut ist veraltet und es sollte das Attribut fontname verwendet werden.*
Der Font (z.B. `Arial`)"#,
            ),
        },
        TagAttribute {
                name: "font-size",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut ist veraltet und es sollte das Attribut fontsize verwendet werden.*
Punkt-Größe des zu verwendenden Fonts (z.b.: `12`) "#,
            ),
        },
        TagAttribute {
                name: "font-weight",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut ist veraltet unde es sollte das Attribut fontweight verwendet werden.*
Die Dicke (Wichtung) des angegebenen Fonts (z.b.: `bold`, `200` oder `900`)."#,
            ),
        },
        TagAttribute {
                name: "fontcolor",
                detail: None,
                documentation: Some(
                r#"
Die Farbe der Schrift. Beispielsweise `AA00DD` oder `ff77ff`"#,
            ),
        },
        TagAttribute {
                name: "fontname",
                detail: None,
                documentation: Some(
                r#"
Der Font (z.B. `Arial`)"#,
            ),
        },
        TagAttribute {
                name: "fontsize",
                detail: None,
                documentation: Some(
                r#"
Punkt-Größe des zu verwendenden Fonts (z.b.: `12`) "#,
            ),
        },
        TagAttribute {
                name: "fontweight",
                detail: None,
                documentation: Some(
                r#"
Die Dicke (Wichtung) des angegebenen Fonts (z.b.: `bold`, `200` oder `900`)."#,
            ),
        },
        TagAttribute {
                name: "format",
                detail: None,
                documentation: Some(
                r#"
Die Formate `png` und `jpeg` können für Thumbnails verwendet werden"#,
            ),
        },
        TagAttribute {
                name: "gravity",
                detail: None,
                documentation: Some(
                r#"
Mit den Werten `n`, `w`, `e`, `s`, `nw`, `ne`, `sw`, `se` oder `Center`, `North`, `South`, `NorthEast` etc. kann die gewünschte Position des Textes im umgebenen Rahmen ausgerichtet werden. `West` ist der Standardwert. Das heißt alle Texte beginnen links auf mittlerer Höhe."#,
            ),
        },
        TagAttribute {
                name: "height",
                detail: None,
                documentation: Some(
                r#"
Die gewünschte Bildhöhe z.B. `100`. Die Breite wird unter Beibehaltung des Seiten-Verhältnisses des Originalbildes oder des gewählten Ausschnittes berechnet. Bei gesetzter Höhe ist daher die Option `padding` zwingend auf `no` gesetzt."#,
            ),
        },
        TagAttribute {
                name: "image",
                detail: None,
                documentation: Some(
                r#"
Bild-Object, das mit dem `spt:imp`-Tag verarbeitet werden soll."#,
            ),
        },
        TagAttribute {
                name: "manipulate",
                detail: None,
                documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut wird nicht mehr unterstützt.*
Erzeugt verschiedene Effekte wie weichzeichnen oder schärfen über `sharp1`, `sharp2`, `sharp3`, `sharp4`, `laplace1`, `laplace2`, `box`, `lowpass`, `neon`, `emboss` und `bw`"#,
            ),
        },
        TagAttribute {
                name: "offset",
                detail: None,
                documentation: Some(
                r#"
Der Anfangspunkt des auszugebenden Textes, die über die Option `gravity` angegeben ist, von der Seite aus gesehen. Ohne Angabe von `gravity` ist dies normalerweise die linke Seite in der Mitte (z.B.: `3,10`)"#,
            ),
        },
        TagAttribute {
                name: "padding",
                detail: None,
                documentation: Some(
                r#"
Der Wert `"on"` erzeugt Rahmen zur Auffüllung der Flächen um das Bild. Damit ist das resultierende Bild immer so groß wie durch die Auflösung gefordert. `padding=on` ist als Standardwert gesetzt, solange es nicht durch andere Optionen ausgeschlossen ist.
Der Wert `"off"` erzeugt keinen Rahmen zur Auffüllung der Flächen um das Bild. Damit ist das resultierende Bild unter Umständen kleiner als die geforderte Auflösung.
Mit `"fit"` wird der größte mögliche Ausschnitt aus dem Originalbild, bzw. aus dem durch `excerpt` gewählten Ausschnitt gesucht, bei dem das Seitenverhältnis der geforderten Auflösung entspricht. Es wird kein Rahmen erzeugt, sondern das Bild in einer Dimension gegebenenfalls gekürzt.
Um eine Abwärtskompatibilität zu gewährleisten, wird auch der Wert `"yes"` (entspricht `"on"`) und `"no"` (entspricht `"off"`) unterstützt."#,
            ),
        },
        TagAttribute {
                name: "paddingcolor",
                detail: None,
                documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut ist veraltet. Es sollte das Attribut `background` verwendet werden.*
Mit `paddingcolor` kann durch Hexadezimalwerte oder `X`-Window-Namen die Farbe des Rahmen bzw. des Hintergrundes, die bei `padding=yes` verwendet wird, angegeben werden. `transparent` ist das Default."#,
            ),
        },
        TagAttribute {
                name: "scalesteps",
                detail: None,
                documentation: Some(
                r#"
Schalter um das Optimierungsverhalten im `In`-Modus auszuschalten."#,
            ),
        },
        TagAttribute {
                name: "text",
                detail: None,
                documentation: Some(
                r#"
Der auszugebende Text in URL-encodeter Form."#,
            ),
        },
        TagAttribute {
            name: "text-transform",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut ist veraltet. Die Umwandlung von Texten sollten über die Attribute und Methoden des `String`-Objektes durchgeführt werden*
Manipulation des Textes, bevor das Bild berechnet wird. Mögliche Werte sind
- `uppercase` Alle Zeichen in Großbuchstaben umwandeln
- `lowercase` Alle Zeichen in Kleinbuchstaben umwandeln"#,
            ),
        },
        TagAttribute {
            name: "transform",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Dieses Attribut ist veraltet. Die Umwandlung von Texten sollten über die Attribute und Methoden des `String`-Objektes durchgeführt werden*
Manipulation des Textes, bevor das Bild berechnet wird. Mögliche Werte sind
- `uppercase` Alle Zeichen in Großbuchstaben umwandeln
- `lowercase` Alle Zeichen in Kleinbuchstaben umwandeln"#,
            ),
        },
        TagAttribute {
            name: "urlonly",
            detail: None,
            documentation: Some(
                r#"
__Deprecated__. *Das Attribut ist veraltet, und sollte nicht mehr verwendet werden. Wird nur die URL benötigt oder weiterer Metadaten des berechneten Bildes sollte der Tag `sp:scaleimage` für Thumbnails oder sp:textimage für Texte verwendet werden.*
schreibt nur die URL ohne `<img>`-Tag heraus"#,
            ),
        },
        TagAttribute {
            name: "width",
            detail: None,
            documentation: Some(
                r#"
Die gewünschte Bildbreite z.B. `100`. Die Höhe wird unter Beibehaltung des Seiten-Verhältnisses des Originalbildes oder des gewählten Ausschnittes berechnet. Bei gesetzter Breite ist daher die Option `padding` zwingend auf `no` gesetzt"#,
            ),
        },
    ]),
    attribute_rules: &[
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
    ],
};

const SPT_ITERATOR: TagProperties = TagProperties {
    name: "spt:iterator",
    detail: None,
    documentation: Some(
        r#"
Erzeugt Wiederholfelder"#,
    ),
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
Liste von Elementen (beginnend mit `1` für das erste Listenelement: `1,2,4,5,8,...`), welche nicht bearbeitet werden dürfen. Sowohl das Listenelement selbst kann nicht gelöscht werden, also auch alle in dem Listenelement enthalten Felder können nicht bearbeitet werden."#,
            ),
        },
        TagAttribute {
            name: "invert",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut (`true`, `false`) kann die Darstellung der Liste beeinflusst werden. Wenn auf `true` gesetzt, werden die Listenelemente mit einem dunklen Rahmen umschlossen, andernfalls mit einem hellen Rahmen. Eine eventuell vorhandene Blocküberschrift (siehe Attribut itemtext passt sich der Darstellung an und erscheint bei invertierter Darstellung in heller Schrift auf dunklem Grund."#,
            ),
        },
        TagAttribute {
            name: "item",
            detail: None,
            documentation: Some(
                r#"
Die in `name` angegebene Liste wird Element für Element durchlaufen. Mit dem, in diesem Attribut angegebenen Namen kann auf das aktuelle Element der Liste zugegriffen werden. Für das aktuelle Element können noch zusätzliche Informationen die den Schleifendurchlauf betreffen abgefragt werden (siehe `IteratorItem`)."#,
            ),
        },
        TagAttribute {
            name: "itemtext",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut kann ein Text definiert werden, der an Stelle der Listennummerierung als Blocküberschrift erscheint, wenn das Attribut `layout` nicht auf `plain` gesetzt wurde. Dem Text folgt bei mehr als 1 Listenelement automatisch die Listennummerierung in der Form `"x von n"`, wobei `x` die Nummer des Listenelementes ist und `n` die Anzahl der Listenelemente."#,
            ),
        },
        TagAttribute {
            name: "layout",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Darstellung der Liste. Wenn `plain`, werden die Listenelemente ohne Rahmen und Schaltflächen ausgegeben. Mögliche Schaltflächen zum Hinzufügen oder Löschen eines Listenelementes müssen manuell im Template gecodet werden."#,
            ),
        },
        TagAttribute {
            name: "max",
            detail: None,
            documentation: Some(
                r#"
Die Anzahl der maximal zu iterierenden Elemente. Enthält die zu iterierende Liste mehr Elemente als in `max` angegeben, so wird die Anzahl der Elemente auf die Anzahl `max` gekürzt."#,
            ),
        },
        TagAttribute {
            name: "min",
            detail: None,
            documentation: Some(
                r#"
Die Anzahl der mindestens zu iterierenden Elemente. Enthält die zu iterierende Liste weniger Elemente als in `min` angegeben werden so viele leere Elemente hinzugefügt, bis mindestens die in `min` angegebene Anzahl von Elementen vorhanden ist."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Variablen, unter der die Liste auch für die Ausgabe erreichbar ist. Dieses Attribut entspricht dem `collection`-Attribut des `sp:iterator`-Tags."#,
            ),
        },
        TagAttribute {
            name: "readonly",
            detail: None,
            documentation: Some(
                r#"
Attribut (`true`, `false`) was bestimmt, ob die Schaltflächen zum Hinzufügen, Löschen und Verschieben von Listenelementen angezeigt werden, wenn das Attribut `layout nicht auf `plain gesetzt wurde."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("name")],
};

const SPT_LINK: TagProperties = TagProperties {
    name: "spt:link",
    detail: None,
    documentation: Some(
        r#"
Erzeugt Links auf Informationen und bindet Bildmedien ein."#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "filter",
                detail: None,
                documentation: Some(
                r#"
Die Filterdefinition für die Filtertypen Wildcard und regulärer Ausdruck. Der mit dem Attribut `mode` angegebene Modus wird verwendet. Ohne Angabe eines Modus wird `simple` verwendet."#,
            ),
        },
        TagAttribute {
            name: "filterattribute",
            detail: None,
            documentation: Some(
                r#"
Attribut, auf das der Filter angewendet werden soll."#,
            ),
        },
        TagAttribute {
            name: "filteric",
            detail: None,
            documentation: Some(
                r#"
Ist Ignore-Case auf `true` gesetzt, wird eine Groß- und Kleinschreibung nicht berücksichtigt. Dieses Attribut gilt nur für die Filtertypen Wildcard und regulärer Ausdruck."#,
            ),
        },
        TagAttribute {
            name: "filterinvert",
            detail: None,
            documentation: Some(
                r#"
Invertiert die Logik des Filters. Alle Elemente die normalerweise herausgefiltert würden, bilden die Filterergebnisse."#,
            ),
        },
        TagAttribute {
            name: "filtermode",
            detail: None,
            documentation: Some(
                r#"
Auswahl des Filter-Mechanismus.
__simple (Wildcard-Filter)__
Der Filter kann die Wildcards `*` für beliebige Zeichen und `?` für ein beliebiges Zeichen enthalten. So würde eine wie folgt gefilterte Liste nur Elemente enthalten, die mit a beginnen.
```regex
a*
```
__regex (Reguläre Ausdrücke)__
Für komplexe Filter stehen Reguläre Ausdrücke (POSIX) zur Verfügung. So würde im regex-Filtermode eine mit
```regex
[a-dA-D].*
```
gefilterte Liste nur Elemente enthalten, die mit dem Buchstaben A, a, B, b, C, c, d oder D beginnen."#,
            ),
        },
        TagAttribute {
            name: "filterquery",
            detail: None,
            documentation: Some(
                r#"
mit diesem Parameter kann eine Suchabfrage definiert werden, welche die anzuzeigenden Elemente für jeden Pool filtert. Als Ergänzung zu den folgenden 5 Parametern, die mit sp:filter arbeiten, ist es so auch möglich, Artikel herauszufiltern, deren Informationen sich in Iteratoren befinden."#,
            ),
        },
        TagAttribute {
            name: "fixvalue",
            detail: None,
            documentation: Some(
                r#"
Überschreibt jeden vorhandenen Inhalt der mit name bestimmten Variablen mit dem durch `fixvalue` angegebenen Wert."#,
            ),
        },
        TagAttribute {
            name: "height",
            detail: None,
            documentation: Some(
                r#"
Bei `type="image"` kann durch dieses Attribut der `'height'`-Wert des generierten `<img ...>`-Tags gesetzt werden."#,
            ),
        },
        TagAttribute {
            name: "hidden",
            detail: None,
            documentation: Some(
                r#"
Macht das Feld unsichtbar."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Diese Attribut bestimmt die Mehrsprachigkeit der Variable."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Variable, unter der der Systemlink in die Datenbank geschrieben wird."#,
            ),
        },
        TagAttribute {
            name: "pools",
                detail: None,
                documentation: Some(
                r#"
Kommaseparierte Liste mit `Anchor`s von Artikelpools oder mit `ID`s von Artikelpools; die Elemente der Pools werden dem Redakteur zur Auswahl angeboten."#,
            ),
        },
        TagAttribute {
            name: "previewimage",
            detail: None,
            documentation: Some(
                r#"
`previewimage=false` verhindert die automatische Anzeige von verküpften Bildern."#,
            ),
        },
        TagAttribute {
            name: "showtree",
            detail: None,
            documentation: Some(
                r#"
wenn `false`, werden nur die im Attribut pools übergebenen Einsprungpunkte in der Baumansicht angezeigt (ohne Aufklappmöglichkeit und ohne Kinder)"#,
            ),
        },
        TagAttribute {
            name: "size",
            detail: None,
            documentation: Some(
                r#"
HTML-size Wert des von `spt:link` erzeugten Eingabefeldes."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Typ der Verlinkung
- `systemlink` bei Änderung des verlinkten Artikels wird der verlinkende Artikel neu publiziert.
- `navlink` bei Änderung des verlinkenden Artikels wird der verlinkte neu publiziert.
- `resultlink` wird auf eine Suchabfrage verlinkt wird bei Änderung der Suchabfrage der verlinkende Artikel neu publiziert.
- `link` es erfolgt keine Aktualisierung in irgendeine Richtung.
- `image` `'image'` erzeugt einen Linktype `'systemlink'`
    Es wird bei Verwendung im Ausgabebereich eines Templates ein `<a href=...>`-Tag generiert. Die Auswahl, die dem Redakteur zur Verfügung gestellt wird, ist von dieser Einstellung abhängig. Ist `'type=image'` gesetzt, kann der Redakteur ein Bildmedium auswählen, mit dem ein `<img ...>`-Tag generiert wird."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Vorgabefeld für das erzeugte Eingabefeld."#,
        ),
        },
        TagAttribute {
            name: "width",
            detail: None,
            documentation: Some(
                r#"
Bei `type="image"` kann durch dieses Attribut der `'width'`-Wert des generierten `<img ...>`-Tags gesetzt werden."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
        AttributeRule::OnlyWith("filterattribute", "filter"),
        AttributeRule::OnlyWith("filteric", "filter"),
        AttributeRule::OnlyWith("filterinvert", "filter"),
        AttributeRule::OnlyWith("filtermode", "filter"),
    ],
};

const SPT_NUMBER: TagProperties = TagProperties {
    name: "spt:number",
    detail: None,
    documentation: Some(
        r#"
Zahleneingabe mit Prüfung auf Gültigkeit"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "align",
            detail: None,
            documentation: Some(
                r#"
Ausrichtung des Inhalts für das erzeugte Eingabefeld."#,
            ),
        },
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "fixvalue",
            detail: None,
            documentation: Some(
                r#"
Überschreibt jeden vorhandenen Inhalt der mit `name` bestimmten Variablen mit dem durch `fixvalue` angegebenen Wert."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "readonly",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "size",
            detail: None,
            documentation: Some(
                r#"
`'size'`-Wert des generierten input-Tags."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Setzt einen Default-Wert für die mit `name` angegebenen Variable, wenn sie leer ist."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const SPT_PERSONALIZATION: TagProperties = TagProperties {
    name: "spt:personalization",
    detail: None,
    documentation: Some(
        r#"
Definiert personalisierte Bereiche"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "information",
            detail: None,
            documentation: Some(
                r#"
Artikel dessen Personalisierungs-Daten für das Apache-Personalisierungsmodul ausgegeben werden sollen."#,
            ),
        },
        TagAttribute {
            name: "mode",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut kann benutzt werden um eine alternative Ausgabe zu erzeugen.
Mit `mode="php"` werden die Personlisierungsinformationen auf PHP-Ebene ausgewertet und anstelle von `<sitepark_authpart>`-Tags wird entsprechender PHP-Code rausgeschrieben.
Unterstützte Werte derzeit: `php`"#,
        ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Variablenname, unter dem die Rechte gespeichert werden."#,
        ),
        },
        TagAttribute {
            name: "publisher",
            detail: None,
            documentation: None,
        },
    ]),
    attribute_rules: &[],
};

const SPT_PREHTML: TagProperties = TagProperties {
    name: "spt:prehtml",
    detail: None,
    documentation: Some(
        r#"
HTML-Code nachbearbeiten."#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Name der Variablen, unter der die ersetzte Zeichenkette gespeichert werden soll."#,
            ),
        },
        TagAttribute {
            name: "object",
            detail: None,
            documentation: Some(
                r#"
Variablenname des Objektes, das die Zeichenkette enthält."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::Required("object"),
    ],
};

const SPT_SMARTEDITOR: TagProperties = TagProperties {
    name: "spt:smarteditor",
    detail: None,
    documentation: Some(
        r#"
Integriert den WYSIWYG-SmartEditor ins CMS"#,
    ),
    // deprecated: true,
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
                name: "cols",
                detail: None,
                documentation: Some(
                r#"
Breite des Eingabefeldes in Spalten."#,
            ),
        },
        TagAttribute {
            name: "hide",
            detail: None,
            documentation: Some(
                r#"
Ist `hide="false"` gesetzt, so wird eine Textarea generiert, die den vom SmartEditor erzeugten Quellcode aufnimmt. Ist `hide="true"` gesetzt, so erscheint lediglich die Schaltfläche, über die sich der SmartEditor starten lässt. Standardwert ist `true`."#,
        ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Eingabefeldes."#,
        ),
        },
        TagAttribute {
            name: "options",
            detail: None,
            documentation: Some(
                r#"
Optionen, die beim Aufruf des Smarteditors an diesen übergeben werden."#,
            ),
        },
        TagAttribute {
            name: "rows",
            detail: None,
            documentation: Some(
                r#"
Höhe des Eingabefeldes in Zeilen."#,
        ),
        },
        TagAttribute {
            name: "textlabel",
            detail: None,
            documentation: Some(
                r#"
Beschriftung des Smarteditorfeldes, oberhalb."#,
        ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Vorgegebener Inhalt des Feldes."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("name")],
};

const SPT_SPML: TagProperties = TagProperties {
    name: "spt:spml",
    detail: None,
    documentation: Some(
        r#"
schreibt den Header für SPML-Live Seiten"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
                name: "api",
                detail: None,
                documentation: Some(
                r#"
Kommaseparierte Liste von APIs, dessen Packages mit Import-Anweisungen eingebunden werden sollen. Mögliche APIs sind:
- `log4j` Siehe [hier](http://logging.apache.org/log4j/1.2/apidocs/index.html)
- `jdom` Siehe [hier](http://www.jdom.org/docs/apidocs/index.html)
- `mail` Siehe [hier](http://java.sun.com/products/javamail/javadocs/index.html)"#,
            ),
        },
    ]),
    attribute_rules: &[],
};

const SPT_TEXT: TagProperties = TagProperties {
    name: "spt:text",
    detail: None,
    documentation: Some(
        r#"
Einzeiliges Textfeld, das Versionierung unterstützt"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "editablePlaceholder",
            detail: None,
            documentation: Some(
                r#"
Mit dem Setzen von `false`, kann die Editierbarkeit von Placeholdern deaktiviert werden."#,
            ),
        },
        TagAttribute {
            name: "fixvalue",
            detail: None,
            documentation: Some(
                r#"
Überschreibt jeden vorhandenen Inhalt der mit `name` bestimmten Variablen mit dem durch `fixvalue` angegebenen Wert."#,
            ),
        },
        TagAttribute {
            name: "format",
                detail: None,
                documentation: Some(
                r#"
Wenn bei type beispielsweise `date` oder `number` angegeben wurde, kann format entsprechend des Types die Formatierung bestimmen (`dd.MM.yyyy` oder `#0.00`)."#,
            ),
        },
        TagAttribute {
            name: "hyphenEditor",
            detail: None,
            documentation: Some(
                r#"
Deaktiviert bei `false` den Hyphen-Editor"#,
            ),
        },
        TagAttribute {
            name: "inputType",
            detail: None,
            documentation: Some(
                r#"
Setzt den [Typ](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#Form_%3Cinput%3E_types) des Eingeabefelds"#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "readonly",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "size",
            detail: None,
            documentation: Some(
                r#"
`'size'`-Wert des generierten `input`-Tags."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Der Typ des Eingabefeldes."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Setzt einen Default-Wert für die mit `name` angegebenen Variable, wenn sie leer ist."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const SPT_TEXTAREA: TagProperties = TagProperties {
    name: "spt:textarea",
    detail: None,
    documentation: Some(
        r#"
Erzeugt ein mehrzeiliges Textfeld, das Versionierung unterstützt"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "editablePlaceholder",
            detail: None,
            documentation: Some(
                r#"
Mit dem Setzen von `false`, kann die Editierbarkeit von Placeholdern deaktiviert werden."#,
            ),
        },
        TagAttribute {
            name: "fixvalue",
            detail: None,
            documentation: Some(
                r#"
Überschreibt jeden vorhandenen Inhalt der mit `name` bestimmten Variablen mit dem durch `fixvalue` angegebenen Wert."#,
            ),
        },
        TagAttribute {
            name: "format",
                detail: None,
                documentation: Some(
                r#"
Wenn bei type beispielsweise `date` oder `number` angegeben wurde, kann format entsprechend des Types die Formatierung bestimmen (`dd.MM.yyyy` oder `#0.00`)."#,
            ),
        },
        TagAttribute {
            name: "hyphenEditor",
            detail: None,
            documentation: Some(
                r#"
Deaktiviert bei `false` den Hyphen-Editor"#,
            ),
        },
        TagAttribute {
            name: "inputType",
            detail: None,
            documentation: Some(
                r#"
Setzt den [Typ](https://developer.mozilla.org/en-US/docs/Web/HTML/Element/input#Form_%3Cinput%3E_types) des Eingeabefelds"#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "readonly",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "size",
            detail: None,
            documentation: Some(
                r#"
`'size'`-Wert des generierten `input`-Tags."#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Der Typ des Eingabefeldes."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Setzt einen Default-Wert für die mit `name` angegebenen Variable, wenn sie leer ist."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const SPT_TIMESTAMP: TagProperties = TagProperties {
    name: "spt:timestamp",
    detail: None,
    documentation: Some(
        r#"
Zeitstempel in ein Eingabefeld schreiben"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "connect",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut wird das Eingabefeld referenziert, in das der Zeitstempel geschrieben werden soll. Das Eingabefeld muss explizit initialisert werden, da der `spt:timestamp`-Tag den Zeitstempel nicht direkt in die Datenbank schreibt."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("connect")],
};

const SPT_TINYMCE: TagProperties = TagProperties {
    name: "spt:tinymce",
    detail: None,
    documentation: Some(
        r#"
Integriert einen Editor"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "cols",
            detail: None,
            documentation: Some(
                r#"
Breite des Eingabefeldes in Spalten."#,
            ),
        },
        TagAttribute {
            name: "config",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut wird der Name einer Konfiguration angegeben. Die in dieser Konfiguration angegebenen Konfigurationsparameter überschreiben die IES-Standardkonfiguration. Die Angaben werden in JSON angegeben, wobei die umschließenden geschweiften Klammern `{}` nicht mit angegeben werden dürfen. Die möglichen Konfigurationsparameter sind unter [TinyMCE:Configuration](http://wiki.moxiecode.com/index.php/TinyMCE:Configuration) aufgelistet."#,
            ),
        },
        TagAttribute {
            name: "configextension",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut wird der Name einer Konfigurations-Ergänzung angegeben. Die in dieser Ergänzung angegebenen Konfigurationsparameter ergänzen und überschreiben die Parameter der IES-Standardkonfiguration. Die Angaben werden in JSON angegeben, wobei die umschließenden geschweiften Klammern `{}` nicht mit angegeben werden dürfen. Die möglichen Konfigurationsparameter sind unter [TinyMCE:Configuration](http://wiki.moxiecode.com/index.php/TinyMCE:Configuration) aufgelistet."#,
            ),
        },
        TagAttribute {
            name: "configvalues",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut wird der Name einer Konfigurations-Ergänzung angegeben. Die in dieser Ergänzung angegebenen Konfigurationsparameter ergänzen und überschreiben die Parameter der IES-Standardkonfiguration. Die Angaben werden in JSON angegeben, wobei die umschließenden geschweiften Klammern `{}` nicht mit angegeben werden dürfen. Die möglichen Konfigurationsparameter sind unter [TinyMCE:Configuration](http://wiki.moxiecode.com/index.php/TinyMCE:Configuration) aufgelistet."#,
            ),
        },
        TagAttribute {
            name: "disabled",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "fixvalue",
            detail: None,
            documentation: Some(
                r#"
Überschreibt jeden vorhandenen Inhalt der mit `name` bestimmten Variablen mit dem durch `fixvalue` angegebenen Wert."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "pools",
            detail: None,
            documentation: Some(
                r#"
Kommaseparierte Liste von `Anchor` von Artikelpools oder von `ID`s von Artikelpools; die Elemente der Pools werden dem Redakteur in einem Linkdialog innerhalb des Editors zur Auswahl angeboten. Voraussetzung, dass der interne Linkdialog überhaupt angezeigt wird, ist die Konfiguration des Editors mit `iesLink` über `theme_advanced_buttons` in der [TinyMCE:Configuration](http://wiki.moxiecode.com/index.php/TinyMCE:Configuration). Ausgabeseitig muss man darauf achten, dass ein interner Link vor der Ausgabe mit `spt:id2url` umgewandelt wird."#,
            ),
        },
        TagAttribute {
            name: "readonly",
            detail: None,
            documentation: Some(
                r#"
HTML-Attribut (`true`, `false`)."#,
            ),
        },
        TagAttribute {
            name: "rows",
            detail: None,
            documentation: Some(
                r#"
Höhe des Eingabefeldes in Zeilen."#,
            ),
        },
        TagAttribute {
            name: "theme",
            detail: None,
            documentation: Some(
                r#"
Konfigurationstypen, die den Funktionsumfang für den Editor beschreiben. Mögliche Themes sind `simple` und `advanced`"#,
            ),
        },
        TagAttribute {
            name: "toggle",
            detail: None,
            documentation: Some(
                r#"
Mit diesem Attribut lässt sich angeben, wie der TinyMce eingeschaltet werden soll. `true` für einen Toggle Button, False für keinen Toggle-Button, auto für automatisches togglen"#,
            ),
        },
        TagAttribute {
            name: "type",
            detail: None,
            documentation: Some(
                r#"
Der Typ des Eingabefeldes."#,
            ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Setzt einen Default-Wert für die mit name angegebenen Variable, wenn sie leer ist."#,
            ),
        },
    ]),
    attribute_rules: &[
        AttributeRule::Required("name"),
        AttributeRule::OnlyOneOf(&["value", "fixvalue"]),
    ],
};

const SPT_UPDOWN: TagProperties = TagProperties {
    name: "spt:updown",
    detail: None,
    documentation: Some(
        r#"
Zahlenfeld, das per Klick auf- und abwärts gezählt werden kann"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "from",
            detail: None,
            documentation: Some(
                r#"
Startwert des Bereichs (Minimalwert)."#,
            ),
        },
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
        ),
        },
        TagAttribute {
            name: "to",
            detail: None,
            documentation: Some(
                r#"
Endwert des Bereichs (Maximalwert, es folgt `'unendlich'`)."#,
        ),
        },
        TagAttribute {
            name: "value",
            detail: None,
            documentation: Some(
                r#"
Default-Wert (Vorgabewert)."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("name")],
};

const SPT_UPLOAD: TagProperties = TagProperties {
    name: "spt:upload",
    detail: None,
    documentation: Some(
        r#"
Upload von Dateien"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "locale",
            detail: None,
            documentation: Some(
                r#"
Dieses Attribut bestimmt die Mehrsprachigkeit der Variablen."#,
            ),
        },
        TagAttribute {
            name: "name",
            detail: None,
            documentation: Some(
                r#"
Bestimmt den Namen des Feldes."#,
            ),
        },
        TagAttribute {
            name: "previewimage",
            detail: None,
            documentation: Some(
                r#"
`true` um ein Vorschaubild von durch diesen Tag hochgeladenen Bildern anzuzeigen (default), `false` um diese Funktion zu deaktivieren."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("name")],
};

const SPT_WORKLIST: TagProperties = TagProperties {
    name: "spt:worklist",
    detail: None,
    documentation: Some(
        r#"
Workflow Management einbinden"#,
    ),
    // deprecated: true,
    children: TagChildren::None,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "command",
            detail: None,
            documentation: Some(
                r#"
Aktion, die der Worklist-Dialog ausführen soll. Mögliche Aktionen sind:
- `create` Erzeugt einen neuen Worklisteintrag.
- `update` Ändert einen existierenden Worklisteintrag. Der zu ändernde Worklisteintrag wird über die `worklistID` bestimmt."#,
            ),
        },
        TagAttribute {
            name: "informationID",
            detail: None,
            documentation: Some(
                r#"
Artikel, zu dem der Worklisteintrag gehören soll."#,
            ),
        },
        TagAttribute {
            name: "poolID",
            detail: None,
            documentation: Some(
                r#"
Pool des Artikels, zu dem der Worklisteintrag gehören soll."#,
            ),
        },
        TagAttribute {
            name: "worklistID",
            detail: None,
            documentation: Some(
                r#"
Worklisteintrag der geändert werden soll."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("command")],
};

impl FromStr for Tag {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        return match string {
            "argument_tag" => Ok(Tag::SpArgument),
            "attribute_tag" => Ok(Tag::SpAttribute),
            "barcode_tag" => Ok(Tag::SpBarcode),
            "break_tag" => Ok(Tag::SpBreak),
            "calendarsheet_tag" => Ok(Tag::SpCalendarsheet),
            "checkbox_tag" => Ok(Tag::SpCheckbox),
            "code_tag" => Ok(Tag::SpCode),
            "collection_tag" => Ok(Tag::SpCollection),
            "condition_tag" => Ok(Tag::SpCondition),
            "diff_tag" => Ok(Tag::SpDiff),
            "else_tag" => Ok(Tag::SpElse),
            "elseif_tag" => Ok(Tag::SpElseif),
            "error_tag" => Ok(Tag::SpError),
            "expire_tag" => Ok(Tag::SpExpire),
            "filter_tag" => Ok(Tag::SpFilter),
            "for_tag" => Ok(Tag::SpFor),
            "form_tag" => Ok(Tag::SpForm),
            "hidden_tag" => Ok(Tag::SpHidden),
            "if_tag" => Ok(Tag::SpIf),
            "include_tag" => Ok(Tag::SpInclude),
            "io_tag" => Ok(Tag::SpIo),
            "iterator_tag" => Ok(Tag::SpIterator),
            "json_tag" => Ok(Tag::SpJson),
            "linktree_tag" => Ok(Tag::SpLinktree),
            "linkedInformation_tag" => Ok(Tag::SpLinkedInformation),
            "livetree_tag" => Ok(Tag::SpLivetree),
            "log_tag" => Ok(Tag::SpLog),
            "login_tag" => Ok(Tag::SpLogin),
            "loop_tag" => Ok(Tag::SpLoop),
            "map_tag" => Ok(Tag::SpMap),
            "option_tag" => Ok(Tag::SpOption),
            "password_tag" => Ok(Tag::SpPassword),
            "print_tag" => Ok(Tag::SpPrint),
            "querytree_tag" => Ok(Tag::SpQuerytree),
            "radio_tag" => Ok(Tag::SpRadio),
            "range_tag" => Ok(Tag::SpRange),
            "return_tag" => Ok(Tag::SpReturn),
            "sass_tag" => Ok(Tag::SpSass),
            "scaleimage_tag" => Ok(Tag::SpScaleimage),
            "scope_tag" => Ok(Tag::SpScope),
            "search_tag" => Ok(Tag::SpSearch),
            "select_tag" => Ok(Tag::SpSelect),
            "set_tag" => Ok(Tag::SpSet),
            "sort_tag" => Ok(Tag::SpSort),
            "subinformation_tag" => Ok(Tag::SpSubinformation),
            "tagbody_tag" => Ok(Tag::SpTagbody),
            "text_tag" => Ok(Tag::SpText),
            "textarea_tag" => Ok(Tag::SpTextarea),
            "textimage_tag" => Ok(Tag::SpTextimage),
            "throw_tag" => Ok(Tag::SpThrow),
            "toggle_tag" => Ok(Tag::SpToggle),
            "upload_tag" => Ok(Tag::SpUpload),
            "url_tag" => Ok(Tag::SpUrl),
            "warning_tag" => Ok(Tag::SpWarning),
            "worklist_tag" => Ok(Tag::SpWorklist),
            "zip_tag" => Ok(Tag::SpZip),
            "spt_counter_tag" => Ok(Tag::SptCounter),
            "spt_date_tag" => Ok(Tag::SptDate),
            "spt_diff_tag" => Ok(Tag::SptDiff),
            "spt_email2img_tag" => Ok(Tag::SptEmail2img),
            "spt_encryptemail_tag" => Ok(Tag::SptEncryptemail),
            "spt_escapeemail_tag" => Ok(Tag::SptEscapeemail),
            "spt_formsolutions_tag" => Ok(Tag::SptFormsolutions),
            "spt_id2url_tag" => Ok(Tag::SptId2url),
            "spt_ilink_tag" => Ok(Tag::SptIlink),
            "spt_imageeditor_tag" => Ok(Tag::SptImageeditor),
            "spt_imp_tag" => Ok(Tag::SptImp),
            "spt_iterator_tag" => Ok(Tag::SptIterator),
            "spt_link_tag" => Ok(Tag::SptLink),
            "spt_number_tag" => Ok(Tag::SptNumber),
            "spt_personalization_tag" => Ok(Tag::SptPersonalization),
            "spt_prehtml_tag" => Ok(Tag::SptPrehtml),
            "spt_smarteditor_tag" => Ok(Tag::SptSmarteditor),
            "spt_spml_tag" => Ok(Tag::SptSpml),
            "spt_text_tag" => Ok(Tag::SptText),
            "spt_textarea_tag" => Ok(Tag::SptTextarea),
            "spt_timestamp_tag" => Ok(Tag::SptTimestamp),
            "spt_tinymce_tag" => Ok(Tag::SptTinymce),
            "spt_updown_tag" => Ok(Tag::SptUpdown),
            "spt_upload_tag" => Ok(Tag::SptUpload),
            "spt_worklist_tag" => Ok(Tag::SptWorklist),
            tag => Err(anyhow::anyhow!("not a valid tag: \"{}\"", tag)),
        };
    }
}

impl Tag {
    pub fn properties(&self) -> TagProperties {
        return match self {
            Tag::SpArgument => SP_ARGUMENT,
            Tag::SpAttribute => SP_ATTRIBUTE,
            Tag::SpBarcode => SP_BARCODE,
            Tag::SpBreak => SP_BREAK,
            Tag::SpCalendarsheet => SP_CALENDARSHEET,
            Tag::SpCheckbox => SP_CHECKBOX,
            Tag::SpCode => SP_CODE,
            Tag::SpCollection => SP_COLLECTION,
            Tag::SpCondition => SP_CONDITION,
            Tag::SpDiff => SP_DIFF,
            Tag::SpElse => SP_ELSE,
            Tag::SpElseif => SP_ELSEIF,
            Tag::SpError => SP_ERROR,
            Tag::SpExpire => SP_EXPIRE,
            Tag::SpFilter => SP_FILTER,
            Tag::SpFor => SP_FOR,
            Tag::SpForm => SP_FORM,
            Tag::SpHidden => SP_HIDDEN,
            Tag::SpIf => SP_IF,
            Tag::SpInclude => SP_INCLUDE,
            Tag::SpIo => SP_IO,
            Tag::SpIterator => SP_ITERATOR,
            Tag::SpJson => SP_JSON,
            Tag::SpLinktree => SP_LINKTREE,
            Tag::SpLinkedInformation => SP_LINKEDINFORMATION,
            Tag::SpLivetree => SP_LIVETREE,
            Tag::SpLog => SP_LOG,
            Tag::SpLogin => SP_LOGIN,
            Tag::SpLoop => SP_LOOP,
            Tag::SpMap => SP_MAP,
            Tag::SpOption => SP_OPTION,
            Tag::SpPassword => SP_PASSWORD,
            Tag::SpPrint => SP_PRINT,
            Tag::SpQuerytree => SP_QUERYTREE,
            Tag::SpRadio => SP_RADIO,
            Tag::SpRange => SP_RANGE,
            Tag::SpReturn => SP_RETURN,
            Tag::SpSass => SP_SASS,
            Tag::SpScaleimage => SP_SCALEIMAGE,
            Tag::SpScope => SP_SCOPE,
            Tag::SpSearch => SP_SEARCH,
            Tag::SpSelect => SP_SELECT,
            Tag::SpSet => SP_SET,
            Tag::SpSort => SP_SORT,
            Tag::SpSubinformation => SP_SUBINFORMATION,
            Tag::SpTagbody => SP_TAGBODY,
            Tag::SpText => SP_TEXT,
            Tag::SpTextarea => SP_TEXTAREA,
            Tag::SpTextimage => SP_TEXTIMAGE,
            Tag::SpThrow => SP_THROW,
            Tag::SpToggle => SP_TOGGLE,
            Tag::SpUpload => SP_UPLOAD,
            Tag::SpUrl => SP_URL,
            Tag::SpWarning => SP_WARNING,
            Tag::SpWorklist => SP_WORKLIST,
            Tag::SpZip => SP_ZIP,
            Tag::SptCounter => SPT_COUNTER,
            Tag::SptDate => SPT_DATE,
            Tag::SptDiff => SPT_DIFF,
            Tag::SptEmail2img => SPT_EMAIL2IMG,
            Tag::SptEncryptemail => SPT_ENCRYPTEMAIL,
            Tag::SptEscapeemail => SPT_ESCAPEEMAIL,
            Tag::SptFormsolutions => SPT_FORMSOLUTIONS,
            Tag::SptId2url => SPT_ID2URL,
            Tag::SptIlink => SPT_ILINK,
            Tag::SptImageeditor => SPT_IMAGEEDITOR,
            Tag::SptImp => SPT_IMP,
            Tag::SptIterator => SPT_ITERATOR,
            Tag::SptLink => SPT_LINK,
            Tag::SptNumber => SPT_NUMBER,
            Tag::SptPersonalization => SPT_PERSONALIZATION,
            Tag::SptPrehtml => SPT_PREHTML,
            Tag::SptSmarteditor => SPT_SMARTEDITOR,
            Tag::SptSpml => SPT_SPML,
            Tag::SptText => SPT_TEXT,
            Tag::SptTextarea => SPT_TEXTAREA,
            Tag::SptTimestamp => SPT_TIMESTAMP,
            Tag::SptTinymce => SPT_TINYMCE,
            Tag::SptUpdown => SPT_UPDOWN,
            Tag::SptUpload => SPT_UPLOAD,
            Tag::SptWorklist => SPT_WORKLIST,
        };
    }

    pub fn iter() -> Iter<'static, Tag> {
        static TAGS: [Tag; 81] = [
            Tag::SpArgument,
            Tag::SpAttribute,
            Tag::SpBarcode,
            Tag::SpBreak,
            Tag::SpCalendarsheet,
            Tag::SpCheckbox,
            Tag::SpCode,
            Tag::SpCollection,
            Tag::SpCondition,
            Tag::SpDiff,
            Tag::SpElse,
            Tag::SpElseif,
            Tag::SpError,
            Tag::SpExpire,
            Tag::SpFilter,
            Tag::SpFor,
            Tag::SpForm,
            Tag::SpHidden,
            Tag::SpIf,
            Tag::SpInclude,
            Tag::SpIo,
            Tag::SpIterator,
            Tag::SpJson,
            Tag::SpLinktree,
            Tag::SpLinkedInformation,
            Tag::SpLivetree,
            Tag::SpLog,
            Tag::SpLogin,
            Tag::SpLoop,
            Tag::SpMap,
            Tag::SpOption,
            Tag::SpPassword,
            Tag::SpPrint,
            Tag::SpQuerytree,
            Tag::SpRadio,
            Tag::SpRange,
            Tag::SpReturn,
            Tag::SpSass,
            Tag::SpScaleimage,
            Tag::SpScope,
            Tag::SpSearch,
            Tag::SpSelect,
            Tag::SpSet,
            Tag::SpSort,
            Tag::SpSubinformation,
            Tag::SpTagbody,
            Tag::SpText,
            Tag::SpTextarea,
            Tag::SpTextimage,
            Tag::SpThrow,
            Tag::SpToggle,
            Tag::SpUpload,
            Tag::SpUrl,
            Tag::SpWarning,
            Tag::SpWorklist,
            Tag::SpZip,
            Tag::SptCounter,
            Tag::SptDate,
            Tag::SptDiff,
            Tag::SptEmail2img,
            Tag::SptEncryptemail,
            Tag::SptEscapeemail,
            Tag::SptFormsolutions,
            Tag::SptId2url,
            Tag::SptIlink,
            Tag::SptImageeditor,
            Tag::SptImp,
            Tag::SptIterator,
            Tag::SptLink,
            Tag::SptNumber,
            Tag::SptPersonalization,
            Tag::SptPrehtml,
            Tag::SptSmarteditor,
            Tag::SptSpml,
            Tag::SptText,
            Tag::SptTextarea,
            Tag::SptTimestamp,
            Tag::SptTinymce,
            Tag::SptUpdown,
            Tag::SptUpload,
            Tag::SptWorklist,
        ];
        return TAGS.iter();
    }
}