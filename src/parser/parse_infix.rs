use crate::{
    ast::{BinOpKind, ExprKind, Expression},
    diagnostic::from_pest_span,
    file_map::FileId,
    parser::{Rule, SVParser},
};
use codespan_reporting::diagnostic::Diagnostic;
use pest::iterators::Pair;
impl SVParser {
    pub fn parse_or_expression(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::and_expression => {
                    if let None = left {
                        left = Some(self.parse_and_expression(pair)?);
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_and_expression(pair)?),
                            },
                        ));
                    }
                }
                Rule::or => op = Some(BinOpKind::Or),
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }
    pub fn parse_and_expression(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::bit_or_expression => {
                    if let None = left {
                        left = Some(self.parse_bit_or_expression(pair)?);
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_bit_or_expression(pair)?),
                            },
                        ));
                    }
                }
                Rule::and => op = Some(BinOpKind::And),
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }
    pub fn parse_bit_or_expression(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::bit_xnor_expression => {
                    if let None = left {
                        left = Some(self.parse_bit_xnor_expression(pair)?);
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_bit_xnor_expression(pair)?),
                            },
                        ));
                    }
                }
                Rule::bit_or => op = Some(BinOpKind::BitOr),
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }
    pub fn parse_bit_xnor_expression(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::bit_and_expression => {
                    if let None = left {
                        left = Some(self.parse_bit_and_expression(pair)?);
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_bit_and_expression(pair)?),
                            },
                        ));
                    }
                }
                Rule::bit_xnor => op = Some(BinOpKind::BitXNor),
                Rule::bit_xor => op = Some(BinOpKind::BitXor),
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }
    pub fn parse_bit_and_expression(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::case_eq_expression => {
                    if let None = left {
                        left = Some(self.parse_case_eq_expression(pair)?);
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_case_eq_expression(pair)?),
                            },
                        ));
                    }
                }
                Rule::bit_and => op = Some(BinOpKind::BitAnd),
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }
    pub fn parse_case_eq_expression(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::le_expression => {
                    if let None = left {
                        left = Some(self.parse_le_expression(pair)?);
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_le_expression(pair)?),
                            },
                        ));
                    }
                }
                Rule::case_eq => op = Some(BinOpKind::CaseEq),
                Rule::case_neq => op = Some(BinOpKind::CaseNeq),
                Rule::wildcard_eq => op = Some(BinOpKind::WildCardEq),
                Rule::wildcard_neq => op = Some(BinOpKind::WildCardNeq),
                Rule::eq => op = Some(BinOpKind::Eq),
                Rule::neq => op = Some(BinOpKind::Neq),
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }
    pub fn parse_le_expression(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::arith_lshift_expression => {
                    if let None = left {
                        left = Some(self.parse_arith_lshift_expression(pair)?);
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_arith_lshift_expression(pair)?),
                            },
                        ));
                    }
                }
                Rule::le => op = Some(BinOpKind::Le),
                Rule::ge => op = Some(BinOpKind::Ge),
                Rule::lt => op = Some(BinOpKind::Lt),
                Rule::gt => op = Some(BinOpKind::Gt),
                Rule::range_list => {
                    left = Some(Expression::new(
                        self.file_id,
                        span,
                        ExprKind::Inside {
                            expr: Box::new(left.unwrap()),
                            ranges: self.parse_range_list(pair)?,
                        },
                    ))
                }
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }
    pub fn parse_arith_lshift_expression(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::add_expression => {
                    if let None = left {
                        left = Some(self.parse_add_expression(pair)?);
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_add_expression(pair)?),
                            },
                        ));
                    }
                }
                Rule::arith_lshift => op = Some(BinOpKind::ArithLShift),
                Rule::arith_rshift => op = Some(BinOpKind::ArithRShift),
                Rule::lshift => op = Some(BinOpKind::LShift),
                Rule::rshift => op = Some(BinOpKind::RShift),
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }
    pub fn parse_add_expression(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::mul_expression => {
                    if let None = left {
                        left = Some(self.parse_mul_expression(pair)?);
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_mul_expression(pair)?),
                            },
                        ));
                    }
                }
                Rule::add => op = Some(BinOpKind::Add),
                Rule::sub => op = Some(BinOpKind::Sub),
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }
    pub fn parse_mul_expression(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::pow_expression => {
                    if let None = left {
                        left = Some(self.parse_pow_expression(pair)?);
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_pow_expression(pair)?),
                            },
                        ));
                    }
                }
                Rule::mul => op = Some(BinOpKind::Mul),
                Rule::div => op = Some(BinOpKind::Div),
                Rule::r#mod => op = Some(BinOpKind::Mod),
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }
    pub fn parse_pow_expression(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::unary_expression => {
                    if let None = left {
                        left = Some(self.parse_unary_expression(pair)?);
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_unary_expression(pair)?),
                            },
                        ));
                    }
                }
                Rule::pow => op = Some(BinOpKind::Pow),
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }
}
