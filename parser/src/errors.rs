/// The errors that parsers can throw.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ParserError {
    NumberWithSeparatorAfterPrefix,
    NumberWithoutDigitsAfterPrefix,

    MissingVariableNameInVariableDeclaration,
    MissingAssignOperatorInVariableDeclaration,
    MissingExpressionInVariableDeclaration,

    MissingExpressionInReturnStatement,

    ExpectedEOFInFile,
    TwoStatementsInSameLineInFile,
}
