use std::sync::{Arc, Mutex};

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

