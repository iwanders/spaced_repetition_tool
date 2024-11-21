use crate::traits::*;

/// Struct that helps maintain the standard flow of a learning session.
/// - Get Question
/// - Propose answer
/// - Rate answer
/// - Submit answer
/// Also provides accessors for transforms and representations.
pub struct Training {
    // learnables: Vec<Box<dyn Learnable>>,
    questions: Vec<Question>,
    recorder: Box<dyn Recorder>,
    selector: Box<dyn Selector>,
    transforms: std::collections::HashMap<TransformId, std::sync::Arc<dyn Transform>>,
    representations:
        std::collections::HashMap<RepresentationId, std::sync::Arc<dyn Representation>>,
}

impl Training {
    /// Load the training object with a collection of learnables, a recorder and a selector.
    /// This sets up the selector with the questions that can be asked from the learnables.
    pub fn new(
        learnables: Vec<Box<dyn Learnable>>,
        recorder: Box<dyn Recorder>,
        selector: Box<dyn Selector>,
    ) -> Self {
        let mut transforms: std::collections::HashMap<TransformId, std::sync::Arc<dyn Transform>> =
            Default::default();
        let mut representations: std::collections::HashMap<
            RepresentationId,
            std::sync::Arc<dyn Representation>,
        > = Default::default();
        // Collect questions;
        let mut questions = vec![];
        for l in learnables.iter() {
            for e in l.edges().iter() {
                transforms.insert(e.transform, l.transform(e.transform));
                representations.insert(e.from, l.representation(e.from));
                representations.insert(e.to, l.representation(e.to));
                questions.push(*e);
            }
        }

        // let mut selector = Box::new(DummySelector::new());
        let mut selector = selector;
        selector.set_questions(&questions, &*recorder);
        Training {
            // learnables,
            questions,
            recorder: recorder,
            selector,
            transforms,
            representations,
        }
    }

    /// Update the selector with the current questions.
    fn update_selector(&mut self) {
        self.selector
            .set_questions(&self.questions, &*self.recorder);
    }

    /// Set the new selector and pass the questions to it.
    pub fn set_selector(&mut self, selector: Box<dyn Selector>) {
        self.selector = selector;
        self.update_selector();
    }

    /// Obtain a new question, or if there's no more questions to ask an empty.
    pub fn question(&mut self) -> Option<Question> {
        self.selector.get_question()
    }

    /// Obtain the representation by id.
    pub fn representation(&self, id: RepresentationId) -> std::sync::Arc<dyn Representation> {
        self.representations
            .get(&id)
            .expect("Requested id must exist")
            .clone()
    }

    /// Obtain the transform by id.
    pub fn transform(&self, id: TransformId) -> std::sync::Arc<dyn Transform> {
        self.transforms
            .get(&id)
            .expect("Requested id must exist")
            .clone()
    }

    /// Get the answer to given question and obtain the proposed record for the given answer.
    /// this proposed record may be modified before it is finalized.
    pub fn get_answer(
        &mut self,
        question: &Question,
    ) -> Result<std::sync::Arc<dyn Representation>, MemorizerError> {
        self.representations
            .get(&question.to)
            .cloned()
            .ok_or(format!("could not find question").into())
    }
    /// Get the answer to given question and obtain the proposed record for the given answer.
    /// this proposed record may be modified before it is finalized.
    pub fn propose_answer(
        &mut self,
        question: &Question,
        given_answer: std::sync::Arc<dyn Representation>,
    ) -> Result<(Record, std::sync::Arc<dyn Representation>), MemorizerError> {
        let representation = self
            .representations
            .get(&question.to)
            .expect("Should exist");
        let score = representation.get_similarity(&*given_answer);
        let time = std::time::SystemTime::now();
        let record = Record {
            question: *question,
            score,
            time,
        };
        Ok((record, representation.clone()))
    }

    /// Finalize the record, storing it in the recorder and selector.
    pub fn finalize_answer(&mut self, record: Record) -> Result<(), MemorizerError> {
        assert!(record.score >= 0.0);
        assert!(record.score <= 1.0);
        self.recorder.store_record(&record)?;
        self.selector.store_record(&record);
        Ok(())
    }
}
