# Spaced repetition tool

This holds a commandline tool to help memorize 'things' via [spaced repetition][spaced_repetition].
Currently only text-based representations of information are supported. It can handle standard 
'flashcard' style facts, but it is generalized to a card with any number of sides. Main use case for
 this is situations where multiple representations of the same information are to be learned, for
example a number in binary, hexadecimal as well as decimal. Each unique representation gets assigned
a unique identifier and transformation get one as well, this will allow for creation of questions
that are not explicitly defined, but instead derived from following matching representation and
transformation pairs.

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
`Selector` does terminate the session if there's no questions to be asked.

[pnas_learning]: https://www.pnas.org/doi/full/10.1073/pnas.1815156116
[supermemo]: https://en.wikipedia.org/wiki/SuperMemo#Description_of_SM-2_algorithm
[spaced_repetition]: https://en.wikipedia.org/wiki/Spaced_repetition
