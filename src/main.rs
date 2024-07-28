#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use main_module::config::parse_arguments;

use crate::main_module::config::execute;

mod main_module;
mod compress;
mod archive;
mod io_utils;


use eframe::egui;
use crate::archive::extractor::Extractor;
use crate::main_module::archive_and_compress;

fn sanitize_path(path: &String) -> String
{
    let mut path = path.replace("\\", "/")
        .replace("\"", "");
    if path.ends_with('/')
    {
        path.pop();
    }

    path
}

fn sanitize_all_paths(paths: Vec<String>) -> Vec<String>
{
    let mut paths: Vec<String> = paths.iter()
        .map(|path| sanitize_path(path))
        .collect();

    paths.sort();
    paths.dedup();

    paths
}


struct MyApp
{
    path_to_display_input: String,
    archive_content_display: String,
    paths_to_archive_input: String,
    output_archive_path_input: String,
    archive_status: String,
}

impl Default for MyApp
{
    fn default() -> Self
    {
        Self
        {
            path_to_display_input: String::new(),
            archive_content_display: String::new(),
            paths_to_archive_input: String::new(),
            output_archive_path_input: String::new(),
            archive_status: String::new(),
        }
    }
}

impl eframe::App for MyApp
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        egui::CentralPanel::default().show(ctx, |ui|
        {
            ui.horizontal(|ui|
            {
                // Pole tekstowe
                ui.add(egui::TextEdit::singleline(&mut self.path_to_display_input)
                    .hint_text("Ścieżka do archiwum"));

                // Przycisk "Pokaż"
                if ui.button("Pokaż").clicked()
                {
                    let input_path = sanitize_path(&self.path_to_display_input);
                    let mut extractor = Extractor::new(input_path)
                        .expect("Bad extractor");

                    let content = extractor.get_archive_info()
                        .iter()
                        .map(|(path, size)| format!("{} {:?}", path, size))
                        .collect::<Vec<String>>()
                        .join("\n");

                    self.archive_content_display = content;
                }
            });

            ui.vertical(|ui|
            {
                ui.label("Zawartość archiwum:");
                ui.monospace(&self.archive_content_display);
            });

            ui.horizontal(|ui|
            {
                ui.add(egui::TextEdit::multiline(&mut self.paths_to_archive_input)
                   .hint_text("Ścieżki plików (katalogów) do spakowania..."));
            });

            ui.horizontal(|ui|
            {
                ui.add(egui::TextEdit::singleline(&mut self.output_archive_path_input)
                    .hint_text("Ścieżka do wynikowego archiwum..."));
                if ui.button("Spakuj").clicked()
                {
                    self.archive_status = String::new();

                    let paths: Vec<String> = self.paths_to_archive_input
                        .lines()
                        .map(|line| line.to_string())
                        .collect();
                    let input_paths = sanitize_all_paths(paths);

                    let output_path = sanitize_path(&self.output_archive_path_input);

                    self.archive_status = match archive_and_compress(input_paths, output_path)
                    {
                        Ok(_) => "Spakowano.".to_string(),
                        Err(err_msg) => err_msg,
                    };
                }
            });

            ui.horizontal(|ui|
            {
                ui.monospace(&mut self.archive_status);
            });
        });
    }
}

fn main() -> eframe::Result
{
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions
    {
        ..Default::default()
    };

    eframe::run_native(
        "Archiwizator boży",
        options,
        Box::new(|_cc|
        {
            Ok(Box::<MyApp>::default())
        }),
    )
}
