use gottlob::expression::*;

fn main() {
  let expression = ExpressionParser::parse_expression("((p ^ q) -> q)").unwrap();
  // println!("expression {:#?}", expression);
  let vars = expression.variables();
  let eval = expression.eval(&vars);
  println!("vars {:?} eval {:?}", vars, eval);

  let expression = ExpressionParser::parse_expression("((p ^ q) -> ~q)").unwrap();
  // println!("expression {:#?}", expression);
  let vars = expression.variables();
  let eval = expression.eval(&vars);
  println!("vars {:?} eval {:?}", vars, eval);
}
