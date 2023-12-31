# TodoLn

![Minimum Supported Rust Version](https://img.shields.io/badge/rustc-1.70.0%2B-orange)
![GitHub License](https://img.shields.io/github/license/Brooklyn-Dev/TodoLn)
![GitHub Repo stars](https://img.shields.io/github/stars/Brooklyn-Dev/TodoLn?color=yellow)

a Blazingly Fast and minimal task organiser written in rust. It provides a simple and efficient command-line interface for managing your tasks.

## Installation

1.  Download the source code
    -   run the command: `git clone https://github.com/Brooklyn-Dev/TodoLn.git`
2.  Use `cargo build --release` to compile todoln
3.  Navigate to the project directory `cd todoln`
4.  Find the compiled executable `todoln.exe` in the `target\release\` directory
5.  Move the executable to a directory in your system's PATH
    -   **Windows**: `C:\Windows\System32`
    -   **Linux**: `\usr\bin`

## Usage

```
  _____               _           _
 |_   _|   ___     __| |   ___   | |      _ __
   | |    / _ \   / _` |  / _ \  | |     | '_ \
   | |   | (_) | | (_| | | (_) | | |___  | | | |
   |_|    \___/   \__,_|  \___/  |_____| |_| |_|

  a Blazingly Fast and minimal task organiser written in rust

Usage: todoln [COMMAND]

Commands:
  add      Adds new tasks [aliases: a, +]
  insert   Adds new tasks at a given index [aliases: ins, i]
  modify   Changes the name of a task [aliases: m, edit]
  list     Lists tasks [aliases: ls, l]
  raw      Prints tasks as plain text [aliases: r, show]
  find     Lists tasks based on the search term [aliases: f, search]
  done     Marks task as done [aliases: dn, complete]
  sort     Sorts tasks (todo -> done) [aliases: s, order]
  remove   Removes tasks [aliases: rm, delete]
  clear    Removes all tasks marked as done [aliases: cls, clean]
  reset    Deletes all tasks [aliases: clearall, deleteall]
  backup   Backs up the task database to the current directory [aliases: b, export]
  restore  Restores a previously saved backup file [aliases: rest, import]
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Like this project?

If you find this project interesting or useful, consider giving it a star ⭐️!
