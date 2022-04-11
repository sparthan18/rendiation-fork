pub type Span = std::ops::Range<usize>;

use crate::lexer::{Lexer, Token};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PrimitiveConstValue {
  Bool(bool),
  Numeric(NumericTypeConstValue),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum NumericTypeConstValue {
  Float(f32),
  Int(i32),
  UnsignedInt(u32),
}

#[derive(Debug)]
pub enum ParseError<'a> {
  Any(&'static str),
  Unexpected(Token<'a>, &'a str),
}

pub trait SyntaxElement: Sized {
  fn parse_input<'a>(input: &'a str) -> Result<Self, ParseError<'a>> {
    Self::parse(&mut Lexer::new(input))
  }
  fn parse<'a>(lexer: &mut Lexer<'a>) -> Result<Self, ParseError<'a>>;
}

#[derive(Debug)]
pub struct FunctionDefine {
  pub name: Ident,
  pub arguments: Vec<(Ident, TypeExpression)>,
  pub return_type: Option<TypeExpression>,
  pub body: Block,
}

#[derive(Debug)]
pub struct Block {
  pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub struct If {
  pub condition: Expression,
  pub accept: Block,
  pub elses: Vec<IfElse>,
  pub reject: Option<Block>,
}

/// https://www.w3.org/TR/WGSL/#switch-statement
#[derive(Debug)]
pub struct Switch {
  pub target: Expression,
  pub cases: Vec<SwitchBody>,
}

#[derive(Debug)]
pub enum CaseType {
  Const(Vec<Expression>), // todo const literal,
  Default,
}

#[derive(Debug)]
pub struct SwitchBody {
  pub case: CaseType,
  pub statements: Vec<Statement>,
  pub fallthrough: bool,
}

#[derive(Debug)]
pub struct IfElse {
  pub condition: Expression,
  pub accept: Block,
}

#[derive(Debug)]
pub struct While {
  pub condition: Expression,
  pub body: Block,
}

#[derive(Debug)]
pub struct For {
  pub init: Box<Statement>,
  pub test: Box<Statement>,
  pub update: Expression,
  pub body: Block,
}

#[derive(Debug)]
pub enum Statement {
  Block(Block),
  Declare {
    declare_ty: DeclarationType,
    ty: Option<TypeExpression>,
    name: Ident,
    init: Option<Expression>,
  },
  Empty,
  Assignment {
    lhs: LhsExpression,
    value: Expression,
  },
  Expression(Expression),
  Return {
    value: Option<Expression>,
  },
  If(If),
  Switch(Switch),
  While(While),
  Loop {
    statements: Vec<Self>,
    // continuing:
  },
  Break,
  Continue,
  Discard,
  For(For),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeclarationType {
  Variable,
  Const,
}

#[derive(Debug)]
pub enum PrimitiveValueType {
  Float32,
  UnsignedInt32,
  Int32,
  Bool,
}

#[derive(Debug)]
pub enum PrimitiveVecDataType {
  Vec2,
  Vec3,
  Vec4,
  Mat2,
  Mat3,
  Mat4,
}

#[derive(Debug)]
pub enum TextureContainerType {
  D1,
  D2,
  D2Array,
  D3,
  Cube,
  CubeArray,
}

#[derive(Debug)]
pub struct TextureType {
  pub value_ty: PrimitiveValueType,
  pub container_ty: TextureContainerType,
}

#[derive(Debug)]
pub struct PrimitiveVectorType {
  pub value_ty: PrimitiveValueType,
  pub vec_ty: PrimitiveVecDataType,
}

#[derive(Debug)]
pub enum PrimitiveType {
  Scalar(PrimitiveValueType),
  Vector(PrimitiveVectorType),
  Texture(TextureType),
  Sampler,
}

#[derive(Debug)]
pub enum TypeExpression {
  Struct(Ident),
  Primitive(PrimitiveType),
}

#[derive(Debug)]
pub enum Expression {
  UnaryOperator {
    op: UnaryOperator,
    expr: Box<Self>,
  },
  BinaryOperator {
    left: Box<Self>,
    op: BinaryOperator,
    right: Box<Self>,
  },
  FunctionCall(FunctionCall),
  PrimitiveConstruct {
    ty: PrimitiveType,
    arguments: Vec<Expression>,
  },
  ArrayAccess {
    array: Box<Self>,
    index: Box<Self>,
  },
  ItemAccess {
    from: Box<Self>,
    to: Ident,
  },
  PrimitiveConst(PrimitiveConstValue),
  Ident(Ident),
}

// pub enum PrimaryExpression{
//   Ident(Ident),
//   PrimitiveConst(PrimitiveConstValue),
//   FunctionCall(FunctionCall),
// }

// pub struct SingularExpression {
//   primary: Box<>,

// }

#[derive(Debug)]
pub struct LhsExpression {
  pub content: LhsExpressionCore,
  pub postfix: Vec<PostFixExpression>,
}

#[derive(Debug)]
pub enum LhsExpressionCore {
  Ident(Ident),
  Deref(Box<LhsExpression>),
  Ref(Box<LhsExpression>),
}

#[derive(Debug)]
pub enum PostFixExpression {
  ArrayAccess { index: Box<Expression> },
  FieldAccess { field: Ident },
}

#[derive(Debug)]
pub struct FunctionCall {
  pub name: Ident,
  pub arguments: Vec<Expression>,
}

#[derive(Debug)]
pub struct Ident {
  pub name: String,
}

impl From<&str> for Ident {
  fn from(name: &str) -> Self {
    Self {
      name: name.to_owned(),
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub enum UnaryOperator {
  Neg,
  Not,
}

impl std::fmt::Display for UnaryOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      UnaryOperator::Neg => write!(f, "-"),
      UnaryOperator::Not => write!(f, "!"),
    }
  }
}

#[derive(Copy, Clone, Debug)]
pub enum BinaryOperator {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Less,
  LessEqual,
  Greater,
  GreaterEqual,
  Equal,
  NotEqual,
  And,
  Or,
  Xor,
  LogicalAnd,
  LogicalOr,
}

impl std::fmt::Display for BinaryOperator {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      BinaryOperator::Add => write!(f, "+"),
      BinaryOperator::Sub => write!(f, "-"),
      BinaryOperator::Mul => write!(f, "*"),
      BinaryOperator::Div => write!(f, "/"),
      BinaryOperator::Mod => write!(f, "%"),
      BinaryOperator::Less => write!(f, "<"),
      BinaryOperator::LessEqual => write!(f, "<="),
      BinaryOperator::Greater => write!(f, ">"),
      BinaryOperator::GreaterEqual => write!(f, ">="),
      BinaryOperator::Equal => write!(f, "=="),
      BinaryOperator::NotEqual => write!(f, "!="),
      BinaryOperator::And => write!(f, "&"),
      BinaryOperator::Or => write!(f, "|"),
      BinaryOperator::Xor => write!(f, "^"),
      BinaryOperator::LogicalAnd => write!(f, "&&"),
      BinaryOperator::LogicalOr => write!(f, "||"),
    }
  }
}