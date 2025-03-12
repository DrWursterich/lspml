use std::fmt::{Display, Formatter};

use lsp_types::{
    CodeActionKind, CodeActionOptions, CodeActionProviderCapability, CompletionOptions,
    CompletionOptionsCompletionItem, DiagnosticOptions, DiagnosticServerCapabilities, HoverOptions,
    HoverProviderCapability, NumberOrString, OneOf, SemanticTokenModifier, SemanticTokenType,
    SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensServerCapabilities, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, WorkDoneProgressOptions,
};

pub const TOKEN_TYPES: &'static [SemanticTokenType] = &[
    SemanticTokenType::ENUM,
    SemanticTokenType::ENUM_MEMBER,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::MACRO,
    SemanticTokenType::METHOD,
    SemanticTokenType::NUMBER,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::PROPERTY,
    SemanticTokenType::REGEXP,
    SemanticTokenType::STRING,
    SemanticTokenType::VARIABLE,
];

pub const TOKEN_MODIFIERS: &'static [SemanticTokenModifier] = &[
    SemanticTokenModifier::DECLARATION,
    SemanticTokenModifier::DEFINITION,
    SemanticTokenModifier::DEPRECATED,
    SemanticTokenModifier::DOCUMENTATION,
    SemanticTokenModifier::MODIFICATION,
];

pub enum CodeActionImplementation {
    GenerateDefaultHeaders,
    NameToCondition,
    ConditionToName,
    FixSpelSyntax,
    RemoveSuperfluousCode,
    AddMissingCode,
}

impl CodeActionImplementation {
    pub const GENERATE_DEFAULT_HEADER_CODE: NumberOrString = NumberOrString::Number(7126);
    pub const FIX_SPEL_SYNTAX_CODE: NumberOrString = NumberOrString::Number(7127);
    pub const REMOVE_SUPERFLUOUS_CODE: NumberOrString = NumberOrString::Number(7128);
    pub const ADD_MISSING_CODE: NumberOrString = NumberOrString::Number(7129);

    pub(crate) fn kinds() -> Vec<CodeActionKind> {
        return vec![
            CodeActionImplementation::GenerateDefaultHeaders.to_kind(),
            CodeActionImplementation::NameToCondition.to_kind(),
            CodeActionImplementation::ConditionToName.to_kind(),
            CodeActionImplementation::FixSpelSyntax.to_kind(),
            CodeActionImplementation::RemoveSuperfluousCode.to_kind(),
            CodeActionImplementation::AddMissingCode.to_kind(),
            CodeActionKind::SOURCE_FIX_ALL,
        ];
    }

    pub fn to_kind(&self) -> CodeActionKind {
        return CodeActionKind::new(match self {
            CodeActionImplementation::GenerateDefaultHeaders => "refactor.generate_default_headers",
            CodeActionImplementation::NameToCondition => "refactor.name_to_condition",
            CodeActionImplementation::ConditionToName => "refactor.condition_to_name",
            CodeActionImplementation::FixSpelSyntax => "quickfix.fix_spel_syntax",
            CodeActionImplementation::RemoveSuperfluousCode => "quickfix.remove_superfluous_code",
            CodeActionImplementation::AddMissingCode => "quickfix.add_missing_code",
        });
    }
}

impl Display for CodeActionImplementation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            CodeActionImplementation::GenerateDefaultHeaders => "refactor.generate_default_headers",
            CodeActionImplementation::NameToCondition => "refactor.name_to_condition",
            CodeActionImplementation::ConditionToName => "refactor.condition_to_name",
            CodeActionImplementation::FixSpelSyntax => "quickfix.fix_spel_syntax",
            CodeActionImplementation::RemoveSuperfluousCode => "quickfix.remove_superfluous_code",
            CodeActionImplementation::AddMissingCode => "quickfix.add_missing_code",
        })
    }
}

pub fn create() -> ServerCapabilities {
    return ServerCapabilities {
        definition_provider: Some(OneOf::Left(true)),
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        document_highlight_provider: Some(OneOf::Left(true)),
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
            SemanticTokensOptions {
                full: Some(SemanticTokensFullOptions::Bool(true)),
                legend: SemanticTokensLegend {
                    token_types: TOKEN_TYPES.to_vec(),
                    token_modifiers: TOKEN_MODIFIERS.to_vec(),
                },
                ..Default::default()
            },
        )),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            inter_file_dependencies: true,
            ..DiagnosticOptions::default()
        })),
        completion_provider: Some(CompletionOptions {
            completion_item: Some(CompletionOptionsCompletionItem {
                label_details_support: Some(true),
            }),
            ..CompletionOptions::default()
        }),
        hover_provider: Some(HoverProviderCapability::Options(HoverOptions {
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: Some(true),
            },
        })),
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(CodeActionImplementation::kinds()),
            ..CodeActionOptions::default()
        })),
        ..ServerCapabilities::default()
    };
}
