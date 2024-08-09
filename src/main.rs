#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide the console window

mod io_utils;
mod archive;
mod compress;
use std::env;
mod gui;
use gui::run;

fn main()
{
    let archive_path = env::args().nth(1);

    if let Err(err) = run("Archiwizator boży", archive_path)
    {
        eprintln!("{}", err);
    }
}
