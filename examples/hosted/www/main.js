"use strict";

class Memorizer {
  constructor() {
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
            link.classList.toggle("buttondiv", true);
            link.classList.toggle("stackedbutton", true);
            r.push(link);
            
            console.log("deck_name", deck_name);
          }

          document.getElementById("deck_username").textContent = self.user;
          document.getElementById("deck_button_list").replaceChildren(...r);
          document.getElementById("deck_select").classList.toggle("hidden", false);
        })
      .catch(function(error) {
        // error handling
        console.log("Something went wrong", error);
      });
  }

  enter_training() {
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
    memorizer.enter_training();
  } else {
    // Deck select.
    memorizer.view_deck_select();
  }

}

