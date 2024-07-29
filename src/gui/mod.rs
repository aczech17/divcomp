use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use eframe::egui;
use crate::archive::{create_extractor_and_execute, display_archive_content, extract_archive};
use crate::compress::compress::archive_and_compress;
use crate::io_utils::path_utils::{parse_paths, sanitize_path};

pub struct GUI
{
    input_archive_path_input: String,
    choose_files_to_extract_input: String,
    output_directory_input: String,

    archive_content_display_result: Arc<Mutex<Option<String>>>,
    archive_content_display: String,

    paths_to_archive_input: String,
    output_archive_path_input: String,

    status_display_result: Arc<Mutex<Option<String>>>,
    status_display: String,

    processing: bool,
}

impl Default for GUI
{
    fn default() -> Self
    {
        Self
        {
            input_archive_path_input: String::new(),
            choose_files_to_extract_input: String::new(),
            output_directory_input: String::new(),
            archive_content_display_result: Arc::new(Mutex::new(None)),
            archive_content_display: String::new(),
            paths_to_archive_input: String::new(),
            output_archive_path_input: String::new(),
            status_display_result: Arc::new(Mutex::new(None)),
            status_display: String::new(),
            processing: false,
        }
    }
}

impl eframe::App for GUI
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        egui::CentralPanel::default().show(ctx, |ui|
        {
            if let Some(display) = self.archive_content_display_result.lock().unwrap().take()
            {
                self.archive_content_display = display;
            }

            if let Some(display) = self.status_display_result.lock().unwrap().take()
            {
                self.status_display = display;
            }

            ui.horizontal(|ui|
            {
                ui.add(egui::TextEdit::singleline(&mut self.input_archive_path_input)
                    .hint_text("Ścieżka do archiwum"));

                if ui.button("Pokaż").clicked()
                {
                    if self.processing
                    {
                        return;
                    }

                    self.processing = true;
                    self.archive_content_display = String::new();
                    let input_path = sanitize_path(&self.input_archive_path_input);
                    let result = Arc::clone(&self.archive_content_display_result);

                    thread::spawn(move ||
                    {
                        let content = create_extractor_and_execute
                            (input_path, None, None, display_archive_content);

                        let mut result_lock = result.lock().unwrap();
                        *result_lock = Some(content)
                    });

                    self.processing = false;
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
                    if self.processing
                    {
                        return;
                    }

                    self.processing = true;
                    self.status_display = String::from("Wypakowywanie...");

                    let input_path = sanitize_path(&self.input_archive_path_input);
                    let output_directory = sanitize_path(&self.output_directory_input);
                    let chosen_paths = parse_paths(&self.choose_files_to_extract_input);
                    let result = Arc::clone(&self.status_display_result);

                    thread::spawn(move ||
                    {
                        let status_message = create_extractor_and_execute
                            (
                                input_path,
                                Some(chosen_paths),
                                Some(output_directory),
                                extract_archive
                            );

                        let mut result_lock = result.lock().unwrap();
                        *result_lock = Some(status_message);
                    });
                    self.processing = false;
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
                    if self.processing
                    {
                        return;
                    }

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

                    self.processing = true;
                    self.status_display = String::from("Pakowanie...");
                    let result = Arc::clone(&self.status_display_result);

                    thread::spawn(move ||
                    {
                        let status_message = match archive_and_compress(input_paths, output_path)
                        {
                            Ok(_) => "Spakowano.".to_string(),
                            Err(err_msg) => err_msg,
                        };

                        let mut result_lock = result.lock().unwrap();
                        *result_lock = Some(status_message);
                    });

                    self.processing = false;
                }
            });

            ui.horizontal(|ui|
            {
                ui.monospace(&mut self.status_display);
            });
        });
    }
}

pub fn run(window_name: &str) -> eframe::Result
{
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions
    {
        ..Default::default()
    };

    eframe::run_native
    (
        window_name,
        options,
        Box::new(|_cc|
        {
            Ok(Box::<GUI>::default())
        }),
    )
}
