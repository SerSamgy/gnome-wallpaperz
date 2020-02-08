use tera::{ Tera, Context };

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
    let tera = match Tera::new("templates/**/*.xml") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    let context = Context::new();
    let result = tera.render("index.xml", &context)?;
    println!("{}", result);

    Ok(())
}
