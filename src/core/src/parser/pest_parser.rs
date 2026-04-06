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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PestRawLineKind {
    Regular,
    IgnoredScriptBlock,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PestRawLine<'a> {
    pub line_number: usize,
    pub raw: &'a str,
    pub kind: PestRawLineKind,
}

#[derive(Debug, Clone)]
pub(crate) struct PestRawHttpFile<'a> {
    pub lines: Vec<PestRawLine<'a>>,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn parse_http_content_to_pest_tree(content: &str) -> Result<PestHttpFile> {
    let file = parse_root_pair(content)?;
    let mut lines = Vec::new();
    let mut next_line_number = 1;

    for pair in file.into_inner().filter(|pair| pair.as_rule() != Rule::EOI) {
        let physical_line_count = count_physical_lines(pair.as_str());
        lines.push(build_line(pair, next_line_number)?);
        next_line_number += physical_line_count;
    }

    Ok(PestHttpFile { lines })
}

/// Split `content` into raw lines without running the full pest grammar.
///
/// The semantic assembler (`assemble_raw_line`) already contains a complete
/// state machine for every line type including `IgnoredScriptBlock` detection
/// via `in_intellij_script`.  Running the PEG parser just to classify lines
/// doubles the work: pest scans every byte to build a parse tree that the
/// assembler immediately discards.
///
/// Zero-copy path: all `raw` fields are sub-slices of `content`; no heap
/// allocation beyond the `Vec` itself.
pub(crate) fn parse_http_content_to_pest_raw_file(content: &str) -> Result<PestRawHttpFile<'_>> {
    Ok(split_lines_fast(content))
}

fn split_lines_fast(content: &str) -> PestRawHttpFile<'_> {
    // `split_terminator` omits the trailing empty element produced by a
    // terminal '\n', which matches the behaviour of `str::lines()` and avoids
    // adding a spurious blank line at the end of the file.
    let mut lines = Vec::new();
    let mut line_number = 1usize;

    for raw in content.split_terminator('\n') {
        // Strip the '\r' so CRLF files are handled transparently.
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        lines.push(PestRawLine {
            line_number,
            raw,
            // All lines are Regular; the semantic assembler drives
            // IgnoredScriptBlock handling via its `in_intellij_script` flag.
            kind: PestRawLineKind::Regular,
        });
        line_number += 1;
    }

    PestRawHttpFile { lines }
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

fn build_line(pair: Pair<'_, Rule>, line_number: usize) -> Result<PestLine> {
    let raw = trim_line_ending(pair.as_str());
    let kind = match pair.as_rule() {
        Rule::BlankLine => PestLineKind::Blank(PestBlankLine {
            whitespace: raw.clone(),
        }),
        Rule::CommentOrDirectiveLine => build_comment_or_directive_line(&raw)?,
        Rule::VariableLine => PestLineKind::Variable(build_variable_line(&raw)?),
        Rule::AssertionLine => PestLineKind::Assertion(build_assertion_line(&raw)?),
        Rule::RequestLine => PestLineKind::Request(build_request_line(&raw)?),
        Rule::HeaderLine => PestLineKind::Header(build_header_line(&raw)?),
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

fn build_comment_or_directive_line(raw: &str) -> Result<PestLineKind> {
    let (_, body) = parse_comment_prefix(raw)?;
    let Some(directive_body) = body.strip_prefix('@') else {
        return Ok(PestLineKind::Comment(build_comment_from_raw(raw)?));
    };

    let Some(separator) = directive_body.find(char::is_whitespace) else {
        return Ok(PestLineKind::Comment(build_comment_from_raw(raw)?));
    };

    let directive_name = &directive_body[..separator];
    match directive_name {
        "name" | "timeout" | "connection-timeout" | "dependsOn" | "if" | "if-not" | "pre-delay"
        | "post-delay" => Ok(PestLineKind::Directive(build_directive_line(raw)?)),
        _ => Ok(PestLineKind::Comment(build_comment_from_raw(raw)?)),
    }
}

fn build_directive_line(raw: &str) -> Result<PestDirectiveLine> {
    let (_, directive_body) = parse_comment_prefix(raw)?;
    let directive_name = directive_body
        .split_whitespace()
        .next()
        .context("directive line did not contain a directive keyword")?;

    match directive_name {
        "@name" => build_name_directive(raw),
        "@timeout" => build_timeout_directive(raw),
        "@connection-timeout" => build_connection_timeout_directive(raw),
        "@dependsOn" => build_depends_on_directive(raw),
        "@if" => build_if_directive(raw),
        "@if-not" => build_if_not_directive(raw),
        "@pre-delay" => build_pre_delay_directive(raw),
        "@post-delay" => build_post_delay_directive(raw),
        other => bail!("unexpected directive keyword: {other}"),
    }
}

fn build_name_directive(raw: &str) -> Result<PestDirectiveLine> {
    let (prefix, name) = parse_directive_value(raw, "@name")?;
    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::Name(name.to_string()),
    })
}

