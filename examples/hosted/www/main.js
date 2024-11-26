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
    this.training_rate_select = undefined;
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
      .catch((error) => console.log("Something went wrong in the request: ", error));
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
        document.getElementById("training_rate_text").textContent = "";
        document.getElementById("training_rate_answer").textContent = "";
        document.getElementById("training_rate_actual_answer").textContent = "";
        document.getElementById("training_question_answer").textContent = "";
        

        document.getElementById("training_rate_submit").classList.add("hidden");
        document.getElementById("training_retrieving").classList.remove("hidden");
        fetch(`/api/question/${this.user}/${this.deck}`)
            .then((response) => response.json())
            .then(function(data) {
              
              console.log("question", data);
              if (data == null) {
                self.training_state = TrainingState.NoMoreQuestions;
                self.redraw_training();
              } else {
                self.training_question = data;
                self.training_state = TrainingState.QuestionAsk;
                self.redraw_training();
              }
            })
            .catch((error) => console.log("Something went wrong in the request: ", error));
        break;
      case TrainingState.QuestionAsk:
        document.getElementById("training_retrieving").classList.add("hidden");
        document.getElementById("training_ask").classList.remove("hidden");
        document.getElementById("training_question_text").textContent = self.training_question.from;
        document.getElementById("training_question_answer").focus();


        break;
      case TrainingState.AnswerGiven:
        document.getElementById("training_ask").classList.add("hidden");
        document.getElementById("training_rate").classList.remove("hidden");

        document.getElementById("training_rate_text").textContent = self.training_question.from;
        document.getElementById("training_rate_answer").textContent = self.training_question.answer;
        document.getElementById("training_rate_actual_answer").textContent = self.training_question.to;
        self.training_rate_highlight(undefined);
        break;

      case TrainingState.RateSubmit:
        document.getElementById("training_rate").classList.add("hidden");
        document.getElementById("training_rate_submit").classList.remove("hidden");
        break;
      case TrainingState.NoMoreQuestions:

        document.getElementById("training_no_questions").classList.remove("hidden");
        document.getElementById("training_retrieving").classList.add("hidden");
        break;
      default:
        console.log(`Unhandled state:`, self.training_state);
    }
  }

  training_answer_submit(e) {
    let self = this;
    if (e != undefined) {
      e.preventDefault();
    }
    this.training_question.answer = document.getElementById("training_question_answer").textContent;
    console.log("submit answer: ", this.training_question.answer);
    self.training_state = TrainingState.AnswerGiven;
    self.redraw_training();
  }

  training_rate_highlight(index) {
    for (let i = 1; i <= 6; i++) {
      document.getElementById("training_rate_" + i).classList.remove("rate_select");
    }
    if (index === undefined){
      this.training_rate_select = index;
      return;
    }
    index = Math.min(Math.max(index, 1), 6);
    document.getElementById("training_rate_" + index).classList.add("rate_select");
    this.training_rate_select = index;
  }

  training_rate_right() {
    if (this.training_rate_select === undefined) {
      this.training_rate_select = 3;
    }
    this.training_rate_highlight(this.training_rate_select + 1);
  }

  training_rate_left() {
    if (this.training_rate_select === undefined) {
      this.training_rate_select = 4;
    }
    this.training_rate_highlight(this.training_rate_select - 1);
  }

  training_rate_submit_selection(e) {
    if (this.training_rate_select === undefined) {
      return;
    }
    // Writen like this to avoid rounding that we get from * 0.2.
    let score = ((this.training_rate_select - 1) * 2.0)/10.0;
    this.training_rate_submit(e, score);
  }

  training_rate_submit(e, score) {
    let self = this;
    if (e != undefined) {
      e.preventDefault();
    }

    console.log("rating answer ", this.training_question.answer, " with ", score);
    self.training_state = TrainingState.RateSubmit;
    self.redraw_training();
    let payload = {
      question: this.training_question,
      score: score,
    };
    fetch(`/api/submit_answer/${self.user}/${self.deck}`, {
        method: "POST",
        body: JSON.stringify(payload),
      })
        .then((response) => response.json())
        .then(function(data) {
          self.training_state = TrainingState.ObtainingQuestion;
          self.redraw_training();
        })
      .catch(function(error) {
        // error handling
        console.log("Something went wrong", error);
      });
    
  }

  register_inputs() {
    let self = this;

    document.getElementById("training_answer_submit").addEventListener("click", (e) => {
      self.training_answer_submit(e);
    });

    document.getElementById("training_rate_1").addEventListener("click", (e) => { self.training_rate_submit(e, 0.0); });
    document.getElementById("training_rate_2").addEventListener("click", (e) => { self.training_rate_submit(e, 0.2); });
    document.getElementById("training_rate_3").addEventListener("click", (e) => { self.training_rate_submit(e, 0.4); });
    document.getElementById("training_rate_4").addEventListener("click", (e) => { self.training_rate_submit(e, 0.6); });
    document.getElementById("training_rate_5").addEventListener("click", (e) => { self.training_rate_submit(e, 0.8); });
    document.getElementById("training_rate_6").addEventListener("click", (e) => { self.training_rate_submit(e, 1.0); });

    document.addEventListener("keydown", function(event) {
      if (event.key == "Enter" && event.ctrlKey && self.training_state == TrainingState.QuestionAsk) {
        event.preventDefault();
        self.training_answer_submit(event);
      }
      console.log(event.key);
      if (self.training_state == TrainingState.AnswerGiven) {
        if (event.key == "ArrowRight") {
          self.training_rate_right();
        }
        if (event.key == "ArrowLeft") {
          self.training_rate_left();
        }
        if (event.key == "Enter" || event.key == " ") {
          self.training_rate_submit_selection(event);
        }
        if (event.key == "1" || event.key == "2" || event.key == "3" || event.key == "4" || event.key == "5" || event.key == "6") {
          self.training_rate_highlight(Number(event.key));
        }
      }
    });
  }
}


function main_setup() {
  console.log("gogo");

  const params = new URLSearchParams(window.location.search);
  let user = params.get("user") ?? "default";

  let memorizer = new Memorizer();
  memorizer.set_user(user);
  memorizer.register_inputs();

  console.log("using user: ", user);

  if (params.has("deck")) {
    memorizer.enter_training(params.get("deck"));
  } else {
    // Deck select.
    memorizer.view_deck_select();
  }
}
