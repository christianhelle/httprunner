pub fn red(text: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", text)
}

pub fn green(text: &str) -> String {
    format!("\x1b[32m{}\x1b[0m", text)
}

pub fn yellow(text: &str) -> String {
    format!("\x1b[33m{}\x1b[0m", text)
}

pub fn blue(text: &str) -> String {
    format!("\x1b[34m{}\x1b[0m", text)
}
