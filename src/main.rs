mod commands;
mod database;
mod utils;

use clap::Parser;
use commands::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add {task_names}) => commands::add(task_names),
        Some(Commands::Insert {index, task_names}) => commands::insert(index, task_names),
        Some(Commands::Modify {task_index, new_name}) => commands::modify(task_index, new_name),
        Some(Commands::List {display_type}) => commands::list(display_type),
        Some(Commands::Find {search_term}) => commands::find(search_term),
        Some(Commands::Raw {display_type}) => commands::raw(display_type),
        Some(Commands::Done {task_indices}) => commands::done(task_indices),
        Some(Commands::Sort) => commands::sort(),
        Some(Commands::Remove {task_indices}) => commands::remove(task_indices),
        Some(Commands::Clear) => commands::clear(),
        Some(Commands::Reset) => commands::reset(),
        Some(Commands::Backup) => commands::backup(),
        Some(Commands::Restore {backup_path}) => commands::restore(backup_path.to_string()),
        None => commands::list(&String::from("all"))
    }
}