use std::env;
use std::env::args;

use crate::compress_stage::decompress::DecompressError;
use crate::io_utils::path_utils::get_superpath;
use crate::main_module::{archive_and_compress, print_archive_info};
use crate::main_module::extractor::Extractor;

#[derive(Eq, PartialEq)]
pub enum ConfigOption
{
    Archive,
    ExtractAll,
    ExtractPath,
    Display,
}

pub struct ProgramConfig
{
    pub option: ConfigOption,
    pub input_filenames: Vec<String>,
    pub output_directory: String,
    pub output_archive_filename: Option<String>,
    pub chosen_path: Option<String>,
}

fn parse_archive_arguments(args: Vec<String>) -> Result<ProgramConfig, String>
{
    let o_position = match args.iter().position(|s| s == "-o")
    {
        Some(pos) => pos,
        None => return Err(String::from("Output path not given.")),
    };

    let input_filenames = match args.iter().position(|s| s == "-a")
    {
        Some(a_position) => args[a_position + 1..o_position].to_vec(),
        None => vec![],
    };

    let output = args[o_position + 1].clone();
    let output_directory = get_superpath(&output);

    let output_archive_filename = output.strip_prefix(&output_directory)
        .unwrap()
        .to_string();


    let config = ProgramConfig
    {
        option: ConfigOption::Archive,
        input_filenames,
        output_directory,
        output_archive_filename: Some(output_archive_filename),
        chosen_path: None,
    };

    Ok(config)
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

    let config = ProgramConfig
    {
        option: ConfigOption::ExtractAll,
        input_filenames: vec![archive_filename],
        output_directory,
        output_archive_filename: None,
        chosen_path: None,
    };


    Ok(config)
}

fn parse_extract_path_arguments(args: Vec<String>) -> Result<ProgramConfig, String>
{
    // divcomp.exe -ep [nazwa archiwum] -c [wybrany plik/katalog do wypakowania]
    // -o [ścieżka do katalogu docelowego]

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
    let chosen_path = args[c_position + 1].to_string();


    let output_directory = match args.iter().position(|s| s == "-o")
    {
        Some(pos) => args[pos + 1].to_string(),
        None => "".to_string(),
    };

    let config = ProgramConfig
    {
        option: ConfigOption::ExtractPath,
        input_filenames: vec![archive_filename],
        output_directory,
        output_archive_filename: None,
        chosen_path: Some(chosen_path),
    };

    Ok(config)
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

    let config = ProgramConfig
    {
        option: ConfigOption::Display,
        input_filenames: vec![archive_filename],
        output_directory: "".to_string(),
        output_archive_filename: None,
        chosen_path: None,
    };

    Ok(config)
}

pub fn parse_arguments() -> Result<ProgramConfig, String>
{
    let args: Vec<String> = args().collect();

    let current_path = env::current_dir().unwrap().to_str().unwrap().to_string();
    let program_name = args[0].clone()
        .strip_prefix(&current_path)
        .unwrap()
        .to_string();

    let usage = format!("\
    1. Spakowanie plików:\n\
    \t{} -a [nazwy plików] -o [wybrana nazwa archiwum]\n\n\
    2. Wypakowanie archiwum\n\
    \t{} -ea [nazwa archiwum] -o [ścieżka do katalogu docelowego]\n\n\
    3. Wypakowanie części archiwum\n\
    \t{} -ep [nazwa archiwum] -c [wybrany plik/katalog do wypakowania] -o [ścieżka do \
     katalogu docelowego]\n\n\
    4. Podejrzenie archiwum\n\
    \t{} -d [nazwa archiwum]\n\
    ",
    program_name, program_name, program_name, program_name);


    if args.len() < 3
    {
        return Err(String::from(usage));
    }

    match args[1].as_str()
    {
        "-a"    => parse_archive_arguments(args),
        "-ea"   => parse_extract_all_arguments(args),
        "-ep"   => parse_extract_path_arguments(args),
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
    if program_config.option == ConfigOption::Archive
    {
        return archive_and_compress
            (program_config.input_filenames, program_config.output_archive_filename.unwrap());
    }

    let archive_filename = program_config.input_filenames[0].clone();
    let mut extractor = Extractor::new(archive_filename)
        .map_err(|err| decompress_error_to_string(err))?;

    match program_config.option
    {
        ConfigOption::ExtractAll => extractor.extract_all(program_config.output_directory)
                .map_err(|err| decompress_error_to_string(err)),

        ConfigOption::ExtractPath =>
        {
            let chosen_path = program_config.chosen_path.unwrap();

            extractor.extract_path(chosen_path, program_config.output_directory)
                .map_err(|err| decompress_error_to_string(err))
        }

        ConfigOption::Display =>
        {
            print_archive_info(&extractor);
            Ok(())
        }

        _ => Ok(()),
    }
}
