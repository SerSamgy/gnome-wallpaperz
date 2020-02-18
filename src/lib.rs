#[macro_use]
extern crate lazy_static;

mod filters;

use chrono::prelude::*;
use serde::Serialize;
use tera::{Context, Tera};

#[derive(Debug)]
pub struct Config {
  pub source_path: String,
  pub output_filename: String,
  pub starttime: DateTime<Local>,
  pub duration: f64,
  pub trans_duration: f64,
}

impl Config {
  pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
    let source_path = match args.next() {
      Some(sp) => sp,
      None => return Err("Didn't get path to source folder"),
    };

    let output_filename = match args.next() {
      Some(out) => out,
      None => return Err("Didn't get the name of output file"),
    };

    // Optional parameters for now
    let starttime = match args.next() {
      Some(st) => match Local.datetime_from_str(&st, "%Y-%m-%d %H:%M:%S").ok() {
        Some(dt) => dt,
        None => return Err("Failed to parse provided start time"),
      },
      None => Local::now(), 
    };

    let default_duration = 300.0;
    let duration = match args.next() {
      Some(dr) => dr.parse::<f64>().unwrap_or(default_duration),
      None => default_duration, 
    };
    
    let default_trans_duration = 60.0;
    let trans_duration = match args.next() {
      Some(td) => td.parse::<f64>().unwrap_or(default_trans_duration),
      None => default_trans_duration, 
    };

    Ok(Config{ source_path, output_filename, starttime, duration, trans_duration })
  }
}

#[derive(Serialize, Debug)]
pub struct IndexContext {
    pub starttime: DateTime<Local>,
    pub duration: f64,
    pub trans_duration: f64,
    pub filenames: Vec<String>,
}

impl IndexContext {
    pub fn new(
        starttime: DateTime<Local>,
        duration: f64,
        trans_duration: f64,
        filenames: Vec<String>,
    ) -> Result<IndexContext, &'static str> {
        Ok(IndexContext {
            starttime,
            duration,
            trans_duration,
            filenames,
        })
    }
}

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::new("templates/**/*.xml").unwrap();
        tera.autoescape_on(vec![]);
        tera.register_filter("with_zero", filters::with_zero);
        tera
    };
}

pub fn render(context: IndexContext) -> Result<String, &'static str> {
    let serialized_context =
        &Context::from_serialize(&context).expect("failed to serialize template context");
    let rendered = TEMPLATES
        .render("index.xml", serialized_context)
        .expect("failed to render template");

    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_returns_one_static_with_one_file() {
        let starttime: DateTime<Local> = Local.ymd(2020, 2, 12).and_hms(16, 2, 2);
        let duration = 300.0;
        let filenames = vec![String::from("./my_awesome_file.jpeg")];
        let expected_tmpl = format!(
            "\
<background>
  <starttime>
    <year>{year}</year>
    <month>{month:02}</month>
    <day>{day}</day>
    <hour>{hour}</hour>
    <minute>{minute:02}</minute>
    <second>{second:02}</second>
  </starttime>

  <static>
    <duration>{duration:.1}</duration>
    <file>{filename}</file>
  </static>
</background>",
            year = starttime.year(),
            month = starttime.month(),
            day = starttime.day(),
            hour = starttime.hour(),
            minute = starttime.minute(),
            second = starttime.second(),
            duration = duration,
            filename = filenames.first().unwrap()
        );
        let render_context = IndexContext::new(starttime, duration, 60.0, filenames).unwrap();
        let rendered = render(render_context).unwrap();

        assert_eq!(expected_tmpl, rendered)
    }

    #[test]
    fn render_returns_static_and_transition_with_two_files() {
        let starttime: DateTime<Local> = Local.ymd(2020, 2, 12).and_hms(16, 2, 2);
        let duration = 300.0;
        let trans_duration = 60.0;
        let filenames = vec![
            String::from("./my_awesome_file_0.jpeg"),
            String::from("./my_awesome_file_1.jpeg"),
        ];
        let expected_tmpl = format!(
            "\
<background>
  <starttime>
    <year>{year}</year>
    <month>{month:02}</month>
    <day>{day}</day>
    <hour>{hour}</hour>
    <minute>{minute:02}</minute>
    <second>{second:02}</second>
  </starttime>

  <static>
    <duration>{duration:.1}</duration>
    <file>{filename_0}</file>
  </static>

  <transition type=\"overlay\">
    <duration>{trans_duration:.1}</duration>
    <from>{filename_0}</from>
    <to>{filename_1}</to>
  </transition>

  <static>
    <duration>{duration:.1}</duration>
    <file>{filename_1}</file>
  </static>

  <transition type=\"overlay\">
    <duration>{trans_duration:.1}</duration>
    <from>{filename_1}</from>
    <to>{filename_0}</to>
  </transition>
</background>",
            year = starttime.year(),
            month = starttime.month(),
            day = starttime.day(),
            hour = starttime.hour(),
            minute = starttime.minute(),
            second = starttime.second(),
            duration = duration,
            filename_0 = filenames.first().unwrap(),
            trans_duration = trans_duration,
            filename_1 = filenames.last().unwrap()
        );
        let render_context =
            IndexContext::new(starttime, duration, trans_duration, filenames).unwrap();
        let rendered = render(render_context).unwrap();

        assert_eq!(expected_tmpl, rendered)
    }
}
