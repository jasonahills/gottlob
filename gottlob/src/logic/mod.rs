pub mod classical;
pub mod modal;

pub type ParsedSentence = String;

// TODO: proper enum with good info about the error.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseError;
pub type LogicResult = Result<(ParsedSentence, bool), ParseError>;

pub trait Logic {
  fn name(&self) -> &'static str;
  fn is_valid_theorem(&self, s: &str) -> LogicResult;
}