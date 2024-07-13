use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use divcomp::compress;
use crate::compress::compress;


fn get_unique_filename(base: &str) -> String
{
    // let id = (std::thread::current().id() as u64).get();
    let id = std::thread::current().id();
    format!("{}_{:?}.bin", base, id)
}

fn get_output(input_data: Vec<u8>) -> Vec<u8>
{
    let input_filename = get_unique_filename("input");
    let output_filename = get_unique_filename("output");

    let mut input_file = File::create(&input_filename)
        .expect("Could not create test input file.");

    input_file.write_all(&input_data).unwrap();

    compress(&input_filename, &output_filename).unwrap();

    let mut output_file = File::open(&output_filename).unwrap();

    let mut output_data: Vec<u8> = vec![];
    let output_file_size = output_file.metadata().unwrap().len() as usize;
    output_data.resize(output_file_size, 0);

    output_file.read_exact(&mut output_data).unwrap();

    fs::remove_file(input_filename).unwrap();
    fs::remove_file(output_filename).unwrap();

    output_data
}

fn do_test(input: Vec<u8>, expected_output: Vec<u8>)
{
    let output = get_output(input);
    assert_eq!(output, expected_output);
}

#[test]
fn empty_file()
{
    let input = vec![];
    let expected_output = vec![];
    do_test(input, expected_output);
}

#[test]
fn six_a_letters()
{
    let input = vec![0x61, 0x61, 0x61, 0x61, 0x61, 0x61];
    let expected_output = vec![0xB0, 0xE0, 0x00];
    do_test(input, expected_output);
}
