use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if args.get(1).and_then(|arg| arg.to_str()) == Some("fmi") {
        args.remove(1);
    }
    cargo_fmi::entrypoint_from(args)?;
    Ok(())
}
