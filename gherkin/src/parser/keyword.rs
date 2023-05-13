#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Feature,
    Scenario,
    Background,
    ScenarioOutline,
    Scenarios,
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
            | Keyword::Scenario
            | Keyword::Background
            | Keyword::ScenarioOutline
            | Keyword::Scenarios => true,
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
            (Self::Scenarios, "examples"),
            (Self::Scenarios, "scenarios"),
            (Self::ScenarioOutline, "scenario outline"),
            (Self::ScenarioOutline, "scenario template"),
            (Self::Feature, "feature"),
            (Self::Scenario, "example"),
            (Self::Scenario, "scenario"),
            (Self::Background, "background"),
            (Self::Given, "given"),
            (Self::When, "when"),
            (Self::Then, "then"),
            (Self::And, "and"),
            (Self::But, "but"),
            (Self::Asterisk, "*"),
        ]
    }

    pub fn parse(line: &str, strip_trailing_colon: bool) -> Option<(Self, &str, &str, bool)> {
        let lowercase = line.to_ascii_lowercase();

        let (keyword, start) = Self::combinations()
            .iter()
            .find(|(_, pattern)| lowercase.starts_with(pattern))?;
        let start_len = start.len();

        let keyword_name = &line[..start_len];
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

        let leftover = if last_is_colon && strip_trailing_colon {
            &leftover[..leftover.len() - 1]
        } else {
            leftover
        };

        Some((*keyword, keyword_name, leftover, last_is_colon))
    }
}
