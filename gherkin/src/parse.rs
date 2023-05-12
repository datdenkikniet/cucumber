use std::iter::Peekable;

use crate::{
    lex::TokenKind, Block, Example, Feature, LexError, LexErrorKind, Prompt, Step, StepInput,
    StepType, Token,
};

#[derive(Debug, Clone)]
pub enum ErrorKind<'a> {
    NotAFeature(Keyword),
    Unexpected(Keyword),
    Expected {
        wanted: &'static [&'static str],
        got: Token<'a>,
    },
    MultipleLanguageTags,
    InvalidBackgroundStep(StepType),
    InvalidBareKeyword(Keyword),
    UnexpectedEof {
        wanted: &'static [&'static str],
    },
    InvalidKeyword,
    Lex(LexErrorKind),
}

#[derive(Debug, Clone)]
pub struct Error<'a> {
    kind: ErrorKind<'a>,
    text: &'a str,
    line_num: usize,
}

impl<'a> Error<'a> {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn line(&self) -> &str {
        self.text.lines().skip(self.line_num).next().unwrap()
    }
}

impl<'a> From<LexError<'a>> for Error<'a> {
    fn from(value: LexError<'a>) -> Self {
        Self {
            kind: ErrorKind::Lex(value.kind),
            text: value.text,
            line_num: value.line_num,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Feature,
    Example,
    Given,
    When,
    Then,
    And,
    But,
    Asterisk,
    Background,
    ScenarioOutline,
    Examples,
}

impl Keyword {
    pub fn combinations() -> &'static [(Self, &'static str)] {
        &[
            (Self::Examples, "examples"),
            (Self::Examples, "scenarios"),
            (Self::ScenarioOutline, "scenario outline"),
            (Self::ScenarioOutline, "scenario template"),
            (Self::Feature, "feature"),
            (Self::Example, "example"),
            (Self::Example, "scenario"),
            (Self::Given, "given"),
            (Self::When, "when"),
            (Self::Then, "then"),
            (Self::And, "and"),
            (Self::But, "but"),
            (Self::Background, "background"),
            (Self::Asterisk, "*"),
        ]
    }
}

#[derive(Debug, Clone)]
struct Line<'a> {
    keyword: Keyword,
    has_kw_colon: bool,
    contents: Option<(&'a str, bool)>,
}

pub fn parse_feature(text: &str) -> Result<Feature, Error> {
    let tokens = crate::lex(text)?;
    Parser::new(text).parse(tokens)
}

