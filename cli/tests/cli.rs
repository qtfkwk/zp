use assert_cmd::Command;

const VERBOSE: &str = include_str!("../../exercise.zip-process-verbose.txt");
const SUMMARY: &str = include_str!("../../exercise.zip-process-summary.txt");

// Helper functions

/// Retrieve the binary to test
pub fn cmd(bin: &str) -> Command {
    Command::cargo_bin(bin).unwrap()
}

/// Print the command
fn p(bin: &str, args: &[&str]) {
    println!(
        "{} {}",
        bin,
        args.iter()
            .map(|x| {
                if x.contains(' ') {
                    format!("\"{}\"", x)
                } else {
                    x.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    );
}

/// Run command that fails
fn fail(bin: &str, args: &[&str], code: i32, msg: &str) {
    p(bin, args);
    cmd(bin)
        .args(args)
        .assert()
        .failure()
        .code(code)
        .stderr(format!("Error: \"{}\"\n", msg.replace("\"", "\\\"")));
}

/// Run command that succeeds
fn pass(bin: &str, args: &[&str], want: &str) {
    p(bin, args);
    cmd(bin)
        .args(args)
        .assert()
        .success()
        .stdout(format!("{}\n", want));
}

// Tests

#[test]
fn version() {
    for i in ["-V", "--version"].iter() {
        pass("zp", &[i], &format!("zp {}", env!("CARGO_PKG_VERSION")));
    }
}

#[test]
fn verbose() {
    pass("zp", &["-v", "../../exercise.zip"], VERBOSE);
}

#[test]
fn summary() {
    pass("zp", &["../../exercise.zip"], SUMMARY);
}

#[test]
fn not_a_file() {
    fail("zp", &["."], 1, "Path is not a file: \".\"");
}

#[test]
fn no_files() {
    fail("zp", &[], 1, "No files provided. Run with `-h` to view usage.");
}

#[test]
fn no_files_verbose() {
    fail("zp", &["-v"], 1, "No files provided. Run with `-h` to view usage.");
}

#[test]
fn eof() {
    fail(
        "zp",
        &["nonexistent.zip"],
        1,
        "No such file or directory (os error 2): \"nonexistent.zip\"",
    );
}
