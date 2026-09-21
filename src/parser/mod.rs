use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "src/parser/grammar.pest"]
pub struct SVParser {}

impl SVParser {
    pub fn new() -> SVParser {
        SVParser {}
    }
}
