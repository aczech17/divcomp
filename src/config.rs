use std::env::args;
use std::fs;
use std::path::Path;
use colored::Colorize;

#[derive(Eq, PartialEq)]
pub enum ConfigOption
{
    Archive, Extract,
}

pub struct Config
{
    pub input_filenames: Vec<String>,
    pub output_archive_filename: Option<String>,
    pub option: ConfigOption,
}

pub fn parse_arguments() -> Result<Config, String>
{
    let usage = "divcomp [-a|-e] [inputs] -o [output]";

    let args: Vec<String> = args().collect();

    if args.len() < 3
    {
        return Err(String::from(usage));
    }

    let option = match args[1].as_str()
    {
        "-a" => ConfigOption::Archive,
        "-e" => ConfigOption::Extract,
        _ => return Err(usage.to_string()),
    };

    let (inputs, output) = match args.iter().position(|s| s == "-o")
    {
        None =>
        {
                if option == ConfigOption::Extract
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

    for input in &inputs
    {
        if !Path::new(input).exists()
        {
            return Err(format!("Path {} does not exist.", input));
        }
    }

    match &output
    {
        Some(name) if Path::new(name).exists() =>
            return Err(format!("Path {} already exists.", name)),
        None => {},
        _ => {}, // ???
    };

    let config = Config
    {
        input_filenames: inputs,
        output_archive_filename: output,
        option,
    };

    Ok(config)
}

//
// pub fn print_statistics(config: Config)
// {
//     if config.option == ConfigOption::Decompress
//     {
//         return;
//     }
//
//     let input_file_size = fs::metadata(config.input_filename).unwrap().len();
//     let output_file_size = fs::metadata(config.output_archive_filename).unwrap().len();
//
//     println!("Rozmiar pliku wejściowego:\t{input_file_size}B");
//     println!("Rozmiar pliku skompresowanego:\t{output_file_size}B ");
//
//     if output_file_size == 0
//     {
//         return;
//     }
//
//     let compression_rate = (input_file_size as f64) / (output_file_size as f64);
//     if compression_rate < 1.0
//     {
//         println!("{}", format!("Współczynnik kompresji:\t\t{compression_rate}").red());
//     }
//     else if compression_rate == 1.0
//     {
//         println!("{}", format!("Współczynnik kompresji:\t\t{compression_rate}").yellow());
//     }
//     else
//     {
//         println!("{}", format!("Współczynnik kompresji:\t\t{compression_rate}").green());
//     }
// }
