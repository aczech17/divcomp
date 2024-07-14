use walkdir::WalkDir;
use std::fs;

struct FilesystemNode
{
    is_directory: bool,
    name: String,
    subnodes: Vec<Box<FilesystemNode>>,
}

impl FilesystemNode
{
    fn new(paths: Vec<String>) -> FilesystemNode
    {
        let mut root = FilesystemNode
        {
            is_directory: true,
            name: "".to_string(),
            subnodes: vec![],
        };

        for ref path in paths
        {
            let mut subroot = FilesystemNode
            {
                is_directory: false,
                name: path.to_string(),
                subnodes: vec![],
            };

            Self::build_tree_recursive(path, &mut subroot);
            root.subnodes.push(Box::from(subroot));
        }

        root
    }

    fn build_tree_recursive(path: &str, node: &mut FilesystemNode)
    {
        let is_directory = fs::metadata(path).unwrap().is_dir();
        if !is_directory
        {
            return;
        }

        node.is_directory = true;
        for entry in WalkDir::new(path)
        {
            let entry = entry.as_ref().unwrap();
            let entry_name = entry.file_name().to_str().unwrap().to_string();

            let mut subnode = FilesystemNode
            {
                is_directory,
                name: entry_name.clone(),
                subnodes: vec![],
            };

            Self::build_tree_recursive(&entry_name, &mut subnode);
            node.subnodes.push(Box::from(subnode));
        }
    }
}

struct ArchiveHeader
{
    data_size: usize,

}
