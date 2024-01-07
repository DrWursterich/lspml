use anyhow::{Error, Result};
use lsp_types::TextDocumentPositionParams;
use std::fmt;
use std::fs;

pub(crate) enum IteratingDirection {
    Forwards,
    Backwards,
}

pub(crate) struct Include {
    pub module: Option<String>,
    pub uri: String,
}

impl fmt::Display for IteratingDirection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return formatter.write_str(&self.to_string());
    }
}

pub(crate) fn get_text_document(text_params: &TextDocumentPositionParams) -> Result<String> {
    return match text_params.text_document.uri.to_file_path() {
        Ok(path) => fs::read_to_string(path.to_owned()).map_err(Error::from),
        Err(_) => Result::Err(anyhow::anyhow!(
            "failed to read file path from uri {}",
            text_params.text_document.uri
        )),
    };
}

pub(crate) fn find_include_uri(string: &str) -> Option<Include> {
    let include_tag = "<sp:include ";
    let module_attribute = "module=\"";
    let uri_attribute = "uri=\"";
    return string.find(include_tag).and_then(|start| {
        let string = &string[start + include_tag.len()..];
        let module = string.find(module_attribute).and_then(|module_start| {
            let string = &string[module_start + module_attribute.len()..];
            return string
                .find('"')
                .map(|module_end| string[..module_end].to_owned());
        });
        return string.find(uri_attribute).and_then(|uri_start| {
            let string = &string[uri_start + uri_attribute.len()..];
            return string.find('"').map(|uri_end| Include {
                module,
                uri: string[..uri_end].to_owned(),
            });
        });
    });
}

pub(crate) fn find_keyword(string: &str, position: usize) -> Option<&str> {
    return find_keyword_boundary(&string[..position], IteratingDirection::Backwards).and_then(
        |start| {
            find_keyword_boundary(&string[position..], IteratingDirection::Forwards)
                .map(|end| &string[position - start..position + end])
        },
    );
}

pub(crate) fn find_keyword_boundary(string: &str, direction: IteratingDirection) -> Option<usize> {
    match direction {
        IteratingDirection::Forwards => {
            for (index, char) in string.chars().enumerate() {
                if !char.is_ascii_alphanumeric() && char != '_' {
                    return Some(index);
                }
            }
        }
        IteratingDirection::Backwards => {
            for (index, char) in string.chars().rev().enumerate() {
                if !char.is_ascii_alphanumeric() && char != '_' {
                    return Some(index);
                }
            }
        }
    };
    eprintln!(
        "did not find a boundary in \"{}\", looking {}",
        string, direction
    );
    return None;
}
