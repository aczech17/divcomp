use std::ffi::OsString;
use std::iter::empty;

fn main()
{
    embed_resource::compile("assets/icon.rc", empty::<OsString>());
}
