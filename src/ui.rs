use std::process::Command;
use colored::Colorize;
use crate::utils::format_command;

pub fn verbose_command(cmd: &Command) {
    let (program, args) = format_command(cmd);
    let cwd = std::env::current_dir().expect("Could not get current working directory.");
    let clean_args = args
        .iter()
        .map(|a| crate::utils::strip_cwd(a, &cwd))
        .map(|s| if s.contains(" ") { format!("\"{}\"", s) } else { s })
        .collect::<Vec<String>>()
        .join(" ");

    println!("[{}] Running: {} {}", "verbose".bold().bright_yellow() ,program, clean_args);
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

pub fn print_forging(target: &str){
    println!("\x1b[1;38;5;208mForging...\n{}\x1b[0m", target);
}

pub fn print_melting(){
    println!("{}", "\x1b[38;5;208mMelting...\x1b[0m".bold())
}

pub fn print_cleaning(){
    println!("{}", "Cleaning...".bold())   
}