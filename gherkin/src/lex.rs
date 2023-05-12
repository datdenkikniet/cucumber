use std::{iter::Peekable, str::Lines};

use crate::Table;

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    StringInconsistentIndent,
    UnclosedDocString,
}

#[derive(Debug, Clone)]
pub struct Error<'a> {
    pub line_num: usize,
    pub kind: ErrorKind,
    pub text: &'a str,
}

#[derive(Debug, Clone)]
pub enum TokenKind<'a> {
    Line(&'a str),
    Comment(&'a str),
    DocString(String),
    DataTable(Table),
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    kind: TokenKind<'a>,
    start_line: usize,
    end_line: usize,
    text: &'a str,
}

impl<'a> Token<'a> {
    pub fn kind(&self) -> &TokenKind<'a> {
        &self.kind
    }

    pub fn start_line(&self) -> usize {
        self.start_line
    }

    pub fn end_line(&self) -> usize {
        self.end_line
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn extract(&self) -> String {
        self.text
            .lines()
            .skip(self.start_line)
            .take(self.end_line - self.start_line)
            .collect()
    }
}

pub fn lex(input: &str) -> Result<Vec<Token>, Error> {
    Lexer::lex(input)
}

struct Lexer<'a> {
    text: &'a str,
    current_line: usize,
}

impl<'a> Lexer<'a> {
    fn make_error(&mut self, kind: ErrorKind) -> Error<'a> {
        Error {
            kind,
            line_num: self.current_line,
            text: self.text,
        }
    }

    fn next_line<'l>(&mut self, lines: &mut Peekable<Lines<'l>>) -> Option<&'l str> {
        if let Some(line) = lines.next() {
            self.current_line += 1;
            Some(line)
        } else {
            None
        }
    }

    fn lex(text: &'a str) -> Result<Vec<Token>, Error> {
        let mut me = Self {
            text,
            current_line: 0,
        };

        let mut lines = text.lines().peekable();
        let lines = &mut lines;
        let mut tokens = Vec::new();

        loop {
            let token = if let Some(token) = me.tokenize_docstring(lines)? {
                token
            } else if let Some(table) = me.tokenize_table(lines) {
                table
            } else if let Some(line) = me.next_line(lines) {
                let line_num = me.current_line - 1;

                let trimmed = line.trim();
                let kind = if trimmed.starts_with('#') {
                    TokenKind::Comment(&trimmed[1..])
                } else {
                    TokenKind::Line(trimmed)
                };

                Token {
                    kind,
                    start_line: line_num,
                    end_line: line_num + 1,
                    text: text,
                }
            } else {
                break;
            };

            tokens.push(token);
        }

        Ok(tokens)
    }

    fn tokenize_docstring<'l>(
        &mut self,
        lines: &mut Peekable<Lines<'l>>,
    ) -> Result<Option<Token<'l>>, Error<'a>>
    where
        'a: 'l,
    {
        let start_line = match lines.peek() {
            Some(line) => line,
            None => return Ok(None),
        };

        if start_line.trim_start().starts_with("\"\"\"") {
            let start_line_idx = self.current_line;
            let start_line = self.next_line(lines).unwrap();
            let start_indent: String = start_line
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect();
            let start_indent_count = start_indent.chars().count();

            let mut string = String::new();

            while let Some(line) = self.next_line(lines) {
                if line.trim().starts_with("\"\"\"") {
                    let end_line = self.current_line;

                    let token = Token {
                        kind: TokenKind::DocString(string),
                        start_line: start_line_idx,
                        end_line,
                        text: self.text,
                    };

                    return Ok(Some(token));
                }

                if !line.starts_with(&start_indent) {
                    return Err(self.make_error(ErrorKind::StringInconsistentIndent));
                }

                let line_chars = line.chars().skip(start_indent_count);

                string.extend(line_chars);
                string.push('\n');
            }

            Err(self.make_error(ErrorKind::UnclosedDocString))
        } else {
            Ok(None)
        }
    }

    fn tokenize_table<'l>(&mut self, lines: &mut Peekable<Lines<'l>>) -> Option<Token<'l>>
    where
        'a: 'l,
    {
        let start_line = self.current_line;

        let first_line = match lines.peek() {
            Some(line) => line,
            None => return None,
        };

        let header: Vec<String> = if first_line.trim_start().starts_with('|') {
            let first_line = self.next_line(lines).unwrap();
            TableRowIter::new(first_line).collect()
        } else {
            return None;
        };

        let mut rows = Vec::new();
        while let Some(line) = lines.peek() {
            if !line.trim_start().starts_with('|') {
                break;
            }

            let row = TableRowIter::new(self.next_line(lines).unwrap()).collect();
            rows.push(row);
        }

        let end_line = self.current_line;

        Some(Token {
            kind: TokenKind::DataTable(Table { header, rows }),
            start_line,
            end_line,
            text: self.text,
        })
    }
}

struct TableRowIter<'a> {
    inner: Peekable<std::iter::Skip<std::str::Split<'a, char>>>,
}

impl<'a> TableRowIter<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            inner: input.split('|').skip(1).peekable(),
        }
    }
}

impl<'a> Iterator for TableRowIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO escape characters and stuff
        let next = self.inner.next().map(|s| s.trim().into());
        if self.inner.peek().is_some() {
            next
        } else {
            None
        }
    }
}
