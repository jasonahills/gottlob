#![recursion_limit="256"]

use custom_debug::*;
use wasm_bindgen::prelude::*;
use yew::web_sys::HtmlInputElement;
use yew::prelude::*;
use log::*;
use gottlob::logic::Logic;
use gottlob::logic::classical::ClassicalLogic;
use gottlob::logic::modal::ModalLogic;

#[derive(CustomDebug)]
struct Repl {
  #[debug(skip)]
  link: ComponentLink<Self>,
  terminal: Vec<String>,
  // TODO: able to arrow through history
  history: Vec<String>,
  command_line: String,
  input_ref: NodeRef,
}

enum Msg {
  CommandUpdate(String),
  CommandExecute,
  TerminalClicked,
}

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

impl Component for Repl {
  type Message = Msg;
  type Properties = ();
  fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
    // TODO: move to App when this component gets separated out.
    wasm_logger::init(wasm_logger::Config::default());
    Self { 
      link, 
      command_line: "".to_owned(),
      terminal: vec!["Welcome to Gottlob!
      
Enter sentences from modal logic to see if they are tautologies.

  - sentence variables: lowercase letters such as 'p' and 'q' (but not 'v', since we use it elsewhere)
  - not: '~'
  - and: '^'
  - or: 'v'
  - conditional: '->'
  - biconditional: '<->'
  - possible: `<>`
  - necessary: `[]`
  
Try a sentence like '~(p ^ q) <-> ~p v ~q'!".to_owned()],
      history: Vec::new(),
      input_ref: NodeRef::default(),
    }
  }

  fn update(&mut self, msg: Self::Message) -> ShouldRender {
    match msg {
      Msg::CommandUpdate(s) => self.command_line = s,
      Msg::CommandExecute => {
        // TODO: do the parsing and execution outside of this "thread"
        // TODO: allow for things other than just evaluating modal logic stuff.
        let command = std::mem::replace(&mut self.command_line, "".to_owned());
        self.terminal.push(format!("gottlob> {}", command));
        if &command == "" { // nothing in the command line? give us a new line!
          return true;
        }
        // TODO: better error display.
        let classical_rs = ClassicalLogic.is_valid_theorem(&command).unwrap_or_else(|e| (format!("invalid classical parse {:?}", e), false));
        let modal_rs = ModalLogic.is_valid_theorem(&command).unwrap_or_else(|e| (format!("invalid modal parse {:?}",e ), false));
        // TODO: ensure that this prompt will match the one the user uses.
        self.terminal.push(classical_rs.0);
        self.terminal.push(modal_rs.0);
        self.history.push(command);
      },
      Msg::TerminalClicked => {
        if let Some(input) = self.input_ref.cast::<HtmlInputElement>() {
          input.focus().expect("should always be able to focus on input");
        }
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
        <header>{"Gottlob "} {VERSION}</header>
        <section class="terminal" onclick=self.link.callback(|_| Msg::TerminalClicked)>
          <div>
            <pre><code>{ self.terminal.join("\n") }</code></pre>
          </div>
          <form 
            onsubmit=self.link.callback(|e: Event| {
              e.prevent_default();
              Msg::CommandExecute
            })
          >
            <span>{"gottlob> "}</span>
            <input 
              ref=self.input_ref.clone()
              value=self.command_line 
              oninput=self.link.callback(|i: InputData| Msg::CommandUpdate(i.value))
            />
            <button 
              kind="submit" 
              class="hidden"
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
