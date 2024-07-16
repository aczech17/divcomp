mod config;

use std::env::args;
use config::*;

mod compress_stage;
use compress_stage::compress::compress;
use compress_stage::decompress::decompress;
use crate::archive_stage::archive::archive;
use crate::archive_stage::extract::extract;

mod archive_stage;
mod io_utils;

extern crate colored;

fn main()
{
    // let inputs = vec!["empty.txt", "lotto.txt", "supdir", "pusty"]
    //     .iter().map(|s| s.to_string())
    //     .collect();
    //
    // archive(inputs, "archive.bin".to_string());

    let args: Vec<String> = args().collect();
    let name = &args[1];

    extract(name)
        .unwrap();
}

