use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    fmi_xtask::entrypoint()?;
    Ok(())
}
