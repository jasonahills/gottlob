use crate::util::powerset::IntoPowerSet;
use std::collections::HashSet;
use strum_macros::*;
use log::*;

pub mod parser;
pub mod parser_reverse_polish;

use parser::ClassicalParser;
use parser_reverse_polish::ClassicalRpParser;
use super::{Logic, LogicResult, ParseError};

pub struct ClassicalLogic;

impl Logic for ClassicalLogic {
  fn name(&self) -> &'static str {
    "Classical"
  }

  fn is_valid_theorem(&self, t: &str) -> LogicResult {
    // TODO: actually parse theorems, not just sentences
    let thm = ClassicalParser::parse_expression(t).or_else(|_| ClassicalRpParser::parse_expression(t)).map_err(|e| {
      // TODO: don't log these, since we expect them to be common; instead put better info in the error
      error!("classical parse error: {:?}", e);
      ParseError
    })?;
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
}

impl Expression {
  pub fn eval(&self, trues: &HashSet<Variable>) -> bool {
    match self {
      Self::Variable(v) => trues.contains(&v),
      Self::Negated(e) => !e.eval(trues),
      Self::And(e1, e2) => e1.eval(trues) && e2.eval(trues),
      Self::Or(e1, e2) => e1.eval(trues) || e2.eval(trues),
      Self::Conditional(e1, e2) => !e1.eval(trues) || e2.eval(trues),
      Self::Biconditional(e1, e2) => e1.eval(trues) == e2.eval(trues),
    }
  }

  // TODO: make more efficient if necessary
  pub fn variables(&self) -> HashSet<Variable> {
    match self {
      Self::Variable(v) => [*v].into_iter().cloned().collect::<HashSet<Variable>>(),
      Self::Negated(e) => e.variables(),
      Self::And(e1, e2) => e1.variables().union(&e2.variables()).cloned().collect(),
      Self::Or(e1, e2) => e1.variables().union(&e2.variables()).cloned().collect(),
      Self::Conditional(e1, e2) => e1.variables().union(&e2.variables()).cloned().collect(),
      Self::Biconditional(e1, e2) => e1.variables().union(&e2.variables()).cloned().collect(),
    }
  }

  /// Uses truth-table to determine if this is a tautology.  Very inefficient if there are many variables.
  pub fn is_tautology(&self) -> bool {
    self.variables().powerset().all(|sub| self.eval(&sub))
  }

  fn is_variable(&self) -> bool {
    ExpressionDiscriminants::from(self) == ExpressionDiscriminants::Variable
  }

  fn is_negated(&self) -> bool {
    ExpressionDiscriminants::from(self) == ExpressionDiscriminants::Negated
  }

  // fn is_and(&self) -> bool {
  //   ExpressionDiscriminants::from(self) == ExpressionDiscriminants::And
  // }

  // fn is_or(&self) -> bool {
  //   ExpressionDiscriminants::from(self) == ExpressionDiscriminants::Or
  // }

  // fn is_conditional(&self) -> bool {
  //   ExpressionDiscriminants::from(self) == ExpressionDiscriminants::Conditional
  // }

  // fn is_biconditional(&self) -> bool {
  //   ExpressionDiscriminants::from(self) == ExpressionDiscriminants::Biconditional
  // }

  fn flatten_and(&self) -> Vec<&Self> {
    match self {
      Expression::And(e1, e2) => [e1.flatten_and(), e2.flatten_and()].concat(),
      e => vec![e],
    }
  }
}

impl std::fmt::Display for Expression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      // TODO: do these without automatically grouping them.
      // TODO: no parens on outermost
      // TODO: use cooler unicode chars
      Self::Variable(v) => write!(f, "{}", v),
      Self::Negated(e) if e.is_variable() || e.is_negated() => write!(f, "¬{}", e),
      Self::Negated(e) => write!(f, "¬{}", e),  // TODO: do I need this after all?
      e @ Self::And(_, _) => write!(
        f,
        "({})",
        e.flatten_and()
          .iter()
          .map(|e| format!("{}", e))
          .collect::<Vec<_>>()
          .join(" ∧ ")
      ),
      Self::Or(e1, e2) => write!(f, "({} ∨ {})", e1, e2),
      Self::Conditional(e1, e2) => write!(f, "({} → {})", e1, e2),
      Self::Biconditional(e1, e2) => write!(f, "({} ↔ {})", e1, e2),
    }
  }
}
