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
use pest::iterators::Pair;

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

    pub fn parse_expression(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut left = None;
        let mut op = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::conditional_expression => {
                    if let None = left {
                        left = Some(self.parse_conditional_expression(pair)?)
                    } else {
                        left = Some(Expression::new(
                            self.file_id,
                            span,
                            ExprKind::BinOp {
                                op: op.unwrap(),
                                left: Box::new(left.unwrap()),
                                right: Box::new(self.parse_conditional_expression(pair)?),
                            },
                        ))
                    }
                }
                Rule::implication => op = Some(BinOpKind::Implication),
                Rule::logical_eq => op = Some(BinOpKind::LogicalEq),
                _ => unreachable!(),
            }
        }

        Ok(left.unwrap())
    }

    pub fn parse_conditional_expression(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut cond = None;
        let mut then = None;
        let mut r#else = None;
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::cond_predicate => cond = Some(self.parse_cond_predicate(pair)?),
                Rule::expression => {
                    if let None = then {
                        then = Some(self.parse_expression(pair)?);
                    } else {
                        r#else = Some(self.parse_expression(pair)?);
                    }
                }
                _ => todo!(),
            }
        }

        if let None = then {
            Ok(cond.unwrap())
        } else {
            Ok(Expression::new(
                self.file_id,
                span,
                ExprKind::Conditional {
                    cond: Box::new(cond.unwrap()),
                    then: Box::new(then.unwrap()),
                    r#else: Box::new(r#else.unwrap()),
                },
            ))
        }
    }

    pub fn parse_cond_predicate(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut exprs = vec![];
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::expression_or_cond_pattern => {
                    exprs.push(self.parse_expression_or_cond_pattern(pair)?)
                }
                _ => unreachable!(),
            }
        }

        if exprs.len() == 1 {
            Ok(exprs.remove(0))
        } else {
            Ok(Expression::new(
                self.file_id,
                span,
                ExprKind::CondPredicate(exprs),
            ))
        }
    }

    pub fn parse_expression_or_cond_pattern(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut expr = None;
        let mut pattern = None;
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::or_expression => expr = Some(self.parse_or_expression(pair)?),
                Rule::pattern => pattern = Some(self.parse_pattern(pair)?),
                _ => unreachable!(),
            }
        }

        if let None = pattern {
            Ok(expr.unwrap())
        } else {
            Ok(Expression::new(
                self.file_id,
                span,
                ExprKind::Matches {
                    expr: Box::new(expr.unwrap()),
                    pattern: Box::new(pattern.unwrap()),
                },
            ))
        }
    }

    pub fn parse_unary_expression(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut prefix = vec![];
        let mut expr = None;
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::primary => expr = Some(self.parse_primary(pair)?),
                //后缀运算符的优先级比前缀高
                Rule::postfix_inc => {
                    expr = Some(Expression::new(
                        self.file_id,
                        span,
                        ExprKind::UnaryOp {
                            op: UnaryOpKind::PostfixInc,
                            operand: Box::new(expr.unwrap()),
                        },
                    ))
                }
                Rule::postfix_dec => {
                    expr = Some(Expression::new(
                        self.file_id,
                        span,
                        ExprKind::UnaryOp {
                            op: UnaryOpKind::PostfixDec,
                            operand: Box::new(expr.unwrap()),
                        },
                    ))
                }
                Rule::positive => prefix.push(UnaryOpKind::Positive),
                Rule::negative => prefix.push(UnaryOpKind::Negative),
                Rule::not => prefix.push(UnaryOpKind::Not),
                Rule::bit_not => prefix.push(UnaryOpKind::BitNot),
                Rule::reduction_and => prefix.push(UnaryOpKind::ReductionAnd),
                Rule::reduction_nand => prefix.push(UnaryOpKind::ReductionNAnd),
                Rule::reduction_or => prefix.push(UnaryOpKind::ReductionOr),
                Rule::reduction_nor => prefix.push(UnaryOpKind::ReductionNOr),
                Rule::reduction_xor => prefix.push(UnaryOpKind::ReductionXor),
                Rule::reduction_xnor => prefix.push(UnaryOpKind::ReductionXNor),
                Rule::prefix_inc => prefix.push(UnaryOpKind::PostfixInc),
                Rule::prefix_dec => prefix.push(UnaryOpKind::PrefixDec),
                _ => todo!(),
            }
        }

        for prefix in prefix.iter().rev() {
            expr = Some(Expression::new(
                self.file_id,
                span,
                ExprKind::UnaryOp {
                    op: *prefix,
                    operand: Box::new(expr.unwrap()),
                },
            ))
        }

        Ok(expr.unwrap())
    }

    pub fn parse_primary(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut expr = None;
        let mut access_kind = None;

        for pair in pair.into_inner() {
            let span = from_pest_span(pair.as_span());
            match pair.as_rule() {
                Rule::operator_assignment => expr = Some(self.parse_operator_assignment(pair)?),
                Rule::number => expr = Some(self.parse_number(pair)?),
                Rule::package_scope => {
                    access_kind = Some(AccessKind::Scope);
                    expr = Some(self.parse_package_scope(pair)?);
                }
                Rule::hierarchical_identifier => {
                    expr = Some(self.parse_hierarchical_identifier(pair, expr, access_kind)?)
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
                Rule::mintypmax_expression => expr = Some(self.parse_mintypmax_expression(pair)?),
                Rule::cast => expr = Some(self.parse_cast(pair)?),
                _ => match pair.as_str() {
                    "this" => expr = Some(Expression::new(self.file_id, span, ExprKind::This)),
                    "$" => expr = Some(Expression::new(self.file_id, span, ExprKind::Dollar)),
                    "null" => expr = Some(Expression::new(self.file_id, span, ExprKind::Null)),
                    "" => {}
                    _ => todo!(),
                },
            }
        }

        Ok(expr.unwrap())
    }
}
