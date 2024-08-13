use std::collections::HashMap;
use std::path::Path;

pub const ARCHIVE_EXTENSION: &str = "xca";

pub fn is_a_subdirectory(superpath: &str, subpath: &str) -> bool
{
    subpath.to_string().starts_with(superpath)
}

pub fn get_superpath(path: &str) -> String
{
    Path::new(path)
        .parent()
        .unwrap_or(Path::new(""))
        .to_str()
        .unwrap_or("")
        .to_string()
}

pub fn sanitize_path(path: &String) -> String
{
    Path::new(path)
        .to_str()
        .unwrap_or("")
        .replace("\"", "")// Remove the quotes.
        .replace("\\", "/")// Replace backslash with slash.
        .trim_end_matches("/")
        .to_string()
}

pub fn sanitize_output_path(path: &String) -> String
{
    let path_initially_sanitized = sanitize_path(path);
    match path_initially_sanitized.ends_with(ARCHIVE_EXTENSION)
    {
        true => path_initially_sanitized,
        false => format!("{}.{}", path_initially_sanitized, ARCHIVE_EXTENSION)
    }
}

pub fn get_display_paths(paths: &[String]) -> HashMap<String, String>
{
    let mut indented_paths: HashMap<String, String> = paths.iter().map(|path|
    {
        let truncated_path = match path.rfind(' ')
        {
            Some(index) => &path[..index],
            None => path,
        };

        let slash_count = truncated_path.matches('/').count();
        let indent = " ".repeat(slash_count * 4); // 4 spaces for one level

        let final_path = truncated_path.rsplit_once('/')
            .map(|(_, remainder)| remainder)
            .unwrap_or(truncated_path);

        let display_path = format!("{}{}", indent, final_path);
        (path.clone(), display_path)
    })
        .collect();

    // Align all paths to the left.
    let min_leading_spaces = indented_paths.values()
        .map(|s| s.chars().take_while(|c| *c == ' ').count())
        .min()
        .unwrap_or(0);

    for indented_path in indented_paths.values_mut()
    {
        let aligned_path = indented_path.chars().skip(min_leading_spaces).collect();
        *indented_path = aligned_path;
    }

    indented_paths
}

