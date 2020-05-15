use gottlob::classical::*;

fn main() {
  let expression = reverse_polish::RpParser::parse_expression("-> ^ p q ~ q").unwrap();
  println!("expression {:#?}", expression);
  let vars = expression.variables();
  let eval = expression.eval(&vars);
  println!("vars {:?} eval {:?}", vars, eval);

  let expression = ExpressionParser::parse_expression("((p ^ q) -> q)").unwrap();
  // println!("expression {:#?}", expression);
  println!("expression {:#?}", expression);
  let vars = expression.variables();
  let eval = expression.eval(&vars);
  println!("vars {:?} eval {:?}", vars, eval);

  let expression = ExpressionParser::parse_expression("((p ^ q) -> ~q)").unwrap();
  // println!("expression {:#?}", expression);
  println!("expression {:#?}", expression);
  let vars = expression.variables();
  let eval = expression.eval(&vars);
  println!("vars {:?} eval {:?}", vars, eval);
}
