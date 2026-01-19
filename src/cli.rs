use std::{fs, io, path::PathBuf, process::Command};

use serde_json::{Value, json};
use sqlx::Encode;

use crate::{
    config::{Config, default_dir},
    date::{date_str_format, now_nanos, page_id_for, today_date, today_date_formatted},
    db::{self, Entity},
    transact::transact_with_fallback,
};

fn edit_buffer_path(date: &str) -> io::Result<PathBuf> {
    let mut dir = default_dir();
    dir.push("tmp");
    fs::create_dir_all(&dir)?;
    dir.push(format!("{}.md", date));
    Ok(dir)
}

fn render_journal(date: &str, entities: &[Entity], bullet: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n", date));
    for entity in entities {
        if let Some(Value::String(title)) = entity.attrs.get("block/content") {
            out.push_str(&format!("{} {}\n", bullet, title));
        }
    }
    out
}

//move this to a separate module
fn open_editor(editor: &str, path: &PathBuf) -> io::Result<()> {
    Command::new(editor).arg(path).status()?;
    Ok(())
}

//todo@chico: add tests for this function
//move this to a separate module
/// Parses the journal content into individual notes.
fn parse_journal(content: &str, bullet: &str) -> Vec<String> {
    let mut notes = Vec::new();
    let mut current = String::new();

    for line in content.lines() {
        let trimmed = line.trim_start();

        // Skip comments
        if trimmed.starts_with('#') {
            continue;
        }

        // Check if this line starts a new bullet point
        if is_bullet_point(trimmed, bullet) {
            // Save previous note if it exists
            if !current.trim().is_empty() {
                notes.push(current);
            }

            // Extract bullet content and start new note
            current = extract_bullet_content(trimmed, bullet).to_string();
        } else if !trimmed.is_empty() {
            // Append to current note (continuation line)
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }

    // Don't forget the last note
    if !current.trim().is_empty() {
        notes.push(current);
    }

    notes
}

fn is_bullet_point(line: &str, custom_bullet: &str) -> bool {
    line.starts_with(custom_bullet) || line.starts_with("- ") || line.starts_with("* ")
}

fn extract_bullet_content<'a>(line: &'a str, custom_bullet: &str) -> &'a str {
    if line.starts_with(custom_bullet) {
        line[custom_bullet.len()..].trim_start()
    } else {
        line[2..].trim_start()
    }
}

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

//TODO@chico: add tests for this function
//TODO@chico: refactor to separate printing logic from data retrieval
//TODO@chico: support nested blocks and indentation
//TODO@chico: support different bullet styles from config
//TODO@chico: move printing logic to a separate module
/// Prints the blocks of a journal page for a given date.
/// # Arguments
/// * `date` - The date of the journal page
/// * `entities` - A vector of journal block entities
fn print_page_blocks(date: &str, entities: Vec<Entity>, bullet: &str) {
    if entities.is_empty() {
        println!("No journal for {}", date);
        return;
    }
    println!("# {}", date_str_format(date));
    for entity in entities {
        if let Some(Value::String(title)) = entity.attrs.get("block/content") {
            println!("{} {}", bullet, title);
        }
    }
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
        // TODO@chico: add test for the "list" command
        // TODO@chico: refactor this function to make it more modular and testable
        "view" => {
            let date = if args.len() > 1 {
                args[1].as_str()
            } else {
                &today_date()
            };
            let page_id = page_id_for(date);
            let entities = match db::list_page_blocks(&page_id).await {
                Ok(entities) => entities,
                Err(err) => {
                    eprintln!("Failed to view journal: {}", err);
                    return;
                }
            };
            print_page_blocks(date, entities, &config.bullet);
        }
        "edit" => {
            let date = if args.len() > 1 {
                args[1].as_str()
            } else {
                &today_date()
            };
            let page_id = page_id_for(date);
            let entities = match db::list_page_blocks(&page_id).await {
                Ok(entities) => entities,
                Err(err) => {
                    eprintln!("Failed to edit journal: {}", err);
                    return;
                }
            };
            let path = match edit_buffer_path(date) {
                Ok(path) => path,
                Err(err) => {
                    eprintln!("Failed to create edit buffer: {}", err);
                    return;
                }
            };

            //this flow is too complex and can be improved
            //can be modularized better
            //TODO@chico: refactor to separate functions
            let content = render_journal(date, &entities, &config.bullet);
            if let Err(err) = fs::write(&path, content) {
                eprintln!("Failed to write temp file: {}", err);
                return;
            }

            if let Err(err) = open_editor(&config.editor, &path) {
                eprintln!("Failed to open editor: {}", err);
                return;
            }

            //TODO@chico: it need to split and move this logic is to coupled
            //and need to delete the tmp/file for edit
            let edited = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Failed to read edited file: {}", err);
                    return;
                }
            };

            let notes = parse_journal(&edited, &config.bullet);
            let mut datoms = Vec::new();

            //TODO@chico: to improve this we need to retract only changed contents
            for entity in &entities {
                if let Some(Value::String(title)) = entity.attrs.get("block/title") {
                    datoms.push(json!(["db/retract", entity.id, "block/title", title]));
                }
                if let Some(Value::String(content)) = entity.attrs.get("block/content") {
                    datoms.push(json!(["db/retract", entity.id, "block/content", content]));
                }
                if let Some(Value::String(page)) = entity.attrs.get("block/page") {
                    datoms.push(json!(["db/retract", entity.id, "block/page", page]));
                }
            }

            for note in notes {
                let block_id = format!("block:{}-{}", date, now_nanos());
                datoms.push(json!([
                    "db/add",
                    &block_id,
                    "block/title",
                    date_str_format(date)
                ]));
                datoms.push(json!(["db/add", &block_id, "block/content", note]));
                datoms.push(json!(["db/add", &block_id, "block/page", page_id.clone()]));
            }
            datoms.push(json!(["db/add", &page_id, "page/name", date]));

            if let Err(err) = transact_with_fallback(datoms).await {
                eprintln!("Failed to save journal: {}", err);
            }
        }
        "list" => {
            let pages = match db::list_pages().await {
                Ok(pages) => pages,
                Err(err) => {
                    eprintln!("Failed to list journals: {}", err);
                    return;
                }
            };
            if pages.is_empty() {
                println!("journals/ (empty)");
                return;
            }
            println!("journals/");
            for page in pages {
                println!("    - {}.md", page);
            }
        }
        "search" => {
            //TODO@chico: implement search command
            eprintln!("Error: 'search' command not implemented yet.");
        }
        "tag" => {
            //TODO@chico: implement tag command
            eprintln!("Error: 'tag' command not implemented yet.");
        }
        _ => {
            eprintln!("Error: Unknown command '{}'.", args[0]);
            print_help();
        }
    }
}
