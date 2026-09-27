use crate::{
    ast::{
        AccessKind, ArgKind, Argument, AssignOp, BinOpKind, CastTarget, ExprKind, Expression,
        IntegerBase, MemberPattern, MemberPatternKind, Pattern, PatternKind, Range, RangeKind,
        SelectDirection, SelectKind, UnaryOpKind,
    },
    diagnostic::from_pest_span,
    file_map::FileId,
    parser::{Rule, SVParser},
};
use codespan_reporting::diagnostic::Diagnostic;
use pest::{
    iterators::Pair,
    pratt_parser::{Assoc, Op, PrattParser},
};
use std::sync::LazyLock;

static PRATT: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::triple_amp, Assoc::Left))
        .op(Op::infix(Rule::implication, Assoc::Right) | Op::infix(Rule::logical_eq, Assoc::Right))
        .op(Op::infix(Rule::cond_then, Assoc::Left) | Op::infix(Rule::cond_else, Assoc::Right))
        .op(Op::infix(Rule::or, Assoc::Left))
        .op(Op::infix(Rule::and, Assoc::Left))
        .op(Op::infix(Rule::bit_or, Assoc::Left))
        .op(Op::infix(Rule::bit_xor, Assoc::Left) | Op::infix(Rule::bit_xnor, Assoc::Left))
        .op(Op::infix(Rule::bit_and, Assoc::Left))
        .op(Op::infix(Rule::eq, Assoc::Left)
            | Op::infix(Rule::neq, Assoc::Left)
            | Op::infix(Rule::case_eq, Assoc::Left)
            | Op::infix(Rule::case_neq, Assoc::Left)
            | Op::infix(Rule::wildcard_eq, Assoc::Left)
            | Op::infix(Rule::wildcard_neq, Assoc::Left))
        .op(Op::infix(Rule::lt, Assoc::Left)
            | Op::infix(Rule::le, Assoc::Left)
            | Op::infix(Rule::gt, Assoc::Left)
            | Op::infix(Rule::ge, Assoc::Left))
        .op(Op::infix(Rule::lshift, Assoc::Left)
            | Op::infix(Rule::arithmetic_lshift, Assoc::Left)
            | Op::infix(Rule::rshift, Assoc::Left)
            | Op::infix(Rule::arithmetic_rshift, Assoc::Left))
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left)
            | Op::infix(Rule::div, Assoc::Left)
            | Op::infix(Rule::r#mod, Assoc::Left))
        .op(Op::infix(Rule::pow, Assoc::Left))
        .op(Op::prefix(Rule::positive)
            | Op::prefix(Rule::negative)
            | Op::prefix(Rule::not)
            | Op::prefix(Rule::bit_not)
            | Op::prefix(Rule::reduction_and)
            | Op::prefix(Rule::reduction_nand)
            | Op::prefix(Rule::reduction_or)
            | Op::prefix(Rule::reduction_nor)
            | Op::prefix(Rule::reduction_xor)
            | Op::prefix(Rule::reduction_xnor))
        .op(Op::postfix(Rule::inside)
            | Op::postfix(Rule::method_call_op)
            | Op::postfix(Rule::cond_pattern)
            | Op::postfix(Rule::cast_op))
});

