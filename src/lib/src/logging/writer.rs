use anyhow::Result;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Log {
    log_file: Option<File>,
    silent: bool,
}

impl Log {
    pub fn new(base_filename: Option<&str>) -> Result<Self> {
        Self::new_with_silent(base_filename, false)
    }

    pub fn new_with_silent(base_filename: Option<&str>, silent: bool) -> Result<Self> {
        let log_file = if let Some(filename) = base_filename {
            Some(create_log_file(filename)?)
        } else {
            None
        };

        Ok(Log { log_file, silent })
    }

    pub fn writeln(&mut self, message: &str) {
        if !self.silent {
            println!("{}", message);
        }
        if let Some(ref mut file) = self.log_file {
            let clean_message = strip_ansi_codes(message);
            let _ = writeln!(file, "{}", clean_message);
        }
    }
}

fn create_log_file(base_filename: &str) -> Result<File> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let log_filename = format!("{}_{}.log", base_filename, timestamp);

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_filename)?;

    Ok(file)
}

/// Strip ANSI escape codes from a string
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Look at the next character to determine if this is an ANSI CSI sequence
            match chars.next() {
                Some('[') => {
                    // Skip until we find a letter (the command character)
                    for next_ch in chars.by_ref() {
                        if next_ch.is_ascii_alphabetic() {
                            break;
                        }
                    }
                }
                Some(other) => {
                    // Not a recognized ANSI sequence: preserve both the escape and the following character
                    result.push(ch);
                    result.push(other);
                }
                None => {
                    // Stray escape at end of string: ignore it
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod strip_ansi_tests {
    use super::*;

    #[test]
    fn empty_string() {
        assert_eq!(strip_ansi_codes(""), "");
    }

    #[test]
    fn no_ansi_codes() {
        let text = "Hello, World!";
        assert_eq!(strip_ansi_codes(text), text);
    }

    #[test]
    fn plain_text_with_special_chars() {
        let text = "Test @#$% 123 🚀 emoji";
        assert_eq!(strip_ansi_codes(text), text);
    }

    #[test]
    fn escape_not_followed_by_bracket() {
        // ESC followed by 'X' should be preserved
        let text = "\x1bX text";
        assert_eq!(strip_ansi_codes(text), "\x1bX text");
    }

    #[test]
    fn escape_followed_by_various_chars() {
        // ESC followed by non-bracket chars should be preserved
        assert_eq!(strip_ansi_codes("\x1bA test"), "\x1bA test");
        assert_eq!(strip_ansi_codes("\x1b7 save"), "\x1b7 save");
        assert_eq!(strip_ansi_codes("\x1b= keypad"), "\x1b= keypad");
    }

    #[test]
    fn incomplete_ansi_sequence_no_terminator() {
        // ESC[ followed by digits and space, then 'i' acts as terminator
        let text = "\x1b[31 incomplete";
        assert_eq!(strip_ansi_codes(text), "ncomplete");
    }

    #[test]
    fn incomplete_ansi_sequence_at_end() {
        // ESC[ at the very end with no terminator
        let text = "text \x1b[31";
        assert_eq!(strip_ansi_codes(text), "text ");
    }

    #[test]
    fn escape_at_end_of_string() {
        // Stray ESC at the end should be ignored
        let text = "text\x1b";
        assert_eq!(strip_ansi_codes(text), "text");
    }

    #[test]
    fn escape_bracket_at_end() {
        // ESC[ at the end with nothing following
        let text = "text\x1b[";
        assert_eq!(strip_ansi_codes(text), "text");
    }

    #[test]
    fn standard_color_codes() {
        assert_eq!(strip_ansi_codes("\x1b[31mRed\x1b[0m"), "Red");
        assert_eq!(strip_ansi_codes("\x1b[34mBlue\x1b[0m"), "Blue");
        assert_eq!(
            strip_ansi_codes("\x1b[1;32mBold Green\x1b[0m"),
            "Bold Green"
        );
    }

    #[test]
    fn multiple_ansi_sequences() {
        let text = "\x1b[31mRed\x1b[0m \x1b[34mBlue\x1b[0m \x1b[32mGreen\x1b[0m";
        assert_eq!(strip_ansi_codes(text), "Red Blue Green");
    }

    #[test]
    fn consecutive_ansi_sequences() {
        // Multiple ANSI codes with no text between them
        let text = "\x1b[31m\x1b[1m\x1b[4mText\x1b[0m";
        assert_eq!(strip_ansi_codes(text), "Text");
    }

    #[test]
    fn nested_style_ansi_sequences() {
        // Overlapping/nested style codes
        let text = "\x1b[1m\x1b[31mBold Red\x1b[0m\x1b[0m";
        assert_eq!(strip_ansi_codes(text), "Bold Red");
    }

    #[test]
    fn malformed_mixed_sequences() {
        // Mix of valid ANSI, incomplete sequences, and non-ANSI escapes
        // Note: 'i' in "incomplete" acts as terminator for \x1b[34
        let text = "\x1b[31mRed\x1b[0m \x1bX \x1b[34 incomplete \x1b[32mGreen\x1b[0m";
        assert_eq!(strip_ansi_codes(text), "Red \x1bX ncomplete Green");
    }

    #[test]
    fn ansi_with_unicode() {
        let text = "\x1b[31m🚀 Rocket\x1b[0m ✨ \x1b[34m星\x1b[0m";
        assert_eq!(strip_ansi_codes(text), "🚀 Rocket ✨ 星");
    }

    #[test]
    fn complex_sgr_parameters() {
        // 256-color and RGB color codes
        assert_eq!(strip_ansi_codes("\x1b[38;5;196mRed256\x1b[0m"), "Red256");
        assert_eq!(
            strip_ansi_codes("\x1b[38;2;255;0;0mRGB Red\x1b[0m"),
            "RGB Red"
        );
    }

    #[test]
    fn cursor_movement_codes() {
        // Non-color ANSI codes should also be stripped
        assert_eq!(strip_ansi_codes("\x1b[2J\x1b[H Clear"), " Clear");
        assert_eq!(strip_ansi_codes("Move\x1b[10;20H Here"), "Move Here");
    }

    #[test]
    fn multiple_escapes_various_patterns() {
        // Multiple stray escapes and escape sequences
        // First \x1b consumes the second \x1b, treating it as non-bracket escape
        let text = "\x1b\x1b[31mText\x1b[0m\x1b";
        assert_eq!(strip_ansi_codes(text), "\x1b\x1b[31mText");
    }

    #[test]
    fn only_ansi_codes() {
        // String with only ANSI codes, no visible text
        let text = "\x1b[31m\x1b[0m\x1b[1m\x1b[0m";
        assert_eq!(strip_ansi_codes(text), "");
    }

    #[test]
    fn mixed_valid_invalid_escapes() {
        // Valid ANSI + invalid escape sequences + normal text
        // Last \x1b consumes space, then "End" remains
        let text = "Start \x1b[31mRed\x1b[0m \x1bQ middle \x1b[34mBlue\x1b[0m \x1b End";
        assert_eq!(
            strip_ansi_codes(text),
            "Start Red \x1bQ middle Blue \x1b End"
        );
    }
}
