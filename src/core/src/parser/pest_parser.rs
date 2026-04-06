use super::pest_parse_tree::{
    CommentPrefix, PestAssertionKind, PestAssertionLine, PestAssertionValue, PestBlankLine,
    PestBodyLine, PestCommentLine, PestConditionExpression, PestDirectiveKind, PestDirectiveLine,
    PestHeaderLine, PestHttpFile, PestLine, PestLineKind, PestRequestLine, PestScriptBlock,
    PestTimeoutLiteral, PestVariableLine,
};
use anyhow::{Context, Result, anyhow, bail};
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/http-file.pest"]
pub(crate) struct HttpFilePestParser;

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn parse_http_content_to_pest_tree(content: &str) -> Result<PestHttpFile> {
    let file = parse_root_pair(content)?;
    let lines = file
        .into_inner()
        .filter(|pair| pair.as_rule() != Rule::EOI)
        .map(build_line)
        .collect::<Result<Vec<_>>>()?;

    Ok(PestHttpFile { lines })
}

fn parse_root_pair(content: &str) -> Result<Pair<'_, Rule>> {
    let mut pairs = HttpFilePestParser::parse(Rule::HttpFile, content)
        .map_err(|error| anyhow!("Failed to parse HTTP file with pest: {error}"))?;

    let file = pairs
        .next()
        .context("pest parser did not yield an HttpFile root pair")?;

    if pairs.next().is_some() {
        bail!("pest parser yielded multiple HttpFile root pairs");
    }

    Ok(file)
}

fn build_line(pair: Pair<'_, Rule>) -> Result<PestLine> {
    let line_number = pair.as_span().start_pos().line_col().0;
    let raw = trim_line_ending(pair.as_str());
    let kind = match pair.as_rule() {
        Rule::BlankLine => PestLineKind::Blank(PestBlankLine {
            whitespace: raw.clone(),
        }),
        Rule::NameDirective => PestLineKind::Directive(build_name_directive(pair)?),
        Rule::TimeoutDirective => PestLineKind::Directive(build_timeout_directive(pair)?),
        Rule::ConnectionTimeoutDirective => {
            PestLineKind::Directive(build_connection_timeout_directive(pair)?)
        }
        Rule::DependsOnDirective => PestLineKind::Directive(build_depends_on_directive(pair)?),
        Rule::IfDirective => PestLineKind::Directive(build_if_directive(pair)?),
        Rule::IfNotDirective => PestLineKind::Directive(build_if_not_directive(pair)?),
        Rule::PreDelayDirective => PestLineKind::Directive(build_pre_delay_directive(pair)?),
        Rule::PostDelayDirective => PestLineKind::Directive(build_post_delay_directive(pair)?),
        Rule::HashComment | Rule::SlashComment => PestLineKind::Comment(build_comment(pair)?),
        Rule::CommentLine => {
            let comment = pair
                .into_inner()
                .next()
                .context("comment line did not contain a concrete comment")?;
            PestLineKind::Comment(build_comment(comment)?)
        }
        Rule::VariableLine => PestLineKind::Variable(build_variable_line(pair)?),
        Rule::AssertionLine => PestLineKind::Assertion(build_assertion_line(pair)?),
        Rule::RequestLine => PestLineKind::Request(build_request_line(pair)?),
        Rule::HeaderLine => PestLineKind::Header(build_header_line(pair)?),
        Rule::BodyLine => PestLineKind::Body(build_body_line(pair)?),
        Rule::IgnoredScriptBlock => PestLineKind::IgnoredScriptBlock(build_script_block(pair)?),
        other => bail!("unexpected top-level pest rule: {other:?}"),
    };

    Ok(PestLine {
        line_number,
        raw,
        kind,
    })
}

fn build_name_directive(pair: Pair<'_, Rule>) -> Result<PestDirectiveLine> {
    let (prefix, mut inner) = split_directive(pair)?;
    let name = required_next(&mut inner, Rule::NameText, "name directive value")?
        .as_str()
        .to_string();

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::Name(name),
    })
}

fn build_timeout_directive(pair: Pair<'_, Rule>) -> Result<PestDirectiveLine> {
    let (prefix, mut inner) = split_directive(pair)?;
    let timeout = build_timeout_literal(required_next(
        &mut inner,
        Rule::TimeoutValue,
        "timeout directive value",
    )?)?;

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::Timeout(timeout),
    })
}

