#[cfg(target_os = "windows")]
pub const SLASH: &str = "\\";

#[cfg(not(target_os = "windows"))]
pub const SLASH: &str = "/";


pub fn is_a_subdirectory(superpath: &str, subpath: &str) -> bool
{
    let superdirectories: Vec<&str> = superpath.split(SLASH)
        .collect();
    let subdirectories: Vec<&str> = subpath.split(SLASH)
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
    match path.rfind(SLASH)
    {
        Some(pos) => path[..pos + 1].to_string(),
        None => String::new(),
    }
}
