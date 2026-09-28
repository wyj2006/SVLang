use crate::file_map::{FileId, source_lookup};
use codespan::Span;

type NodeId = usize;

pub const DUMMY_NODE_ID: NodeId = NodeId::MAX;

#[derive(Debug, Clone)]
pub struct CompilationUnit {
    pub node_id: NodeId,
    pub file_id: FileId,
    pub span: Span,
    pub decls: Vec<Declaration>,
}

impl CompilationUnit {
    pub fn new(file_id: FileId, span: Span) -> CompilationUnit {
        let (file_id, span) = source_lookup(file_id, span);
        CompilationUnit {
            node_id: DUMMY_NODE_ID,
            file_id,
            span,
            decls: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub node_id: NodeId,
    pub file_id: FileId,
    pub span: Span,
    pub kind: DeclKind,
}

#[derive(Debug, Clone)]
pub enum DeclKind {
    Module {
        name: String,
        ports: Vec<Declaration>,
        items: Vec<Item>,
    },
    Port {
        name: Option<String>,
        connection: Option<Expression>,
        r#type: Option<Type>,
    },
}

impl Declaration {
    pub fn new(file_id: FileId, span: Span, kind: DeclKind) -> Declaration {
        Declaration {
            node_id: DUMMY_NODE_ID,
            file_id,
            span,
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    Decl(Declaration),
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub node_id: NodeId,
    pub file_id: FileId,
    pub span: Span,
    pub kind: ExprKind,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Concat(Vec<Expression>),
    MultipleConcat {
        repeat: Box<Expression>,
        expr: Box<Expression>,
    },
    Select {
        base: Box<Expression>,
        kind: SelectKind,
    },
    NameRef(String),
    Access {
        base: Box<Expression>,
        name: String,
        kind: AccessKind,
    },
    Conditional {
        cond: Box<Expression>,
        then: Box<Expression>,
        r#else: Box<Expression>,
    },
    BinOp {
        op: BinOpKind,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOpKind,
        operand: Box<Expression>,
    },
    Cast {
        expr: Box<Expression>,
        target: Box<CastTarget>,
    },
    Inside {
        expr: Box<Expression>,
        ranges: Vec<Range>,
    },
    Call {
        base: Box<Expression>,
        args: Vec<Argument>,
    },
    Matches {
        expr: Box<Expression>,
        pattern: Box<Pattern>,
    },
    Assignment {
        op: AssignOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    This,
    Super,
    ThisSuper,
    Dollar,
    Null,
    Integer {
        size: String,
        base: IntegerBase,
        digits: String,
    },
    Real(String),
    MinTypicalMax {
        min: Box<Expression>,
        typical: Box<Expression>,
        max: Box<Expression>,
    },
    CondPredicate(Vec<Expression>),
}

impl Expression {
    pub fn new(file_id: FileId, span: Span, kind: ExprKind) -> Expression {
        Expression {
            node_id: DUMMY_NODE_ID,
            file_id,
            span,
            kind,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IntegerBase {
    Binary,
    Decimal,
    Octal,
    Hex,
}

#[derive(Debug, Clone, Copy)]
pub enum AccessKind {
    Member,
    Scope,
}

#[derive(Debug, Clone)]
pub enum CastTarget {
    Ty(Type),
    Expr(Expression),
    Signing(bool), //true表示unsigned
    String,
    Const,
}

#[derive(Debug, Clone)]
pub enum SelectKind {
    BitSelect(Box<Expression>),
    RangeSelect {
        msb: Box<Expression>,
        lsb: Box<Expression>,
    },
    IndexedSelect {
        base: Box<Expression>,
        direction: SelectDirection,
        width: Box<Expression>,
    },
}

#[derive(Debug, Clone)]
pub enum SelectDirection {
    Positive,
    Negative,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    CaseEq,
    CaseNeq,
    WildCardEq,
    WildCardNeq,
    And,
    Or,
    Pow,
    Lt,
    Le,
    Gt,
    Ge,
    BitAnd,
    BitOr,
    BitXor,
    BitXNor,
    RShift,
    ArithRShift,
    LShift,
    ArithLShift,
    Implication,
    LogicalEq,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOpKind {
    Positive,
    Negative,
    Not,
    BitNot,
    ReductionAnd,
    ReductionNAnd,
    ReductionOr,
    ReductionNOr,
    ReductionXor,
    ReductionXNor,
    PostfixInc,
    PostfixDec,
    PrefixInc,
    PrefixDec,
}

#[derive(Debug, Clone, Copy)]
pub enum AssignOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    LShift,
    ArithLShift,
    RShift,
    ArithRShift,
}

#[derive(Debug, Clone)]
pub struct Range {
    pub node_id: NodeId,
    pub file_id: FileId,
    pub span: Span,
    pub kind: RangeKind,
}

#[derive(Debug, Clone)]
pub enum RangeKind {
    Value(Expression),
    Bounded {
        min: Option<Expression>, //为None表示类型的最小值
        max: Option<Expression>, //为None表示类型的最大值
    },
    AbsoluteTolerance {
        center: Expression,
        tolerance: Expression,
    },
    RelativeTolerance {
        center: Expression,
        percent: Expression,
    },
}

impl Range {
    pub fn new(file_id: FileId, span: Span, kind: RangeKind) -> Range {
        Range {
            node_id: DUMMY_NODE_ID,
            file_id,
            span,
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Argument {
    pub node_id: NodeId,
    pub file_id: FileId,
    pub span: Span,
    pub kind: ArgKind,
}

#[derive(Debug, Clone)]
pub enum ArgKind {
    Position(Option<Expression>),
    Named {
        name: String,
        expr: Option<Expression>,
    },
}

impl Argument {
    pub fn new(file_id: FileId, span: Span, kind: ArgKind) -> Argument {
        Argument {
            node_id: DUMMY_NODE_ID,
            file_id,
            span,
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub node_id: NodeId,
    pub file_id: FileId,
    pub span: Span,
    pub kind: PatternKind,
}

#[derive(Debug, Clone)]
pub enum PatternKind {
    Identifier(String),
    Wildcard,
    Expr(Expression),
    Tagged {
        name: String,
        pattern: Option<Box<Pattern>>,
    },
    Member(Vec<MemberPattern>),
}

impl Pattern {
    pub fn new(file_id: FileId, span: Span, kind: PatternKind) -> Pattern {
        Pattern {
            node_id: DUMMY_NODE_ID,
            file_id,
            span,
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemberPattern {
    pub node_id: NodeId,
    pub file_id: FileId,
    pub span: Span,
    pub kind: MemberPatternKind,
}

#[derive(Debug, Clone)]
pub enum MemberPatternKind {
    Position(Pattern),
    Named { name: String, pattern: Pattern },
}

impl MemberPattern {
    pub fn new(file_id: FileId, span: Span, kind: MemberPatternKind) -> MemberPattern {
        MemberPattern {
            node_id: DUMMY_NODE_ID,
            file_id,
            span,
            kind,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Type {
    pub node_id: NodeId,
    pub file_id: FileId,
    pub span: Span,
    pub kind: TypeKind,
}

#[derive(Debug, Clone)]
pub enum TypeKind {
    Byte,
    ShortInt,
    Int,
    LongInt,
    Integer,
    Time,
    Bit,
    Logic,
    Reg,
    ShortReal,
    Real,
    RealTime,
}

impl Type {
    pub fn new(file_id: FileId, span: Span, kind: TypeKind) -> Type {
        Type {
            node_id: DUMMY_NODE_ID,
            file_id,
            span,
            kind,
        }
    }
}
