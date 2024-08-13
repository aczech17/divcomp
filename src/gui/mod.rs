mod util;
use util::MultithreadedData;
use std::collections::{HashMap, HashSet};
use std::{path::Path, thread, sync::Arc};
use crate::io_utils::path_utils::
    {ARCHIVE_EXTENSION, get_display_paths, sanitize_path, sanitize_output_path};
use crate::archive::extractor::Extractor;
use crate::compress::
{
    CompressionMethod,
    CompressionMethod::{HUFFMAN, LZ77},
    pack_and_compress,
};
use eframe::egui;
use egui::Ui;
use egui::InnerResponse;
use rfd::FileDialog;


pub struct Gui
{
    compression_method: CompressionMethod,

    input_archive_path: String,
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
            input_archive_path: String::new(),
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
    fn spawn_task(&mut self, task: impl FnOnce() -> String + Send + 'static)
    {
        self.processing = true;

        let exec_result = Arc::clone(&self.status_display.result);

        thread::spawn(move ||
        {
            let result = task();

            let mut exec_result_lock = exec_result.lock().unwrap();
            *exec_result_lock = Some(result);
        });
        self.processing = false;
    }
}

impl Gui // packing
{
    fn select_files_to_pack(&mut self)
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

    fn select_folders_to_pack(&mut self)
    {
        if let Some(paths) = FileDialog::new()
            .set_title("Wybierz foldery")
            .pick_folders()
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

    fn show_paths_to_pack(&mut self, ui: &mut Ui) -> InnerResponse<()>
    {
        ui.vertical(|ui|
        {
            let mut right_clicked_paths = Vec::new();

            // Find paths that were right-clicked...
            for path in &self.paths_to_pack
            {
                let path_label = ui.selectable_label(false, path);
                if path_label.hovered() && ui.input(|i| i.pointer.secondary_clicked())
                {
                    right_clicked_paths.push(path.clone());
                }
            }

            // ... and remove them.
            for right_clicked_path in right_clicked_paths
            {
                self.paths_to_pack.retain(|path| path != &right_clicked_path);
            }
        })
    }

    fn do_packing(&mut self)
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

        self.spawn_task(move ||
        {
            match pack_and_compress(input_paths, output_path, compression_method)
            {
                Ok(_) => "Spakowano.".to_string(),
                Err(err_msg) => err_msg,
            }
        });
    }

    fn packing_vertical(&mut self, ui: &mut Ui) -> InnerResponse<()>
    {
        ui.vertical(|ui|
        {
            ui.label("Pakowanie:");
            ui.horizontal(|ui|
            {
                if ui.button("Dodaj pliki do spakowania").clicked()
                {
                    self.select_files_to_pack();
                }

                if ui.button("Dodaj foldery do spakowania").clicked()
                {
                    self.select_folders_to_pack();
                }
            });

            ui.label("Ścieżki do spakowania:");

            egui::ScrollArea::vertical()
                .min_scrolled_height(300.0)
                .max_height(600.0)
                .show(ui, |ui| self.show_paths_to_pack(ui));

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
                    self.do_packing();
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
        })
    }
}

impl Gui // extraction
{
    fn select_input_archive_path(&mut self)
    {
        let chosen_path = FileDialog::new()
            .add_filter("Archiwa xca", &[ARCHIVE_EXTENSION])
            .add_filter("Wszystkie pliki", &["*"])
            .pick_file();

        if let Some(path) = chosen_path
        {
            let path = path.to_str().unwrap().to_string();
            let path = sanitize_path(&path);

            self.input_archive_path = path.clone();
            self.show_archive_content();
        }
    }

    fn select_output_directory(&mut self)
    {
        if let Some(path) = FileDialog::new().pick_folder()
        {
            self.output_directory = path.to_str().unwrap().to_string();
        }
    }

    fn show_packed_files_selection(&mut self, ui: &mut Ui) -> InnerResponse<()>
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
    }

    fn show_archive_content(&mut self)
    {
        if self.processing
        {
            return;
        }

        self.processing = true;
        let input_path = sanitize_path(&self.input_archive_path);
        let result = Arc::clone(&self.archive_content.result);

        thread::spawn(move ||
        {
            let content = match Extractor::new(input_path)
            {
                Ok(extractor) => extractor.to_string(),
                Err(err) => err.to_string(),
            };

            let paths: Vec<String> = content
                .lines()
                .map(|line| line.to_string())
                .collect();

            let mut result_lock = result.lock().unwrap();
            *result_lock = Some(paths)
        });

        self.processing = false;
    }

    fn do_extraction(&mut self)
    {
        if self.processing
        {
            return;
        }

        let input_path = sanitize_path(&self.input_archive_path);
        let output_directory = sanitize_path(&self.output_directory);

        // Get chosen_paths from clicked position of the selection menu
        // and remove everything after the actual path,
        // e.g., "Some", "None" and all that shit.
        let chosen_paths: Vec<String> = self.selected_archive_items.clone()
            .into_iter()
            .map(|s| s.clone().split_once(' ').map_or(s, |(before, _)| before.to_string()))
            .collect();

        if chosen_paths.is_empty()
        {
            self.status_display
                .set_content(String::from("Wybierz pliki do wypakowania."));
            return;
        }

        if input_path.is_empty()
        {
            self.status_display
                .set_content(String::from("Podaj ścieżkę archiwum."));
            return;
        }

        if output_directory.is_empty()
        {
            self.status_display
                .set_content(String::from("Podaj ścieżkę do wypakowania."));
            return;
        }

        self.processing = true;
        self.status_display.set_content(String::from("Wypakowywanie..."));

        self.spawn_task(||
        {
            match Extractor::new(input_path)
            {
                Ok(mut extractor) =>
                {
                    match extractor.extract_paths(chosen_paths, output_directory)
                    {
                        Ok(_) => "Wypakowano".to_string(),
                        Err(err) => err.to_string(),
                    }
                },
                Err(err) => err.to_string(),
            }
        });
    }

    fn extraction_vertical(&mut self, ui: &mut Ui) -> InnerResponse<()>
    {
        ui.vertical(|ui|
        {
            ui.label("Wypakowywanie");
            ui.horizontal(|ui|
            {
                if ui.button("Wybierz archiwum").clicked()
                {
                    self.select_input_archive_path();
                }
            });

            ui.horizontal(|ui|
            {
                ui.add(egui::TextEdit::singleline(&mut self.input_archive_path)
                    .hint_text("Ścieżka do archiwum"));

                if ui.button("Pokaż").clicked()
                {
                    self.show_archive_content();
                }
            });

            ui.vertical(|ui|
            {
                ui.label("Zawartość archiwum:");
                egui::ScrollArea::vertical()
                    .min_scrolled_height(300.0)
                    .max_height(600.0)
                    .show(ui, |ui| self.show_packed_files_selection(ui));
            });

            ui.horizontal(|ui|
            {
                ui.add(egui::TextEdit::singleline(&mut self.output_directory)
                    .hint_text("Wypakuj do..."));

                if ui.button("Wybierz folder do wypakowania").clicked()
                {
                    self.select_output_directory();
                }

                if ui.button("Wypakuj").clicked()
                {
                    self.do_extraction();
                }
            });
        })
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
                self.packing_vertical(ui);
                self.extraction_vertical(ui);
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
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    let mut gui = Gui::default();
    if let Some(path) = archive_argument
    {
        gui.input_archive_path = path.clone();
        gui.show_archive_content();
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
