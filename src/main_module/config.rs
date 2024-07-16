use std::env::args;
use std::path::Path;

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
