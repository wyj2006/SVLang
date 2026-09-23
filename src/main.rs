pub mod parser;

use crate::parser::{Rule, SVParser};
use pest::Parser;

fn main() {
    let a = SVParser::parse(
        Rule::source_text,
        "module a(); bit b; assign b=1+2; endmodule",
    )
    .unwrap();
    println!("{:#?}", a);
}
