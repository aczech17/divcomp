#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide the console window

mod io_utils;
mod archive;
mod compress;
mod gui;

use std::env::args;
use gui::run;

fn main()
{
    let args: Vec<String> = args().collect();
    let archive_path = match args.len() >= 2
    {
        false => None,
        true => Some(args[1].clone()),
    };

    if let Err(err) = run("Archiwizator boży", archive_path)
    {
        eprintln!("{}", err.to_string());
    }
}
