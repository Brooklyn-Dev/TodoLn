use crossterm::style::Stylize;

pub fn print_success(s: &str) { 
    println!("{}", s.green());
}

pub fn print_error(s: &str) {
    println!("{}", s.red());
}

pub fn print_title(s: &str) { 
    println!("{}", s.bold().underlined());
    println!();
}

pub fn bold_text(s: &str) -> String {
    s.bold().to_string()
}

pub fn todo_text(s: &str) -> String {
    s.to_string()
}

pub fn done_text(s: &str) -> String {
    s.dark_grey().crossed_out().to_string()
}