use std::env::args;

use crate::compress::decompress::DecompressError;
use crate::main_module::{archive_and_compress, print_archive_info};
use crate::archive::extractor::Extractor;

#[derive(Eq, PartialEq)]
pub enum ProgramConfig
{
    Archive { input_paths: Vec<String>, output_archive_path: String},
    ExtractAll { archive_path: String, output_directory: String},
    ExtractPaths { archive_path: String, chosen_paths: Vec<String>, output_directory: String},
    Display { archive_path: String},
}

fn parse_archive_arguments(args: Vec<String>) -> Result<ProgramConfig, String>
{
    let o_position = match args.iter().position(|s| s == "-o")
    {
        Some(pos) => pos,
        None => return Err(String::from("Output path not given.")),
    };

    let mut input_paths = match args.iter().position(|s| s == "-a")
    {
        Some(a_position) => args[a_position + 1..o_position].to_vec(),
        None => vec![],
    };
    // Remove duplicated filenames so that the files aren't archived multiple times.
    input_paths.sort();
    input_paths.dedup();

    // Convert backslash to slash.
    let mut input_paths: Vec<String> = input_paths.iter()
        .map(|input_path| input_path.replace("\\", "/").to_string())
        .collect();

    // Remove trailing slashes.
    for path in &mut input_paths
    {
        if path.ends_with('/')
        {
            path.pop();
        }
    }


    let output_archive_path = args[o_position + 1].clone();

    let option = ProgramConfig::Archive { input_paths, output_archive_path };
    Ok(option)
}

fn parse_extract_all_arguments(args: Vec<String>) -> Result<ProgramConfig, String>
{

    let ea_position = args.iter().position(|s| s == "-ea")
        .unwrap();

    let archive_filename = args[ea_position + 1].clone();

    let output_directory = match args.iter().position(|s| s == "-o")
    {
        Some(pos) => args[pos + 1].to_string(),
        None => "".to_string(),
    };

    let option = ProgramConfig::ExtractAll { archive_path: archive_filename, output_directory };
    Ok(option)
}

fn parse_extract_paths_arguments(args: Vec<String>) -> Result<ProgramConfig, String>
{
    // -ep [nazwa archiwum] -c [wybrane pliki] -o [ścieżka do katalogu docelowego]

    let ep_position = args.iter().position(|s| s == "-ep")
        .unwrap();
    let c_position = match args.iter().position(|s| s == "-c")
    {
        Some(pos) => pos,
        None => return Err("No chosen path given.".to_string()),
    };

    if c_position - ep_position != 2
    {
        return Err("Bad arguments format.".to_string());
    }

    let archive_filename = args[ep_position + 1].to_string();

    if c_position + 1 >= args.len()
    {
        return Err("No chosen path given.".to_string());
    }

    let o_position = args.iter().position(|s| s == "-o");

    let mut chosen_paths = match o_position
    {
        None => args[c_position + 1..].to_vec(),
        Some(pos) => args[c_position + 1..pos].to_vec(),
    };
    // Remove duplicates.
    chosen_paths.sort();
    chosen_paths.dedup();


    let output_directory = match o_position
    {
        Some(pos) => args[pos + 1].to_string(),
        None => "".to_string(),
    };

    let option = ProgramConfig::ExtractPaths { archive_path: archive_filename, chosen_paths, output_directory };
    Ok(option)
}

fn parse_display_arguments(args: Vec<String>) -> Result<ProgramConfig, String>
{
    let archive_filename = match args.iter().position(|s| s == "-d")
    {
        None => return Err("No archive path was given.".to_string()),
        Some(pos) =>
        {
            if pos + 1 >= args.len()
            {
                return Err("Bad arguments format.".to_string());
            }
            args[pos + 1].to_string()
        }
    };

    let option = ProgramConfig::Display { archive_path: archive_filename };
    Ok(option)
}

pub fn parse_arguments() -> Result<ProgramConfig, String>
{
    let args: Vec<String> = args().collect();

    let usage = "\
    1. Spakowanie plików:\n\
    \t-a [nazwy plików] -o [wybrana nazwa archiwum]\n\
    2. Wypakowanie archiwum\n\
    \t-ea [nazwa archiwum] -o [ścieżka do katalogu docelowego]\n\
    3. Wypakowanie części archiwum\n\
    \t-ep [nazwa archiwum] -c [wybrane pliki/katalogi do wypakowania] -o [ścieżka do \
     katalogu docelowego]\n\
    4. Podejrzenie archiwum\n\
    \t-d [nazwa archiwum]";


    if args.len() < 3
    {
        return Err(String::from(usage));
    }

    match args[1].as_str()
    {
        "-a"    => parse_archive_arguments(args),
        "-ea"   => parse_extract_all_arguments(args),
        "-ep"   => parse_extract_paths_arguments(args),
        "-d"    => parse_display_arguments(args),
        _       => Err(usage.to_string()),
    }
}

fn decompress_error_to_string(error: DecompressError) -> String
{
    match error
    {
        DecompressError::BadFormat | DecompressError::FileTooShort | DecompressError::EmptyFile =>
            "Invalid archive.",
        DecompressError::FileOpenError => "Could not open the file.",
        DecompressError::Other => "Error while decompressing.",
    }.to_string()
}

pub fn execute(program_config: ProgramConfig) -> Result<(), String>
{
    match program_config
    {
        ProgramConfig::Archive { input_paths, output_archive_path } =>
            archive_and_compress(input_paths, output_archive_path),

        ProgramConfig::ExtractAll { archive_path, output_directory } =>
        {
            let mut extractor = Extractor::new(archive_path)
                .map_err(|err| decompress_error_to_string(err))?;

            extractor.extract_all(output_directory)
                .map_err(|err| decompress_error_to_string(err))
        },

        ProgramConfig::ExtractPaths
        {archive_path, chosen_paths, output_directory} =>
        {
            let mut extractor = Extractor::new(archive_path)
                .map_err(|err| decompress_error_to_string(err))?;

            extractor.extract_paths(chosen_paths, output_directory)
                .map_err(|err| decompress_error_to_string(err))
        },

        ProgramConfig::Display { archive_path } =>
        {
            let extractor = Extractor::new(archive_path)
                .map_err(|err| decompress_error_to_string(err))?;

            print_archive_info(&extractor);
            Ok(())
        }
    }
}
