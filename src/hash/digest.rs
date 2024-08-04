use std::error;
use std::path;

use crate::libs::hash;
use crate::libs::input;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Clone, Copy)]
pub enum Style {
    BSD,
    GNU,
}

pub fn println(f: &path::PathBuf, hf: hash::Func, style: Style) -> Result<()> {
    let r = input::Input::new(&f)?;
    let digest = hash::digest(r, hf)?;

    // TODO: handle unwrap
    let name = f.to_str().unwrap();

    match style {
        Style::BSD => println!("{} ({}) = {}", hf, name, digest),
        Style::GNU => println!("{}  {}", digest, name),
    }

    Ok(())
}
