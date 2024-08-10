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

#[macro_export]
macro_rules! spawn_thread
{
    ($self: ident, $result_variable: ident, $code: block) =>
    {
        use std::sync::Arc;

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

#[macro_export]
macro_rules! display_archive
{
    ($self: ident, $input_path:expr) =>
    {
        use std::sync::Arc;
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
