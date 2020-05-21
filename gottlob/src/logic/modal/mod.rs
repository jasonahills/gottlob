use log::*;
use strum_macros::*;

pub mod parser;

use parser::ModalParser;
use super::{Logic, LogicResult, ParseError};

pub struct ModalLogic;

impl Logic for ModalLogic {
  fn name(&self) -> &'static str {
    "Modal Logic"
  }

  fn is_valid_theorem(&self, t: &str) -> LogicResult {
    let thm = ModalParser::parse_theorem(t).map_err(|e| {
      error!("parse error: {:?}", e);
      ParseError
    })?;
    // TODO: actually prove validity
    Ok((format!("{}", thm), true))
  }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
pub struct Variable(char);

impl std::fmt::Display for Variable {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let Variable(v) = self;
    write!(f, "{}", v)
  }
}

#[derive(Debug, Eq, PartialEq, Clone, EnumDiscriminants)] // Maybe we don't want to derive Eq for these, but instead impl something.
pub enum Expression {
  Variable(Variable),
  Negated(Box<Expression>),
  And(Box<Expression>, Box<Expression>),
  Or(Box<Expression>, Box<Expression>),
  Conditional(Box<Expression>, Box<Expression>),
  Biconditional(Box<Expression>, Box<Expression>),
  Necessary(Box<Expression>),
  Possible(Box<Expression>),
}

impl Expression {
  // TODO: consider being more DRY with these flattening functions
  fn flatten_and(&self) -> Vec<&Self> {
    match self {
      Expression::And(e1, e2) => [e1.flatten_and(), e2.flatten_and()].concat(),
      e => vec![e],
    }
  }

  fn flatten_or(&self) -> Vec<&Self> {
    match self {
      Expression::Or(e1, e2) => [e1.flatten_or(), e2.flatten_or()].concat(),
      e => vec![e],
    }
  }

  fn flatten_biconditional(&self) -> Vec<&Self> {
    match self {
      Expression::Biconditional(e1, e2) => [e1.flatten_biconditional(), e2.flatten_biconditional()].concat(),
      e => vec![e],
    }
  }
}

impl std::fmt::Display for Expression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      // TODO: no parens on outermost
      Self::Variable(v) => write!(f, "{}", v),
      Self::Negated(e) => write!(f, "¬{}", e),
      Self::Necessary(e) => write!(f, "◻{}", e),
      Self::Possible(e) => write!(f, "◇{}", e),
      e @ Self::And(_, _) => write!(
        f,
        "({})",
        e.flatten_and()
          .iter()
          .map(|e| format!("{}", e))
          .collect::<Vec<_>>()
          .join(" ∧ ")
      ),
      e @ Self::Or(_, _) => write!(
        f,
        "({})",
        e.flatten_or()
          .iter()
          .map(|e| format!("{}", e))
          .collect::<Vec<_>>()
          .join(" ∨ ")
      ),
      e @ Self::Biconditional(_, _) => write!(
        f,
        "({})",
        e.flatten_biconditional()
          .iter()
          .map(|e| format!("{}", e))
          .collect::<Vec<_>>()
          .join(" ↔ ")
      ),
      Self::Conditional(e1, e2) => write!(f, "({} → {})", e1, e2),
    }
  }
}

#[derive(Debug)]
pub enum Theorem {
  Proves { 
    assumptions: Vec<Expression>,
    conclusion: Expression,
  },
  DoesNotProve {
    assumptions: Vec<Expression>,
    conclusion: Expression,
  }
}

impl std::fmt::Display for Theorem {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let (assumptions, conclusion, op) = match self {
      Self::Proves{assumptions, conclusion} => (assumptions, conclusion, "⊢"),
      Self::DoesNotProve{assumptions, conclusion} => (assumptions, conclusion, "⊬"),
    };
    let assumptions = assumptions.iter().map(|e| format!("{}", e)).collect::<Vec<_>>().join(", ");
    write!(f, "{} {} {}", assumptions, op, conclusion)
  }
}
