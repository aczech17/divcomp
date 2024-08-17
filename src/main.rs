#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide the console window

mod io_utils;
mod archive;
mod compress;
mod gui;

fn main()
{
    let archive_path = std::env::args().nth(1);

    if let Err(err) = gui::run("Archiwizator boży", archive_path)
    {
        eprintln!("{}", err);
    }
}
