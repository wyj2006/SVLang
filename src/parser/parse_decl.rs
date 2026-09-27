use crate::{
    ast::{DeclKind, Declaration, ExprKind, Expression},
    diagnostic::from_pest_span,
    file_map::FileId,
    parser::{Rule, SVParser},
};
use codespan_reporting::diagnostic::Diagnostic;
use pest::iterators::Pair;

impl SVParser {
    pub fn parse_module_declaration(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Declaration, Diagnostic<FileId>> {
        let span = from_pest_span(pair.as_span());
        let mut name = None;
        let mut ports = vec![];
        let mut items = vec![];

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::module_keyword => {}
                Rule::module_identifier => name = Some(pair.as_str().to_string()),
                Rule::list_of_ports => ports.extend(self.parse_list_of_ports(pair)?),
                _ => todo!(),
            }
        }

        Ok(Declaration::new(
            self.file_id,
            span,
            DeclKind::Module {
                name: name.unwrap(),
                ports,
                items,
            },
        ))
    }

    pub fn parse_list_of_ports(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Vec<Declaration>, Diagnostic<FileId>> {
        let mut ports = vec![];

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::port => ports.push(self.parse_port(pair)?),
                _ => unreachable!(),
            }
        }

        Ok(ports)
    }

    pub fn parse_port(&self, pair: Pair<Rule>) -> Result<Declaration, Diagnostic<FileId>> {
        let mut name = None;
        let mut connection = None;
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::port_identifier => name = Some(pair.as_str().to_string()),
                Rule::port_expression => connection = Some(self.parse_port_expression(pair)?),
                _ => unreachable!(),
            }
        }

        Ok(Declaration::new(
            self.file_id,
            span,
            DeclKind::Port {
                name,
                connection,
                r#type: None,
            },
        ))
    }

    pub fn parse_port_expression(
        &self,
        pair: Pair<Rule>,
    ) -> Result<Expression, Diagnostic<FileId>> {
        let mut exprs = vec![];
        let span = from_pest_span(pair.as_span());

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::port_reference => exprs.push(self.parse_port_reference(pair)?),
                _ => unreachable!(),
            }
        }

        if exprs.len() == 1 {
            Ok(exprs.remove(0))
        } else {
            Ok(Expression::new(self.file_id, span, ExprKind::Concat(exprs)))
        }
    }

    pub fn parse_port_reference(&self, pair: Pair<Rule>) -> Result<Expression, Diagnostic<FileId>> {
        let mut expr = None;

        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::port_identifier => {
                    expr = Some(Expression::new(
                        self.file_id,
                        from_pest_span(pair.as_span()),
                        ExprKind::NameRef(pair.as_str().to_string()),
                    ))
                }
                Rule::select => expr = Some(self.parse_select(expr.unwrap(), pair)?),
                _ => unreachable!(),
            }
        }

        Ok(expr.unwrap())
    }
}
