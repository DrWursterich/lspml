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
    OnlyDynamic,
    These(&'static [TagAttribute]),
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
Die Bedingung ist erfüllt, wenn die Variable in `name` kleiner als in der Variable in `lte` ist."#
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
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "code",
            detail: None,
            documentation: Some(
                r#"
Zu prüfender Error-Code."#,
            ),
        },
    ]),
    attribute_rules: &[AttributeRule::Required("code")],
};

const SP_EXPIRE: TagProperties = TagProperties {
    name: "sp:expire",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::These(&[
        TagAttribute {
            name: "date",
            detail: None,
            documentation: Some(
                r#"
Long-Wert mit dem Unix-Timestamp des gewünschten Datums"#,
            ),
        },
    ]),
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
    attributes: TagAttributes::None,
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
Die Bedingung ist erfüllt, wenn die Variable in `name` kleiner als in der Variable in `lte` ist."#
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
    attribute_rules: &[AttributeRule::Required("collection")],
};

const SP_JSON: TagProperties = TagProperties {
    name: "sp:json",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
    attribute_rules: &[AttributeRule::Required("level")],
};

const SP_LOGIN: TagProperties = TagProperties {
    name: "sp:login",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
    attribute_rules: &[AttributeRule::Required("name")],
};

const SP_RANGE: TagProperties = TagProperties {
    name: "sp:range",
    detail: None,
    documentation: None,
    children: TagChildren::Any,
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
    attribute_rules: &[AttributeRule::Required("name")],
};

const SPT_DATE: TagProperties = TagProperties {
    name: "spt:date",
    detail: None,
    documentation: Some(
        r#"
Datums- und Uhrzeiteingabe mit Prüfung auf Gültigkeit"#,
    ),
    children: TagChildren::None,
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
    attributes: TagAttributes::None,
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
