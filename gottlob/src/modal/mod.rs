use lazy_static::*;
use pest::error::Error;
use pest::prec_climber::{Assoc, Operator, PrecClimber};
use pest::Parser;
use pest_derive::*;
use strum_macros::*;

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
#[grammar = "modal/modal.pest"]
pub struct ModalParser;

impl ModalParser {
  pub fn parse_theorem(s: &str) -> Result<Theorem, Error<Rule>> {
    let thm = Self::parse(Rule::whole_theorem, s)?.next().unwrap();
    let mut inner = thm.into_inner().collect::<Vec<_>>();
    let conclusion = Self::handle_expression_parse_tree(inner.pop().expect("always has a conclusion"));
    let proves = Self::handle_theorem_op_parse_tree(inner.pop().expect("always has an op"));
    let assumptions = inner.into_iter().map(|expr| Self::handle_expression_parse_tree(expr)).collect::<Vec<_>>();
    if proves {
      Ok(Theorem::Proves { assumptions, conclusion })
    } else {
      Ok(Theorem::DoesNotProve { assumptions, conclusion })
    }
  }

  pub fn parse_expression(s: &str) -> Result<Expression, Error<Rule>> {
    let expr = Self::parse(Rule::whole_expr, s)?.next().unwrap();
    Ok(Self::handle_expression_parse_tree(expr))
  }

  /// You _must give this the parse tree for a theorem op.
  fn handle_theorem_op_parse_tree(expr_tree: pest::iterators::Pair<Rule>) -> bool {
    match expr_tree.as_rule() {
      Rule::proves => true,
      Rule::does_not_prove => false,
      _ => panic!("parse tree must be for a theorem op")
    }
  }

  /// You _must_ give this the parse tree for an expression.
  fn handle_expression_parse_tree(expr_tree: pest::iterators::Pair<Rule>) -> Expression {
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
          Rule::necessary => Expression::Necessary(Box::new(with_prec(pair.into_inner()))),
          Rule::possible => Expression::Possible(Box::new(with_prec(pair.into_inner()))),
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

    with_prec(expr_tree.into_inner())
  }
}

#[cfg(test)]
mod test {
  use super::*;
  #[test]
  fn test_parens_or_no() {
    assert_eq!(
      ModalParser::parse_expression("(p)").unwrap(),
      ModalParser::parse_expression("p").unwrap()
    );
    assert_eq!(
      ModalParser::parse_expression("(p ^ q)").unwrap(),
      ModalParser::parse_expression("p ^ q").unwrap()
    );
  }

  #[test]
  fn test_order_of_operations() {
    use Expression::*;
    assert_eq!(
      ModalParser::parse_expression("a ^ b v c -> d <-> e").unwrap(),
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
      ModalParser::parse_expression("a <-> b -> c v d ^ e").unwrap(),
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
      ModalParser::parse_expression("(p -> (q -> r))").unwrap(),
      ModalParser::parse_expression("p -> q -> r").unwrap(),
      "conditional should be right-associative"
    );
  }

  #[test]
  fn test_grouping_overrides_ooo() {}

  #[test]
  fn test_some_tautologies() {

  }

  #[test]
  fn test_parse_theorem() {
    ModalParser::parse_theorem("p,q|-p^q").unwrap();
    ModalParser::parse_theorem("p, q |- p ^ q").unwrap();
    ModalParser::parse_theorem("[]p, []q |- [](p ^ q)").unwrap();
  }
}
