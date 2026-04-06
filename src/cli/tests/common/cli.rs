use assert_cmd::Command;
use std::path::Path;

pub fn command_in(dir: &Path) -> Command {
    let mut cmd =
        Command::cargo_bin("httprunner").expect("httprunner binary should be available to tests");
    cmd.current_dir(dir);
    cmd
}