fn build_connection_timeout_directive(pair: Pair<'_, Rule>) -> Result<PestDirectiveLine> {
    let (prefix, mut inner) = split_directive(pair)?;
    let timeout = build_timeout_literal(required_next(
        &mut inner,
        Rule::TimeoutValue,
        "connection-timeout directive value",
    )?)?;

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::ConnectionTimeout(timeout),
    })
}

fn build_depends_on_directive(pair: Pair<'_, Rule>) -> Result<PestDirectiveLine> {
    let (prefix, mut inner) = split_directive(pair)?;
    let depends_on = required_next(&mut inner, Rule::ReferenceName, "dependsOn directive value")?
        .as_str()
        .to_string();

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::DependsOn(depends_on),
    })
}

fn build_if_directive(pair: Pair<'_, Rule>) -> Result<PestDirectiveLine> {
    let (prefix, mut inner) = split_directive(pair)?;
    let condition = build_condition_expression(required_next(
        &mut inner,
        Rule::ConditionExpression,
        "@if directive condition",
    )?)?;

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::If(condition),
    })
}

fn build_if_not_directive(pair: Pair<'_, Rule>) -> Result<PestDirectiveLine> {
    let (prefix, mut inner) = split_directive(pair)?;
    let condition = build_condition_expression(required_next(
        &mut inner,
        Rule::ConditionExpression,
        "@if-not directive condition",
    )?)?;

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::IfNot(condition),
    })
}

fn build_pre_delay_directive(pair: Pair<'_, Rule>) -> Result<PestDirectiveLine> {
    let (prefix, mut inner) = split_directive(pair)?;
    let delay = required_next(&mut inner, Rule::Digits, "@pre-delay directive value")?
        .as_str()
        .to_string();

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::PreDelay(delay),
    })
}

fn build_post_delay_directive(pair: Pair<'_, Rule>) -> Result<PestDirectiveLine> {
    let (prefix, mut inner) = split_directive(pair)?;
    let delay = required_next(&mut inner, Rule::Digits, "@post-delay directive value")?
        .as_str()
        .to_string();

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::PostDelay(delay),
    })
}

fn build_timeout_literal(pair: Pair<'_, Rule>) -> Result<PestTimeoutLiteral> {
    let mut inner = pair.into_inner();
    let amount = required_next(&mut inner, Rule::Digits, "timeout amount")?
        .as_str()
        .to_string();
    let unit = optional_next(&mut inner, Rule::TimeoutUnit, "timeout unit")?
        .map(|pair| pair.as_str().to_string());

    Ok(PestTimeoutLiteral { amount, unit })
}

fn build_condition_expression(pair: Pair<'_, Rule>) -> Result<PestConditionExpression> {
    let condition = pair
        .into_inner()
        .next()
        .context("condition expression did not contain a concrete condition")?;

    match condition.as_rule() {
        Rule::StatusCondition => build_status_condition(condition),
        Rule::BodyCondition => build_body_condition(condition),
        other => bail!("unexpected condition expression rule: {other:?}"),
    }
}

fn build_status_condition(pair: Pair<'_, Rule>) -> Result<PestConditionExpression> {
    let mut inner = pair.into_inner();
    let request_name = required_next(&mut inner, Rule::ReferenceName, "status condition request")?
        .as_str()
        .to_string();
    let operator_or_expected = inner
        .next()
        .context("status condition did not contain an expected value")?;

    let (has_equality_operator, expected) = if operator_or_expected.as_rule() == Rule::EqualityOp {
        let expected = required_next(&mut inner, Rule::ExpectedText, "status condition expected")?
            .as_str()
            .to_string();
        (true, expected)
    } else {
        ensure_rule(
            &operator_or_expected,
            Rule::ExpectedText,
            "status condition expected",
        )?;
        (false, operator_or_expected.as_str().to_string())
    };

    Ok(PestConditionExpression::Status {
        request_name,
        has_equality_operator,
        expected,
    })
}