impl SVParser {
    pub fn parse_select(
        &self,
        base: Expression,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut expr = base;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::member_identifier => {
                    expr = Expression::new(
                        self.file_id,
                        from_pest_span(pair.as_span()),
                        ExprKind::Access {
                            base: Box::new(expr),
                            name: pair.as_str().to_string(),
                            kind: AccessKind::Member,
                        },
                    )
                }
                Rule::bit_select => expr = self.parse_bit_select(expr, pair)?,
                Rule::part_select_range => expr = self.parse_part_select_range(expr, pair)?,
                _ => todo!(),
            }
        }

        Ok(expr)
    }

    pub fn parse_bit_select(
        &self,
        base: Expression,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut expr = base;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::expression => {
                    expr = Expression::new(
                        self.file_id,
                        from_pest_span(pair.as_span()),
                        ExprKind::Select {
                            base: Box::new(expr),
                            kind: SelectKind::BitSelect(Box::new(self.parse_expression(pair)?)),
                        },
                    )
                }
                _ => unreachable!(),
            }
        }

        Ok(expr)
    }

    pub fn parse_part_select_range(
        &self,
        base: Expression,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::constant_range => return self.parse_constant_range(base, pair),
                Rule::indexed_range => return self.parse_indexed_range(base, pair),
                _ => unreachable!(),
            }
        }
        unreachable!()
    }

    pub fn parse_constant_range(
        &self,
        base: Expression,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut exprs = vec![];
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::expression => exprs.push(self.parse_expression(pair)?),
                _ => unreachable!(),
            }
        }

        Ok(Expression::new(
            self.file_id,
            span,
            ExprKind::Select {
                base: Box::new(base),
                kind: SelectKind::RangeSelect {
                    msb: Box::new(exprs.remove(0)),
                    lsb: Box::new(exprs.remove(0)),
                },
            },
        ))
    }

    pub fn parse_indexed_range(
        &self,
        base: Expression,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let direction = if pair.as_str().contains("+:") {
            SelectDirection::Positive
        } else if pair.as_str().contains("-:") {
            SelectDirection::Negative
        } else {
            unreachable!()
        };
        let mut exprs = vec![];
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::expression => exprs.push(self.parse_expression(pair)?),
                _ => unreachable!(),
            }
        }

        Ok(Expression::new(
            self.file_id,
            span,
            ExprKind::Select {
                base: Box::new(base),
                kind: SelectKind::IndexedSelect {
                    base: Box::new(exprs.remove(0)),
                    direction,
                    width: Box::new(exprs.remove(0)),
                },
            },
        ))
    }

    pub fn parse_expression(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        PRATT
            .map_primary(|pair| {
                let mut expr = None;
                let mut access_kind = None;

                for pair in pair.into_inner() {
                    let span = from_pest_span(pair.as_span());
                    match pair.as_rule() {
                        Rule::operator_assignment => {
                            expr = Some(self.parse_operator_assignment(pair)?)
                        }
                        Rule::inc_or_dec_expression => {
                            expr = Some(self.parse_inc_or_dec_expression(pair)?)
                        }
                        Rule::number => expr = Some(self.parse_number(pair)?),
                        Rule::package_scope => {
                            access_kind = Some(AccessKind::Scope);
                            expr = Some(self.parse_package_scope(pair)?);
                        }
                        Rule::hierarchical_identifier => {
                            expr =
                                Some(self.parse_hierarchical_identifier(pair, expr, access_kind)?)
                        }
                        Rule::select => expr = Some(self.parse_select(expr.unwrap(), pair)?),
                        Rule::empty_unpacked_array_concatenation => {
                            expr = Some(Expression::new(
                                self.file_id,
                                span,
                                ExprKind::Concat(vec![]),
                            ))
                        }
                        Rule::concatenation => expr = Some(self.parse_concatenation(pair)?),
                        Rule::range_expression => {
                            expr = Some(self.parse_range_expression(expr.unwrap(), pair)?)
                        }
                        Rule::multiple_concatenation => {
                            expr = Some(self.parse_multiple_concatenation(pair)?)
                        }
                        Rule::subroutine_call => expr = Some(self.parse_subroutine_call(pair)?),
                        Rule::mintypmax_expression => {
                            expr = Some(self.parse_mintypmax_expression(pair)?)
                        }
                        Rule::cast => expr = Some(self.parse_cast(pair)?),
                        _ => match pair.as_str() {
                            "this" => {
                                expr = Some(Expression::new(self.file_id, span, ExprKind::This))
                            }
                            "$" => {
                                expr = Some(Expression::new(self.file_id, span, ExprKind::Dollar))
                            }
                            "null" => {
                                expr = Some(Expression::new(self.file_id, span, ExprKind::Null))
                            }
                            "" => {}
                            _ => todo!(),
                        },
                    }
                }

                Ok(expr.unwrap())
            })
            .map_prefix(|pair, operand| {
                let operand = operand?;
                let span = from_pest_span(pair.as_span());
                Ok(Expression::new(
                    self.file_id,
                    span,
                    ExprKind::UnaryOp {
                        op: match pair.as_rule() {
                            Rule::positive => UnaryOpKind::Positive,
                            Rule::negative => UnaryOpKind::Negative,
                            Rule::not => UnaryOpKind::Not,
                            Rule::bit_not => UnaryOpKind::BitNot,
                            Rule::reduction_and => UnaryOpKind::ReductionAnd,
                            Rule::reduction_nand => UnaryOpKind::ReductionNAnd,
                            Rule::reduction_or => UnaryOpKind::ReductionOr,
                            Rule::reduction_nor => UnaryOpKind::ReductionNOr,
                            Rule::reduction_xor => UnaryOpKind::ReductionXor,
                            Rule::reduction_xnor => UnaryOpKind::ReductionXNor,
                            _ => unreachable!(),
                        },
                        operand: Box::new(operand),
                    },
                ))
            })
            .map_postfix(|operand, pair| {
                let operand = operand?;
                let span = from_pest_span(pair.as_span());

                match pair.as_rule() {
                    Rule::cast_op => {
                        let mut expr = None;
                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::expression => expr = Some(self.parse_expression(pair)?),
                                _ => unreachable!(),
                            }
                        }
                        Ok(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::Cast {
                                expr: Box::new(expr.unwrap()),
                                target: Box::new(CastTarget::Expr(operand)),
                            },
                        ))
                    }
                    Rule::inside => {
                        let mut ranges = vec![];

                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::range_list => ranges = self.parse_range_list(pair)?,
                                _ => unreachable!(),
                            }
                        }

                        Ok(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::Inside {
                                expr: Box::new(operand),
                                ranges,
                            },
                        ))
                    }
                    Rule::cond_pattern => {
                        let mut pattern = None;

                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::pattern => pattern = Some(self.parse_pattern(pair)?),
                                _ => unreachable!(),
                            }
                        }

                        Ok(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::Matches {
                                expr: Box::new(operand),
                                pattern: Box::new(pattern.unwrap()),
                            },
                        ))
                    }
                    _ => todo!(),
                }
            })
            .map_infix(|left, pair, right| {
                let mut left = left?;
                let right = right?;
                let span = from_pest_span(pair.as_span());

                match pair.as_rule() {
                    Rule::cond_then => Ok(Expression::new(
                        self.file_id,
                        span,
                        ExprKind::Conditional {
                            cond: Box::new(left),
                            then: Box::new(right),
                            r#else: Box::new(Expression::new_error(self.file_id, span)),
                        },
                    )),
                    Rule::cond_else => {
                        match &mut left.kind {
                            ExprKind::Conditional { r#else, .. } => *r#else = Box::new(right),
                            _ => unreachable!(),
                        }
                        Ok(left)
                    }
                    _ => Ok(Expression::new(
                        self.file_id,
                        span,
                        ExprKind::BinOp {
                            op: match pair.as_rule() {
                                Rule::add => BinOpKind::Add,
                                Rule::sub => BinOpKind::Sub,
                                Rule::mul => BinOpKind::Mul,
                                Rule::div => BinOpKind::Div,
                                Rule::r#mod => BinOpKind::Mod,
                                Rule::eq => BinOpKind::Eq,
                                Rule::neq => BinOpKind::Neq,
                                Rule::case_eq => BinOpKind::CaseEq,
                                Rule::case_neq => BinOpKind::CaseNeq,
                                Rule::wildcard_eq => BinOpKind::WildCardEq,
                                Rule::wildcard_neq => BinOpKind::WildCardNeq,
                                Rule::and => BinOpKind::And,
                                Rule::or => BinOpKind::Or,
                                Rule::pow => BinOpKind::Pow,
                                Rule::lt => BinOpKind::Lt,
                                Rule::le => BinOpKind::Le,
                                Rule::gt => BinOpKind::Gt,
                                Rule::ge => BinOpKind::Ge,
                                Rule::bit_and => BinOpKind::BitAnd,
                                Rule::bit_or => BinOpKind::BitOr,
                                Rule::bit_xor => BinOpKind::BitXor,
                                Rule::bit_xnor => BinOpKind::BitXNor,
                                Rule::rshift => BinOpKind::RShift,
                                Rule::arithmetic_rshift => BinOpKind::ArithRShift,
                                Rule::lshift => BinOpKind::LShift,
                                Rule::arithmetic_lshift => BinOpKind::ArithLShift,
                                Rule::implication => BinOpKind::Implication,
                                Rule::logical_eq => BinOpKind::LogicalEq,
                                Rule::triple_amp => BinOpKind::TripleAmp,
                                _ => todo!(),
                            },
                            left: Box::new(left),
                            right: Box::new(right),
                        },
                    )),
                }
            })
            .parse(pair.into_inner())
    }

    pub fn parse_operator_assignment(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut op = AssignOp::Assign;
        let mut left = None;
        let mut right = None;
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::variable_lvalue => left = Some(self.parse_variable_lvalue(pair)?),
                Rule::assignment_operator => {
                    op = match pair.as_str() {
                        "<<<=" => AssignOp::ArithLShift,
                        ">>>=" => AssignOp::ArithRShift,
                        "<<=" => AssignOp::LShift,
                        ">>=" => AssignOp::RShift,
                        "+=" => AssignOp::Add,
                        "-=" => AssignOp::Sub,
                        "*=" => AssignOp::Mul,
                        "/=" => AssignOp::Div,
                        "%=" => AssignOp::Mod,
                        "&=" => AssignOp::BitAnd,
                        "|=" => AssignOp::BitOr,
                        "^=" => AssignOp::BitXor,
                        "=" => AssignOp::Assign,
                        _ => unreachable!(),
                    }
                }
                Rule::expression => right = Some(self.parse_expression(pair)?),
                _ => unreachable!(),
            }
        }

        Ok(Expression::new(
            self.file_id,
            span,
            ExprKind::Assignment {
                op,
                left: Box::new(left.unwrap()),
                right: Box::new(right.unwrap()),
            },
        ))
    }

    pub fn parse_variable_lvalue(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut expr = None;
        let mut concat_exprs = vec![];
        let mut access_kind = None;
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::implicit_class_handle => {
                    access_kind = Some(AccessKind::Member);
                    expr = Some(self.parse_implicit_class_handle(pair)?);
                }
                Rule::package_scope => {
                    access_kind = Some(AccessKind::Scope);
                    expr = Some(self.parse_package_scope(pair)?);
                }
                Rule::hierarchical_identifier => {
                    expr = Some(self.parse_hierarchical_identifier(pair, expr, access_kind)?)
                }
                Rule::select => expr = Some(self.parse_select(expr.unwrap(), pair)?),
                Rule::variable_lvalue => concat_exprs.push(self.parse_variable_lvalue(pair)?),
                Rule::assignment_pattern_expression_type => todo!(),
                Rule::assignment_pattern_variable_lvalue => todo!(),
                Rule::streaming_concatenation => todo!(),
                _ => unreachable!(),
            }
        }

        if let Some(t) = expr {
            Ok(t)
        } else if concat_exprs.len() > 0 {
            Ok(Expression::new(
                self.file_id,
                span,
                ExprKind::Concat(concat_exprs),
            ))
        } else {
            unreachable!()
        }
    }

    pub fn parse_implicit_class_handle(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let str = pair.as_str();
        let span = from_pest_span(pair.as_span());
        if str.contains("this") && str.contains("super") {
            Ok(Expression::new(self.file_id, span, ExprKind::ThisSuper))
        } else if str.contains("this") {
            Ok(Expression::new(self.file_id, span, ExprKind::This))
        } else if str.contains("super") {
            Ok(Expression::new(self.file_id, span, ExprKind::Super))
        } else {
            unreachable!()
        }
    }

    pub fn parse_package_scope(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut name = "$unit".to_string();
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::package_identifier => name = pair.as_str().to_string(),
                _ => unreachable!(),
            }
        }

        Ok(Expression::new(self.file_id, span, ExprKind::NameRef(name)))
    }

    pub fn parse_hierarchical_identifier(
        &self,
        pair: Pair<Rule>,
        base: Option<Expression>,
        access_kind: Option<AccessKind>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut expr = base;
        let mut access_kind = access_kind.unwrap_or(AccessKind::Member);
        let span = from_pest_span(pair.as_span());

        if pair.as_str().starts_with("$root") {
            match expr {
                Some(t) => {
                    expr = Some(Expression::new(
                        self.file_id,
                        span,
                        ExprKind::Access {
                            base: Box::new(t),
                            name: "$root".to_string(),
                            kind: access_kind,
                        },
                    ))
                }
                None => {
                    expr = Some(Expression::new(
                        self.file_id,
                        span,
                        ExprKind::NameRef("$root".to_string()),
                    ))
                }
            }
            access_kind = AccessKind::Member;
        }

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::identifier => {
                    match expr {
                        Some(t) => {
                            expr = Some(Expression::new(
                                self.file_id,
                                span,
                                ExprKind::Access {
                                    base: Box::new(t),
                                    name: pair.as_str().to_string(),
                                    kind: access_kind,
                                },
                            ))
                        }
                        None => {
                            expr = Some(Expression::new(
                                self.file_id,
                                span,
                                ExprKind::NameRef(pair.as_str().to_string()),
                            ))
                        }
                    }

                    access_kind = AccessKind::Member;
                }
                Rule::bit_select => expr = Some(self.parse_bit_select(expr.unwrap(), pair)?),
                _ => unreachable!(),
            }
        }

        Ok(expr.unwrap())
    }

    pub fn parse_inc_or_dec_expression(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut op = UnaryOpKind::PrefixInc;
        let mut operand = None;
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::inc_or_dec_operator => match (pair.as_str(), op) {
                    ("++", UnaryOpKind::PrefixDec | UnaryOpKind::PrefixInc) => {
                        op = UnaryOpKind::PrefixInc
                    }
                    ("++", UnaryOpKind::PostfixDec | UnaryOpKind::PostfixInc) => {
                        op = UnaryOpKind::PostfixInc
                    }
                    ("--", UnaryOpKind::PrefixDec | UnaryOpKind::PrefixInc) => {
                        op = UnaryOpKind::PrefixDec
                    }
                    ("--", UnaryOpKind::PostfixDec | UnaryOpKind::PostfixInc) => {
                        op = UnaryOpKind::PostfixDec
                    }
                    _ => unreachable!(),
                },
                Rule::variable_lvalue => {
                    op = UnaryOpKind::PostfixInc;
                    operand = Some(self.parse_variable_lvalue(pair)?);
                }
                Rule::attribute_instance => todo!(),
                _ => unreachable!(),
            }
        }

        Ok(Expression::new(
            self.file_id,
            span,
            ExprKind::UnaryOp {
                op,
                operand: Box::new(operand.unwrap()),
            },
        ))
    }

    pub fn parse_number(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::integral_number => return self.parse_integral_number(pair),
                Rule::real_number => return self.parse_real_number(pair),
                _ => unreachable!(),
            }
        }
        unreachable!()
    }

    pub fn parse_integral_number(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut size = String::new();
        let mut base = IntegerBase::Decimal;
        let mut digits = String::new();
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::size => size = pair.as_str().to_string(),
                Rule::decimal_base => base = IntegerBase::Decimal,
                Rule::binary_base => base = IntegerBase::Binary,
                Rule::octal_base => base = IntegerBase::Octal,
                Rule::hex_base => base = IntegerBase::Hex,
                Rule::unsigned_number
                | Rule::binary_value
                | Rule::octal_value
                | Rule::hex_value
                | Rule::x_digit
                | Rule::z_digit => digits = pair.as_str().to_string(),
                _ => unreachable!(),
            }
        }

        Ok(Expression::new(
            self.file_id,
            span,
            ExprKind::Integer { size, base, digits },
        ))
    }

    pub fn parse_real_number(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        Ok(Expression::new(
            self.file_id,
            from_pest_span(pair.as_span()),
            ExprKind::Real(pair.as_str().to_string()),
        ))
    }

    pub fn parse_concatenation(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut exprs = vec![];
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::expression => exprs.push(self.parse_expression(pair)?),
                _ => unreachable!(),
            }
        }

        Ok(Expression::new(self.file_id, span, ExprKind::Concat(exprs)))
    }

    pub fn parse_range_expression(
        &self,
        base: Expression,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::expression => {
                    return Ok(Expression::new(
                        self.file_id,
                        span,
                        ExprKind::Select {
                            base: Box::new(base),
                            kind: SelectKind::BitSelect(Box::new(self.parse_expression(pair)?)),
                        },
                    ));
                }
                Rule::part_select_range => return self.parse_part_select_range(base, pair),
                _ => unreachable!(),
            }
        }

        unreachable!()
    }

    pub fn parse_multiple_concatenation(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut repeat = None;
        let mut expr = None;
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::expression => repeat = Some(self.parse_expression(pair)?),
                Rule::concatenation => expr = Some(self.parse_concatenation(pair)?),
                _ => unreachable!(),
            }
        }

        Ok(Expression::new(
            self.file_id,
            span,
            ExprKind::MultipleConcat {
                repeat: Box::new(repeat.unwrap()),
                expr: Box::new(expr.unwrap()),
            },
        ))
    }

    pub fn parse_subroutine_call(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::tf_call => return self.parse_tf_call(pair),
                _ => todo!(),
            }
        }
        unreachable!()
    }

    pub fn parse_tf_call(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let span = from_pest_span(pair.as_span());
        let mut expr = None;
        let mut args = vec![];

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::ps_or_hierarchical_tf_identifier => {
                    expr = Some(self.parse_ps_or_hierarchical_tf_identifier(pair)?)
                }
                Rule::attribute_instance => todo!(),
                Rule::list_of_arguments => args = self.parse_list_of_arguments(pair)?,
                _ => unreachable!(),
            }
        }

        Ok(Expression::new(
            self.file_id,
            span,
            ExprKind::Call {
                base: Box::new(expr.unwrap()),
                args,
            },
        ))
    }

    pub fn parse_ps_or_hierarchical_tf_identifier(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut expr = None;
        let mut access_kind = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::package_scope => {
                    access_kind = Some(AccessKind::Scope);
                    expr = Some(self.parse_package_scope(pair)?);
                }
                Rule::tf_identifier => match expr {
                    Some(t) => {
                        expr = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::Access {
                                base: Box::new(t),
                                name: pair.as_str().to_string(),
                                kind: access_kind.unwrap(),
                            },
                        ))
                    }
                    None => {
                        expr = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::NameRef(pair.as_str().to_string()),
                        ))
                    }
                },
                Rule::hierarchical_identifier => {
                    expr = Some(self.parse_hierarchical_identifier(pair, expr, access_kind)?)
                }
                _ => todo!(),
            }
        }

        Ok(expr.unwrap())
    }

    pub fn parse_list_of_arguments(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Vec<Argument>, Diagnostic<FileId>> {
        let mut args = vec![];
        let mut name = None;
        let mut expr = None;
        let mut span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::identifier => name = Some(pair.as_str().to_string()),
                Rule::expression => expr = Some(self.parse_expression(pair)?),
                Rule::argument_separator => {
                    args.push(Argument::new(
                        self.file_id,
                        span,
                        match name {
                            Some(t) => ArgKind::Named { name: t, expr },
                            None => ArgKind::Position(expr),
                        },
                    ));
                    name = None;
                    expr = None;
                }
                _ => unreachable!(),
            }
        }

        args.push(Argument::new(
            self.file_id,
            span,
            match name {
                Some(t) => ArgKind::Named { name: t, expr },
                None => ArgKind::Position(expr),
            },
        ));

        Ok(args)
    }

    pub fn parse_mintypmax_expression(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let span = from_pest_span(pair.as_span());
        let mut exprs = vec![];

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::expression => exprs.push(self.parse_expression(pair)?),
                _ => unreachable!(),
            }
        }

        if exprs.len() == 3 {
            Ok(Expression::new(
                self.file_id,
                span,
                ExprKind::MinTypicalMax {
                    min: Box::new(exprs.remove(0)),
                    typical: Box::new(exprs.remove(0)),
                    max: Box::new(exprs.remove(0)),
                },
            ))
        } else if exprs.len() == 1 {
            Ok(exprs.remove(0))
        } else {
            unreachable!()
        }
    }

    pub fn parse_cast(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut target = None;
        let mut expr = None;
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::casting_type => target = Some(self.parse_casting_type(pair)?),
                Rule::expression => expr = Some(self.parse_expression(pair)?),
                _ => unreachable!(),
            }
        }

        Ok(Expression::new(
            self.file_id,
            span,
            ExprKind::Cast {
                expr: Box::new(expr.unwrap()),
                target: Box::new(target.unwrap()),
            },
        ))
    }

    pub fn parse_casting_type(&self, pair: Pair<Rule>) -> Result<CastTarget, Diagnostic<FileId>> {
        match pair.as_str() {
            "string" => Ok(CastTarget::String),
            "const" => Ok(CastTarget::Const),
            "signed" => Ok(CastTarget::Signing(false)),
            "unsigned" => Ok(CastTarget::Signing(true)),
            _ => {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::simple_type => {
                            return Ok(CastTarget::Ty(self.parse_simple_type(pair)?));
                        }
                        _ => unreachable!(),
                    }
                }
                unreachable!()
            }
        }
    }

    pub fn parse_range_list(&self, pair: Pair<Rule>) -> Result<Vec<Range>, Diagnostic<FileId>> {
        let mut ranges = vec![];

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::value_range => ranges.push(self.parse_value_range(pair)?),
                _ => unreachable!(),
            }
        }

        Ok(ranges)
    }

    pub fn parse_value_range(&self, pair: Pair<Rule>) -> Result<Range, Diagnostic<FileId>> {
        let mut exprs = vec![];
        let mut is_max = false;
        let mut is_min = false;
        let str = pair.as_str();
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::expression => exprs.push(self.parse_expression(pair)?),
                Rule::min_bound => is_max = true,
                Rule::max_bound => is_min = true,
                _ => unreachable!(),
            }
        }

        Ok(Range::new(
            self.file_id,
            span,
            if str.contains("+/-") {
                RangeKind::AbsoluteTolerance {
                    center: exprs.remove(0),
                    tolerance: exprs.remove(0),
                }
            } else if str.contains("+%-") {
                RangeKind::RelativeTolerance {
                    center: exprs.remove(0),
                    percent: exprs.remove(0),
                }
            } else if exprs.len() == 2 {
                RangeKind::Bounded {
                    min: Some(exprs.remove(0)),
                    max: Some(exprs.remove(0)),
                }
            } else if is_max {
                RangeKind::Bounded {
                    min: None,
                    max: Some(exprs.remove(0)),
                }
            } else if is_min {
                RangeKind::Bounded {
                    min: Some(exprs.remove(0)),
                    max: None,
                }
            } else {
                RangeKind::Value(exprs.remove(0))
            },
        ))
    }

    pub fn parse_pattern(&self, pair: Pair<Rule>) -> Result<Pattern, Diagnostic<FileId>> {
        let span = from_pest_span(pair.as_span());
        let str = pair.as_str();
        let mut patterns = vec![];
        let mut names = vec![];
        let mut expr = None;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::pattern => patterns.push(self.parse_pattern(pair)?),
                Rule::variable_identifier | Rule::member_identifier => {
                    names.push(pair.as_str().to_string())
                }
                Rule::expression => expr = Some(self.parse_expression(pair)?),
                _ => unreachable!(),
            }
        }

        Ok(Pattern::new(
            self.file_id,
            span,
            if let Some(expr) = expr {
                PatternKind::Expr(expr)
            } else if str.starts_with(".") {
                if names.len() == 0 {
                    PatternKind::Wildcard
                } else {
                    PatternKind::Identifier(names.remove(0))
                }
            } else if str.starts_with("tagged") {
                PatternKind::Tagged {
                    name: names.remove(0),
                    pattern: if patterns.len() > 0 {
                        Some(Box::new(patterns.remove(0)))
                    } else {
                        None
                    },
                }
            } else if str.starts_with("'") {
                let mut member_patterns = vec![];
                if names.len() == 0 {
                    for pattern in patterns {
                        member_patterns.push(MemberPattern::new(
                            pattern.file_id,
                            pattern.span,
                            MemberPatternKind::Position(pattern),
                        ));
                    }
                } else {
                    for (name, pattern) in names.iter().zip(patterns) {
                        member_patterns.push(MemberPattern::new(
                            pattern.file_id,
                            pattern.span,
                            MemberPatternKind::Named {
                                name: name.clone(),
                                pattern,
                            },
                        ));
                    }
                }

                PatternKind::Member(member_patterns)
            } else {
                return Ok(patterns.remove(0));
            },
        ))
    }
}
