use std::str::FromStr;

pub struct FunctionDefinition {
    pub name: &'static str,
    pub argument_number: ArgumentNumber,
    pub documentation: &'static str,
}

impl FunctionDefinition {
    pub(crate) const CEIL: FunctionDefinition = FunctionDefinition::new(
        "ceil",
        ArgumentNumber::Exactly(1),
        "Rundet eine Zahl zum nächsten Integer auf",
    );
    pub(crate) const COALESCE: FunctionDefinition = FunctionDefinition::new(
        "coalesce",
        ArgumentNumber::AtLeast(1),
        concat!(
            "Gibt das erste Object zurück welches nicht zu `null` oder Leerstring evaluiert ",
            "werden kann"
        ),
    );
    pub(crate) const COLLECTION: FunctionDefinition = FunctionDefinition::new(
        "collection",
        ArgumentNumber::Any,
        "Liefert ein `Collection`-Objekt welches alle übergebenen Werte beinhaltet",
    );
    pub(crate) const COLOR: FunctionDefinition = FunctionDefinition::new(
        "color",
        ArgumentNumber::Exactly(1),
        "Liefert ein `Color`-Objekt anhand eines RGB-Hex-Wertes (\"#12ab34\")",
    );
    pub(crate) const EVAL_CONDITION: FunctionDefinition = FunctionDefinition::new(
        "evalCondition",
        ArgumentNumber::Exactly(1),
        "Evaluiert den Ausdruck und liefert das Ergebnis zurück.",
    );
    pub(crate) const EVAL_EXPRESSION: FunctionDefinition = FunctionDefinition::new(
        "evalExpression",
        ArgumentNumber::Exactly(1),
        "Evaluiert den Ausdruck und liefert das Ergebnis zurück.",
    );
    pub(crate) const EVAL_TEXT: FunctionDefinition = FunctionDefinition::new(
        "evalText",
        ArgumentNumber::Exactly(1),
        "Evaluiert den Text und liefert das Ergebnis zurück.",
    );
    pub(crate) const FLOOR: FunctionDefinition = FunctionDefinition::new(
        "floor",
        ArgumentNumber::Exactly(1),
        "Rundet eine Zahl zum nächsten Integer ab",
    );
    pub(crate) const IS_EMAIL: FunctionDefinition = FunctionDefinition::new(
        "isEmail",
        ArgumentNumber::Exactly(1),
        "Liefert `true`, wenn das Objekt einer gültigen E-Mail-Adresse entspricht.",
    );
    pub(crate) const IS_LIST: FunctionDefinition = FunctionDefinition::new(
        "isList",
        ArgumentNumber::Exactly(1),
        "Liefert `true`, wenn das übergebene Objekte eine Liste ist",
    );
    pub(crate) const IS_MAP: FunctionDefinition = FunctionDefinition::new(
        "isMap",
        ArgumentNumber::Exactly(1),
        "Liefert `true`, wenn das übergebene Objekte eine Map ist",
    );
    pub(crate) const IS_NULL: FunctionDefinition = FunctionDefinition::new(
        "isNull",
        ArgumentNumber::Exactly(1),
        concat!(
            "Liefert `true`, wenn das Objekt nicht existiert eine leere Zeichenkette oder ein ",
            "leeres Artikelfeld ist. Die Abfrage, ob ein Objekt NICHT existiert kann durch ein ",
            "vorangestellte \"!\" erreicht werden."
        ),
    );
    pub(crate) const IS_NUMBER: FunctionDefinition = FunctionDefinition::new(
        "isNumber",
        ArgumentNumber::Exactly(1),
        concat!(
            "Liefert `true`, wenn das Objekt ein numerischer Wert ist. Die Abfrage, ob ein Objekt ",
            "KEINE Zahl ist, kann durch ein vorangestellte \"!\" erreicht werden."
        ),
    );
    pub(crate) const RANDOM_UUID: FunctionDefinition =
        FunctionDefinition::new("randomUUID", ArgumentNumber::None, "Erzeugt eine UUID");
    pub(crate) const ROUND: FunctionDefinition = FunctionDefinition::new(
        "floor",
        ArgumentNumber::Exactly(1),
        "Rundet eine Zahl zum nächsten Integer",
    );
    pub(crate) const TRANSLATABLE: FunctionDefinition = FunctionDefinition::new(
        "translatable",
        ArgumentNumber::Exactly(1),
        "Markiert einen Text als übersetzbar",
    );

    const fn new(
        name: &'static str,
        argument_number: ArgumentNumber,
        documentation: &'static str,
    ) -> Self {
        return FunctionDefinition {
            name,
            argument_number,
            documentation,
        };
    }
}

impl FromStr for FunctionDefinition {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        return match string {
            "ceil" => Ok(FunctionDefinition::CEIL),
            "coalesce" => Ok(FunctionDefinition::COALESCE),
            "collection" => Ok(FunctionDefinition::COLLECTION),
            "color" => Ok(FunctionDefinition::COLOR),
            "evalCondition" => Ok(FunctionDefinition::EVAL_CONDITION),
            "evalExpression" => Ok(FunctionDefinition::EVAL_EXPRESSION),
            "evalText" => Ok(FunctionDefinition::EVAL_TEXT),
            "floor" => Ok(FunctionDefinition::FLOOR),
            "isEmail" => Ok(FunctionDefinition::IS_EMAIL),
            "isList" => Ok(FunctionDefinition::IS_LIST),
            "isMap" => Ok(FunctionDefinition::IS_MAP),
            "isNull" => Ok(FunctionDefinition::IS_NULL),
            "isNumber" => Ok(FunctionDefinition::IS_NUMBER),
            "randomUUID" => Ok(FunctionDefinition::RANDOM_UUID),
            "round" => Ok(FunctionDefinition::ROUND),
            "translatable" => Ok(FunctionDefinition::TRANSLATABLE),
            name => Err(anyhow::anyhow!("unknown function \"{}\"", name)),
        };
    }
}

pub enum ArgumentNumber {
    Any,
    AtLeast(usize),
    Exactly(usize),
    None,
}
