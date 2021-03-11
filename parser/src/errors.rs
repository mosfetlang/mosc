/// The errors that parsers can throw.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ParserError {
    MultilineCommentWithoutEndToken,

    NumberWithSeparatorAfterPrefix,
    NumberWithoutDigitsAfterPrefix,

    MissingNameInVariableDeclaration,
    MissingAssignOperatorInVariableDeclaration,
    MissingExpressionInVariableDeclaration,

    MissingExpressionInReturnStatement,

    NotAMosfetFile,
    ExpectedEOFInFile,
    TwoStatementsInSameLineInFile,
}
