FormatSettings {
    blank_line_before_statement: BlankLineBeforeStatements::try_from(vec![
        BlankLineBeforeStatement::Phpdoc,
        BlankLineBeforeStatement::Throw,
        BlankLineBeforeStatement::Include,
        BlankLineBeforeStatement::YieldFrom,
        BlankLineBeforeStatement::Case,
        BlankLineBeforeStatement::Default,
        BlankLineBeforeStatement::Return,
    ])
    .unwrap(),
    empty_line_before_return: false,
    ..FormatSettings::default()
}
