use pest_derive::Parser;

// Phase 1 scaffolding: the handwritten parser in file_parser.rs remains the
// production path until later migration phases consume pest pairs.
#[allow(dead_code)]
#[derive(Parser)]
#[grammar = "parser/http-file.pest"]
pub(crate) struct HttpFilePestParser;

#[cfg(test)]
mod tests {
    use super::*;
    use pest::Parser;

    #[test]
    fn parses_mixed_http_file_shapes() {
        let input = r#"# @name login
# @timeout 30s
POST https://api.example.com/login HTTP/1.1 trailing-token
Content-Type: application/json

{"token":"abc"}
> EXPECTED_RESPONSE_STATUS 200"#;

        let mut pairs =
            HttpFilePestParser::parse(Rule::HttpFile, input).expect("grammar should parse");
        let file = pairs.next().expect("root pair");

        assert_eq!(file.as_rule(), Rule::HttpFile);
        assert!(pairs.next().is_none());
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
