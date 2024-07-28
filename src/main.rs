#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::path::Path;
use main_module::config::parse_arguments;

use crate::main_module::config::execute;

mod main_module;
mod compress;
mod archive;
mod io_utils;


use eframe::egui;
use crate::archive::extractor::Extractor;
use crate::compress::decompress::{decompress_error_to_string, DecompressError};
use crate::io_utils::{parse_paths, sanitize_path};
use crate::main_module::archive_and_compress;


struct MyApp
{
    input_archive_path_input: String,
    choose_files_to_extract_input: String,
    output_directory_input: String,
    archive_content_display: String,
    paths_to_archive_input: String,
    output_archive_path_input: String,
    status_display: String,
}

impl Default for MyApp
{
    fn default() -> Self
    {
        Self
        {
            input_archive_path_input: String::new(),
            choose_files_to_extract_input: String::new(),
            output_directory_input: String::new(),
            archive_content_display: String::new(),
            paths_to_archive_input: String::new(),
            output_archive_path_input: String::new(),
            status_display: String::new(),
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
                ui.add(egui::TextEdit::singleline(&mut self.input_archive_path_input)
                    .hint_text("Ścieżka do archiwum"));

                if ui.button("Pokaż").clicked()
                {
                    let input_path = sanitize_path(&self.input_archive_path_input);
                    let mut extractor = match Extractor::new(input_path)
                    {
                        Ok(ext) => ext,
                        Err(err) =>
                        {
                            self.archive_content_display = decompress_error_to_string(err);
                            return;
                        }
                    };

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

            ui.vertical(|ui|
            {
                ui.add(egui::TextEdit::multiline(&mut self.choose_files_to_extract_input)
                    .hint_text("Wybrane ścieżki do wypakowania..."));
            });

            ui.horizontal(|ui|
            {
                ui.add(egui::TextEdit::singleline(&mut self.output_directory_input)
                    .hint_text("Wypakuj do..."));

                if ui.button("Wypakuj").clicked()
                {
                    let input_path = sanitize_path(&self.input_archive_path_input);
                    let mut extractor = match Extractor::new(input_path)
                    {
                        Ok(ext) => ext,
                        Err(err) =>
                            {
                                self.archive_content_display = decompress_error_to_string(err);
                                return;
                            }
                    };

                    let output_directory = self.output_directory_input.clone();
                    let extraction_result =
                        match self.choose_files_to_extract_input.as_str()
                    {
                        "" => extractor.extract_all(output_directory),
                        input =>
                        {
                            let chosen_paths = parse_paths(input);
                            extractor.extract_paths(chosen_paths, output_directory)
                        }
                    };

                    self.status_display = match extraction_result
                    {
                        Ok(_) => "Wypakowano".to_string(),
                        Err(err) => decompress_error_to_string(err),
                    };
                }
            });

            ui.vertical(|ui|
            {
                ui.label("Pakowanie:");
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
                    self.status_display = String::new();
                    let input_paths = parse_paths(&self.paths_to_archive_input);

                    for path in &input_paths
                    {
                        if !Path::new(path).exists()
                        {
                            self.status_display = format!("Plik {} nie istnieje.", path);
                            return;
                        }
                    }

                    let output_path = sanitize_path(&self.output_archive_path_input);

                    self.status_display = match archive_and_compress(input_paths, output_path)
                    {
                        Ok(_) => "Spakowano.".to_string(),
                        Err(err_msg) => err_msg,
                    };
                }
            });

            ui.horizontal(|ui|
            {
                ui.monospace(&mut self.status_display);
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
