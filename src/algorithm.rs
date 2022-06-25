use crate::traits::*;

#[derive(Debug)]
pub struct DummySelector {
    edges: Vec<(Question, Vec<Score>)>,
}

impl Selector for DummySelector {
    /// Constructor, takes recorder of past event and a set of learnables.
    fn new(
        questions: &[Question],
        recorder: &dyn Recorder,
    ) -> Result<Box<dyn Selector>, MemorizerError> {
        let mut edges = vec![];
        for q in questions.iter() {
            let records = recorder.get_records_by_learnable(q.learnable)?;
            let scores = records.iter().map(|x| x.score).collect::<_>();
            edges.push((*q, scores));
        }
        Ok(Box::new(DummySelector { edges }))
    }

    /// Retrieve a question to ask.
    fn get_question(&mut self) -> Question {
        // Just return things in order.
        let first = self.edges.remove(0);
        self.edges.push(first.clone());
        first.0
    }

    /// Store answer to a question.
    fn store_record(&mut self, _record: &Record) {}
}
