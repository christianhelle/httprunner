use super::*;

#[test]
fn color_helpers_return_ansi_codes() {
    assert_eq!(red("hello"), "\x1b[31mhello\x1b[0m");
    assert_eq!(green("world"), "\x1b[32mworld\x1b[0m");
    assert_eq!(yellow("warn"), "\x1b[33mwarn\x1b[0m");
    assert_eq!(blue("info"), "\x1b[34minfo\x1b[0m");
}
