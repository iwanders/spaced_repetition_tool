# Spaced repetition tool

This holds a commandline tool to help memorize 'things' via [spaced repetition][spaced_repetition].
Currently only text-based representations of information are supported. It can handle standard 
'flashcard' style facts, but it is generalized to a card with any number of sides. Main use case for
 this is situations where multiple representations of the same information are to be learned, for
example a number in binary, hexadecimal as well as decimal. Each unique representation gets assigned
a unique identifier and transformation get one as well, this will allow for creation of questions
that are not explicitly defined, but instead derived from following matching representation and
transformation pairs.

Note from self (trying to use this over 2.5 years after initially writing it): Definitely a bit clunky
and clearly one of my earlier Rust projects when I thought everything needed a trait, even though
that abstraction is not used anywhere in the codebase. Use at your own risk =)

## Quick Start

The [`example_files`](/example_files/) folder contains an English to French conjucation file. Generate the learnables by running

```
cargo run --example generate_from_text -- example_files/eng_fr_verbs.txt --output example_files/eng_fr_learnables.yaml
```

Run the `tui` example 
```
cargo run --example tui -- /tmp/log.yaml example_files/eng_fr_learnables.yaml
```

## Design
The [`traits.rs`](/src/traits.rs) file describes the main concepts;
- `Representation` this represents a concept/fact to learn in a particular representation, so
  `0x13`, `19` or `10011` would be different representations of the same `Learnable` concept.
- `Transform` A particular transform that can map one representation to another one, so
  `hexadecimal to decimal`, or `decimal to binary`.
- `Selector` is what algorithms should implement.
- `Recorder` can record (and load) past performance on questions.


## Algorithms
Started with implementing the algorithm described in [Enhancing human learning via spaced repetition optimization][pnas_learning],
but only the first half of the paper is implemented at the moment, it uses the forgetting curve that
gets adjusted to weight random selection from the learnables. Currently, this `Selector` never
declares the session complete, but still shows the hardest questions the most often.

The second algorithm (currently default in the cli example) is the [SuperMemo2][supermemo]
algorithm. Which is well known and also implemented by other spaced repetition software. This
`Selector` does terminate the session if there's no questions to be asked, this is the default.

## Examples

The `hosted` example hosts a webserver with a minimalistic webinterface that one can use to practice.
It can act as a iOS 'webapp' when added to the homescreen. It supports multiple users and each user
can have multiple decks available to them. There is no listing of the available users, but
[this](http://localhost:8080/?user=Ivor) url would be for the user `Ivor` (case sensitive). If no 
user is provided `default` is used as a username.

The `generate_deck` example can be used to create the yaml files consumed by the hosted example or text ui example.
Two types of files are accepted, a `.txt` file where each line is one learnable, with the answer delimited by a `|` character.
Or a more elaborate yaml file. Examples for both are found in `./example_files/`.

The decks used by the hosted example need to be created by running the following to convert the simple txt format:
```
cargo run --example generate_deck -- ./example_files/hex_dec_conversions.txt --output /tmp/hex.yaml
```
and converting the yaml file to the deck with:
```
cargo run --example generate_deck -- ./example_files/learnables_elaborate.yaml  --output /tmp/elaborate.yaml
```

The hosted example can then be ran with:
```
cargo r --release  --example hosted -- ./examples/hosted/example_config.yaml
```

After which going to [http://localhost:8080/](http://localhost:8080/) should show these two decks for the `default` user.


[pnas_learning]: https://www.pnas.org/doi/full/10.1073/pnas.1815156116
[supermemo]: https://en.wikipedia.org/wiki/SuperMemo#Description_of_SM-2_algorithm
[spaced_repetition]: https://en.wikipedia.org/wiki/Spaced_repetition
