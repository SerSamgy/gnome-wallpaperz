#[macro_use]
extern crate lazy_static;

mod filters;

use chrono::prelude::*;
use serde::Serialize;
use std::error::Error;
use std::fs::{DirEntry, File};
use std::io::prelude::*;
use std::path::PathBuf;
use std::{fs, io};
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
        args.next();

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

        Ok(Config {
            source_path,
            output_filename,
            starttime,
            duration,
            trans_duration,
        })
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

pub fn render(context: IndexContext) -> Result<String, Box<dyn Error>> {
    let serialized_context = &Context::from_serialize(&context)?;
    let rendered = TEMPLATES.render("index.xml", serialized_context)?;

    Ok(rendered)
}

fn get_file_path(dir_entry: DirEntry) -> Option<PathBuf> {
    let file_path = dir_entry.path();
    if file_path.is_file() {
        return Some(file_path);
    }

    return None;
}

fn get_filenames(path_to_directory: String) -> io::Result<Vec<String>> {
    // TODO: check if folder contains 2 or more files
    // TODO: filter files by picture types
    let mut entries = fs::read_dir(path_to_directory)?
        .map(|res| {
            res.map(|dir_entry| match get_file_path(dir_entry) {
                Some(file_path) => match file_path.to_str() {
                    Some(file_path) => file_path.to_string(),
                    None => String::from("bad_filename"),
                },
                None => String::from("bad_filename"),
            })
        })
        .filter(|entry| entry.as_ref().unwrap() != &String::from("bad_filename"))
        .collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort();

    Ok(entries)
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let entries = get_filenames(config.source_path)?;

    let context = IndexContext::new(
        config.starttime,
        config.duration,
        config.trans_duration,
        entries,
    )?;
    let rendered = render(context)?;

    let mut buffer = File::create(config.output_filename)?;
    write!(&mut buffer, "{}", rendered)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_returns_static_and_transition() {
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
