use serde_json::json;

use crate::{
    config::Config,
    date::{now_nanos, page_id_for, today_date, today_date_formatted},
    transact::transact_with_fallback,
};

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
        // TODO@chico: add test for the "add" command
        // TODO@chico: refactor this function to make it more modular and testable
        "add" => {
            if args.len() < 2 {
                eprintln!("Error: 'add' command requires note text.");
                print_help();
                return;
            }
            //TODO@chico this can be improved to handle multi-word notes better
            //TODO@chico this can be improved.
            //it would be better page_id be uuid and date be a property
            let note = args[1..].join(" ");
            let date = today_date();
            let page_id = page_id_for(&date);
            let title = today_date_formatted();

            let block_id = format!("block:{}-{}", date, now_nanos());
            let datoms = vec![
                json!(["db/add", &block_id, "block/title", title]),
                json!(["db/add", &block_id, "block/content", note]),
                json!(["db/add", &block_id, "block/page", page_id.clone()]),
                json!(["db/add", &page_id, "page/name", date]),
            ];
            if let Err(err) = transact_with_fallback(datoms).await {
                eprintln!("Failed to add note: {}", err);
            }
        }
        "view" => {
            // TODO@chico: implement the "view" command
            println!("'view' command is not yet implemented.");
        }
        _ => {
            eprintln!("Error: Unknown command '{}'.", args[0]);
            print_help();
        }
    }
}
