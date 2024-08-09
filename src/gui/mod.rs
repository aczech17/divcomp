use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use eframe::egui;
use rfd::FileDialog;
use crate::archive::{create_extractor_and_execute, display_archive_content, extract_archive};
use crate::compress::{archive_and_compress, CompressionMethod};
use crate::compress::CompressionMethod::{HUFFMAN, LZ77};
use crate::io_utils::path_utils::{parse_paths, sanitize_output_path, sanitize_path};
use crate::io_utils::path_utils::ARCHIVE_EXTENSION;

pub struct Gui
{
    compression_method: CompressionMethod,

    input_archive_path_input: String,
    choose_files_to_extract_input: String,
    output_directory_input: String,

    archive_content_display: (Arc<Mutex<Option<String>>>, String),

    paths_to_archive_input: String,
    output_archive_path_input: String,

    status_display: (Arc<Mutex<Option<String>>>, String),

    processing: bool,
}

impl Default for Gui
{
    fn default() -> Self
    {
        Self
        {
            compression_method: HUFFMAN,
            input_archive_path_input: String::new(),
            choose_files_to_extract_input: String::new(),
            output_directory_input: String::new(),
            archive_content_display: (Arc::new(Mutex::new(None)), String::new()),
            paths_to_archive_input: String::new(),
            output_archive_path_input: String::new(),
            status_display: (Arc::new(Mutex::new(None)), String::new()),
            processing: false,
        }
    }
}

macro_rules! spawn_thread
{
    ($self: ident, $result_variable: ident, $code: block) =>
    {
        $self.processing = true;

        let exec_result = Arc::clone(&$self.$result_variable.0);

        thread::spawn(move ||
        {
            let result = $code;

            let mut exec_result_lock = exec_result.lock().unwrap();
            *exec_result_lock = Some(result);
        });
        $self.processing = false;
    };
}

macro_rules! display_archive
{
    ($self: ident, $input_path:expr) =>
    {
        if !$self.processing
        {
            $self.processing = true;
            $self.archive_content_display.1 = String::new();
            let input_path = sanitize_path($input_path);
            let result = Arc::clone(&$self.archive_content_display.0);

            thread::spawn(move ||
            {
                let content = create_extractor_and_execute
                    (input_path, None, None, display_archive_content);

                let mut result_lock = result.lock().unwrap();
                *result_lock = Some(content)
            });

            $self.processing = false;
        }
    }
}

impl eframe::App for Gui
{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
    {
        egui::CentralPanel::default().show(ctx, |ui|
        {
            if let Some(display) = self.archive_content_display.0.lock().unwrap().take()
            {
                self.archive_content_display.1 = display;
            }

            if let Some(display) = self.status_display.0.lock().unwrap().take()
            {
                self.status_display.1 = display;
            }

            ui.horizontal(|ui|
            {
                if ui.button("Wybierz archiwum").clicked()
                {
                    if let Some(path) = FileDialog::new()
                        .add_filter("Archiwa xca", &[ARCHIVE_EXTENSION])
                        .add_filter("Wszystkie pliki", &["*"])
                        .pick_file()
                    {
                        let path = path.to_str().unwrap().to_string();
                        let path = sanitize_path(&path);

                        self.input_archive_path_input = path.clone();
                        display_archive!(self, &path);
                    }
                }
            });

            ui.horizontal(|ui|
            {
                ui.add(egui::TextEdit::singleline(&mut self.input_archive_path_input)
                    .hint_text("Ścieżka do archiwum"));

                if ui.button("Pokaż").clicked()
                {
                    display_archive!(self, &self.input_archive_path_input);
                }
            });

            ui.vertical(|ui|
            {
                ui.label("Zawartość archiwum:");
                ui.monospace(&self.archive_content_display.1);
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
                    self.status_display.1 = String::from("Wypakowywanie...");

                    let input_path = sanitize_path(&self.input_archive_path_input);
                    let output_directory = sanitize_path(&self.output_directory_input);
                    let chosen_paths = parse_paths(&self.choose_files_to_extract_input);

                    spawn_thread!(self, status_display,
                    {
                        create_extractor_and_execute
                        (
                            input_path,
                            Some(chosen_paths),
                            Some(output_directory),
                            extract_archive
                        )
                    });
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
                            self.status_display.1 = format!("Plik {} nie istnieje.", path);
                            return;
                        }
                    }


                    let output_path = sanitize_output_path(&self.output_archive_path_input);
                    self.status_display.1 = String::from("Pakowanie...");


                    let compression_method = self.compression_method;

                    spawn_thread!(self, status_display,
                    {
                        match archive_and_compress(input_paths, output_path, compression_method)
                        {
                            Ok(_) => "Spakowano.".to_string(),
                            Err(err_msg) => err_msg,
                        }
                    });
                }

                ui.vertical(|ui|
                {
                    ui.label("Wybierz metodę kompresji:");
                    ui.horizontal(|ui|
                    {
                        ui.radio_value(&mut self.compression_method, HUFFMAN, "Huffman");
                        ui.radio_value(&mut self.compression_method, LZ77, "LZ77");
                    });
                });
            });

            ui.horizontal(|ui|
            {
                ui.monospace(&mut self.status_display.1);
            });
        });
    }
}

pub fn run(window_name: &str, archive_argument: Option<String>) -> eframe::Result
{
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let options = eframe::NativeOptions
    {
        ..Default::default()
    };

    let mut gui = Gui::default();
    if let Some(path) = archive_argument
    {
        gui.input_archive_path_input = path.clone();
        display_archive!(gui, &path);
    }

    eframe::run_native
    (
        window_name,
        options,
        Box::new(|_cc|
        {
            Ok(Box::new(gui))
        }),
    )
}
