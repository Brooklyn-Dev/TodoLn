use std::fs;
use std::io;

use dirs::data_local_dir;
use rusqlite::{Connection, Error, Result, params};

use crate::commands::Task;

pub fn establish_connection() -> Connection {
    let mut db_path = data_local_dir().unwrap_or_default();
    db_path.push("Todoln");
    db_path.push("todoln.db");

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create data directory");
    }

    match Connection::open(&db_path) {
        Ok(conn) => {
            if let Err(e) = conn.execute(
                "CREATE TABLE IF NOT EXISTS tasks (
                    id INTEGER PRIMARY KEY,
                    idx INTEGER UNIQUE,
                    name TEXT NOT NULL UNIQUE,
                    done INTEGER DEFAULT 0
                )",
                (),
            ) {
                panic!("Failed to create table: {}", e);
            }

            conn
        }
        Err(e) => {
            panic!("Failed to connect to the database: {}", e);
        }
    }
}

pub fn update_task_indices(conn: &Connection, tasks: &[Task]) -> Result<(), Error>{
    conn.execute("UPDATE tasks SET idx = NULL", [])?;

    for (i, task) in tasks.iter().enumerate() {
        conn.execute("UPDATE tasks SET idx = ?1 WHERE id = ?2", params![i as i32 + 1, task.id])?;
    }

    Ok(())
}

pub fn shift_task_indices(conn: &mut Connection, index: &i32, size: &i32) -> Result<(), Error>  {
    let mut stmt = conn.prepare("SELECT idx FROM tasks WHERE idx >= ?1")?;
    let rows = stmt.query_map([index], |row| row.get(0))?;
    
    let mut indices: Vec<i32> = rows.map(|row| row.unwrap()).collect();
    indices.sort_by(|a, b| b.cmp(a));
    
    drop(stmt);

    let transaction = conn.transaction()?;
    for index_to_shift in indices.iter() {
        transaction.execute(
            "UPDATE tasks SET idx = idx + ?1 WHERE idx = ?2",
            params![&size, &index_to_shift],
        )?;
    }

    transaction.commit()?;

    Ok(())
}

pub fn get_tasks_length(conn: &Connection) -> i32 {
    match conn.query_row("SELECT COUNT(*) FROM tasks", (), |row| row.get::<_, i64>(0)) {
        Ok(count) => {
            if count > i32::MAX as i64 {
                eprintln!("Warning: Count exceeds i32 range. Truncating to i32::MAX.");
                i32::MAX
            } else {
                count as i32
            }
        },
        Err(e) => {
            panic!("Failed to get the length of tasks: {}", e);
        }
    }
}

fn add_task_to_db(conn: &mut Connection, task: &Task) {
    match conn.execute(
        "INSERT INTO tasks (name) VALUES (?1)",
        params![&task.name],
    ) {
        Ok(_) => {},
        Err(e) => { 
            panic!("Failed to add task {}: {}", &task.name, e);
        },
    }
}

pub fn add_tasks_to_db(conn: &mut Connection, tasks: &[Task]) {
    for task in tasks {
        add_task_to_db(conn, task);
    }

    match get_tasks_from_db_and_update_indices(conn) {
        Ok(_) => {},
        Err(e) => {
            panic!("Failed to update indices after adding tasks: {}", e);
        }
    };
}

fn insert_task_to_db(conn: &mut Connection, task: &Task) {
    match conn.execute(
        "INSERT INTO tasks (idx, name) VALUES (?1, ?2)",
        params![&task.idx, &task.name],
    ) {
        Ok(_) => {},
        Err(e) => panic!("Failed to insert task {}: {}", &task.name, e),
    }
}

pub fn insert_tasks_to_db(conn: &mut Connection, idx: &i32, tasks: &[Task]) {
    match shift_task_indices(conn, idx, &(tasks.len() as i32)) {
        Ok(()) => {},
        Err(e) => panic!("Failed to shift indices when inserting tasks: {}", e)
    }

    for task in tasks {
        insert_task_to_db(conn, task);
    }
}

