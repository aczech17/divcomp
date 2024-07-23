use main_module::config::parse_arguments;

use crate::main_module::config::execute;

mod main_module;
mod compress;
mod archive;
mod io_utils;

fn main()
{
    let config = match parse_arguments()
    {
        Ok(c) => c,
        Err(err_msg) =>
        {
            eprintln!("{}", err_msg);
            return;
        }
    };


    if let Err(err_msg) = execute(config)
    {
        eprintln!("{}", err_msg);
    }
}

