use std::{borrow::Cow, str::FromStr};

pub(crate) struct Function {
    pub(crate) name: Cow<'static, str>,
    pub(crate) argument_number: ArgumentNumber,
    pub(crate) documentation: Cow<'static, str>,
}

impl Function {
    pub(crate) const CEIL: Function = Function::new(
        "ceil",
        ArgumentNumber::Exactly(1),
        "Rundet eine Zahl zum nächsten Integer auf",
    );
    pub(crate) const COALESCE: Function = Function::new(
        "coalesce",
        ArgumentNumber::AtLeast(1),
        concat!(
            "Gibt das erste Object zurück welches nicht zu `null` oder Leerstring evaluiert ",
            "werden kann"
        ),
    );
    pub(crate) const COLLECTION: Function = Function::new(
        "collection",
        ArgumentNumber::Any,
        "Liefert ein `Collection`-Objekt welches alle übergebenen Werte beinhaltet",
    );
    pub(crate) const COLOR: Function = Function::new(
        "color",
        ArgumentNumber::Exactly(1),
        "Liefert ein `Color`-Objekt anhand eines RGB-Hex-Wertes (\"#12ab34\")",
    );
    pub(crate) const EVAL_CONDITION: Function = Function::new(
        "evalCondition",
        ArgumentNumber::Exactly(1),
        "Evaluiert den Ausdruck und liefert das Ergebnis zurück.",
    );
    pub(crate) const EVAL_EXPRESSION: Function = Function::new(
        "evalExpression",
        ArgumentNumber::Exactly(1),
        "Evaluiert den Ausdruck und liefert das Ergebnis zurück.",
    );
    pub(crate) const EVAL_TEXT: Function = Function::new(
        "evalText",
        ArgumentNumber::Exactly(1),
        "Evaluiert den Text und liefert das Ergebnis zurück.",
    );
    pub(crate) const FLOOR: Function = Function::new(
        "floor",
        ArgumentNumber::Exactly(1),
        "Rundet eine Zahl zum nächsten Integer ab",
    );
    pub(crate) const IS_EMAIL: Function = Function::new(
        "isEmail",
        ArgumentNumber::Exactly(1),
        "Liefert `true`, wenn das Objekt einer gültigen E-Mail-Adresse entspricht.",
    );
    pub(crate) const IS_LIST: Function = Function::new(
        "isList",
        ArgumentNumber::Exactly(1),
        "Liefert `true`, wenn das übergebene Objekte eine Liste ist",
    );
    pub(crate) const IS_MAP: Function = Function::new(
        "isMap",
        ArgumentNumber::Exactly(1),
        "Liefert `true`, wenn das übergebene Objekte eine Map ist",
    );
    pub(crate) const IS_NULL: Function = Function::new(
        "isNull",
        ArgumentNumber::Exactly(1),
        concat!(
            "Liefert `true`, wenn das Objekt nicht existiert eine leere Zeichenkette oder ein ",
            "leeres Artikelfeld ist. Die Abfrage, ob ein Objekt NICHT existiert kann durch ein ",
            "vorangestellte \"!\" erreicht werden."
        ),
    );
    pub(crate) const IS_NUMBER: Function = Function::new(
        "isNumber",
        ArgumentNumber::Exactly(1),
        concat!(
            "Liefert `true`, wenn das Objekt ein numerischer Wert ist. Die Abfrage, ob ein Objekt ",
            "KEINE Zahl ist, kann durch ein vorangestellte \"!\" erreicht werden."
        ),
    );
    pub(crate) const RANDOM_UUID: Function =
        Function::new("randomUUID", ArgumentNumber::None, "Erzeugt eine UUID");
    pub(crate) const ROUND: Function = Function::new(
        "floor",
        ArgumentNumber::Exactly(1),
        "Rundet eine Zahl zum nächsten Integer",
    );
    pub(crate) const TRANSLATABLE: Function = Function::new(
        "translatable",
        ArgumentNumber::Exactly(1),
        "Markiert einen Text als übersetzbar",
    );

    const fn new(
        name: &'static str,
        argument_number: ArgumentNumber,
        documentation: &'static str,
    ) -> Self {
        return Function {
            name: Cow::Borrowed(name),
            argument_number,
            documentation: Cow::Borrowed(documentation),
        };
    }
}

impl FromStr for Function {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        return match string {
            "ceil" => Ok(Function::CEIL),
            "coalesce" => Ok(Function::COALESCE),
            "collection" => Ok(Function::COLLECTION),
            "color" => Ok(Function::COLOR),
            "evalCondition" => Ok(Function::EVAL_CONDITION),
            "evalExpression" => Ok(Function::EVAL_EXPRESSION),
            "evalText" => Ok(Function::EVAL_TEXT),
            "floor" => Ok(Function::FLOOR),
            "isEmail" => Ok(Function::IS_EMAIL),
            "isList" => Ok(Function::IS_LIST),
            "isMap" => Ok(Function::IS_MAP),
            "isNull" => Ok(Function::IS_NULL),
            "isNumber" => Ok(Function::IS_NUMBER),
            "randomUUID" => Ok(Function::RANDOM_UUID),
            "round" => Ok(Function::ROUND),
            "translatable" => Ok(Function::TRANSLATABLE),
            name => Err(anyhow::anyhow!("unknown function \"{}\"", name)),
        };
    }
}

pub(crate) enum ArgumentNumber {
    Any,
    AtLeast(usize),
    Exactly(usize),
    None,
}
