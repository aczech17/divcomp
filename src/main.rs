#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide a console window on Windows in release

mod io_utils;
mod archive;
mod compress;
mod gui;
use gui::run;

fn main()
{
    if let Err(err) = run("Archiwizator boży")
    {
        eprintln!("{}", err.to_string());
    }
}
