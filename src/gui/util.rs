use std::sync::{Arc, Mutex};
use eframe::egui::IconData;

pub struct MultithreadedData<T>
{
    pub result: Arc<Mutex<Option<T>>>,
    content: T,
}

impl<T> MultithreadedData<T>
{
    pub fn new(content: T) -> Self
    {
        Self
        {
            result: Arc::new(Mutex::new(None)),
            content,
        }
    }

    pub fn set_new_content(&mut self) -> bool
    {
        if let Some(content) = self.result.lock().unwrap().take()
        {
            self.content = content;
            return true;
        }

        false
    }

    pub fn set_content(&mut self, value: T)
    {
        self.content = value;
    }

    pub fn get_content(&self) -> &T
    {
        &self.content
    }
}

pub fn load_icon() -> IconData
{
    let icon_bytes = include_bytes!("../../assets/icon.ico");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to open icon path.")
        .into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    IconData
    {
        rgba,
        width,
        height,
    }
}
