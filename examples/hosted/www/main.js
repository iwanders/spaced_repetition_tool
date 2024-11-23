"use strict";

const TrainingState = Object.freeze({
    /// Internal state while we are retrieving a question.
    ObtainingQuestion:  Symbol("ObtainingQuestion"),
    /// A question ask is setup.
    QuestionAsk: Symbol("QuestionAsk"),
    /// The true answer is displayed, score can be adjusted. Pressing enter submits this record.
    AnswerGiven: Symbol("AnswerGiven"),
    /// State happens when the selector returns an empty optional. This is a termination state.
    NoMoreQuestions: Symbol("NoMoreQuestions")
});

class Memorizer {
  constructor() {
    this.user = "default";
    this.deck = undefined;
    this.training_state = TrainingState.ObtainingQuestion;
    this.training_question = undefined;
  }

  set_user(user) {
    this.user = user;
    
  }

  view_deck_select() {
    let self = this;
    fetch(`/api/deck/${this.user}`)
        .then((response) => response.json())
        .then(function(data) {
          let r = []
          for (const deck_name of data.decks) {
            let link = document.createElement("a");
            link.text = deck_name;
            link.href = `?user=${self.user}&deck=${deck_name}`;
            link.classList.add("buttondiv");
            link.classList.add("stackedbutton");
            r.push(link);
            
            console.log("deck_name", deck_name);
          }

          document.getElementById("deck_username").textContent = self.user;
          document.getElementById("deck_button_list").replaceChildren(...r);
          document.getElementById("deck_select").classList.remove("hidden");
        })
      .catch(function(error) {
        // error handling
        console.log("Something went wrong", error);
      });
  }

  enter_training(deck) {
    document.getElementById("deck_select").classList.add("hidden");
    let self = this;
    self.deck = deck;
    console.log("deck name", deck, "state", self.training_state);
    self.training_state = TrainingState.ObtainingQuestion;
    self.redraw_training();
  }

  redraw_training() {
    let self = this;
    switch (self.training_state) {
      case TrainingState.ObtainingQuestion:
        document.getElementById("training_retrieving").classList.remove("hidden");
        fetch(`/api/question/${this.user}/${this.deck}`)
            .then((response) => response.json())
            .then(function(data) {
              
              console.log("question", data);
              self.training_question = data;
              self.training_state = TrainingState.QuestionAsk;
              self.redraw_training();
            })
          .catch(function(error) {
            // error handling
            console.log("Something went wrong", error);
          });
        break;
      case TrainingState.QuestionAsk:
        document.getElementById("training_retrieving").classList.add("hidden");
        document.getElementById("training_ask").classList.remove("hidden");
        document.getElementById("training_question_text").textContent = self.training_question.from;
        break;
      case TrainingState.AnswerGiven:
        break;
      case TrainingState.NoMoreQuestions:
        break;
      default:
        console.log(`Unhandled state:`, self.training_state);
    }
  }
}


var memorizer;
function main_setup() {
  console.log("gogo");

  const params = new URLSearchParams(window.location.search);
  let user = params.get("user") ?? "default";

  memorizer = new Memorizer();
  memorizer.set_user(user);

  console.log("using user: ", user);

  if (params.has("deck")) {
    memorizer.enter_training(params.get("deck"));
  } else {
    // Deck select.
    memorizer.view_deck_select();
  }

}

