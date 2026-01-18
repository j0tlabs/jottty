use crate::config::Config;

/// Prints the help message for the CLI application.
fn print_help() {
    println!("Usage:");
    println!("  jottty add \"note text\"");
    println!("  jottty list");
    println!("  jottty view");
    println!("  jottty view [YYYY-MM-DD]");
    println!("  jottty edit");
    println!("  jottty edit [YYYY-MM-DD]");
    println!("  jottty search \"term\"");
    println!("  jottty tag --filter \"TERM\"");
}
//TODO@chico use a command-line argument parser like clap or structopt
//
/// Entry point for the CLI application.
/// Handles command-line arguments and executes corresponding actions.
pub async fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    if args.is_empty() {
        print_help();
        return;
    }

    match args[0].as_str() {
        "add" => {
            if args.len() < 2 {
                eprintln!("Error: 'add' command requires note text.");
                print_help();
                return;
            }
        }
        _ => {
            eprintln!("Error: Unknown command '{}'.", args[0]);
            print_help();
        }
    }
}
