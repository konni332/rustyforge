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

    println!("[verbose] Running: {} {}", program, clean_args);
}

pub fn verbose_command_hard(cmd: &Command) {
    let (program, args) = format_command(cmd);
    let args_string = args
        .iter()
        .map(|s| if s.contains(" ") { format!("\"{}\"", s) } else { s.clone() })
        .collect::<Vec<String>>()
        .join(" ");

    println!("[verbose-hard] Running (raw): {} {}", program, args_string);
}

pub fn print_forging(){
    println!("{}", "Forging...".bold().white())
}

pub fn print_melting(){
    println!("{}", "Melting...".bold().red())
}

pub fn print_smelting(){
    todo!()
}

pub fn print_cleaning(){
    todo!()
}



