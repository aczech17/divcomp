use std::env::args;
use std::path::Path;
use crate::compress_stage::decompress::DecompressError;
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
    pub input_filenames: Vec<String>,
    pub output_filename: Option<String>,
    pub option: ConfigOption,
}

pub fn parse_arguments() -> Result<ProgramConfig, String>
{
    let args: Vec<String> = args().collect();

    let program_name = args[0].clone();

    let usage = format!("\
    1. Spakowanie plików:\n\
    \t{} -a [nazwy plików] -o [wybrana nazwa archiwum]\n\n\
    2. Wypakowanie archiwum\n\
    \t{} -ea [nazwa archiwum]\n\n\
    3. Wypakowanie części archiwum\n\
    \t{} -ep [nazwa archiwum] -o [ścieżka do wypakowania]\n\n\
    4. Podejrzenie archiwum\n\
    \t{} -d [nazwa archiwum]\n\
    ",
    program_name, program_name, program_name, program_name);



    if args.len() < 3
    {
        return Err(String::from(usage));
    }

    let option = match args[1].as_str()
    {
        "-a"    => ConfigOption::Archive,
        "-ea"   => ConfigOption::ExtractAll,
        "-ep"   => ConfigOption::ExtractPath,
        "-d"    => ConfigOption::Display,
        _       => return Err(usage.to_string()),
    };

    let (inputs, output) = match args.iter().position(|s| s == "-o")
    {
        None =>
        {
            if option == ConfigOption::ExtractAll || option == ConfigOption::Display
            {
                let ins = args[2..].to_vec();
                let outs = None;

                (ins, outs)
            }
            else
            {
                return Err(String::from(usage));
            }
        }

        Some(pos) =>
        {
            let ins = args[2..pos].to_vec();
            let outs = Some(args[pos + 1].to_owned());

            (ins, outs)
        }
    };

    let inputs = inputs.iter()
        .map(|input| input.replace("\\", "/"))
        .collect();

    let output = match output
    {
        Some(out) => Some(out.replace("\\", "/")),
        None => None,
    };

    for input in &inputs
    {
        if !Path::new(input).exists()
        {
            return Err(format!("Path {} does not exist.", input));
        }
    }

    let config = ProgramConfig
    {
        input_filenames: inputs,
        output_filename: output,
        option,
    };

    Ok(config)
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
            (program_config.input_filenames, program_config.output_filename.unwrap());
    }

    let archive_filename = program_config.input_filenames[0].clone();
    let mut extractor = Extractor::new(archive_filename)
        .map_err(|err| decompress_error_to_string(err))?;

    match program_config.option
    {
        ConfigOption::ExtractAll => extractor.extract_all()
            .map_err(|err| decompress_error_to_string(err)),
        ConfigOption::ExtractPath => extractor.extract_path(program_config.output_filename.unwrap())
            .map_err(|err| decompress_error_to_string(err)),
        ConfigOption::Display =>
        {
            print_archive_info(&extractor);
            Ok(())
        }

        _ => Ok(()),
    }
}
