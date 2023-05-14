use crate::{parser::ParserInner, DataTable, Parser};

#[test]
pub fn data_table() {
    const DATA_TABLE: &str = r#"
            | Header 1 | Header 2 | Header 3 |
            | Value 11 | Value 12 | Value 13 |
            | Value 21 | Value 22 | Value 23 |
        "#;

    let mut inner = ParserInner::new(DATA_TABLE);
    let datatable = inner.try_datatable().unwrap().unwrap();
    let expected = DataTable::new_populated(
        vec!["Header 1".into(), "Header 2".into(), "Header 3".into()],
        vec![
            vec!["Value 11".into(), "Value 12".into(), "Value 13".into()],
            vec!["Value 21".into(), "Value 22".into(), "Value 23".into()],
        ],
    )
    .unwrap();

    assert_eq!(datatable, expected);
}

#[test]
pub fn doc_string() {
    const DOC_STRING: &str = r#"
    """

    This is my doc string
    There are many like it
    But this one is mine

    :)

    """
    "#;

    let mut inner = ParserInner::new(DOC_STRING);
    let doc_string = inner.try_docstring().unwrap().unwrap();

    assert_eq!(
        doc_string,
        "This is my doc string\nThere are many like it\nBut this one is mine\n\n:)"
    );
}

#[test]
pub fn basic() {
    const KIND_OF_EMPTY: &str = r#"
    Feature: a feature
        Hehe a freeform text!
        Uh oh my indentation!
        Background:
            Given some flour
            And some eggs
    
        Example:
            Given the following text:
            """
            Hello there
            General Kenobi
            """
            When the base is empty
            Then nothing happens

        Scenario Outline:
            Given <count> biscuits
            And <count> cups of tea
            Then I should be stuffed
            @1-to-3
            Scenarios:
                |count|
                | 1 |
                | 2 |
                | 3 |

            @4-to-6
            Scenarios:
                |count|
                | 4 |
                | 5 |
                | 6 |
    "#;

    let feature = Parser::parse_feature(KIND_OF_EMPTY).unwrap();

    panic!("{feature:#?}");
}
