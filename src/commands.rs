use std::env;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::database::{
    establish_connection,
    get_tasks_length,
    add_tasks_to_db,
    insert_tasks_to_db,
    edit_task_in_db,
    get_tasks_from_db_and_update_indices,
    mark_task_in_db_as_done,
    find_tasks_from_db,
    sort_tasks_in_db,
    remove_task_from_db,
    delete_tasks_from_db,
    backup_db,
    restore_db,
};

use crate::utils::{
    print_success,
    print_error,
    print_title,
    bold_text,
    todo_text,
    done_text,
};

const ABOUT_TEXT: &str = "
  _____               _           _             
 |_   _|   ___     __| |   ___   | |      _ __  
   | |    / _ \\   / _` |  / _ \\  | |     | '_ \\ 
   | |   | (_) | | (_| | | (_) | | |___  | | | |
   |_|    \\___/   \\__,_|  \\___/  |_____| |_| |_|

  a \x1b[38;2;255;135;0mBlazingly Fast\x1b[0m and minimal task organiser written in rust\r";


#[derive(Parser)] 
#[command(author = "Brooklyn Baylis", version = "1.1.0", long_about = ABOUT_TEXT)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Adds new tasks
    #[command(name = "add", visible_aliases = &["a", "+"], arg_required_else_help = true)]
    Add {
        /// The task(s) to add
        #[arg(value_name = "task_names", use_value_delimiter = true,)]
        task_names: Vec<String>,
    },
    /// Adds new tasks at a given index
    #[command(name = "insert", visible_aliases = &["ins", "i"], arg_required_else_help = true)]
    Insert {
        /// The index to insert at
        #[arg(value_name = "index")]
        index: i32,

        /// The task(s) to add
        #[arg(value_name = "task_names", use_value_delimiter = true,)]
        task_names: Vec<String>,
    },
    /// Changes the name of a task
    #[command(name = "modify", visible_aliases = &["m", "edit"], arg_required_else_help = true)]
    Modify {
        /// The task to modify
        #[arg(value_name = "task_index")]
        task_index: i32,

        /// The new name for the task
        #[arg(value_name = "new_name")]
        new_name: String,
    },
    /// Lists tasks
    #[command(name = "list", visible_aliases = &["ls", "l"], arg_required_else_help = true)]
    List {
        /// The type of tasks to display (All, Todo, Done)
        #[arg(value_name = "display_type")]
        display_type: String,
    },
    /// Prints tasks as plain text
    #[command(name = "raw", visible_aliases = &["r", "show"], arg_required_else_help = true)]
    Raw {
        /// The type of tasks to display (All, Todo, Done)
        #[arg(value_name = "display_type")]
        display_type: String,
    },
    /// Lists tasks based on the search term
    #[command(name = "find", visible_aliases = &["f", "search"], arg_required_else_help = true)]
    Find {
        /// The term to search for
        #[arg(value_name = "search_term")]
        search_term: String,
    },
    /// Marks task as done
    #[command(name = "done", visible_aliases = &["dn", "complete"], arg_required_else_help = true)]
    Done {
        /// The task(s) to mark as done
        #[arg(value_name = "task_indices", use_value_delimiter = true)]
        task_indices: Vec<i32>,
    },
    /// Sorts tasks (todo -> done)
    #[command(name = "sort", visible_aliases = &["s", "order"])]
    Sort,
    /// Removes tasks
    #[command(name = "remove", visible_aliases = &["rm", "del", "delete", "-"], arg_required_else_help = true)]
    Remove {
        /// The task(s) to remove
        #[arg(value_name = "task_indices", use_value_delimiter = true,)]
        task_indices: Vec<i32>,
    },
    /// Removes all tasks marked as done
    #[command(name = "clear", visible_aliases = &["cls", "clean"])]
    Clear,
    /// Deletes all tasks
    #[command(name = "reset", visible_aliases = &["clearall", "deleteall"])]
    Reset,
    /// Backs up the task database to the current directory
    #[command(name = "backup", visible_aliases = &["b", "export"])]
    Backup,
    /// Restores a previously saved backup file
    #[command(name = "restore", visible_aliases = &["rest", "import"], arg_required_else_help = true)]
    Restore {
        /// The path to the backuped file
        #[arg(value_name = "backup_path")]
        backup_path: String,
    },
}

pub struct Task {
    pub id: Option<i32>,
    pub idx: Option<i32>,
    pub name: String,
    pub done: bool,
}

pub fn add(task_names: &[String]) {
    let mut conn = establish_connection();

    let tasks_to_add: Vec<Task> = task_names
        .iter()
        .filter(|task_name| !task_name.trim().is_empty()) // Filter out empty or whitespace-only names
        .map(|task_name| Task {
            id: None,
            idx: None,
            name: task_name.clone(),
            done: false,
        })
        .collect();

    if tasks_to_add.is_empty() {
        print_error("Error: No valid tasks provided.");
    }

    add_tasks_to_db(&mut conn, &tasks_to_add);

    print_success(&format!("Task(s) added successfully: {}", task_names.join(", ")));
}

pub fn insert(index: &i32, task_names: &[String]) {
    let mut conn = establish_connection();

    if *index < 0 {
        print_error("Error: Index must be non-negative.");
    }

    let tasks_length = get_tasks_length(&conn);
    if *index > tasks_length {
        print_error(&format!("Error: Cannot insert at index {} as the total number of tasks is: {}", *index, tasks_length));
    }

    let tasks_to_insert: Vec<Task> = task_names
        .iter()
        .filter(|task_name| !task_name.trim().is_empty())
        .enumerate()
        .map(|(i, task_name)| Task {
            id: None,
            idx: Some(*index + i as i32),
            name: task_name.clone(),
            done: false,
        })
        .collect();

    if tasks_to_insert.is_empty() {
        print_error("Error: No valid tasks provided.");
    }

    insert_tasks_to_db(&mut conn, index, &tasks_to_insert);

    print_success(&format!("Task(s) inserted successfully: {}", tasks_to_insert.iter().map(|t| t.name.clone()).collect::<Vec<_>>().join(", ")));
}

pub fn modify(task_index: &i32, new_name: &String) {
    let mut conn = establish_connection();

    if *task_index <= 0 || *task_index > get_tasks_length(&conn) {
        print_error(&format!("Error: Invalid index '{}'.", task_index));
        return;
    }

    if new_name.trim().is_empty() {
        print_error("Error: New task cannot be empty or whitespace-only.");
        return;
    }

    match edit_task_in_db(&mut conn, task_index, new_name) {
        Ok(_) => print_success(&format!("Task modifed successfully: '{}'", new_name)),
        Err(e) => print_error(&format!("Failed to modify task {}: {}", task_index, e)),
    }
}

pub enum DisplayType {
    All,
    Todo,
    Done
}

impl DisplayType {
    pub fn from_str(s: &str) -> Option<DisplayType> {
        match s.trim().to_lowercase().as_str() {
            "all" => Some(DisplayType::All),
            "todo" => Some(DisplayType::Todo),
            "done" => Some(DisplayType::Done),
            _ => None,
        }
    }
}

pub fn list(display_type: &str) {
    let mut conn = establish_connection();

    match get_tasks_from_db_and_update_indices(&mut conn) {
        Ok(tasks) => {        
            match DisplayType::from_str(display_type) {
                Some(display_type) => {
                    match display_type {
                        DisplayType::All => {
                            if tasks.is_empty() {
                                println!("No tasks found.");
                                return;
                            }

                            print_title("Tasks:");
                            for task in tasks {
                                if task.done { 
                                    println!("  [{}] {}", bold_text(&task.idx.unwrap().to_string()), done_text(&task.name));
                                }
                                else {
                                    println!("  [{}] {}", bold_text(&task.idx.unwrap().to_string()), todo_text(&task.name));
                                };
                            }
                        }
                        DisplayType::Todo => {
                            let tasks_todo = tasks.iter().filter(|t| !t.done).collect::<Vec<_>>();
                            if tasks_todo.is_empty() {
                                println!("No tasks found.");
                            }

                            print_title("Tasks todo:");
                            for task in tasks_todo {
                                println!("  [{}] {}", bold_text(&task.idx.unwrap().to_string()), todo_text(&task.name));
                            }
                        }
                        DisplayType::Done => {
                            let tasks_done = tasks.iter().filter(|t| t.done).collect::<Vec<_>>();
                            if tasks_done.is_empty() {
                                println!("No tasks found.");
                            }
                      
                            print_title("Tasks done:");
                            for task in tasks_done {
                                println!("  [{}] {}", bold_text(&task.idx.unwrap().to_string()), done_text(&task.name));
                            }
                        }
                    }
                }
                None => {
                    println!("Invalid display type");
                }
            }
        }
        Err(e) => print_error(&format!("Failed to retrieve tasks: {}", e)),
    }
}

pub fn raw(display_type: &str) {
    let mut conn = establish_connection();

    match get_tasks_from_db_and_update_indices(&mut conn) {
        Ok(tasks) => {        
            if let Some(display_type) = DisplayType::from_str(display_type) {
                match display_type {
                    DisplayType::All => {
                        for task in tasks {
                            println!("{}", task.name);
                        }
                    }
                    DisplayType::Todo => {
                        let tasks_todo = tasks.iter().filter(|t| !t.done).collect::<Vec<_>>();
                        for task in tasks_todo {
                            println!("{}", task.name);
                        }
                    }
                    DisplayType::Done => {
                        let tasks_done = tasks.iter().filter(|t| t.done).collect::<Vec<_>>();
                        for task in tasks_done {
                            println!("{}", task.name);
                        }
                    }
                }
            }         
        }
        Err(e) => print_error(&format!("Failed to retrieve tasks: {}", e)),
    }
}

pub fn find(search_term: &str) {
    let mut conn = establish_connection();

    match find_tasks_from_db(&mut conn, search_term) {
        Ok(tasks_found) => {        
            for task in tasks_found {
                println!("{} {}", bold_text(&task.idx.unwrap().to_string()), task.name);
            }   
        }
        Err(e) => print_error(&format!("Failed to find tasks: {}", e)),
    }
}

pub fn done(task_indices: &[i32]) {
    let mut conn = establish_connection();

    for id in task_indices.iter() {
        match mark_task_in_db_as_done(&mut conn, id) {
            Ok(_) => {},
            Err(e) => {
                print_error(&format!("Failed to mark task {} as done: {}", id, e));
                return;
            }
        }
    }

    print_success(&format!("Task(s) completed successfully: {}", task_indices.iter().map(|&i| i.to_string()).collect::<Vec<_>>().join(", ")));
}

pub fn sort() {
    let mut conn = establish_connection();

    match sort_tasks_in_db(&mut conn) {
        Ok(_) => {},
        Err(e) => {
            print_error(&format!("Failed to sort tasks: {}", e));
            return;
        }
    }

    print_success("Tasks sorted successfully");
}

pub fn remove(task_indices: &[i32]) {
    let mut conn = establish_connection();

    for index in task_indices.iter() {
        match remove_task_from_db(&mut conn, index) {
            Ok(_) => {},
            Err(e) => {
                print_error(&format!("Failed to remove task {}: {}", index, e));
                return;
            }
        }
    }

    print_success(&format!("Task(s) removed successfully: {}", task_indices.iter().map(|&i| i.to_string()).collect::<Vec<_>>().join(", ")));
}

pub fn clear() {
    let mut conn = establish_connection();

    match get_tasks_from_db_and_update_indices(&mut conn) {
        Ok(tasks) => {
            let completed_tasks: Vec<_> = tasks.iter().filter(|t| t.done).collect();

            for task in &completed_tasks.clone() {
                match remove_task_from_db(&mut conn, &task.idx.unwrap()) {
                    Ok(_) => {},
                    Err(e) => print_error(&format!("Failed to remove task {}: {}", &task.idx.unwrap(), e)),
                }
            }

            print_success(&format!("Completed task(s) cleared successfully: {}", completed_tasks.iter().map(|t| t.name.to_string()).collect::<Vec<_>>().join(", ")));
        },
        Err(e) => print_error(&format!("Failed to retrieve tasks: {}", e)),
    }
}

pub fn reset() {
    let mut conn = establish_connection();
    
    if let Err(e) = delete_tasks_from_db(&mut conn) {
        print_error(&format!("Failed to delete all tasks: {}", e));
        return;
    }

    print_success("Tasks reset successfully");
}

pub fn backup() {
    if let Ok(mut current_dir) = env::current_dir() {
        current_dir.push("todoln_backup.db");
        let backup_path = current_dir.to_str().expect("Invalid Unicode in current path");

        if let Err(e) = backup_db(backup_path) {
            print_error(&format!("Failed to backup database: {}", e));
            return;
        }

        print_success("Task database backuped successfully");
    } else {
        print_error("Failed to get current directory");
    }
}

pub fn restore(backup_path: String) {
    let mut backup_path = backup_path.clone();
    let backup_path_buf = PathBuf::from(&backup_path);

    if !backup_path_buf.is_absolute() {
        if let Ok(mut current_dir) = env::current_dir() {
            current_dir.push(backup_path_buf);
            backup_path = current_dir.to_string_lossy().into_owned();
        } else {
            print_error("Failed to get current directory");
            return;
        }
    }

    if let Err(e) = restore_db(&backup_path) {
        print_error(&format!("Failed to restore database: {}", e));
        return;
    }

    print_success("Task database restored successfully");
}