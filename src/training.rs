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
    // learnables: Vec<Box<dyn Learnable>>,
    recorder: Box<dyn Recorder>,
    selector: Box<dyn Selector>,
    transforms: std::collections::HashMap<TransformId, std::rc::Rc<dyn Transform>>,
    representations: std::collections::HashMap<RepresentationId, std::rc::Rc<dyn Representation>>,
}

impl Training {
    pub fn new(learnables: Vec<Box<dyn Learnable>>, recorder: Box<dyn Recorder>) -> Self {
        let mut transforms: std::collections::HashMap<TransformId, std::rc::Rc<dyn Transform>> =
            Default::default();
        let mut representations: std::collections::HashMap<
            RepresentationId,
            std::rc::Rc<dyn Representation>,
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
        // make selector
        let selector = DummySelector::new(&questions, &*recorder);
        Training {
            // learnables,
            recorder: recorder,
            selector,
            transforms,
            representations,
        }
    }

    pub fn question(&mut self) -> Question {
        self.selector.get_question()
    }

    pub fn representation(&self, id: RepresentationId) -> std::rc::Rc<dyn Representation> {
        self.representations
            .get(&id)
            .expect("Requested id must exist")
            .clone()
    }

    pub fn transform(&self, id: TransformId) -> std::rc::Rc<dyn Transform> {
        self.transforms
            .get(&id)
            .expect("Requested id must exist")
            .clone()
    }

    pub fn answer(
        &mut self,
        question: &Question,
        answer: Box<dyn Representation>,
    ) -> Result<(), MemorizerError> {
        let representation = self
            .representations
            .get(&question.to)
            .expect("Should exist");
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
