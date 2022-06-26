use crate::traits::*;

#[derive(Debug)]
pub struct DummySelector {
    edges: Vec<(Question, Vec<Score>)>,
}
impl DummySelector {
    pub fn new() -> Self {
        DummySelector { edges: vec![] }
    }
}

impl Selector for DummySelector {
    fn set_questions(&mut self, questions: &[Question], recorder: &dyn Recorder) {
        self.edges.clear();
        for q in questions.iter() {
            let records = recorder
                .get_records_by_learnable(q.learnable)
                .expect("Should return empty if unknown");
            let scores = records.iter().map(|x| x.score).collect::<_>();
            self.edges.push((*q, scores));
        }
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