fn build_timeout_directive(raw: &str) -> Result<PestDirectiveLine> {
    let (prefix, value) = parse_directive_value(raw, "@timeout")?;
    let timeout = build_timeout_literal(value)?;

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::Timeout(timeout),
    })
}

fn build_connection_timeout_directive(raw: &str) -> Result<PestDirectiveLine> {
    let (prefix, value) = parse_directive_value(raw, "@connection-timeout")?;
    let timeout = build_timeout_literal(value)?;

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::ConnectionTimeout(timeout),
    })
}

fn build_depends_on_directive(raw: &str) -> Result<PestDirectiveLine> {
    let (prefix, value) = parse_directive_value(raw, "@dependsOn")?;
    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::DependsOn(value.to_string()),
    })
}

fn build_if_directive(raw: &str) -> Result<PestDirectiveLine> {
    let (prefix, value) = parse_directive_value(raw, "@if")?;
    let condition = build_condition_expression(value)?;

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::If(condition),
    })
}

fn build_if_not_directive(raw: &str) -> Result<PestDirectiveLine> {
    let (prefix, value) = parse_directive_value(raw, "@if-not")?;
    let condition = build_condition_expression(value)?;

    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::IfNot(condition),
    })
}

fn build_pre_delay_directive(raw: &str) -> Result<PestDirectiveLine> {
    let (prefix, value) = parse_directive_value(raw, "@pre-delay")?;
    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::PreDelay(value.to_string()),
    })
}

fn build_post_delay_directive(raw: &str) -> Result<PestDirectiveLine> {
    let (prefix, value) = parse_directive_value(raw, "@post-delay")?;
    Ok(PestDirectiveLine {
        prefix,
        kind: PestDirectiveKind::PostDelay(value.to_string()),
    })
}

fn build_timeout_literal(value: &str) -> Result<PestTimeoutLiteral> {
    let value = value.trim();
    let amount_end = value
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(value.len());

    if amount_end == 0 {
        bail!("timeout literal did not contain a numeric amount");
    }

    let amount = value[..amount_end].to_string();
    let unit = {
        let unit = value[amount_end..].trim();
        (!unit.is_empty()).then(|| unit.to_string())
    };

    Ok(PestTimeoutLiteral { amount, unit })
}

fn build_condition_expression(value: &str) -> Result<PestConditionExpression> {
    let value = value.trim();
    let split_at = value
        .find(char::is_whitespace)
        .context("condition expression did not contain an expected value")?;
    let reference = &value[..split_at];
    let remainder = value[split_at..].trim_start();
    let (has_equality_operator, expected) = if let Some(expected) = remainder.strip_prefix("==") {
        (true, expected.trim_start())
    } else {
        (false, remainder)
    };

    if expected.is_empty() {
        bail!("condition expression did not contain an expected value");
    }

    if let Some(request_name) = reference.strip_suffix(".response.status") {
        return Ok(PestConditionExpression::Status {
            request_name: request_name.to_string(),
            has_equality_operator,
            expected: expected.to_string(),
        });
    }

    if let Some((request_name, path)) = reference.split_once(".response.body.") {
        return Ok(PestConditionExpression::Body {
            request_name: request_name.to_string(),
            path: path.to_string(),
            has_equality_operator,
            expected: expected.to_string(),
        });
    }

    bail!("unexpected condition expression: {value}")
}

fn build_comment_from_raw(raw: &str) -> Result<PestCommentLine> {
    let (prefix, body) = parse_comment_prefix(raw)?;
    Ok(PestCommentLine {
        prefix,
        text: body.to_string(),
    })
}

fn build_variable_line(raw: &str) -> Result<PestVariableLine> {
    let separator = raw.find('=').context("variable line did not contain '='")?;
    let name = raw[1..separator].trim().to_string();
    let value = raw[separator + 1..].trim().to_string();

    Ok(PestVariableLine { name, value })
}

