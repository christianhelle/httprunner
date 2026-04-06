#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestHttpFile {
    pub lines: Vec<PestLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestLine {
    pub line_number: usize,
    pub raw: String,
    pub kind: PestLineKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PestLineKind {
    Blank(PestBlankLine),
    Directive(PestDirectiveLine),
    Comment(PestCommentLine),
    Variable(PestVariableLine),
    Assertion(PestAssertionLine),
    Request(PestRequestLine),
    Header(PestHeaderLine),
    Body(PestBodyLine),
    IgnoredScriptBlock(PestScriptBlock),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestBlankLine {
    pub whitespace: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestDirectiveLine {
    pub prefix: CommentPrefix,
    pub kind: PestDirectiveKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CommentPrefix {
    Hash,
    SlashSlash,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PestDirectiveKind {
    Name(String),
    Timeout(PestTimeoutLiteral),
    ConnectionTimeout(PestTimeoutLiteral),
    DependsOn(String),
    If(PestConditionExpression),
    IfNot(PestConditionExpression),
    PreDelay(String),
    PostDelay(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestTimeoutLiteral {
    pub amount: String,
    pub unit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PestConditionExpression {
    Status {
        request_name: String,
        has_equality_operator: bool,
        expected: String,
    },
    Body {
        request_name: String,
        path: String,
        has_equality_operator: bool,
        expected: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestCommentLine {
    pub prefix: CommentPrefix,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestVariableLine {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestAssertionLine {
    pub uses_prompt_prefix: bool,
    pub kind: PestAssertionKind,
    pub value: PestAssertionValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PestAssertionKind {
    Status,
    Body,
    Headers,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PestAssertionValue {
    Raw(String),
    DoubleQuoted(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestRequestLine {
    pub method: String,
    pub target: String,
    pub http_version: Option<String>,
    pub trailing_tokens: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestHeaderLine {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestBodyLine {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PestScriptBlock {
    pub start: String,
    pub lines: Vec<String>,
    pub end: Option<String>,
}
