use std::env::args;
use std::fs;
use std::path::Path;

mod huffman_tree;
mod file_reader;
mod bit_vector;
mod compress;
mod file_writer;

extern crate colored;
use colored::*;
use divcomp::compress::compress;
use divcomp::decompress::decompress;

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

    match decompress(&input_filename, &output_filename)
    {
        Ok(()) => {},
        Err(err_msg) =>
        {
            eprintln!("{}", err_msg);
            return;
        }
    };

    let input_file_size = fs::metadata(input_filename).unwrap().len();
    let output_file_size = fs::metadata(output_filename).unwrap().len();

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


