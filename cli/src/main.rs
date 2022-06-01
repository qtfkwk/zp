use clap::Parser;

/// Zip Parser
#[derive(Parser)]
#[clap(name = "zp", version, about)]
struct Args {
    /// Verbosity
    #[clap(short, parse(from_occurrences))]
    verbose: u8,

    /// One or more zip files
    files: Vec<String>,
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    if args.files.len() < 1 {
        return Err(String::from(
            "No files provided. Run with `-h` to view usage.",
        ));
    }
    let verbose = args.verbose > 0;
    for i in args.files {
        match zp_lib::process_file(&i, verbose) {
            Ok(o) => {
                println!("{o}");
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
    Ok(())
}
