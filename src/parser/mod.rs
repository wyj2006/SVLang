pub mod parse_decl;
pub mod parse_expr;
pub mod parse_type;

use crate::{
    ast::{CompilationUnit, Declaration},
    diagnostic::{from_pest_span, map_pest_err},
    file_map::FileId,
    files,
};
use codespan_reporting::{diagnostic::Diagnostic, files::Files};
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "src/parser/grammar.pest"]
pub struct SVParser {
    pub file_id: FileId,
}

impl SVParser {
    pub fn new(file_id: FileId) -> SVParser {
        SVParser { file_id }
    }

    pub fn parse_to_ast(&self) -> Result<CompilationUnit, Diagnostic<FileId>> {
        let source = files.lock().unwrap().source(self.file_id).unwrap();
        let pairs = map_pest_err(
            self.file_id,
            SVParser::parse(Rule::source_text, source.as_str()),
        )?;
        println!("{pairs:#?}");
        for pair in pairs {
            let span = pair.as_span();
            match pair.as_rule() {
                Rule::source_text => {
                    return Ok(CompilationUnit {
                        decls: self.parse_source_text(pair)?,
                        ..CompilationUnit::new(self.file_id, from_pest_span(span))
                    });
                }
                _ => unreachable!(),
            }
        }
        unreachable!()
    }

    pub fn parse_source_text(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Vec<Declaration>, Diagnostic<FileId>> {
        let mut decls = vec![];

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::module_declaration => decls.push(self.parse_module_declaration(pair)?),
                Rule::EOI => {}
                _ => todo!(),
            }
        }
        Ok(decls)
    }
}
