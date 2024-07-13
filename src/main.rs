use std::env::args;
use std::path::Path;
use crate::compress::compress;

mod huffman_tree;
mod file_reader;
mod bit_vector;
mod compress;
mod file_writer;


fn parse_arguments() -> Result<(String, String), String>
{
    let usage = "divcomp [input] [output]";
    let args: Vec<String> = args().collect();

    if args.len() < 3
    {
        return Err(String::from(usage));
    }

    let input_filename = &args[1];
    let output_filename = &args[2];

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

    Ok((input_filename.to_string(), output_filename.to_string()))
}

fn main()
{
    let (input_filename, output_filename) = match parse_arguments()
    {
        Ok((i, o)) => (i, o),
        Err(err_msg) =>
        {
            eprintln!("{}", err_msg);
            return;
        }
    };

    match compress(&input_filename, &output_filename)
    {
        Ok(()) => {},
        Err(err_msg) => eprintln!("{}", err_msg),
    };
}

