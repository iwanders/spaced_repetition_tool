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


// Enhancing human learning via spaced repetition optimization
// https://www.pnas.org/doi/full/10.1073/pnas.1815156116
// https://github.com/Networks-Learning/memorize/
pub mod memorize {
    /*
        Summarized from the paper;

        Exponential forgetting curve model of memory
        Recall m(t) of item after review is 1.0, m(t) decays with forgetting rate n(t)

            m(t) = exp( -n(t) * (t - t_last_review) )

        Estimated forgetting rate changes based on correct or failed recall.
            Correct recall:
                n(t + dt) = (1 - alpha) * n(t)
            Failed recall:
                n(t + dt) = (1 + beta) * n(t)

        The reviewing intensity is then given by:
            u(t) = q^(-0.5) * (1 - m(t))

        Perfect recall m(t) = 1 results in rate of review of 0.
        Forgotten recall m(t) = 0 results in review events at rate q^(-0.5) per unit time.

        In the paper, this is then transformed from rate-of-reviewing to next-review-time with a
        thinning algorithm.
    */


    /// n_t is the forgetting rate, t is the current time, t_last_review is previous test.
    pub fn recall(n_t: f64, t: f64, t_last_review: f64) -> f64 {
        assert!(t_last_review <= t, "t should be greater than t_last_review");
        ((-n_t) * ( t - t_last_review )).exp() 
    }

    pub fn review_intensity(q: f64, recall: f64) -> f64 {
        q.powf(-0.5) * (1.0 - recall)
    }

    // Time transform.
    pub fn intensity(n_t: f64, t: f64, q: f64) -> f64 {
        (1.0 / q.sqrt()) * (1.0 - (-n_t * t).exp())
    }

    /// Calculate the next time to review based on the intensity.
    pub fn next_review_time(n_t: f64, q: f64, t_max: f64) -> Option<f64> {
        use rand_distr::Distribution;
        let max_interval = 1.0 / q.sqrt();
        let exp = rand_distr::Exp::new(1.0 / max_interval).unwrap();
        let mut t = 0.0;
        loop {
            let t_ = exp.sample(&mut rand::thread_rng());
            if t_ + t > t_max {
                return None; // Beyond max scheduling interval.
            }
            t = t + t_;
            let proposed_interval = intensity(n_t, t, q);
            if rand::random::<f64>() < (proposed_interval / t_max) {
                return Some(t);
            }
        }
    }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples()
    {
        // We want time in seconds... but that makes n(t) really small.
        // First day recall: 0.64
        // Second day recall: 0.42
        // Third day recall: 0.27
        for t in 0..10 {
            let t = (t as f64) * (3600.0 * 24.0);
            let n_t = 5e-6;
            let recallt = recall(n_t, t, 0.0);
            let q = 1.0;
            let t_max = 5.0 * 3600.0 * 24.0;
            let reviewt = review_intensity(q, recallt);
            // let review_next = next_review_time(n_t, q, t_max);
            println!("Recall {t: >10}: {recallt}");
            println!("           review: {reviewt}");
            // println!("                 : {review_next:?}");
        }
    }
}

}