#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;
extern crate handlebars;
extern crate clang;
extern crate clap;
extern crate rand;

#[cfg(test)]
extern crate tempdir;

extern crate log;
extern crate env_logger;

use clang::{Clang, Index};

use handlebars::Handlebars;

use std::io::Write;
use std::path::Path;
use std::boxed::Box;
use std::env;
use std::fs::File;

mod model;
mod template;
mod cmdline;
mod parser;
mod response_file;

use response_file::ExtendWithResponseFile;

#[cfg(test)]
mod parser_tests;

fn main() {
    let matches = cmdline::build_argument_parser().get_matches();

    let clang = Clang::new().expect("create clang parser");
    let index = Index::new(&clang, false, false);
    let input = matches.value_of("INPUT").expect("input missing");
    let clang_flags =
        match matches.values_of("FLAGS") {
            None => vec!["-x".to_string(), "c++".to_string()],
            Some(vals) => vec!["-x", "c++"]
                .into_iter()
                .chain(vals)
                .map(String::from)
                .extend_with_response_file()
                .collect::<Vec<_>>(),
        };
    let mut builder = env_logger::LogBuilder::new();
    let verbosity = &match matches.occurrences_of("verbose") {
        1 => "info".to_string(),
        2 => "debug".to_string(),
        _ => env::var("RUST_LOG").unwrap_or("".to_string()),
    };
    builder.parse(&verbosity);
    builder.init().unwrap();

    let tu = match index.parser(input).arguments(&clang_flags).parse() {
        Ok(x) => x,
        Err(e) => panic!("{:?}", e),
    };
    let model = model::Model::new(&tu);

    let template_file_name = matches.values_of("template").unwrap().nth(0).unwrap();

    let mut handlebars = Handlebars::new();
    handlebars.register_helper("len", Box::new(template::len));
    handlebars.register_escape_fn(|s| -> String { return s.to_string(); });

    match handlebars.register_template_file("template", &Path::new(template_file_name)) {
        Err(e) => panic!("{:?}", e),
        _ => (),
    };

    let output = handlebars.render("template", &model)
        .unwrap_or_else(|e| e.to_string().to_owned());

    match matches.value_of("OUTPUT") {
        None => println!("{}", output),
        Some(val) => {
            let output_file = File::create(val);         
            let mut file = match output_file {
                Err(error) => panic!("Problem opening the file: {:?}", error),
                Ok(file) =>  file,
            };
            
            let _ = file.write_all(output.as_bytes());
        },
    };
    
}
