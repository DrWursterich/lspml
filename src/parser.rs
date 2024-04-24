use anyhow::Result;
use lsp_types::Position;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Tree {
    header: Header,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Header {
    java_header: JavaHeader,
    taglib_imports: Vec<TagLibImport>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct JavaHeader {
    open_bracket: Location,
    page: Location,
    language: Attribute,
    page_encoding: Attribute,
    content_type: Attribute,
    // TODO: java_class_imports?
    close_bracket: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TagLibImport {
    open_bracket: Location,
    taglib: Location,
    origin: TagLibOrigin,
    prefix: Attribute,
    close_bracket: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TagLibOrigin {
    Uri(Attribute),
    TagDir(Attribute),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Attribute {
    key_location: Location,
    equals_location: Location,
    opening_quote_location: Location,
    value: String,
    closing_quote_location: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Location {
    char: usize,
    line: usize,
    length: usize,
}

impl Location {
    pub(crate) fn new(char: usize, line: usize, length: usize) -> Self {
        return Location { char, line, length };
    }
}

struct TreeParser<'a, 'b> {
    cursor: tree_sitter::TreeCursor<'b>,
    text_bytes: &'a [u8],
}

impl TreeParser<'_, '_> {
    pub(crate) fn new<'a, 'b>(
        cursor: tree_sitter::TreeCursor<'b>,
        text: &'a String,
    ) -> TreeParser<'a, 'b> {
        return TreeParser {
            cursor,
            text_bytes: text.as_bytes(),
        };
    }

    pub(crate) fn parse_header(&mut self) -> Result<Header> {
        println!("parsing header");
        let root = self.cursor.node();
        if root.kind() != "document" {
            return Err(anyhow::anyhow!(
                "missplaced cursor. the header should be the first thing that a TreeParser parses"
            ));
        }
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("document is empty"));
        }
        let mut java_header = None;
        let mut taglib_imports = Vec::new();
        loop {
            let header_node = self.cursor.node();
            println!("header: {}", header_node.kind());
            match header_node.kind() {
                "page_header" => match java_header {
                    Some(_) => return Err(anyhow::anyhow!("found multiple java headers")),
                    None => java_header = Some(self.parse_java_header()?),
                },
                "taglib_header" => taglib_imports.push(self.parse_taglib_header()?),
                _ => break,
            }
            if !self.cursor.goto_next_sibling() {
                // document contains nothing but the header
                break;
            }
        }
        println!("{:#?}", taglib_imports);
        return match java_header {
            Some(java_header) => Ok(Header {
                java_header,
                taglib_imports,
            }),
            None => Err(anyhow::anyhow!("document has no java header")),
        };
    }

    fn parse_java_header(&mut self) -> Result<JavaHeader> {
        println!("parse java header");
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("java header is empty"));
        }
        let open_bracket = node_location(self.cursor.node());
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"page\" keyword"
            ));
        }
        let page = node_location(self.cursor.node());
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"language\" attribute"
            ));
        }
        // TODO: attribute order should not matter
        let (_, language) = self.parse_attribute()?;
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"pageEncoding\" attribute"
            ));
        }
        let (_, page_encoding) = self.parse_attribute()?;
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"contentType\" attribute"
            ));
        }
        let (_, content_type) = self.parse_attribute()?;
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"page\" keyword"
            ));
        }
        let close_bracket = node_location(self.cursor.node());
        self.cursor.goto_parent();
        return Ok(JavaHeader {
            open_bracket,
            page,
            language,
            page_encoding,
            content_type,
            close_bracket,
        });
    }

    fn parse_taglib_header(&mut self) -> Result<TagLibImport> {
        println!("parse taglib header");
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("java header is empty"));
        }
        let open_bracket = node_location(self.cursor.node());
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"page\" keyword"
            ));
        }
        let taglib = node_location(self.cursor.node());
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"language\" attribute"
            ));
        }
        // TODO: attribute order should not matter
        let (name, attribute) = self.parse_attribute()?;
        let origin = match name.as_str() {
            "uri" => TagLibOrigin::Uri(attribute),
            "tagdir" => TagLibOrigin::TagDir(attribute),
            name => {
                return Err(anyhow::anyhow!(
                    "unexpected \"{}\" attribute in taglib header",
                    name
                ))
            }
        };
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"pageEncoding\" attribute"
            ));
        }
        let (_, prefix) = self.parse_attribute()?;
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!(
                "java header is missing the \"page\" keyword"
            ));
        }
        let close_bracket = node_location(self.cursor.node());
        self.cursor.goto_parent();
        return Ok(TagLibImport {
            open_bracket,
            taglib,
            origin,
            prefix,
            close_bracket,
        });
    }

    fn parse_attribute(&mut self) -> Result<(String, Attribute)> {
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("attribute is empty"));
        }
        let key_node = self.cursor.node();
        let key_location = node_location(key_node);
        let key = key_node.kind().to_string();
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!("attribute is missing a value"));
        }
        let equals_location = node_location(self.cursor.node());
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!("attribute is missing a value"));
        }
        let _string_node = self.cursor.node();
        if !self.cursor.goto_first_child() {
            return Err(anyhow::anyhow!("attribute value is empty"));
        }
        let opening_quote_location = node_location(self.cursor.node());
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!("attribute value string is unclosed"));
        }
        let value = self.cursor.node().utf8_text(self.text_bytes)?.to_string();
        if !self.cursor.goto_next_sibling() {
            return Err(anyhow::anyhow!("attribute value string is unclosed"));
        }
        let closing_quote_location = node_location(self.cursor.node());
        self.cursor.goto_parent();
        self.cursor.goto_parent();
        let attribute = Attribute {
            key_location,
            equals_location,
            opening_quote_location,
            value,
            closing_quote_location,
        };
        return Ok((key, attribute));
    }
}

