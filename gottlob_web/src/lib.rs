#![recursion_limit="256"]

use custom_debug::*;
use wasm_bindgen::prelude::*;
use yew::prelude::*;
use log::*;

#[derive(CustomDebug)]
struct Repl {
  #[debug(skip)]
  link: ComponentLink<Self>,
  terminal: Vec<String>,
  // TODO: able to arrow through history
  history: Vec<String>,
  command_line: String,
}

enum Msg {
  CommandUpdate(String),
  CommandExecute,
}

impl Component for Repl {
  type Message = Msg;
  type Properties = ();
  fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
    // TODO: move to App when this component gets separated out.
    wasm_logger::init(wasm_logger::Config::default());
    Self { 
      link, 
      command_line: "".to_owned(),
      history: Vec::new(),
      terminal: Vec::new(),
    }
  }

  fn update(&mut self, msg: Self::Message) -> ShouldRender {
    match msg {
      Msg::CommandUpdate(s) => self.command_line = s,
      Msg::CommandExecute => {
        // TODO: do the parsing and execution outside of this "thread"
        // TODO: allow for things other than just evaluating classical logic stuff.
        let command = std::mem::replace(&mut self.command_line, "".to_owned());
        self.terminal.push(format!("> {}", command));
        if &command == "" { // nothing in the command line? give us a new line!
          return true;
        }
        // TODO: better error display.
        let response = gottlob::classical::ExpressionParser::parse_expression(&command).map(|expr| format!("{}", expr)).unwrap_or_else(|e| format!("{:?}", e));
        // TODO: ensure that this prompt will match the one the user uses.
        self.terminal.push(response);
        self.history.push(command);
      }
    }
    true
  }

  fn change(&mut self, _props: Self::Properties) -> ShouldRender {
    false
  }

  fn view(&self) -> Html {
    info!("{:?}", self);
    // TODO focus on input whenever clicked.
    html! {
      <div>
        <header>{"Gottlob"}</header>
        <section class="terminal">
          <div>
            <pre><code>{ self.terminal.join("\n") }</code></pre>
          </div>
          <form 
            onsubmit=self.link.callback(|e: Event| {
              e.prevent_default();
              Msg::CommandExecute
            })
          >
            <span>{"> "}</span>
            <input value=self.command_line oninput=self.link.callback(|i: InputData| Msg::CommandUpdate(i.value))/>
            <button 
              kind="submit" 
            >{"Submit"}</button>
          </form>
        </section>
        <aside></aside>
      </div>
    }
  }
}

#[wasm_bindgen(start)]
pub fn run_app() {
  App::<Repl>::new().mount_to_body();
}
