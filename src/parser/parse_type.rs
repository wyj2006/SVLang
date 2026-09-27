use crate::{
    ast::{Type, TypeKind},
    diagnostic::from_pest_span,
    file_map::FileId,
    parser::{Rule, SVParser},
};
use codespan_reporting::diagnostic::Diagnostic;
use pest::iterators::Pair;

impl SVParser {
    pub fn parse_simple_type(&self, pair: Pair<Rule>) -> Result<Type, Diagnostic<FileId>> {
        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::integer_type => return self.parse_integer_type(pair),
                Rule::non_integer_type => return self.parse_non_integer_type(pair),
                _ => todo!(),
            }
        }
        unreachable!()
    }

    pub fn parse_integer_type(&self, pair: Pair<Rule>) -> Result<Type, Diagnostic<FileId>> {
        Ok(Type::new(
            self.file_id,
            from_pest_span(pair.as_span()),
            match pair.as_str() {
                "byte" => TypeKind::Byte,
                "shortint" => TypeKind::ShortInt,
                "int" => TypeKind::Int,
                "longint" => TypeKind::LongInt,
                "integer" => TypeKind::Integer,
                "time" => TypeKind::Time,
                "bit" => TypeKind::Bit,
                "logic" => TypeKind::Logic,
                "reg" => TypeKind::Reg,
                _ => unreachable!(),
            },
        ))
    }

    pub fn parse_non_integer_type(&self, pair: Pair<Rule>) -> Result<Type, Diagnostic<FileId>> {
        Ok(Type::new(
            self.file_id,
            from_pest_span(pair.as_span()),
            match pair.as_str() {
                "shortreal" => TypeKind::ShortReal,
                "real" => TypeKind::Real,
                "realtime" => TypeKind::RealTime,
                _ => unreachable!(),
            },
        ))
    }
}