fn node_location(node: tree_sitter::Node) -> Location {
    let start = node.start_position();
    return Location {
        char: start.column,
        line: start.row,
        length: node.end_position().column - start.column,
    };
}

impl Tree {
    pub(crate) fn new(ts: tree_sitter::Tree, text: String) -> Result<Self> {
        let parser = &mut TreeParser::new(ts.walk(), &text);
        let header = parser.parse_header()?;
        return Ok(Tree { header });
    }
}

// =============================================================================================
// || OLD STUFF || OLD STUFF || OLD STUFF || OLD STUFF || OLD STUFF || OLD STUFF || OLD STUFF ||
// =============================================================================================

pub(crate) fn find_current_node<'tree>(
    tree: &'tree tree_sitter::Tree,
    position: Position,
) -> Option<tree_sitter::Node<'tree>> {
    let trigger_point =
        tree_sitter::Point::new(position.line as usize, position.character as usize);
    let mut cursor = tree.root_node().walk();
    loop {
        let node = cursor.node();
        if match node.end_position() <= trigger_point {
            true => !cursor.goto_next_sibling() || cursor.node().start_position() > trigger_point,
            false => !cursor.goto_first_child(),
        } {
            log::debug!("current node: {:?}", node);
            return Some(node);
        }
    }
}

pub(crate) fn attribute_name_of<'a>(
    attribute: tree_sitter::Node<'_>,
    source: &'a str,
) -> Option<&'a str> {
    return attribute
        .child(0)
        .and_then(|node| node.utf8_text(source.as_bytes()).ok());
}

pub(crate) fn attribute_value_of<'a>(
    attribute: tree_sitter::Node<'_>,
    source: &'a str,
) -> Option<&'a str> {
    return attribute
        .child(2)
        .and_then(|node| node.child(1))
        .and_then(|node| node.utf8_text(source.as_bytes()).ok());
}

pub(crate) fn attribute_name_and_value_of<'a>(
    attribute: tree_sitter::Node<'_>,
    source: &'a str,
) -> Option<(&'a str, &'a str)> {
    return attribute
        .child(0)
        .and_then(|node| node.utf8_text(source.as_bytes()).ok())
        .map(|name| {
            (
                name,
                attribute
                    .child(2)
                    .and_then(|node| node.child(1))
                    .filter(|node| node.kind() == "string_content")
                    .and_then(|node| node.utf8_text(source.as_bytes()).ok())
                    .unwrap_or(""),
            )
        });
}

// ============================================================================================
// || TESTS || TESTS || TESTS || TESTS || TESTS || TESTS || TESTS || TESTS || TESTS || TESTS ||
// ============================================================================================

#[cfg(test)]
mod tests {
    use crate::parser::{Attribute, Header, JavaHeader, Location, TagLibImport, TagLibOrigin};

    use super::TreeParser;
    use anyhow::{Error, Result};