fn build_body_condition(pair: Pair<'_, Rule>) -> Result<PestConditionExpression> {
    let mut inner = pair.into_inner();
    let request_name = required_next(&mut inner, Rule::ReferenceName, "body condition request")?
        .as_str()
        .to_string();
    let path = required_next(&mut inner, Rule::ConditionPath, "body condition path")?
        .as_str()
        .to_string();
    let operator_or_expected = inner
        .next()
        .context("body condition did not contain an expected value")?;

    let (has_equality_operator, expected) = if operator_or_expected.as_rule() == Rule::EqualityOp {
        let expected = required_next(&mut inner, Rule::ExpectedText, "body condition expected")?
            .as_str()
            .to_string();
        (true, expected)
    } else {
        ensure_rule(
            &operator_or_expected,
            Rule::ExpectedText,
            "body condition expected",
        )?;
        (false, operator_or_expected.as_str().to_string())
    };

    Ok(PestConditionExpression::Body {
        request_name,
        path,
        has_equality_operator,
        expected,
    })
}

fn build_comment(pair: Pair<'_, Rule>) -> Result<PestCommentLine> {
    let raw = trim_line_ending(pair.as_str());
    let (prefix, text) = match pair.as_rule() {
        Rule::HashComment => (
            CommentPrefix::Hash,
            raw.strip_prefix('#').unwrap_or(&raw).to_string(),
        ),
        Rule::SlashComment => (
            CommentPrefix::SlashSlash,
            raw.strip_prefix("//").unwrap_or(&raw).to_string(),
        ),
        other => bail!("unexpected comment rule: {other:?}"),
    };

    Ok(PestCommentLine { prefix, text })
}

fn build_variable_line(pair: Pair<'_, Rule>) -> Result<PestVariableLine> {
    let mut inner = pair.into_inner();
    let name = required_next(&mut inner, Rule::VariableName, "variable name")?
        .as_str()
        .to_string();
    let value = optional_next(&mut inner, Rule::VariableValue, "variable value")?
        .map(|pair| pair.as_str().to_string())
        .unwrap_or_default();

    Ok(PestVariableLine { name, value })
}

fn build_assertion_line(pair: Pair<'_, Rule>) -> Result<PestAssertionLine> {
    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .context("assertion line did not contain any assertion tokens")?;

    let (uses_prompt_prefix, keyword_pair) = if first.as_rule() == Rule::AssertionPrefix {
        (
            true,
            required_next(&mut inner, Rule::AssertionKeyword, "assertion keyword")?,
        )
    } else {
        ensure_rule(&first, Rule::AssertionKeyword, "assertion keyword")?;
        (false, first)
    };

    let kind = match keyword_pair.as_str() {
        "EXPECTED_RESPONSE_STATUS" => PestAssertionKind::Status,
        "EXPECTED_RESPONSE_BODY" => PestAssertionKind::Body,
        "EXPECTED_RESPONSE_HEADERS" => PestAssertionKind::Headers,
        other => bail!("unexpected assertion keyword: {other}"),
    };

    let value_pair = required_next(&mut inner, Rule::AssertionValue, "assertion value")?;
    let literal = value_pair
        .into_inner()
        .next()
        .context("assertion value did not contain a literal")?;
    let value = match literal.as_rule() {
        Rule::QuotedText => PestAssertionValue::DoubleQuoted(strip_wrapping_quotes(
            literal.as_str(),
            '"',
            "double-quoted assertion value",
        )?),
        Rule::ExpectedText => PestAssertionValue::Raw(literal.as_str().to_string()),
        other => bail!("unexpected assertion value rule: {other:?}"),
    };

    Ok(PestAssertionLine {
        uses_prompt_prefix,
        kind,
        value,
    })
}

fn build_request_line(pair: Pair<'_, Rule>) -> Result<PestRequestLine> {
    let mut inner = pair.into_inner();
    let method = required_next(&mut inner, Rule::RequestMethod, "request method")?
        .as_str()
        .to_string();
    let target = required_next(&mut inner, Rule::RequestTarget, "request target")?
        .as_str()
        .to_string();
    let mut http_version = None;
    let mut trailing_tokens = Vec::new();

    if let Some(tail) = optional_next(&mut inner, Rule::RequestLineTail, "request line tail")? {
        for token in tail.into_inner() {
            match token.as_rule() {
                Rule::HttpVersion => http_version = Some(token.as_str().to_string()),
                Rule::IgnoredRequestToken => trailing_tokens.push(token.as_str().to_string()),
                other => bail!("unexpected request line tail rule: {other:?}"),
            }
        }
    }

    Ok(PestRequestLine {
        method,
        target,
        http_version,
        trailing_tokens,
    })
}

