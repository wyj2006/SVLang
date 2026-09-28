import os

infix_precedence = [
    ["or"],
    ["and"],
    ["bit_or"],
    ["bit_xnor", "bit_xor"],
    ["bit_and"],
    ["case_eq", "case_neq", "wildcard_eq", "wildcard_neq", "eq", "neq"],
    ["le", "ge", "lt", "gt", "inside"],
    ["arith_lshift", "arith_rshift", "lshift", "rshift"],
    ["add", "sub"],
    ["mul", "div", "mod"],
    ["pow"],
]
infix_op = {
    "or": "BinOpKind::Or",
    "and": "BinOpKind::And",
    "bit_or": "BinOpKind::BitOr",
    "bit_and": "BinOpKind::BitAnd",
    "bit_xor": "BinOpKind::BitXor",
    "bit_xnor": "BinOpKind::BitXNor",
    "eq": "BinOpKind::Eq",
    "neq": "BinOpKind::Neq",
    "case_eq": "BinOpKind::CaseEq",
    "case_neq": "BinOpKind::CaseNeq",
    "wildcard_eq": "BinOpKind::WildCardEq",
    "wildcard_neq": "BinOpKind::WildCardNeq",
    "lt": "BinOpKind::Lt",
    "le": "BinOpKind::Le",
    "gt": "BinOpKind::Gt",
    "ge": "BinOpKind::Ge",
    "lshift": "BinOpKind::LShift",
    "rshift": "BinOpKind::RShift",
    "arith_lshift": "BinOpKind::ArithLShift",
    "arith_rshift": "BinOpKind::ArithRShift",
    "add": "BinOpKind::Add",
    "sub": "BinOpKind::Sub",
    "mul": "BinOpKind::Mul",
    "div": "BinOpKind::Div",
    "mod": "BinOpKind::Mod",
    "pow": "BinOpKind::Pow",
}


def gen_infix_grammar() -> str:
    code = ""
    for i, ops in enumerate(infix_precedence):
        next_rule = (
            f"{infix_precedence[i + 1][0]}_expression"
            if i + 1 < len(infix_precedence)
            else "unary_expression"
        )
        if "inside" in ops:
            ops = ops[:]
            ops.remove("inside")
            code += f'{ops[0]}_expression={{{next_rule}~((({"|".join(ops)})~{next_rule})|("inside"~"{{"~range_list~"}}"))*}}\n'
        else:
            code += f"{ops[0]}_expression={{{next_rule}~(({"|".join(ops)})~{next_rule})*}}\n"
    return code


def gen_infix_parser():
    code = """use crate::{
    ast::{BinOpKind, ExprKind, Expression},
    diagnostic::from_pest_span,
    file_map::FileId,
    parser::{Rule, SVParser},
};
use codespan_reporting::diagnostic::Diagnostic;
use pest::iterators::Pair;
impl SVParser{"""
    for i, ops in enumerate(infix_precedence):

        rule = f"{ops[0]}_expression"
        next_rule = (
            f"{infix_precedence[i + 1][0]}_expression"
            if i + 1 < len(infix_precedence)
            else "unary_expression"
        )

        extension = ""
        if "inside" in ops:
            ops = ops[:]
            ops.remove("inside")
            extension = "Rule::range_list=>left=Some(Expression::new(self.file_id,span,ExprKind::Inside{expr:Box::new(left.unwrap()),ranges:self.parse_range_list(pair)?})),"
        code += f"""
pub fn parse_{rule}(&self,pair:Pair<Rule>)->Result<Expression,Diagnostic<FileId>>{{
    let mut left=None;
    let mut op=None;

    for pair in pair.into_inner()
    {{
        let span=from_pest_span(pair.as_span());
        match pair.as_rule(){{
            Rule::{next_rule}=>if let None=left{{
                left=Some(self.parse_{next_rule}(pair)?);
            }}else{{
                left=Some(Expression::new(self.file_id,span,ExprKind::BinOp{{op:op.unwrap(),left:Box::new(left.unwrap()),right:Box::new(self.parse_{next_rule}(pair)?)}}));
            }},
            {",".join([f"Rule::{op if op!="mod" else "r#mod"}=>op=Some({infix_op[op]})" for op in ops])},{extension}
            _=>unreachable!()
        }}
    }}

    Ok(left.unwrap())
}}"""
    code += "}"
    open(
        os.path.join(os.path.dirname(__file__), "..", "parse_infix.rs"),
        encoding="utf-8",
        mode="w",
    ).write(code)


if __name__ == "__main__":
    gen_infix_parser()
