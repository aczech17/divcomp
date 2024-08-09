const ARCHIVE_EXTENSION: &str = ".xca";

pub fn is_a_subdirectory(superpath: &str, subpath: &str) -> bool
{
    let superdirectories: Vec<&str> = superpath.split("/")
        .collect();
    let subdirectories: Vec<&str> = subpath.split("/")
        .collect();

    if superdirectories.len() > subdirectories.len()
    {
        return false;
    }

    for (i, elem) in superdirectories.iter().enumerate()
    {
        if &subdirectories[i] != elem
        {
            return false;
        }
    }
    true
}

pub fn get_superpath(path: &str) -> String
{
    match path.rfind("/")
    {
        Some(pos) => path[..pos + 1].to_string(),
        None => String::new(),
    }
}

pub fn sanitize_path(path: &String) -> String
{
    let mut path = path.replace("\\", "/")
        .replace("\"", "");
    if path.ends_with('/')
    {
        path.pop();
    }

    path
}

pub fn sanitize_output_path(path: &String) -> String
{
    let path_initially_sanitized = sanitize_path(path);
    match path_initially_sanitized.ends_with(ARCHIVE_EXTENSION)
    {
        true => path_initially_sanitized,
        false => format!("{}{}", path_initially_sanitized, ARCHIVE_EXTENSION)
    }
}

fn sanitize_all_paths(paths: Vec<String>) -> Vec<String>
{
    let mut paths: Vec<String> = paths.iter()
        .map(|path| sanitize_path(path))
        .collect();

    paths.sort();
    paths.dedup();

    paths
}

pub fn parse_paths(text: &str) -> Vec<String>
{
    let paths: Vec<String> = text
        .lines()
        .map(|line| line.to_string())
        .collect();

    sanitize_all_paths(paths)
}