    #[test]
    fn test_parse_header() -> Result<()> {
        let document = String::from(concat!(
            "<%@ page language=\"java\" pageEncoding=\"UTF-8\" contentType=\"text/html; charset=UTF-8\"\n",
            "%><%@ taglib uri=\"http://www.sitepark.com/taglibs/core\" prefix=\"sp\"\n",
            "%><%@ taglib tagdir=\"/WEB-INF/tags/spt\" prefix=\"spt\"\n",
            "%>\n"
        ));
        let expected = Header {
            java_header: JavaHeader {
                open_bracket: Location::new(0, 0, 3),
                page: Location::new(4, 0, 4),
                language: Attribute {
                    key_location: Location::new(9, 0, 8),
                    equals_location: Location::new(17, 0, 1),
                    opening_quote_location: Location::new(18, 0, 1),
                    value: "java".to_string(),
                    closing_quote_location: Location::new(23, 0, 1),
                },
                page_encoding: Attribute {
                    key_location: Location::new(25, 0, 12),
                    equals_location: Location::new(37, 0, 1),
                    opening_quote_location: Location::new(38, 0, 1),
                    value: "UTF-8".to_string(),
                    closing_quote_location: Location::new(44, 0, 1),
                },
                content_type: Attribute {
                    key_location: Location::new(46, 0, 11),
                    equals_location: Location::new(57, 0, 1),
                    opening_quote_location: Location::new(58, 0, 1),
                    value: "text/html; charset=UTF-8".to_string(),
                    closing_quote_location: Location::new(83, 0, 1),
                },
                close_bracket: Location::new(0, 1, 2),
            },
            taglib_imports: vec![
                TagLibImport {
                    open_bracket: Location {
                        char: 2,
                        line: 1,
                        length: 3,
                    },
                    taglib: Location {
                        char: 6,
                        line: 1,
                        length: 6,
                    },
                    origin: TagLibOrigin::Uri(Attribute {
                        key_location: Location {
                            char: 13,
                            line: 1,
                            length: 3,
                        },
                        equals_location: Location {
                            char: 16,
                            line: 1,
                            length: 1,
                        },
                        opening_quote_location: Location {
                            char: 17,
                            line: 1,
                            length: 1,
                        },
                        value: "http://www.sitepark.com/taglibs/core".to_string(),
                        closing_quote_location: Location {
                            char: 54,
                            line: 1,
                            length: 1,
                        },
                    }),
                    prefix: Attribute {
                        key_location: Location {
                            char: 56,
                            line: 1,
                            length: 6,
                        },
                        equals_location: Location {
                            char: 62,
                            line: 1,
                            length: 1,
                        },
                        opening_quote_location: Location {
                            char: 63,
                            line: 1,
                            length: 1,
                        },
                        value: "sp".to_string(),
                        closing_quote_location: Location {
                            char: 66,
                            line: 1,
                            length: 1,
                        },
                    },
                    close_bracket: Location {
                        char: 0,
                        line: 2,
                        length: 2,
                    },
                },
                TagLibImport {
                    open_bracket: Location {
                        char: 2,
                        line: 2,
                        length: 3,
                    },
                    taglib: Location {
                        char: 6,
                        line: 2,
                        length: 6,
                    },
                    origin: TagLibOrigin::TagDir(Attribute {
                        key_location: Location {
                            char: 13,
                            line: 2,
                            length: 6,
                        },
                        equals_location: Location {
                            char: 19,
                            line: 2,
                            length: 1,
                        },
                        opening_quote_location: Location {
                            char: 20,
                            line: 2,
                            length: 1,
                        },
                        value: "/WEB-INF/tags/spt".to_string(),
                        closing_quote_location: Location {
                            char: 38,
                            line: 2,
                            length: 1,
                        },
                    }),
                    prefix: Attribute {
                        key_location: Location {
                            char: 40,
                            line: 2,
                            length: 6,
                        },
                        equals_location: Location {
                            char: 46,
                            line: 2,
                            length: 1,
                        },
                        opening_quote_location: Location {
                            char: 47,
                            line: 2,
                            length: 1,
                        },
                        value: "spt".to_string(),
                        closing_quote_location: Location {
                            char: 51,
                            line: 2,
                            length: 1,
                        },
                    },
                    close_bracket: Location {
                        char: 0,
                        line: 3,
                        length: 2,
                    },
                },
            ],
        };
        let mut ts_parser = tree_sitter::Parser::new();
        ts_parser
            .set_language(&tree_sitter_spml::language())
            .map_err(Error::from)?;
        let ts_tree = ts_parser
            .parse(&document, None)
            .ok_or_else(|| anyhow::anyhow!("treesitter parsing failed"))?;
        let parser = &mut TreeParser::new(ts_tree.walk(), &document);
        println!("start parsing");
        let header = parser.parse_header()?;
        println!("finished parsing");
        assert_eq!(header, expected);
        return Ok(());
    }
}
