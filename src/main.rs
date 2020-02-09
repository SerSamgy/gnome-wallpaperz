#[macro_use]
extern crate lazy_static;

use tera::{ Tera, Context };

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*.xml") {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![]);
        tera
    };
}

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    let context = Context::new();
    let result = TEMPLATES.render("index.xml", &context)?;
    println!("{}", result);

    Ok(())
}
