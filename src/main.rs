use slint::*;
use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let app = Main::new()?;
    app.run()?;
    Ok(())
}
