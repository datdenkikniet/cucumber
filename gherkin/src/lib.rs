// #[cfg(test)]
// mod test;

use std::{iter::Peekable, str::Lines};

#[derive(Debug, Clone, Copy)]
pub enum StepType {
    Given,
    When,
    Then,
    And,
    But,
    Asterisk,
}

#[derive(Debug, Clone)]
pub struct DataTable {
    pub header: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum StepData {
    DocString(String),
    DataTable(DataTable),
}

#[derive(Debug, Clone)]
pub struct Step {
    pub ty: StepType,
    pub data: Option<StepData>,
}

#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone)]
pub struct ScenarioOutline {
    pub name: Option<String>,
    pub description: Option<String>,
    pub steps: Vec<Step>,
    pub examples: DataTable,
}

#[derive(Debug, Clone)]
pub struct Feature {
    pub name: Option<String>,
    pub description: Option<String>,
    pub background: Vec<Step>,
    pub scenarios: Vec<Scenario>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Feature,
    Example,
    Background,
    ScenarioOutline,
    Examples,
    Given,
    When,
    Then,
    And,
    But,
    Asterisk,
}

impl Keyword {
    pub fn has_colon(&self) -> bool {
        match self {
            Keyword::Feature
            | Keyword::Example
            | Keyword::Background
            | Keyword::ScenarioOutline
            | Keyword::Examples => true,
            Keyword::Given
            | Keyword::When
            | Keyword::Then
            | Keyword::And
            | Keyword::But
            | Keyword::Asterisk => false,
        }
    }

    fn combinations() -> &'static [(Self, &'static str)] {
        &[
            (Self::Examples, "examples:"),
            (Self::Examples, "scenarios:"),
            (Self::ScenarioOutline, "scenario outline"),
            (Self::ScenarioOutline, "scenario template"),
            (Self::Feature, "feature"),
            (Self::Example, "example"),
            (Self::Example, "scenario"),
            (Self::Background, "background"),
            (Self::Given, "given"),
            (Self::When, "when"),
            (Self::Then, "then"),
            (Self::And, "and"),
            (Self::But, "but"),
            (Self::Asterisk, "*"),
        ]
    }

    pub fn parse(line: &str, strip_trailing_colon: bool) -> Option<(Self, &str, bool)> {
        let lowercase = line.to_ascii_lowercase();

        let (keyword, start) = Self::combinations()
            .iter()
            .find(|(_, pattern)| lowercase.starts_with(pattern))?;
        let start_len = start.len();

        let leftover = &line[start_len..];
        let has_colon = keyword.has_colon();

        let leftover = if has_colon ^ leftover.starts_with(':') {
            return None;
        } else if has_colon {
            leftover[1..].trim_start()
        } else {
            leftover.trim_start()
        };

        let last_is_colon = leftover.trim_end().ends_with(':');

        if last_is_colon && strip_trailing_colon {
            Some((*keyword, &leftover[..leftover.len() - 1], true))
        } else {
            Some((*keyword, &leftover[..leftover.len()], false))
        }
    }
}

pub struct Parser;

struct ParserInner<'a> {
    current_line: usize,
    lines: Peekable<Lines<'a>>,
}

