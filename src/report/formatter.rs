pub fn escape_markdown(s: &str) -> String {
    s.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_markdown_escapes_pipe_character() {
        assert_eq!(escape_markdown("hello|world"), "hello\\|world");
        assert_eq!(escape_markdown("no pipes here"), "no pipes here");
        assert_eq!(escape_markdown("|||"), "\\|\\|\\|");
    }
}
