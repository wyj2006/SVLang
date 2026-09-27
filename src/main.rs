pub mod ast;
pub mod diagnostic;
pub mod file_map;
pub mod parser;

use crate::{diagnostic::print_diag, file_map::FileMap, parser::SVParser};
use lazy_static::lazy_static;
use std::{fs, sync::Mutex};

lazy_static! {
    pub static ref files: Mutex<FileMap> = Mutex::new(FileMap::new());
}

fn main() {
    let input_path = "test.sv".to_string();
    let source_id = files
        .lock()
        .unwrap()
        .add(input_path.clone(), fs::read_to_string(input_path).unwrap());
    let parser = SVParser::new(source_id);
    let ast = match parser.parse_to_ast() {
        Ok(t) => t,
        Err(e) => {
            print_diag(e);
            return;
        }
    };
    println!("{ast:#?}");
}