impl<'a> ParserInner<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            current_line: 0,
            lines: input.lines().peekable(),
        }
    }

    fn peek(&mut self) -> Option<&&str> {
        self.lines.peek()
    }

    fn make_error<T>(&mut self, message: &str) -> Result<T, String> {
        let _ = self.current_line;
        Err(message.into())
    }

    fn take_empty_or_comment(&mut self) {
        loop {
            if let Some(line) = self.peek() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    break;
                } else {
                    self.next();
                }
            } else {
                break;
            }
        }
    }

    fn match_kw_line(
        &mut self,
        wanted_kw: Option<Keyword>,
        strip_colon: bool,
    ) -> Result<(Keyword, Option<&'a str>, bool), String> {
        self.take_empty_or_comment();

        let kw_line = self
            .next()
            .map(str::trim_start)
            .ok_or("Expected keyword, got End Of Input")?;

        if let Some((keyword, rest_of_str, has_trailing_colon)) =
            Keyword::parse(kw_line, strip_colon)
        {
            if let Some(wanted) = wanted_kw {
                if keyword != wanted {
                    return self
                        .make_error(&format!("Expected keyword `{wanted:?}`, got `{keyword:?}`"));
                }
            }

            let rest_of_str = if rest_of_str.is_empty() {
                None
            } else {
                Some(rest_of_str)
            };

            Ok((keyword, rest_of_str, has_trailing_colon))
        } else {
            self.make_error(&format!("Unknown keyword {kw_line}"))
        }
    }

    fn try_freeform_text(&mut self) -> Result<Option<String>, String> {
        self.take_empty_or_comment();

        if self.peek().is_none() {
            return Ok(None);
        };

        let mut string = String::new();
        let mut indent: Option<String> = None;
        loop {
            if let Some(next_line) = self.peek() {
                let trimmed = next_line.trim_start();
                if let Some((kw, _, _)) = Keyword::parse(trimmed, false) {
                    match kw {
                        Keyword::Example | Keyword::Background | Keyword::ScenarioOutline => {
                            break;
                        }
                        _ => {}
                    }
                }

                if let Some(indent) = &indent {
                    if !next_line.starts_with(indent) {
                        return self.make_error("Inconsistent indentation in freeform text");
                    }
                    string.push_str(&next_line[indent.len()..]);
                    string.push('\n');
                } else {
                    let indent_value: String = next_line
                        .chars()
                        .take_while(|c| c.is_ascii_whitespace())
                        .collect();
                    let indent_len = indent_value.len();
                    indent = Some(indent_value);
                    string.push_str(&next_line[indent_len..]);
                    string.push('\n');
                }

                self.next();
            } else {
                break;
            }
        }

        if !string.is_empty() {
            Ok(Some(string))
        } else {
            Ok(None)
        }
    }

    fn try_docstring(&mut self) -> Result<Option<String>, String> {
        self.take_empty_or_comment();

        let first = if let Some(line) = self.peek() {
            line
        } else {
            return Ok(None);
        };

        if first.trim_start().starts_with("\"\"\"") {
            let indent: String = first
                .chars()
                .take_while(|c| c.is_ascii_whitespace())
                .collect();
            let indent_len = indent.len();

            self.next();

            let mut string = String::new();

            loop {
                if let Some(line) = self.next() {
                    if line.trim_start().starts_with("\"\"\"") {
                        return Ok(Some(string));
                    } else if line.starts_with(&indent) {
                        let actual_line = &line[indent_len..];
                        string.push_str(actual_line);
                        string.push('\n');
                    } else {
                        return self.make_error("Inconsistent whitespace in docstring");
                    }
                } else {
                    return self.make_error("Unterminated newline");
                }
            }
        } else {
            Ok(None)
        }
    }
}

impl<'a> Iterator for ParserInner<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next()
    }
}

impl Parser {
    pub fn parse_feature(input: &str) -> Result<Feature, String> {
        let mut inner = ParserInner::new(input);
        let (_, rest_of_line, _) = inner.match_kw_line(Some(Keyword::Feature), false)?;

        let feature_name = rest_of_line.map(String::from);
        let description = inner.try_freeform_text()?;

        let background = inner.match_kw_line(Some(Keyword::Background), false);

        todo!("{feature_name:?}, {description:?}")
    }
}

const KIND_OF_EMPTY: &str = r#"
Feature: a feature
    Hehe a freeform text!
    Uh oh my indentation!
    Example:
        Given nothing
        When the base is empty
        Then nothing happens
"#;

#[test]
pub fn basic() {
    let feature = Parser::parse_feature(KIND_OF_EMPTY).unwrap();
}
