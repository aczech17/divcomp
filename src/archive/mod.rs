use crate::archive::extractor::Extractor;
use crate::compress::decompress_error_to_string;

pub mod directory_info;
mod archive_header;
pub mod archive;
pub mod extractor;

pub fn create_extractor_and_execute
(
    input_path: String,
    paths: Option<Vec<String>>,
    output_path: Option<String>,
    function: fn(Extractor, Option<Vec<String>>, Option<String>) -> String
)
    -> String
{
    let extractor = match Extractor::new(input_path)
    {
        Ok(ext) => ext,
        Err(err) => return decompress_error_to_string(err),
    };

    function(extractor, paths, output_path)
}

pub fn display_archive_content
(
    extractor: Extractor,
    _paths: Option<Vec<String>>,
    _output_path: Option<String>
)
    -> String
{
    extractor.get_archive_info()
        .iter()
        .map(|(path, size)| format!("{} {:?}", path, size))
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn extract_archive
(
    mut extractor: Extractor,
    paths: Option<Vec<String>>,
    output_path: Option<String>
)
    -> String
{
    let chosen_paths = paths.unwrap();
    let output_path = output_path.unwrap();

    let extraction_status = match chosen_paths.is_empty()
    {
        true => extractor.extract_all(output_path),
        false => extractor.extract_paths(chosen_paths, output_path)
    };

    match extraction_status
    {
        Ok(_) => "Wypakowano.".to_string(),
        Err(err) => decompress_error_to_string(err),
    }
}
