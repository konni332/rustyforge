use std::process::Command;
use colored::Colorize;
use crate::utils::{format_command, strip_cwd};

use crossterm::{
    cursor::{MoveUp, MoveDown, MoveToColumn},
    execute,
    style::{Color, Print, SetForegroundColor, ResetColor},
    terminal::{Clear, ClearType},
};
use crate::arguments::DiscoverOptions;
use crate::discovery::should_be_ignored;

pub fn verbose_command(cmd: &Command) {
    let (program, args) = format_command(cmd);
    let cwd = std::env::current_dir().expect("Could not get current working directory.");
    let clean_args = args
        .iter()
        .map(|a| strip_cwd(a, &cwd))
        .map(|s| if s.contains(" ") { format!("\"{}\"", s) } else { s })
        .collect::<Vec<String>>()
        .join(" ");
    
    let clean_program = strip_cwd(&program, &cwd);
    
    println!("[{}] Running: {} {}", "verbose".bold().bright_yellow() ,clean_program, clean_args);
}

pub fn verbose_command_hard(cmd: &Command) {
    let (program, args) = format_command(cmd);
    let args_string = args
        .iter()
        .map(|s| if s.contains(" ") { format!("\"{}\"", s) } else { s.clone() })
        .collect::<Vec<String>>()
        .join(" ");

    println!("[{}] Running (raw): {} {}", "verbose-hard".bold().bright_red(),program, args_string);
}

pub fn event_file_found(options: &DiscoverOptions, name: &String) -> bool {
    if should_be_ignored(name, &options.ignore) {
        return false;
    }

    let mut stdout = std::io::stdout();
    // if auto, don't ask for permission
    if options.auto {
        execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print(format!("[{}]\n", name)),
                ResetColor
            ).unwrap();
        return true;   
    }
    
    execute!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print("[file found]: "),
        ResetColor,
        Print(name),
        Print("\nAdd? [Y/N] "),
    ).unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let input = input.trim().to_lowercase();

    execute!(
        stdout,
        MoveUp(2),
        MoveToColumn(0),
        Clear(ClearType::CurrentLine),
        MoveDown(1),
        MoveToColumn(0),
        Clear(ClearType::CurrentLine),
    ).unwrap();

    match input.as_str() {
        "y" => {
            execute!(
                stdout,
                SetForegroundColor(Color::Green),
                Print(format!("[{}]\n", name)),
                ResetColor
            ).unwrap();
            true
        }
        "n" => {
            execute!(
                stdout,
                SetForegroundColor(Color::Red),
                Print(format!("[{}]\n", name)),
                ResetColor
            ).unwrap();
            false
        }
        _ => {
            execute!(
                stdout,
                SetForegroundColor(Color::Yellow),
                Print("[invalid input]\n"),
                ResetColor
            ).unwrap();
            false
        }
    }
}

pub fn print_forging(target: &str){
    println!("\x1b[1;38;5;208mForging...\n{}\x1b[0m", target);
}

pub fn print_melting(){
    println!("{}", "\x1b[38;5;208mMelting...\x1b[0m".bold())
}

pub fn print_cleaning(){
    println!("{}", "Cleaning...".bold())   
}