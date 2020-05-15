use pest::error::Error;
use pest::Parser;
use pest_derive::*;
use std::collections::HashSet;

pub mod reverse_polish;
// TODO: consider `grammar_inline`
#[derive(Parser)]
#[grammar = "classical/classical.pest"]
pub struct ExpressionParser;

impl ExpressionParser {
  pub fn parse_expression(s: &str) -> Result<Expression, Error<Rule>> {
    let expr = Self::parse(Rule::expression, s)?.next().unwrap();
    use pest::iterators::Pair;

    fn parse_value(p: Pair<Rule>) -> Expression {
      match p.as_rule() {
        Rule::literal => {
          let c = p.as_str().chars().next().unwrap();
          Expression::Variable(Variable(c))
        }
        Rule::negated => Expression::Negated(Box::new(parse_value(
          p.into_inner().next().expect("Negated has inner"),
        ))),
        Rule::and => {
          let (left, right) = parse_two_inner(p);
          Expression::And(left, right)
        }
        Rule::or => {
          let (left, right) = parse_two_inner(p);
          Expression::Or(left, right)
        }
        Rule::conditional => {
          let (left, right) = parse_two_inner(p);
          Expression::Conditional(left, right)
        }
        Rule::biconditional => {
          let (left, right) = parse_two_inner(p);
          Expression::Biconditional(left, right)
        }
        Rule::groupable => parse_value(p.into_inner().next().unwrap()),
        Rule::grouped => parse_value(p.into_inner().next().unwrap()),
        Rule::expression => parse_value(p.into_inner().next().unwrap()),
        Rule::WHITESPACE => unreachable!(),
      }
    }

    fn parse_two_inner(p: Pair<Rule>) -> (Box<Expression>, Box<Expression>) {
      let mut inner = p.into_inner();
      let left = Box::new(parse_value(inner.next().unwrap()));
      let right = Box::new(parse_value(inner.next().unwrap()));
      (left, right)
    }

    Ok(parse_value(expr))
  }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
pub struct Variable(char);

#[derive(Debug, Eq, PartialEq, Clone)] // Maybe we don't want to derive Eq for these, but instead impl something.
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
}
