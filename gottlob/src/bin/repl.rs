use gottlob::classical;
use structopt::*;

#[derive(Debug, StructOpt)]
enum Opt {
  Classical,
  ClassicalRp,
}

fn main() {
  let opt = Opt::from_args();

  // TODO: make DRY (trait time, I think!)
  // TODO: clean quit
  println!("Welcome to Gottlob REPL!  Enter expressions.");
  match opt {
    Opt::Classical => loop {
      print!("> ");
      let mut l = String::new();
      std::io::stdin()
        .read_line(&mut l)
        .expect("Failed to read line");
      let expr = match classical::ExpressionParser::parse_expression(&l) {
        Ok(expr) => expr,
        Err(e) => {
          println!("Unable to parse {:?}", e);
          continue;
        }
      };

      println!("{}", expr);
      println!("{}", expr.is_tautology());
    },
    Opt::ClassicalRp => loop {
      let mut l = String::new();
      print!("> ");
      std::io::stdin()
        .read_line(&mut l)
        .expect("Failed to read line");
      let expr = match classical::reverse_polish::RpParser::parse_expression(&l) {
        Ok(expr) => expr,
        Err(e) => {
          println!("Unable to parse {:?}", e);
          continue;
        }
      };

      println!("{}", expr);
      println!("{}", expr.is_tautology());
    },
  }
}
