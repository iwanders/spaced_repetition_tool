use crate::algorithm::DummySelector;
use crate::traits::*;

/*
Implements the generic flow;
    Main flow;
        Load Learnable
        Load Recorder

        Filter learnable based on rules.

        Create Selector(learnables, recorder)

        Get question
        Present question
        Obtain answer

        store answer
            -> Selector
            -> Recorder

        Go to get question.
*/

pub struct Training {
    learnables: Vec<Box<dyn Learnable>>,
    recorder: Box<dyn Recorder>,
    selector: Box<dyn Selector>,
}

impl Training {
    pub fn new(learnables: Vec<Box<dyn Learnable>>, recorder: Box<dyn Recorder>) -> Self {
        // Collect questions;
        let mut questions = vec![];
        for l in learnables.iter() {
            for e in l.get_edges().iter() {
                questions.push(Question {
                    learnable: l.get_id(),
                    from: e.0.get_id(),
                    transform: e.1.get_id(),
                    to: e.2.get_id(),
                });
            }
        }
        // make selector
        let selector = DummySelector::new(&questions, &*recorder);
        Training {
            learnables,
            recorder: recorder,
            selector,
        }
    }

    pub fn question(&mut self) -> LearnableEdge {
        let q = self.selector.get_question();
        // Find the actual representation.
        let learnable = self
            .learnables
            .iter()
            .find(|z| z.get_id() == q.learnable)
            .expect("Should be present");
        learnable.get_question(&q)
    }

    pub fn answer(
        &mut self,
        question: &Question,
        answer: Box<dyn Representation>,
    ) -> Result<(), MemorizerError> {
        let learnable = self
            .learnables
            .iter()
            .find(|z| z.get_id() == question.learnable)
            .expect("Should be present");
        let representation = learnable.get_question(&question).2;
        let score = representation.get_similarity(&*answer);
        let time = std::time::SystemTime::now();
        let record = Record {
            question: *question,
            score,
            time,
        };
        self.recorder.store_record(&record)?;
        self.selector.store_record(&record);
        Ok(())
    }
}
