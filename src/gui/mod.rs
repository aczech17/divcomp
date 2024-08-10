use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use eframe::egui;
use rfd::FileDialog;
use crate::archive::{create_extractor_and_execute, display_archive_content, extract_archive};
use crate::compress::{archive_and_compress, CompressionMethod};
use crate::compress::CompressionMethod::{HUFFMAN, LZ77};
use crate::io_utils::path_utils::{sanitize_output_path, sanitize_path, get_display_paths};
use crate::io_utils::path_utils::ARCHIVE_EXTENSION;

struct MultithreadedData<T>
{
    pub result: Arc<Mutex<Option<T>>>,
    content: T,
}

impl<T> MultithreadedData<T>
{
    fn new(content: T) -> Self
    {
        Self
        {
            result: Arc::new(Mutex::new(None)),
            content,
        }
    }

    fn set_new_content(&mut self) -> bool
    {
        if let Some(content) = self.result.lock().unwrap().take()
        {
            self.content = content;
            return true;
        }

        false
    }

    fn set_content(&mut self, value: T)
    {
        self.content = value;
    }

    fn get_content(&self) -> &T
    {
        &self.content
    }
}

pub struct Gui
{
    compression_method: CompressionMethod,

    input_archive_path_input: String,
    output_directory: String,

    archive_content: MultithreadedData<Vec<String>>,
    selected_archive_items: HashSet<String>,
    display_path_map: HashMap<String, String>,

    paths_to_pack: Vec<String>,
    output_archive_path: String,

    status_display: MultithreadedData<String>,

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
            output_directory: String::new(),
            archive_content: MultithreadedData::new(vec![]),
            selected_archive_items: HashSet::new(),
            display_path_map: HashMap::new(),
            paths_to_pack: Vec::new(),
            output_archive_path: String::new(),
            status_display: MultithreadedData::new(String::new()),
            processing: false,
        }
    }
}

impl Gui
{
    fn select_output_archive_path(&mut self)
    {
        if let Some(path) = FileDialog::new()
            .set_title("Wybierz lokalizację i wpisz nazwę.")
            .add_filter("Archiwum xca", &[ARCHIVE_EXTENSION])
            .save_file()
        {
            self.output_archive_path = path.to_str().unwrap().to_string();
        }
    }
}

macro_rules! spawn_thread
{
    ($self: ident, $result_variable: ident, $code: block) =>
    {
        $self.processing = true;

        let exec_result = Arc::clone(&$self.$result_variable.result);

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
            let input_path = sanitize_path($input_path);
            let result = Arc::clone(&$self.archive_content.result);

            thread::spawn(move ||
            {
                let content = create_extractor_and_execute
                    (input_path, None, None, display_archive_content);
                let paths: Vec<String> = content
                    .lines()
                    .map(|line| line.to_string())
                    .collect();


                let mut result_lock = result.lock().unwrap();
                *result_lock = Some(paths)
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
            if self.archive_content.set_new_content()
            {
                let archive_content = self.archive_content.get_content();
                self.display_path_map = get_display_paths(archive_content);
            }

            self.status_display.set_new_content();

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
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui|
                {
                    ui.vertical(|ui|
                    {
                        for path in self.archive_content.get_content().iter()
                        {
                            let is_selected = self.selected_archive_items.contains(path);
                            let display_path = self.display_path_map.get(path).unwrap();

                            let response = ui.selectable_label(is_selected, display_path);

                            if response.clicked()
                            {
                                if is_selected
                                {
                                    // Unclick
                                    self.selected_archive_items.remove(path);
                                }
                                else
                                {
                                    // Click
                                    self.selected_archive_items.insert(path.clone());
                                }
                            }
                        }
                    })
                })
            });

            ui.horizontal(|ui|
            {
                ui.add(egui::TextEdit::singleline(&mut self.output_directory)
                    .hint_text("Wypakuj do..."));

                if ui.button("Wybierz folder do wypakowania").clicked()
                {
                    if let Some(path) = FileDialog::new().pick_folder()
                    {
                        self.output_directory = path.to_str().unwrap().to_string();
                    }
                }

                if ui.button("Wypakuj").clicked()
                {
                    if self.processing
                    {
                        return;
                    }

                    self.processing = true;
                    self.status_display.set_content(String::from("Wypakowywanie..."));

                    let input_path = sanitize_path(&self.input_archive_path_input);
                    let output_directory = sanitize_path(&self.output_directory);


                    // Get chosen_paths from clicked position of the selection menu
                    // and remove everything after the actual path,
                    // e.g., "Some", "None" and all that shit.
                    let chosen_paths = self.selected_archive_items.clone()
                        .into_iter()
                        .map(|s| s.clone().split_once(' ')
                            .map_or(s, |(before, _)| before.to_string()))
                        .collect();

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
                if ui.button("Dodaj pliki do spakowania").clicked()
                {
                    if let Some(paths) = FileDialog::new()
                        .set_title("Wybierz pliki")
                        .pick_files()
                    {
                        let paths_to_pack: Vec<String> = paths.into_iter()
                            .map(|path| path.to_str().unwrap_or("").to_string())
                            .collect();

                        self.paths_to_pack.extend(paths_to_pack);

                        // Now remove the duplicates.
                        let unique_paths: HashSet<String> = self.paths_to_pack.iter().cloned().collect();
                        self.paths_to_pack = unique_paths.into_iter().collect();
                    }
                }
            });

            ui.label("Ścieżki do spakowania:");
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui|
            {
                ui.vertical(|ui|
                {
                    let mut paths_to_remove = Vec::new();

                    for path in &self.paths_to_pack
                    {
                        let path_label = ui.selectable_label(false, path);
                        if path_label.hovered() && ui.input(|i| i.pointer.secondary_clicked())
                        {
                            paths_to_remove.push(path.clone());
                        }
                    }

                    // Remove the paths that were right-clicked.
                    for path_to_remove in paths_to_remove
                    {
                        self.paths_to_pack.retain(|path| path != &path_to_remove);
                    }
                })
            });

            if ui.button("Wyczyść").clicked()
            {
                self.paths_to_pack.clear();
            }

            ui.horizontal(|ui|
            {
                ui.add(egui::TextEdit::singleline(&mut self.output_archive_path)
                    .hint_text("Ścieżka do wynikowego archiwum..."));

                if ui.button("Wybierz lokalizację archiwum").clicked()
                {
                    self.select_output_archive_path();
                }

                if ui.button("Spakuj").clicked()
                {
                    if self.processing
                    {
                        return;
                    }

                    let input_paths = self.paths_to_pack.clone();

                    for path in &input_paths
                    {
                        if !Path::new(path).exists()
                        {
                            self.status_display.set_content(format!("Plik {} nie istnieje.", path));
                            return;
                        }
                    }

                    let output_path = sanitize_output_path(&self.output_archive_path);
                    self.status_display.set_content(String::from("Pakowanie..."));


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
                ui.monospace(self.status_display.get_content());
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