fn build_header_line(pair: Pair<'_, Rule>) -> Result<PestHeaderLine> {
    let mut inner = pair.into_inner();
    let name = required_next(&mut inner, Rule::HeaderName, "header name")?
        .as_str()
        .to_string();
    let value = optional_next(&mut inner, Rule::HeaderValue, "header value")?
        .map(|pair| pair.as_str().to_string())
        .unwrap_or_default();

    Ok(PestHeaderLine { name, value })
}

fn build_body_line(pair: Pair<'_, Rule>) -> Result<PestBodyLine> {
    let text = pair
        .into_inner()
        .next()
        .map(|line| trim_line_ending(line.as_str()))
        .unwrap_or_default();

    Ok(PestBodyLine { text })
}

fn build_script_block(pair: Pair<'_, Rule>) -> Result<PestScriptBlock> {
    let mut inner = pair.into_inner();
    let start = trim_line_ending(
        required_next(&mut inner, Rule::ScriptBlockStart, "script block start")?.as_str(),
    );
    let mut lines = Vec::new();
    let mut end = None;

    for part in inner {
        match part.as_rule() {
            Rule::ScriptBlockLine => {
                let body = part
                    .into_inner()
                    .next()
                    .map(|line| trim_line_ending(line.as_str()))
                    .unwrap_or_default();
                lines.push(body);
            }
            Rule::ScriptBlockEnd => {
                end = Some(trim_line_ending(part.as_str()));
            }
            other => bail!("unexpected script block rule: {other:?}"),
        }
    }

    Ok(PestScriptBlock { start, lines, end })
}

fn split_directive(pair: Pair<'_, Rule>) -> Result<(CommentPrefix, Pairs<'_, Rule>)> {
    let mut inner = pair.into_inner();
    let prefix = parse_directive_prefix(required_next(
        &mut inner,
        Rule::DirectivePrefix,
        "directive prefix",
    )?)?;
    Ok((prefix, inner))
}

fn parse_directive_prefix(pair: Pair<'_, Rule>) -> Result<CommentPrefix> {
    let prefix = pair
        .into_inner()
        .next()
        .context("directive prefix did not contain a concrete prefix")?;

    match prefix.as_rule() {
        Rule::HashDirectivePrefix => Ok(CommentPrefix::Hash),
        Rule::SlashDirectivePrefix => Ok(CommentPrefix::SlashSlash),
        other => bail!("unexpected directive prefix rule: {other:?}"),
    }
}

fn required_next<'a>(
    pairs: &mut Pairs<'a, Rule>,
    expected: Rule,
    context: &str,
) -> Result<Pair<'a, Rule>> {
    let pair = pairs
        .next()
        .with_context(|| format!("missing {context} ({expected:?})"))?;
    ensure_rule(&pair, expected, context)?;
    Ok(pair)
}

fn optional_next<'a>(
    pairs: &mut Pairs<'a, Rule>,
    expected: Rule,
    context: &str,
) -> Result<Option<Pair<'a, Rule>>> {
    match pairs.next() {
        Some(pair) => {
            ensure_rule(&pair, expected, context)?;
            Ok(Some(pair))
        }
        None => Ok(None),
    }
}

fn ensure_rule(pair: &Pair<'_, Rule>, expected: Rule, context: &str) -> Result<()> {
    if pair.as_rule() == expected {
        Ok(())
    } else {
        bail!(
            "expected {context} to be {expected:?}, found {:?}",
            pair.as_rule()
        )
    }
}

fn strip_wrapping_quotes(value: &str, quote: char, context: &str) -> Result<String> {
    if value.len() >= 2 && value.starts_with(quote) && value.ends_with(quote) {
        Ok(value[1..value.len() - 1].to_string())
    } else {
        bail!("expected {context} to be wrapped in {quote}");
    }
}

