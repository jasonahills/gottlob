use crate::util::powerset::IntoPowerSet;
use lazy_static::*;
use pest::error::Error;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use pest_derive::*;
use std::collections::HashSet;
use strum_macros::*;

pub mod reverse_polish;

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
      Self::Negated(e) if e.is_variable() || e.is_negated() => write!(f, "~{}", e),
      Self::Negated(e) => write!(f, "~({})", e),
      e @ Self::And(_, _) => write!(
        f,
        "({})",
        e.flatten_and()
          .iter()
          .map(|e| format!("{}", e))
          .collect::<Vec<_>>()
          .join(" ^ ")
      ),
      Self::Or(e1, e2) => write!(f, "({} v {})", e1, e2),
      Self::Conditional(e1, e2) => write!(f, "({} -> {})", e1, e2),
      Self::Biconditional(e1, e2) => write!(f, "({} <-> {})", e1, e2),
    }
  }
}

lazy_static! {
  static ref PREC_CLIMBER: PrecClimber<Rule> = {
    PrecClimber::new(vec![
      Operator::new(Rule::biconditional, Assoc::Left),
      Operator::new(Rule::conditional, Assoc::Right),
      Operator::new(Rule::or, Assoc::Left),
      Operator::new(Rule::and, Assoc::Left),
    ])
  };
}
// TODO: consider `grammar_inline`
#[derive(Parser)]
#[grammar = "classical/classical.pest"]
pub struct ExpressionParser;

impl ExpressionParser {
  pub fn parse_expression(s: &str) -> Result<Expression, Error<Rule>> {
    let expr = Self::parse(Rule::expr, s)?.next().unwrap();
    use pest::iterators::Pair;
    use pest::iterators::Pairs;

    // println!("{:#?}", expr);
    fn with_prec(pairs: Pairs<Rule>) -> Expression {
      PREC_CLIMBER.climb(
        pairs,
        |pair: Pair<Rule>| match pair.as_rule() {
          Rule::expr => with_prec(pair.into_inner()),
          Rule::term => with_prec(pair.into_inner()),
          Rule::negated => Expression::Negated(Box::new(with_prec(pair.into_inner()))),
          Rule::literal => {
            let c = pair.as_str().chars().next().unwrap();
            Expression::Variable(Variable(c))
          }
          Rule::grouped => with_prec(pair.into_inner()),
          _ => {
            // println!("pair {:#?}", pair);
            unreachable!()
          }
        },
        |lhs: Expression, op: Pair<Rule>, rhs: Expression| match op.as_rule() {
          Rule::and => Expression::And(Box::new(lhs), Box::new(rhs)),
          Rule::or => Expression::Or(Box::new(lhs), Box::new(rhs)),
          Rule::conditional => Expression::Conditional(Box::new(lhs), Box::new(rhs)),
          Rule::biconditional => Expression::Biconditional(Box::new(lhs), Box::new(rhs)),
          _ => {
            // println!("op {:#?}", op);
            unreachable!()
          }
        },
      )
    }

    Ok(with_prec(expr.into_inner()))
  }
}

#[cfg(test)]
mod test {
  use super::*;
  #[test]
  fn test_parens_or_no() {
    assert_eq!(
      ExpressionParser::parse_expression("(p)").unwrap(),
      ExpressionParser::parse_expression("p").unwrap()
    );
    assert_eq!(
      ExpressionParser::parse_expression("(p ^ q)").unwrap(),
      ExpressionParser::parse_expression("p ^ q").unwrap()
    );
  }

  #[test]
  fn test_order_of_operations() {
    use Expression::*;
    assert_eq!(
      ExpressionParser::parse_expression("a ^ b v c -> d <-> e").unwrap(),
      Biconditional(
        Box::new(Conditional(
          Box::new(Or(
            Box::new(And(
              Box::new(Variable(super::Variable('a'))),
              Box::new(Variable(super::Variable('b')))
            )),
            Box::new(Variable(super::Variable('c')))
          )),
          Box::new(Variable(super::Variable('d')))
        )),
        Box::new(Variable(super::Variable('e')))
      ),
      "correct precedence"
    );
    assert_eq!(
      ExpressionParser::parse_expression("a <-> b -> c v d ^ e").unwrap(),
      Biconditional(
        Box::new(Variable(super::Variable('a'))),
        Box::new(Conditional(
          Box::new(Variable(super::Variable('b'))),
          Box::new(Or(
            Box::new(Variable(super::Variable('c'))),
            Box::new(And(
              Box::new(Variable(super::Variable('d'))),
              Box::new(Variable(super::Variable('e')))
            )),
          )),
        )),
      ),
      "correct precedence"
    );

    assert_eq!(
      ExpressionParser::parse_expression("(p -> (q -> r))").unwrap(),
      ExpressionParser::parse_expression("p -> q -> r").unwrap(),
      "conditional should be right-associative"
    );
  }

  #[test]
  fn test_grouping_overrides_ooo() {}

  #[test]
  fn test_some_tautologies() {
    assert!(ExpressionParser::parse_expression("p ^ q -> p")
      .unwrap()
      .is_tautology());
    assert!(ExpressionParser::parse_expression("p v ~p")
      .unwrap()
      .is_tautology());

    // TODO: Need some that are not tautologies
    // TODO: need to parse negations correctly
    let expr = ExpressionParser::parse_expression("~(p ^ q) <-> (~p v ~q)").unwrap();
    println!("expr {:#?}", expr);
    assert!(expr.is_tautology())
  }
}
