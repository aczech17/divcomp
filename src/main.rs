use main_module::config::parse_arguments;

use crate::main_module::config::execute;

mod main_module;
mod compress_stage;
mod archive_stage;
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

    let execution_status = execute(config);
    if let Err(err_msg) = execution_status
    {
        eprintln!("{}", err_msg);
    }
}

