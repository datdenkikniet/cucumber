use crate::scenario_outline::TaggedScenarios;

use super::*;

mod keyword;
use keyword::Keyword;

#[cfg(test)]
mod test;

use std::{collections::HashSet, iter::Peekable, str::Lines};

struct ParserInner<'a> {
    current_line: usize,
    text: &'a str,
    lines: Peekable<Lines<'a>>,
    feature_name: Option<String>,
}

impl<'a> Iterator for ParserInner<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(line) = self.lines.next() {
            self.current_line += 1;
            Some(line)
        } else {
            None
        }
    }
}

impl<'a> ParserInner<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            text: input,
            current_line: 0,
            lines: input.lines().peekable(),
            feature_name: None,
        }
    }

    fn format_error<T>(message: &str, text: &str, line_number: usize) -> Result<T, String> {
        let line = text.lines().skip(line_number).next().unwrap();
        Err(format!("{message}.\n--> {line} <--"))
    }

    fn make_error<T>(&mut self, message: &str) -> Result<T, String> {
        Self::format_error(message, self.text, self.current_line)
    }

    fn take_empty_or_comment(&mut self) {
        loop {
            if let Some(line) = self.lines.peek() {
                let trimmed = line.trim_start();
                if !trimmed.starts_with('#') && !trimmed.trim_end().is_empty() {
                    break;
                } else {
                    self.next();
                }
            } else {
                break;
            }
        }
    }

    fn try_tags(&mut self) -> Result<Vec<String>, String> {
        let mut tags = Vec::new();

        let line = if let Some(line) = self.lines.peek() {
            line
        } else {
            return Ok(tags);
        };

        let trimmed = line.trim();

        if !trimmed.starts_with('@') {
            return Ok(tags);
        }

        for tag in trimmed.split(' ') {
            let trimmed = tag.trim();
            if !trimmed.starts_with('@') {
                return self
                    .make_error(&format!("Invalid tag {trimmed} (does not start with '@')"));
            }
            tags.push(String::from(&trimmed[1..]));
        }

        self.next();

        self.take_empty_or_comment();

        if self.lines.peek().is_none() {
            return self.make_error("Standalone tags are not allowed");
        }

        Ok(tags)
    }

    fn match_steps(&mut self, in_keyword: Keyword) -> Result<Vec<Step>, String> {
        let mut steps = Vec::new();
        let mut lines = Vec::new();

        loop {
            self.take_empty_or_comment();

            let next_kw = self.peek_kw_line(true);

            let (kw, description, _) = match (steps.is_empty(), next_kw) {
                (true, Err(_)) => {
                    return self.make_error("Expected step keyword, but got invalid keyword line")
                }
                (true, Ok(None)) => {
                    return self.make_error("Expected step keyword, but got end of input")
                }
                (false, Err(_)) | (false, Ok(None)) => break,
                (_, Ok(Some((kw, desc, colon)))) => (kw, desc.map(String::from), colon),
            };

            let step_type = match kw {
                Keyword::Given => StepType::Given,
                Keyword::When => StepType::When,
                Keyword::Then => StepType::Then,
                Keyword::And => StepType::And,
                Keyword::But => StepType::But,
                Keyword::Asterisk => StepType::Asterisk,
                _ => break,
            };

            self.next();

            let description = if let Some(description) = description {
                description.to_string()
            } else {
                return self.make_error(&format!("{kw:?} step without description."));
            };

            let step_data = if let Some(table) = self.try_datatable()? {
                Some(StepData::DataTable(table))
            } else if let Some(docstring) = self.try_docstring()? {
                Some(StepData::DocString(docstring))
            } else {
                None
            };

            steps.push(Step::new(step_type, description, step_data));
            lines.push(self.current_line);
        }

        if steps.is_empty() {
            return self.make_error(&format!("`{in_keyword:?} Must have at least 1 step"));
        }

        // Find duplicated steps (according to gherkin spec)
        #[cfg(feature = "step-duplicate-check")]
        {
            let mut step_set = HashSet::new();
            if let Some((_, description)) = steps.iter().enumerate().find_map(|(idx, s)| {
                if !step_set.insert(s.description.as_str()) {
                    Some((lines[idx], s.description.as_str()))
                } else {
                    None
                }
            }) {
                log::warn!(
                    "Duplicate step definition '{description}' in feature {}",
                    self.feature_name.as_ref().unwrap()
                );
            }
        }

        Ok(steps)
    }

    fn peek_kw_line(
        &mut self,
        strip_colon: bool,
    ) -> Result<Option<(Keyword, Option<&str>, bool)>, String> {
        self.take_empty_or_comment();

        let kw_line = if let Some(line) = self.lines.peek() {
            line
        } else {
            return Ok(None);
        };

        if let Some((keyword, _, rest_of_str, has_trailing_colon)) =
            Keyword::parse(kw_line.trim_start(), strip_colon)
        {
            let rest_of_str = if rest_of_str.is_empty() {
                None
            } else {
                Some(rest_of_str)
            };

            Ok(Some((keyword, rest_of_str, has_trailing_colon)))
        } else {
            let message = format!("Unknown keyword {kw_line}");
            Self::format_error(&message, self.text, self.current_line)
        }
    }

    fn match_kw_line(
        &mut self,
        wanted: Keyword,
        strip_colon: bool,
    ) -> Result<(Keyword, Option<&'a str>, bool), String> {
        let kw_line = if let Some(keyword_line) = self.next().map(str::trim_start) {
            keyword_line
        } else {
            return self.make_error(&format!("Expected keyword `{wanted:?}`, got end of input"));
        };

        if let Some((keyword, _, rest_of_str, has_trailing_colon)) =
            Keyword::parse(kw_line, strip_colon)
        {
            if keyword != wanted {
                return self
                    .make_error(&format!("Expected keyword `{wanted:?}`, got `{keyword:?}`"));
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

    fn try_datatable(&mut self) -> Result<Option<DataTable>, String> {
        fn row_iter<'a>(row: &'a str) -> impl Iterator<Item = &'a str> {
            struct Inner<'a> {
                iter: Peekable<std::iter::Skip<std::str::Split<'a, char>>>,
            }

            impl<'a> Iterator for Inner<'a> {
                type Item = &'a str;

                fn next(&mut self) -> Option<Self::Item> {
                    // TODO: escape and stuff
                    let next_line = self.iter.next();
                    if self.iter.peek().is_none() {
                        None
                    } else {
                        next_line.map(str::trim)
                    }
                }
            }

            Inner {
                iter: row.split('|').skip(1).peekable(),
            }
        }

        self.take_empty_or_comment();
        let first_line = if let Some(line) = self.lines.peek() {
            line.trim()
        } else {
            return Ok(None);
        };

        if !first_line.starts_with('|') || !first_line.ends_with('|') {
            return Ok(None);
        }

        let header = row_iter(first_line).map(String::from).collect();

        self.next();

        let mut table = DataTable::new(header);

        loop {
            self.take_empty_or_comment();
            if let Some(next_line) = self.lines.peek() {
                let next_line = next_line.trim();
                if next_line.starts_with('|') && next_line.ends_with('|') {
                    let row: Vec<_> = row_iter(next_line).map(String::from).collect();
                    let row_len = row.len();
                    if table.add_row(row).is_err() {
                        return self.make_error(&format!(
                            "Invalid column count in datatable. Expected {}, got {row_len}",
                            table.header().len(),
                        ));
                    }
                    self.next();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(Some(table))
    }

    fn try_background(&mut self) -> Result<Vec<Step>, String> {
        if let Ok(Some((Keyword::Background, _, _))) = self.peek_kw_line(true) {
            self.next();
            let steps = self.match_steps(Keyword::Background)?;
            Ok(steps)
        } else {
            return Ok(Vec::new());
        }
    }

    fn try_freeform_text(&mut self) -> Result<Option<String>, String> {
        self.take_empty_or_comment();

        if self.lines.peek().is_none() {
            return Ok(None);
        };

        let mut string = String::new();
        let mut indent: Option<String> = None;
        loop {
            if let Some(next_line) = self.lines.peek() {
                let trimmed = next_line.trim();

                if trimmed.is_empty() {
                    self.next();
                    continue;
                }

                if let Some((_, _, _, _)) = Keyword::parse(trimmed, false) {
                    break;
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

        let trimmed = string.trim_end();

        if !trimmed.is_empty() {
            Ok(Some(trimmed.to_string()))
        } else {
            Ok(None)
        }
    }

    fn try_docstring(&mut self) -> Result<Option<String>, String> {
        self.take_empty_or_comment();

        let first = if let Some(line) = self.lines.peek() {
            line
        } else {
            return Ok(None);
        };

        if first.trim() == "\"\"\"" {
            let indent: String = first
                .chars()
                .take_while(|c| c.is_ascii_whitespace())
                .collect();
            let indent_len = indent.len();

            self.next();

            let mut string = String::new();

            loop {
                if let Some(line) = self.next() {
                    let trimmed = line.trim();
                    if trimmed == "\"\"\"" {
                        return Ok(Some(string.trim().to_string()));
                    } else if line.starts_with(&indent) {
                        let actual_line = &line[indent_len..];
                        string.push_str(actual_line);
                        string.push('\n');
                    } else if trimmed.is_empty() {
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

    fn try_scenario_outline(&mut self) -> Result<Option<ScenarioOutline>, String> {
        let outline_tags = self.try_tags()?;

        self.take_empty_or_comment();

        let name = if let Ok(Some((Keyword::ScenarioOutline, name, _))) = self.peek_kw_line(false) {
            name.map(String::from)
        } else {
            return Ok(None);
        };

        self.next();

        let description = self.try_freeform_text()?;

        let steps = self.match_steps(Keyword::ScenarioOutline)?;

        let mut scenarios = Vec::new();
        let mut first_placeholders: Option<HashSet<String>> = None;

        loop {
            self.take_empty_or_comment();

            let tags = self.try_tags()?;

            self.take_empty_or_comment();

            let _ = match (self.peek_kw_line(false), scenarios.is_empty()) {
                (Ok(Some((Keyword::Scenarios, _, _))), _) => {}
                (Err(e), true) => return Err(e),
                (Ok(_), true) => {
                    return self.make_error(
                        "Must have at least one `Scenarios` section in a `Scenario Outline`",
                    )
                }
                (Err(_), false) | (Ok(_), false) => break,
            };

            self.next();

            self.take_empty_or_comment();

            let DataTable {
                header: placeholders,
                rows: values,
            } = if let Some(table) = self.try_datatable()? {
                table
            } else {
                return self.make_error("Expected data table to follow `Examples`.");
            };

            if let Some(first_placeholders) = &first_placeholders {
                if placeholders.iter().any(|p| !first_placeholders.contains(p)) {
                    return self.make_error(
                        "Differing amount of or differently named placeholders in examples",
                    );
                }
            } else {
                first_placeholders = Some(placeholders.clone().into_iter().collect::<HashSet<_>>());
            }
            let placeholders = placeholders.into_iter().collect();
            scenarios.push(TaggedScenarios::new(tags, placeholders, values)?);
        }

        Ok(Some(ScenarioOutline {
            tags: outline_tags,
            name,
            description,
            steps,
            scenarios,
        }))
    }

    fn try_scenario(&mut self) -> Result<Option<Scenario>, String> {
        let tags = self.try_tags()?;

        self.take_empty_or_comment();

        let name = if let Ok(Some((Keyword::Scenario, name, _))) = self.peek_kw_line(false) {
            name.map(String::from)
        } else {
            return Ok(None);
        };

        self.next();

        let description = self.try_freeform_text()?;

        let steps = self.match_steps(Keyword::Scenario)?;

        Ok(Some(Scenario {
            tags,
            name,
            description,
            steps,
        }))
    }

    fn match_feature(mut self) -> Result<Feature, String> {
        self.take_empty_or_comment();

        let feature_tags = self.try_tags()?;

        self.take_empty_or_comment();

        let (_, rest_of_line, _) = self.match_kw_line(Keyword::Feature, false)?;

        let feature_name = rest_of_line.map(String::from);

        self.feature_name = Some(
            feature_name
                .clone()
                .unwrap_or("Unnamed feature".to_string()),
        );

        let description = self.try_freeform_text()?;

        self.take_empty_or_comment();
        let background = self.try_background()?;

        let mut scenarios = Vec::new();
        let mut scenario_outlines = Vec::new();

        loop {
            self.take_empty_or_comment();

            if let Some(scenario) = self.try_scenario()? {
                scenarios.push(scenario);
            } else if let Some(scenario_outline) = self.try_scenario_outline()? {
                scenario_outlines.push(scenario_outline);
            } else if self.lines.peek().is_none() {
                break;
            } else {
                return self.make_error(
                    "Expected `Scenario`, `Example`, `Scenario Outline`, or `Scenario Template`.",
                );
            }
        }

        Ok(Feature {
            tags: feature_tags,
            name: feature_name,
            description,
            background,
            scenarios,
            scenario_outlines,
        })
    }
}

pub struct Parser;

impl Parser {
    pub fn parse_feature(input: &str) -> Result<Feature, String> {
        let inner = ParserInner::new(input);
        inner.match_feature()
    }
}
