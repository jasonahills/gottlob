use super::{Expression, Variable};
use pest::error::Error;
use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "classical/reverse_polish.pest"]
pub struct RpParser;

impl RpParser {
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
