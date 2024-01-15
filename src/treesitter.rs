
enum IncludeParsingState {
    AttributeName,
    AttributeValue,
    TagBeginning,
    Unknown,
    UnknownText,
    Whitespace,
}

fn parse_include_tag(string: String, position: usize) -> Option<String> {
    let mut state = IncludeParsingState::Unknown;
    // let mut previous_value = "".to_string();
    let mut value = "".to_string();
    let mut is_value_complete = false;
    for (_index, char) in string[..position].chars().rev().enumerate() {
        match char {
            ' ' => match state {
                IncludeParsingState::Unknown => {
                    state = IncludeParsingState::Whitespace;
                    is_value_complete = true;
                }
                IncludeParsingState::AttributeName => {
                    if is_value_complete {

                    }
                    state = IncludeParsingState::Whitespace;
                }
                _ => {
                    eprintln!("error!");
                    return None;
                }
            },
            '"' => match state {
                IncludeParsingState::Unknown => {
                    state = IncludeParsingState::UnknownText;
                }
                IncludeParsingState::Whitespace => {
                    state = IncludeParsingState::AttributeValue;
                }
                IncludeParsingState::AttributeValue | IncludeParsingState::UnknownText => {
                    state = IncludeParsingState::AttributeName;
                }
                _ => {
                    eprintln!("error!");
                    return None;
                }
            },
            '=' => match state {
                IncludeParsingState::Unknown
                | IncludeParsingState::AttributeValue
                | IncludeParsingState::UnknownText => {
                    state = IncludeParsingState::AttributeName;
                }
                _ => {
                    eprintln!("error!");
                    return None;
                }
            },
            '<' => match state {
                IncludeParsingState::Unknown | IncludeParsingState::UnknownText => {
                    state = IncludeParsingState::TagBeginning;
                }
                _ => {
                    eprintln!("error!");
                    return None;
                }
            }
            _ => {
                value.push(char);
            }
            // 'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '.' {
            //     return Some(Include {
            //         module: None,
            //         uri: "".to_string(),
            //     });
            // }
        }
    }
    return None;
}
