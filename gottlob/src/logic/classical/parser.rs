use lazy_static::*;
use pest::error::Error;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use pest_derive::*;
use super::{Expression, Variable};

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
#[grammar = "logic/classical/grammar.pest"]
pub struct ClassicalParser;

impl ClassicalParser {
  pub fn parse_expression(s: &str) -> Result<Expression, Error<Rule>> {
    let expr = Self::parse(Rule::whole_expr, s)?.next().unwrap();
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
      ClassicalParser::parse_expression("(p)").unwrap(),
      ClassicalParser::parse_expression("p").unwrap()
    );
    assert_eq!(
      ClassicalParser::parse_expression("(p ^ q)").unwrap(),
      ClassicalParser::parse_expression("p ^ q").unwrap()
    );
  }

  #[test]
  fn test_order_of_operations() {
    use Expression::*;
    assert_eq!(
      ClassicalParser::parse_expression("a ^ b v c -> d <-> e").unwrap(),
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
      ClassicalParser::parse_expression("a <-> b -> c v d ^ e").unwrap(),
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
      ClassicalParser::parse_expression("(p -> (q -> r))").unwrap(),
      ClassicalParser::parse_expression("p -> q -> r").unwrap(),
      "conditional should be right-associative"
    );
  }

  #[test]
  fn test_grouping_overrides_ooo() {}

  #[test]
  fn test_some_tautologies() {
    assert!(ClassicalParser::parse_expression("p ^ q -> p")
      .unwrap()
      .is_tautology());
    assert!(ClassicalParser::parse_expression("p v ~p")
      .unwrap()
      .is_tautology());

    // TODO: Need some that are not tautologies
    // TODO: need to parse negations correctly
    let expr = ClassicalParser::parse_expression("~(p ^ q) <-> ~p v ~q").unwrap();
    println!("expr {:#?}", expr);
    assert!(expr.is_tautology())
  }
}
