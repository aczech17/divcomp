use std::env::args;
use std::fs;
use std::path::Path;
use colored::Colorize;

#[derive(Eq, PartialEq)]
pub enum Option
{
    Compress, Decompress,
}

pub struct Config
{
    pub input_filename: String,
    pub output_filename: String,
    pub option: Option,
}

pub fn parse_arguments() -> Result<Config, String>
{
    let usage = "divcomp [-c|-d] [input] [output]";

    let args: Vec<String> = args().collect();

    if args.len() < 4
    {
        return Err(String::from(usage));
    }

    let option = match args[1].as_str()
    {
        "-c" => Option::Compress,
        "-d" => Option::Decompress,
        _ => return Err(usage.to_string()),
    };

    let input_filename = &args[2];
    let output_filename = &args[3];

    if !Path::new(input_filename).exists()
    {
        return Err(format!("File {input_filename} does not exist."));
    }

    if !Path::new(input_filename).is_file()
    {
        return Err(format!("{input_filename} is not a regular file."));
    }

    if Path::new(output_filename).exists()
    {
        return Err(format!("{output_filename} already exists."));
    }

    let config = Config
    {
        input_filename: input_filename.to_string(),
        output_filename: output_filename.to_string(),
        option,
    };

    Ok(config)
}

pub fn print_statistics(config: Config)
{
    if config.option == Option::Decompress
    {
        return;
    }

    let input_file_size = fs::metadata(config.input_filename).unwrap().len();
    let output_file_size = fs::metadata(config.output_filename).unwrap().len();

    let compression_rate = (input_file_size as f64) / (output_file_size as f64);

    println!("Rozmiar pliku wejściowego:\t{input_file_size}B");
    println!("Rozmiar pliku skompresowanego:\t{output_file_size}B ");

    if compression_rate > 1.0
    {
        println!("{}", format!("Współczynnik kompresji:\t\t{compression_rate}").green());
    }
    else
    {
        println!("{}", format!("Współczynnik kompresji:\t\t{compression_rate}").red());
    }
}
