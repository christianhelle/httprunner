pub fn escape_markdown(s: &str) -> String {
    s.replace('|', "\\|")
}