fn trim_line_ending(text: &str) -> String {
    text.strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .or_else(|| text.strip_suffix('\r'))
        .unwrap_or(text)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::super::pest_parse_tree::{
        PestConditionExpression, PestDirectiveKind, PestLineKind, PestRequestLine,
    };
    use super::*;

    #[test]
    fn builds_line_oriented_parse_tree_from_mixed_http_file_content() {
        let input = r#"# @name login
// @if auth.response.status == 200
POST https://api.example.com/login HTTP/1.1 trailing-token
Content-Type: application/json

> EXPECTED_RESPONSE_BODY "ok""#;

        let tree = parse_http_content_to_pest_tree(input).expect("parse tree should build");

        assert_eq!(tree.lines.len(), 6);
        assert_eq!(
            tree.lines[0],
            PestLine {
                line_number: 1,
                raw: "# @name login".to_string(),
                kind: PestLineKind::Directive(PestDirectiveLine {
                    prefix: CommentPrefix::Hash,
                    kind: PestDirectiveKind::Name("login".to_string()),
                }),
            }
        );
        assert_eq!(
            tree.lines[1].kind,
            PestLineKind::Directive(PestDirectiveLine {
                prefix: CommentPrefix::SlashSlash,
                kind: PestDirectiveKind::If(PestConditionExpression::Status {
                    request_name: "auth".to_string(),
                    has_equality_operator: true,
                    expected: "200".to_string(),
                }),
            })
        );
        assert_eq!(
            tree.lines[2].kind,
            PestLineKind::Request(PestRequestLine {
                method: "POST".to_string(),
                target: "https://api.example.com/login".to_string(),
                http_version: Some("HTTP/1.1".to_string()),
                trailing_tokens: vec!["trailing-token".to_string()],
            })
        );
        assert_eq!(
            tree.lines[3].kind,
            PestLineKind::Header(PestHeaderLine {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            })
        );
        assert_eq!(
            tree.lines[4].kind,
            PestLineKind::Blank(PestBlankLine {
                whitespace: String::new(),
            })
        );
        assert_eq!(
            tree.lines[5].kind,
            PestLineKind::Assertion(PestAssertionLine {
                uses_prompt_prefix: true,
                kind: PestAssertionKind::Body,
                value: PestAssertionValue::DoubleQuoted("ok".to_string()),
            })
        );
    }

    #[test]
    fn keeps_syntactic_classification_before_body_mode_semantics() {
        let input = r#"POST https://api.example.com/items
Content-Type: application/json

@token = abc
X-Trace-Id: 1234"#;

        let tree = parse_http_content_to_pest_tree(input).expect("parse tree should build");

        assert!(matches!(
            tree.lines[3].kind,
            PestLineKind::Variable(PestVariableLine { .. })
        ));
        assert!(matches!(
            tree.lines[4].kind,
            PestLineKind::Header(PestHeaderLine { .. })
        ));
    }

    #[test]
    fn groups_intellij_script_blocks_into_single_parse_tree_nodes() {
        let input = r#"> {% client.global.set("token", "abc")
console.log("still ignored")
%}
GET https://api.example.com/profile"#;

        let tree = parse_http_content_to_pest_tree(input).expect("parse tree should build");

        assert_eq!(tree.lines.len(), 2);
        assert_eq!(
            tree.lines[0].kind,
            PestLineKind::IgnoredScriptBlock(PestScriptBlock {
                start: r#"> {% client.global.set("token", "abc")"#.to_string(),
                lines: vec!(r#"console.log("still ignored")"#.to_string()),
                end: "%}".to_string().into(),
            })
        );
        assert_eq!(
            tree.lines[1].kind,
            PestLineKind::Request(PestRequestLine {
                method: "GET".to_string(),
                target: "https://api.example.com/profile".to_string(),
                http_version: None,
                trailing_tokens: Vec::new(),
            })
        );
    }

    #[test]
    fn parses_request_variable_references() {
        let mut pairs = HttpFilePestParser::parse(
            Rule::RequestVariableReference,
            "{{login.response.body.$.token}}",
        )
        .expect("request variable syntax should parse");

        assert_eq!(
            pairs.next().expect("request variable pair").as_rule(),
            Rule::RequestVariableReference
        );
    }

    #[test]
    fn parses_builtin_function_calls() {
        let mut pairs = HttpFilePestParser::parse(Rule::FunctionCall, "upper('TeSt')")
            .expect("function syntax should parse");

        assert_eq!(
            pairs.next().expect("function pair").as_rule(),
            Rule::FunctionCall
        );
    }
}
