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