pub fn get_tasks_from_db_and_update_indices(conn: &mut Connection) -> Result<Vec<Task>> {
    let mut stmt = conn.prepare("SELECT id, name, done FROM tasks ORDER BY idx ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            idx: None,
            name: row.get(1)?,
            done: row.get(2)?,
        })
    })?;

    let tasks: Vec<Task> = rows.map(|row| row.unwrap()).collect();

    match update_task_indices(conn, &tasks) {
        Ok(_) => {},
        Err(e) =>  panic!("Failed to update task indices: {}", e)
    }
    
    let mut stmt = conn.prepare("SELECT id, idx, name, done FROM tasks ORDER BY idx ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            idx: row.get(1)?,
            name: row.get(2)?,
            done: row.get(3)?,
        })
    })?;

    let tasks: Vec<Task> = rows.map(|row| row.unwrap()).collect();
    Ok(tasks)
}

pub fn find_tasks_from_db(conn: &mut Connection, query: &str) -> Result<Vec<Task>, Error> {
    let mut stmt = conn.prepare("SELECT id, name, done FROM tasks WHERE name LIKE ?1 ORDER BY idx ASC")?;
    let pattern = format!("%{}%", query);

    let rows = stmt.query_map([&pattern], |row| {
        Ok(Task {
            id: row.get(0)?,
            idx: None,
            name: row.get(1)?,
            done: row.get(2)?,
        })
    })?;

    let mut tasks_found: Vec<Task> = rows.map(|row| row.unwrap()).collect();
    for (i, task) in tasks_found.iter_mut().enumerate() {
        task.idx = Some(i as i32 + 1);
    }
    
    Ok(tasks_found)
}

pub fn edit_task_in_db(conn: &mut Connection, task_index: &i32, new_name: &String) -> Result<(), Error> {
    conn.execute("UPDATE tasks SET name = ?1 WHERE idx = ?2", params![new_name, task_index])?;
    Ok(())
}

pub fn mark_task_in_db_as_done(conn: &mut Connection, task_index: &i32) -> Result<(), Error> {
    conn.execute("UPDATE tasks SET done = true WHERE idx = ?1", [task_index])?;
    Ok(())
}

pub fn sort_tasks_in_db(conn: &mut Connection) -> Result<(), Error> {
    let mut stmt = conn.prepare("SELECT id, idx, name, done FROM tasks ORDER BY done ASC, idx ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(Task {
            id: row.get(0)?,
            idx: row.get(1)?,
            name: row.get(2)?,
            done: row.get(3)?,
        })
    })?;

    let tasks: Vec<Task> = rows.map(|row| row.unwrap()).collect();

    conn.execute("UPDATE tasks SET idx = NULL", [])?;

    let (done_tasks, not_done_tasks): (Vec<_>, Vec<_>) = tasks.into_iter().partition(|task| task.done);

    for (i, task) in not_done_tasks.iter().enumerate() {
        conn.execute("UPDATE tasks SET idx = ?1 WHERE id = ?2", params![i as i32 + 1, task.id])?;
    }

    for (i, task) in done_tasks.iter().enumerate() {
        conn.execute("UPDATE tasks SET idx = ?1 WHERE id = ?2", params![i as i32 + not_done_tasks.len() as i32 + 1, task.id])?;
    }

    Ok(())
}

pub fn remove_task_from_db(conn: &mut Connection, task_index: &i32) -> Result<(), Error> {
    conn.execute("DELETE FROM tasks WHERE idx = ?1", params![task_index])?;
    Ok(())
}

pub fn delete_tasks_from_db(conn: &mut Connection) -> Result<(), Error> {
    conn.execute("DELETE FROM tasks", ())?;
    Ok(())
}

pub fn backup_db(destination_path: &str) -> io::Result<()> {
    let source_path = data_local_dir().unwrap_or_default().join("TodoLn").join("todoln.db");

    fs::copy(source_path, destination_path)?;

    Ok(())
}

pub fn restore_db(backup_path: &str) -> io::Result<()> {
    let source_path = data_local_dir().unwrap_or_default().join("TodoLn").join("todoln.db");

    fs::copy(backup_path, &source_path)?;

    match Connection::open(source_path.clone()) {
        Ok(_) => Ok(()),
        Err(e) => {
            fs::remove_file(&source_path).ok();

            Err(io::Error::new(io::ErrorKind::Other, format!("Failed to open the database: {}", e)))
        }
    }
}