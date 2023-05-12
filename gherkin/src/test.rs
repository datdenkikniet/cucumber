use crate::{Block, Example, Feature, Prompt, Step, StepInput, StepType, Table};

#[test]
fn empty() {
    assert!(crate::parse_feature("").is_err());
}

const KIND_OF_EMPTY: &str = r#"
Feature:
    Example:
        Given nothing
        When the base is empty
        Then nothing happens
"#;

#[test]
fn kind_of_emtpy() {
    let parsed = crate::parse_feature(KIND_OF_EMPTY).unwrap();

    let expected = Feature {
        language: "en".into(),
        name: None,
        freeform_text: None,
        background_and: Vec::new(),
        background_but: Vec::new(),
        contents: vec![Block::Example(Example {
            name: None,
            steps: vec![
                Step {
                    ty: StepType::Given,
                    ands: vec![Prompt::Bare {
                        prompt: "nothing".into(),
                    }],
                    buts: Vec::new(),
                },
                Step {
                    ty: StepType::When,
                    ands: vec![Prompt::Bare {
                        prompt: "the base is empty".into(),
                    }],
                    buts: Vec::new(),
                },
                Step {
                    ty: StepType::Then,
                    ands: vec![Prompt::Bare {
                        prompt: "nothing happens".into(),
                    }],
                    buts: Vec::new(),
                },
            ],
        })],
    };

    assert_eq!(expected, parsed);
}

const ALL: &str = r#"
#language: gb

Feature: Basic
    This is some freeform text
    Hello there

    Background:
        Given a bunch of battle droids
        And a lot of hope
        But little time

    Example: To the rescue!
        Given an empty table
        and the following text:
        """
        General kenobi
        """
        When the rebels are extracted
        Then we are all home safe
        And this table should be output:
        | Header1 | Header2 |
        | Value1  | Value2  |
        but not all of us survived   
"#;

#[test]
fn parse_basic() {
    let parsed = crate::parse_feature(ALL).unwrap();

    let expected = Feature {
        name: Some("Basic".into()),
        language: "gb".into(),
        freeform_text: Some("This is some freeform text\nHello there".into()),
        background_and: vec![
            Prompt::Bare {
                prompt: "a bunch of battle droids".into(),
            },
            Prompt::Bare {
                prompt: "a lot of hope".into(),
            },
        ],
        background_but: vec![Prompt::Bare {
            prompt: "little time".into(),
        }],
        contents: vec![Block::Example(Example {
            name: Some("To the rescue!".into()),
            steps: vec![
                Step {
                    ty: StepType::Given,
                    ands: vec![
                        Prompt::Bare {
                            prompt: "an empty table".into(),
                        },
                        Prompt::WithInput {
                            prompt: "the following text".into(),
                            input: StepInput::String("General kenobi\n".into()),
                        },
                    ],
                    buts: vec![],
                },
                Step {
                    ty: StepType::When,
                    ands: vec![Prompt::Bare {
                        prompt: "the rebels are extracted".into(),
                    }],
                    buts: vec![],
                },
                Step {
                    ty: StepType::Then,
                    ands: vec![
                        Prompt::Bare {
                            prompt: "we are all home safe".into(),
                        },
                        Prompt::WithInput {
                            prompt: "this table should be output".into(),
                            input: StepInput::Table(Table {
                                header: vec!["Header1".into(), "Header2".into()],
                                rows: vec![vec!["Value1".into(), "Value2".into()]],
                            }),
                        },
                    ],
                    buts: vec![Prompt::Bare {
                        prompt: "not all of us survived".into(),
                    }],
                },
            ],
        })],
    };

    assert_eq!(parsed, expected);
}