struct Parser<'a> {
    current_end_line: usize,
    text: &'a str,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            current_end_line: 0,
            text,
        }
    }

    fn make_error<O>(&mut self, kind: ErrorKind<'a>) -> Result<O, Error<'a>>
    where
        O: 'a,
    {
        Err(Error {
            kind,
            text: self.text,
            line_num: self.current_end_line,
        })
    }

    fn peek_token<'s, T>(tokens: &mut Peekable<T>) -> Option<&TokenKind<'s>>
    where
        T: Iterator<Item = Token<'s>>,
    {
        tokens.peek().map(|t| t.kind())
    }

    fn consume_token<'s, T>(&mut self, tokens: &mut Peekable<T>) -> Option<Token<'s>>
    where
        T: Iterator<Item = Token<'s>>,
    {
        if let Some(token) = tokens.next() {
            self.current_end_line = token.end_line();
            Some(token)
        } else {
            None
        }
    }

    fn parse(mut self, tokens: Vec<Token<'a>>) -> Result<Feature, Error<'a>> {
        let mut language_tags = tokens.iter().filter_map(|t| match t.kind() {
            TokenKind::Comment(c) => {
                if let Some(("language", language)) =
                    c.split_once(':').map(|(l, ll)| (l, ll.trim()))
                {
                    Some((language, t))
                } else {
                    None
                }
            }
            _ => None,
        });

        if language_tags.clone().count() > 1 {
            return self.make_error(ErrorKind::MultipleLanguageTags);
        }

        let language_tag = language_tags.next().map(|(lang, _)| lang).unwrap_or("en");

        let mut peekable = tokens.into_iter().peekable();
        let tokens = &mut peekable;

        let first_line = self.match_kw_line(tokens)?;
        if first_line.keyword != Keyword::Feature {
            return self.make_error(ErrorKind::NotAFeature(first_line.keyword));
        }
        let feature_name = first_line.contents.map(|(v, _)| v.to_string());

        let freeform_text = self.parse_freeform_text(tokens)?;

        let (background_and, background_but) = self.parse_background(tokens)?;

        let mut contents: Vec<Block> = Vec::new();

        loop {
            let line = self.peek_kw_line(tokens)?;
            let token = self.consume_token(tokens).unwrap();

            let block = match line.keyword {
                Keyword::Example => {
                    let name = line.contents.map(|(s, _)| s.to_string());
                    Block::Example(Example {
                        name,
                        steps: self.parse_steps(tokens)?,
                    })
                }
                Keyword::ScenarioOutline => todo!("Scenario outline not supported"),
                Keyword::Background => {
                    return self.make_error(ErrorKind::Unexpected(Keyword::Background));
                }
                _ => {
                    return self.make_error(ErrorKind::Expected {
                        wanted: &["`Example`", "`Scenario Outline`"],
                        got: token.clone(),
                    });
                }
            };

            contents.push(block);

            if Self::peek_token(tokens).is_none() {
                break;
            }
        }

        Ok(Feature {
            language: language_tag.into(),
            name: feature_name,
            freeform_text,
            contents,
            background_and,
            background_but,
        })
    }

    fn parse_freeform_text<'t, T>(
        &mut self,
        tokens: &mut Peekable<T>,
    ) -> Result<Option<String>, Error<'a>>
    where
        T: Iterator<Item = Token<'t>>,
        't: 'a,
    {
        let mut freeform_text = String::new();
        while let Some(token) = Self::peek_token(tokens) {
            match token {
                TokenKind::Line(line) => match self.parse_line(line) {
                    Ok(Some(Line {
                        keyword: Keyword::Background | Keyword::Example | Keyword::ScenarioOutline,
                        ..
                    })) => {
                        break;
                    }
                    _ => {
                        freeform_text.push_str(line.trim_start());
                        freeform_text.push('\n');
                        self.consume_token(tokens);
                    }
                },
                TokenKind::Comment(_) => {}
                _ => {
                    let got = self.consume_token(tokens).unwrap();
                    return self.make_error(ErrorKind::Expected {
                        wanted: &[
                            "freeform text",
                            "`Background`",
                            "`Example`",
                            "`ScenarioOutline`",
                        ],
                        got,
                    });
                }
            }
        }

        let trimmed = freeform_text.trim().to_string();

        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed))
        }
    }

    fn parse_background<'t, T>(
        &mut self,
        tokens: &mut Peekable<T>,
    ) -> Result<(Vec<Prompt>, Vec<Prompt>), Error<'a>>
    where
        T: Iterator<Item = Token<'t>>,
        't: 'a,
    {
        let res = if let Line {
            keyword: Keyword::Background,
            ..
        } = self.peek_kw_line(tokens)?
        {
            self.consume_token(tokens);
            let as_example = self.parse_steps(tokens)?;

            if let Some(step) = as_example.iter().find(|s| s.ty != StepType::Given) {
                let ty = step.ty;
                return self.make_error(ErrorKind::InvalidBackgroundStep(ty));
            }

            let givens = as_example.into_iter().filter_map(|g| {
                if matches!(
                    g,
                    Step {
                        ty: StepType::Given,
                        ..
                    }
                ) {
                    Some((g.ands, g.buts))
                } else {
                    None
                }
            });

            let ands = givens.clone().flat_map(|(g, _)| g.into_iter()).collect();
            let buts = givens.clone().flat_map(|(_, b)| b.into_iter()).collect();
            (ands, buts)
        } else {
            (Vec::new(), Vec::new())
        };

        Ok(res)
    }

    fn parse_steps<'t, T>(&mut self, tokens: &mut Peekable<T>) -> Result<Vec<Step>, Error<'a>>
    where
        T: Iterator<Item = Token<'t>>,
        't: 'a,
    {
        let mut steps: Vec<Step> = Vec::new();

        while let Ok(line) = self.peek_kw_line(tokens) {
            let step_type = match line.keyword {
                Keyword::Given => StepType::Given,
                Keyword::When => StepType::When,
                Keyword::Then => StepType::Then,
                Keyword::Asterisk => panic!("Placeholder not supported yet"),
                _ => break,
            };

            self.consume_token(tokens);

            let mut ands = if let Some((prompt, _)) = line.contents {
                let prompt = prompt.to_string();
                let prompt = match self.peek_input(tokens).ok() {
                    Some(input) => {
                        self.consume_token(tokens);
                        Prompt::WithInput { prompt, input }
                    }
                    None => Prompt::Bare { prompt },
                };
                vec![prompt]
            } else {
                return self.make_error(ErrorKind::InvalidBareKeyword(line.keyword));
            };

            let mut buts = Vec::new();

            while let Ok(line) = self.peek_kw_line(tokens) {
                match line.keyword {
                    Keyword::And => {
                        self.consume_token(tokens);

                        let prompt = if let Some((contents, _)) = line.contents {
                            contents.to_string()
                        } else {
                            return self.make_error(ErrorKind::InvalidBareKeyword(Keyword::And));
                        };

                        let prompt = match self.peek_input(tokens).ok() {
                            Some(input) => {
                                self.consume_token(tokens);
                                Prompt::WithInput { prompt, input }
                            }
                            None => Prompt::Bare { prompt },
                        };

                        ands.push(prompt);
                    }
                    Keyword::But => {
                        self.consume_token(tokens);

                        let prompt = if let Some((contents, _)) = line.contents {
                            contents.to_string()
                        } else {
                            return self.make_error(ErrorKind::InvalidBareKeyword(Keyword::But));
                        };

                        let prompt = match self.peek_input(tokens).ok() {
                            Some(input) => {
                                self.consume_token(tokens);
                                Prompt::WithInput { prompt, input }
                            }
                            None => Prompt::Bare { prompt },
                        };

                        buts.push(prompt);
                    }
                    Keyword::Asterisk => panic!("Placeholder not supported yet"),
                    _ => break,
                }
            }

            steps.push(Step {
                ty: step_type,
                ands,
                buts,
            });
        }

        Ok(steps)
    }

    fn match_kw_line<'t, T>(&mut self, tokens: &mut Peekable<T>) -> Result<Line<'t>, Error<'a>>
    where
        T: Iterator<Item = Token<'t>>,
        't: 'a,
    {
        let line = self.peek_kw_line(tokens)?;
        self.consume_token(tokens);
        Ok(line)
    }

    fn peek_kw_line<'t, T>(&mut self, tokens: &mut Peekable<T>) -> Result<Line<'t>, Error<'a>>
    where
        T: Iterator<Item = Token<'t>>,
        't: 'a,
    {
        while let Some(token) = Self::peek_token(tokens) {
            let line = match token {
                TokenKind::Line(line) => line,
                TokenKind::Comment(_) => {
                    self.consume_token(tokens);
                    continue;
                }
                _ => {
                    let got = self.consume_token(tokens).unwrap();
                    return self.make_error(ErrorKind::Expected {
                        wanted: &["`Line`"],
                        got,
                    });
                }
            };

            if let Some(line) = self.parse_line(line)? {
                // Ignore lines with incorrect colon placement
                match (line.keyword, line.has_kw_colon) {
                    (
                        Keyword::Feature
                        | Keyword::Example
                        | Keyword::Background
                        | Keyword::ScenarioOutline
                        | Keyword::Examples,
                        true,
                    )
                    | (
                        Keyword::Given
                        | Keyword::When
                        | Keyword::Then
                        | Keyword::And
                        | Keyword::But
                        | Keyword::Asterisk,
                        false,
                    ) => {}
                    _ => {
                        self.consume_token(tokens);
                        continue;
                    }
                }

                return Ok(line);
            } else {
                self.consume_token(tokens);
            }
        }

        self.make_error(ErrorKind::UnexpectedEof {
            wanted: &["`Line`"],
        })
    }

    fn peek_input<'t, T>(&mut self, tokens: &mut Peekable<T>) -> Result<StepInput, Error>
    where
        T: Iterator<Item = Token<'t>>,
        't: 'a,
    {
        while let Some(token) = Self::peek_token(tokens) {
            let input = match token {
                TokenKind::Comment(_) => {
                    self.consume_token(tokens);
                    continue;
                }
                TokenKind::DataTable(table) => StepInput::Table(table.clone()),
                TokenKind::DocString(string) => StepInput::String(string.clone()),
                _ => {
                    let got = self.consume_token(tokens).unwrap();
                    return self.make_error(ErrorKind::Expected {
                        wanted: &["`DocString", "`DataTable`"],
                        got,
                    });
                }
            };

            return Ok(input);
        }

        self.make_error(ErrorKind::UnexpectedEof {
            wanted: &["`StepInput`"],
        })
    }

    fn parse_line<'l>(&mut self, line: &'l str) -> Result<Option<Line<'l>>, Error<'a>> {
        if line.is_empty() {
            return Ok(None);
        }

        let content_lower = line.to_lowercase();

        let (keyword, contents) = if let Some(keyword) =
            Keyword::combinations().iter().find_map(|(kw, text)| {
                if content_lower.starts_with(text) {
                    Some((*kw, &line[text.bytes().len()..]))
                } else {
                    None
                }
            }) {
            keyword
        } else {
            return self.make_error(ErrorKind::InvalidKeyword);
        };

        let (kw_colon, contents) = if contents.chars().next() == Some(':') {
            (true, &contents[1..])
        } else {
            (false, contents)
        };

        let contents = contents.trim();

        let contents = if contents.ends_with(':') {
            let contents = &contents[..contents.len() - 1];
            if contents.is_empty() {
                None
            } else {
                Some((contents, true))
            }
        } else if contents.is_empty() {
            None
        } else {
            Some((contents, false))
        };

        Ok(Some(Line {
            keyword,
            has_kw_colon: kw_colon,
            contents,
        }))
    }
}