fn build_assertion_line(raw: &str) -> Result<PestAssertionLine> {
    let (uses_prompt_prefix, assertion_line) = if let Some(assertion_line) = raw.strip_prefix('>') {
        (
            true,
            strip_required_horizontal_ws(assertion_line, "assertion keyword")?,
        )
    } else {
        (false, raw)
    };

    let (kind, value_text) =
        if let Some(value) = assertion_line.strip_prefix("EXPECTED_RESPONSE_STATUS") {
            (
                PestAssertionKind::Status,
                strip_required_horizontal_ws(value, "status assertion value")?,
            )
        } else if let Some(value) = assertion_line.strip_prefix("EXPECTED_RESPONSE_BODY") {
            (
                PestAssertionKind::Body,
                strip_required_horizontal_ws(value, "body assertion value")?,
            )
        } else if let Some(value) = assertion_line.strip_prefix("EXPECTED_RESPONSE_HEADERS") {
            (
                PestAssertionKind::Headers,
                strip_required_horizontal_ws(value, "headers assertion value")?,
            )
        } else {
            bail!("unexpected assertion keyword in '{raw}'");
        };

    let value = if value_text.starts_with('"') && value_text.ends_with('"') {
        PestAssertionValue::DoubleQuoted(strip_wrapping_quotes(
            value_text,
            '"',
            "double-quoted assertion value",
        )?)
    } else {
        PestAssertionValue::Raw(value_text.to_string())
    };

    Ok(PestAssertionLine {
        uses_prompt_prefix,
        kind,
        value,
    })
}

fn build_request_line(raw: &str) -> Result<PestRequestLine> {
    let parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.len() < 2 {
        bail!("request line did not contain a method and target");
    }

    let method = parts[0].to_string();
    let target = parts[1].to_string();
    let mut http_version = None;
    let mut trailing_tokens = Vec::new();

    if parts.len() >= 3 {
        let tail = &parts[2..];
        if tail[0].starts_with("HTTP/") {
            http_version = Some(tail[0].to_string());
            trailing_tokens.extend(tail[1..].iter().map(|token| (*token).to_string()));
        } else {
            trailing_tokens.extend(tail.iter().map(|token| (*token).to_string()));
        }
    }

    Ok(PestRequestLine {
        method,
        target,
        http_version,
        trailing_tokens,
    })
}

fn build_header_line(raw: &str) -> Result<PestHeaderLine> {
    let separator = raw.find(':').context("header line did not contain ':'")?;
    let name = raw[..separator].to_string();
    let value = trim_horizontal_ws_start(&raw[separator + 1..]).to_string();

    Ok(PestHeaderLine { name, value })
}

fn parse_directive_value<'a>(raw: &'a str, directive: &str) -> Result<(CommentPrefix, &'a str)> {
    let (prefix, directive_body) = parse_comment_prefix(raw)?;
    let directive_body = directive_body
        .strip_prefix(directive)
        .with_context(|| format!("directive did not start with {directive}"))?;
    let value = strip_required_horizontal_ws(directive_body, directive)?;
    Ok((prefix, value))
}

fn parse_comment_prefix(raw: &str) -> Result<(CommentPrefix, &str)> {
    if let Some(body) = raw.strip_prefix('#') {
        return Ok((CommentPrefix::Hash, trim_horizontal_ws_start(body)));
    }

    if let Some(body) = raw.strip_prefix("//") {
        return Ok((CommentPrefix::SlashSlash, trim_horizontal_ws_start(body)));
    }

    bail!("line did not start with a supported comment prefix")
}

fn trim_horizontal_ws_start(text: &str) -> &str {
    text.trim_start_matches([' ', '\t'])
}

fn strip_required_horizontal_ws<'a>(text: &'a str, context: &str) -> Result<&'a str> {
    let stripped = trim_horizontal_ws_start(text);
    if stripped.len() == text.len() {
        bail!("expected {context} to be followed by whitespace")
    } else {
        Ok(stripped)
    }
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

fn trim_line_ending_ref(text: &str) -> &str {
    text.strip_suffix("\r\n")
        .or_else(|| text.strip_suffix('\n'))
        .or_else(|| text.strip_suffix('\r'))
        .unwrap_or(text)
}

fn trim_line_ending(text: &str) -> String {
    trim_line_ending_ref(text).to_string()
}

fn count_physical_lines(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let bytes = text.as_bytes();
    let mut line_count = 0;
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'\n' => {
                line_count += 1;
                index += 1;
            }
            b'\r' => {
                line_count += 1;
                index += if bytes.get(index + 1) == Some(&b'\n') {
                    2
                } else {
                    1
                };
            }
            _ => {
                index += 1;
            }
        }
    }

    if text.ends_with('\n') || text.ends_with('\r') {
        line_count
    } else {
        line_count + 1
    }
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
        assert_eq!(tree.lines[1].line_number, 4);
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
