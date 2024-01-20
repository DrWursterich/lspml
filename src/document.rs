use anyhow::Error;
use lsp_types::{DiagnosticSeverity, Range};
use std::str::FromStr;

// #[derive(Debug)]
pub(crate) struct Document {
    top_level_nodes: Vec<TopLevelNode>,
    issues: Option<Vec<Issue>>,
}

impl FromStr for Document {
    type Err = Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_spml::language())
            .map_err(|err| {
                anyhow::anyhow!("failed to set tree sitter language to spml: {}", err)
            })?;
        let tree = parser
            .parse(&string, None)
            .ok_or_else(|| anyhow::anyhow!("tree sitter parser failed"))?;
        tree.root_node()
            .children(&mut tree.root_node().walk())
            .map(|node| match node.kind() {
                "comment" => Ok(TopLevelNode::Comment(
                    node.utf8_text(string.as_bytes()).unwrap().to_string(),
                )),
                "include_tag" => {
                    // TODO: probably have to either:
                    //     - name attributes via treesitter
                    //     - have one variable for each attribute
                    let locale = None;
                    let lookup = None;
                    let mode = None;
                    let r#return = None;
                    let target = None;
                    let arguments = None;
                    for node in node.children(&mut node.walk()) {
                        match node.kind() {
                            "include_tag_open" | "include_tag_close" => {}
                            "module_attribute" => target = IncludeTarget::Uri,
                            "uri_attribute" => {}
                            "return_attribute" => {}
                            "argument_tag" => {}
                            kind => {
                                // TODO: mark as issue
                                return Result::Err(anyhow::anyhow!(""));
                            }
                        }
                    }
                    return Ok(TopLevelNode::Include(Include {
                        locale,
                        lookup,
                        mode,
                        r#return,
                        target,
                        arguments,
                    }));
                }
                "text" | _ => Ok(TopLevelNode::Text(
                    node.utf8_text(string.as_bytes()).unwrap().to_string(),
                )),
            });
        return Result::Err(anyhow::anyhow!("asdf"));
    }
}

enum TopLevelNode {
    Include(Include),
    Text(String),
    Comment(String),
}

struct Argument {
    name: String,
    value: ArgumentValue,
    locale: Option<String>,
}

enum ArgumentValue {
    Condition(String),
    Expression(String),
    Object(ArgumentObjectValue),
    Text(ArgumentTextValue),
}

struct ArgumentObjectValue {
    object: String,
    default: Option<String>,
}

struct ArgumentTextValue {
    text: String,
    default: Option<String>,
}

type DefaultValue = String;

struct Include {
    locale: Option<String>,
    lookup: Option<String>,
    mode: Option<IncludeMode>,
    r#return: Option<String>,
    target: IncludeTarget,
    arguments: Option<Arguments>,
}

enum IncludeMode {
    In,
    Out,
}

struct Arguments {
    map: Option<String>,
    values: Vec<Argument>,
}

enum IncludeTarget {
    Template(String),
    Anchor(String),
    Uri(IncludeTargetUri),
}

struct IncludeTargetUri {
    uri: String,
    module: Option<InlcudeTargetUriModule>,
}

enum InlcudeTargetUriModule {
    Context(String),
    Module(String),
}

struct Issue {
    pub range: Range,
    pub severity: Option<DiagnosticSeverity>,
    pub message: String,
}
