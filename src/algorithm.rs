use crate::traits::*;

/// Trivial selector that yields entries in order.
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
    fn get_question(&mut self) -> Option<Question> {
        // Just return things in order.
        let first = self.edges.remove(0);
        self.edges.push(first.clone());
        Some(first.0)
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
        ((-n_t) * (t - t_last_review)).exp()
    }

    /// Review intensity based on the recall curve and review rate.
    pub fn review_intensity(q: f64, recall: f64) -> f64 {
        q.powf(-0.5) * (1.0 - recall)
    }

    /// Time transform.
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
            // println!("t_: {t_}");
            if t_ + t > t_max {
                return None; // Beyond max scheduling interval.
            }
            t = t + t_;
            let proposed_interval = intensity(n_t, t, q);
            // println!("Proposed: {proposed_interval}");
            if rand::random::<f64>() < (proposed_interval / t_max) {
                return Some(t);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_examples() {
            // We want time in seconds... but that makes n(t) really small. Which sort of makes sense
            // as 1 day is 86400 ~ 100e3 s, which puts 5-e6 (reasonable decay for seconds) at 0.5 per
            // day.
            // First day recall: 0.64
            // Second day recall: 0.42
            // Third day recall: 0.27
            for t in 0..10 {
                let t = (t as f64) * (3600.0 * 24.0);
                let n_t = 5e-6;
                let recallt = recall(n_t, t, 0.0);
                let q = 1.0;
                let reviewt = review_intensity(q, recallt);
                println!("Recall {t: >10}: {recallt}");
                println!("           review: {reviewt}");

                /*
                // Even trying to correct for the days->seconds shift here, this seems pretty
                // odd in behaviour.
                let t_max = 10.0;
                let n_t = n_t * 86400.0;
                let q = q;
                let review_next = next_review_time(n_t, q, t_max);
                println!("                 : {review_next:?}");
                */
            }
        }
    }

    /// A selector based on the recall curve of this paper.
    pub mod recall_curve {
        use super::{recall, review_intensity};
        use crate::traits::*;
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Deserialize, Serialize)]
        pub struct RecallCurveConfig {
            n_t_alpha_correct: f64,
            n_t_beta_incorrect: f64,
            n_t_default: f64,
            q: f64,
        }

        impl Default for RecallCurveConfig {
            fn default() -> RecallCurveConfig {
                RecallCurveConfig {
                    n_t_alpha_correct: 0.05,
                    n_t_beta_incorrect: 0.2,
                    q: 1.0,
                    n_t_default: 5e-6,
                }
            }
        }

        #[derive(Debug, Clone)]
        struct QuestionInfo {
            question: Question,
            records: Vec<Record>,
            last_time: std::time::SystemTime,
            n_t: f64,
        }

        #[derive(Debug)]
        pub struct RecallCurveSelector {
            questions: Vec<QuestionInfo>,
            config: RecallCurveConfig,
        }
        impl RecallCurveSelector {
            pub fn new(config: RecallCurveConfig) -> Self {
                RecallCurveSelector {
                    questions: vec![],
                    config,
                }
            }
        }

        impl Selector for RecallCurveSelector {
            fn set_questions(&mut self, questions: &[Question], recorder: &dyn Recorder) {
                self.questions.clear();
                let now = std::time::SystemTime::now();
                for question in questions.iter() {
                    let records = recorder
                        .get_records_by_learnable(question.learnable)
                        .expect("Should return empty if unknown");

                    let mut last_time = now;
                    let mut n_t = self.config.n_t_default;
                    // Now, update n_t based on the past performance.
                    for record in records.iter() {
                        last_time = record.time;
                        if record.score == 1.0 {
                            // correct.
                            n_t = (1.0 - self.config.n_t_alpha_correct) * n_t;
                        } else {
                            // fail.
                            n_t = (1.0 + self.config.n_t_beta_incorrect) * n_t;
                        }
                    }

                    self.questions.push(QuestionInfo {
                        question: *question,
                        records,
                        last_time,
                        n_t,
                    });
                }
            }

            /// Retrieve a question to ask.
            fn get_question(&mut self) -> Option<Question> {
                // Here, we calculate the review intensity for each question on hand.
                // then we pick with a weighting.
                let now = std::time::SystemTime::now();
                use rand_distr::Distribution;
                let weights = self
                    .questions
                    .iter()
                    .map(|z| {
                        let t = now
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .expect("can this fail?");
                        let t_last = z
                            .last_time
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .expect("can this fail?");
                        let recallt = recall(z.n_t, t.as_secs_f64(), t_last.as_secs_f64());
                        let reviewt = review_intensity(self.config.q, recallt);
                        reviewt
                    })
                    .collect::<Vec<f64>>();

                // Create the distribution based on these weights and return the picked index.
                let dist = rand::distributions::WeightedIndex::new(&weights).unwrap();
                let index = dist.sample(&mut rand::thread_rng());

                Some(self.questions[index].question)
            }

            /// Store answer to a question.
            fn store_record(&mut self, record: &Record) {
                // Update the internal record for this question.
                let z = self
                    .questions
                    .iter_mut()
                    .find(|v| v.question == record.question)
                    .expect("Passed question for which we don't have a record.");
                z.records.push(*record);
                z.last_time = std::time::SystemTime::now();
                if record.score == 1.0 {
                    // correct.
                    z.n_t = (1.0 - self.config.n_t_alpha_correct) * z.n_t;
                } else {
                    // fail.
                    z.n_t = (1.0 + self.config.n_t_beta_incorrect) * z.n_t;
                }
            }
        }
    }
}

// As retrieved from https://en.wikipedia.org/wiki/SuperMemo
// https://en.wikipedia.org/w/index.php?title=SuperMemo&oldid=1087602144
pub mod supermemo2 {
    use crate::traits::*;

    #[derive(Debug, Clone)]
    struct QuestionState {
        /// The repetition number n, which is the number of times the card has been
        /// successfully recalled (meaning it was given a grade ≥ 3) in a row since the last
        /// time it was not.
        repetition_number: u64, // n
        /// The easiness factor EF, which loosely indicates how "easy" the card is (more
        /// precisely, it determines how quickly the inter-repetition interval grows).
        /// The initial value of EF is 2.5.
        easiness_factor: f64, // EF, initially 2.5
        /// The inter-repetition interval I, which is the length of time (in days) SuperMemo
        /// will wait after the previous review before asking the user to review the card again.
        inter_repetition: u64, // I, Inter repetition interval, days
    }
    /*
        Every time the user starts a review session, SuperMemo provides the user with the cards
        whose last review occurred at least I days ago. For each review, the user tries to
        recall the information and (after being shown the correct answer) specifies a grade q
        (from 0 to 5) indicating a self-evaluation the quality of their response, with each
        grade having the following meaning:

        0: "Total blackout", complete failure to recall the information.
        1: Incorrect response, but upon seeing the correct answer it felt familiar.
        2: Incorrect response, but upon seeing the correct answer it seemed easy to remember.
        3: Correct response, but required significant effort to recall.
        4: Correct response, after some hesitation.
        5: Correct response with perfect recall.

        After all scheduled reviews are complete, SuperMemo asks the user to re-review any cards
        they marked with a grade less than 4 repeatedly until they give a grade ≥ 4.
    */

    impl Default for QuestionState {
        fn default() -> Self {
            QuestionState {
                repetition_number: 0,
                easiness_factor: 2.5,
                inter_repetition: 0,
            }
        }
    }

    impl QuestionState {
        pub fn score_to_grade(score: f64) -> u64 {
            if score <= 0.0 {
                0
            } else if score <= 0.2 {
                1
            } else if score <= 0.4 {
                2
            } else if score <= 0.6 {
                3
            } else if score <= 0.8 {
                4
            } else {
                5
            }
        }

        pub fn update(&mut self, user_grade: u64) {
            assert!(user_grade <= 5);
            if user_grade >= 3 {
                // correct response
                if self.repetition_number == 1 {
                    self.inter_repetition = 1;
                } else if self.repetition_number == 1 {
                    self.inter_repetition = 6;
                } else {
                    self.inter_repetition =
                        ((self.inter_repetition as f64) * self.easiness_factor).round() as u64;
                }
                self.repetition_number += 1;
            } else {
                // incorrect response
                self.repetition_number = 0;
                self.inter_repetition = 1;
            }
            // update EF based on correctness.
            let s = (5 - user_grade) as f64;
            self.easiness_factor = self.easiness_factor + (0.1 - s * (0.08 + s * 0.02));
            if self.easiness_factor < 1.3 {
                self.easiness_factor = 1.3;
            }
        }

        pub fn inter_repetition(&self) -> u64 {
            self.inter_repetition
        }
    }

    #[derive(Debug, Clone)]
    struct QuestionInfo {
        /// The question itself.
        question: Question,

        /// Last time this question was asked.
        last_time: std::time::SystemTime,

        /// If question was asked and last grade is less than 4.
        pending_re_review: bool,

        /// The internal state for this question.
        state: QuestionState,
    }

    /// A selector that implements the SuperMemo2 algorithm.
    #[derive(Debug)]
    pub struct SuperMemo2Selector {
        questions: Vec<QuestionInfo>,
    }
    impl SuperMemo2Selector {
        pub fn new() -> Self {
            SuperMemo2Selector { questions: vec![] }
        }
    }

    impl Selector for SuperMemo2Selector {
        fn set_questions(&mut self, questions: &[Question], recorder: &dyn Recorder) {
            self.questions.clear();
            let now = std::time::SystemTime::now();
            for question in questions.iter() {
                let records = recorder
                    .get_records_by_learnable(question.learnable)
                    .expect("Should return empty if unknown");

                // Create the state and iterate through all records to update the state.
                let mut state = QuestionState::default();
                let mut last_time = now;
                for record in records.iter() {
                    last_time = record.time;
                    let grade = QuestionState::score_to_grade(record.score);
                    state.update(grade);
                }

                self.questions.push(QuestionInfo {
                    question: *question,
                    last_time,
                    state,
                    pending_re_review: false,
                });
            }
        }

        /// Retrieve a question to ask.
        fn get_question(&mut self) -> Option<Question> {
            use rand::seq::SliceRandom;
            // Stage one:
            // Every time the user starts a review session, SuperMemo provides the user with the
            // cards whose last review occurred at least I days ago.

            let now = std::time::SystemTime::now();

            // Subtract a few hours, this allows for testing at an earlier timestamp than exactly 24 hours for
            // a day, preventing the interval from 'moving forward' in time when reviewing at roughly the same
            // time each day.
            let interval_subtract = std::time::Duration::new(60 * 60 * 6, 0);
            let questions_pending_review = self
                .questions
                .iter()
                .filter(|z| {
                    let duration_since_last =
                        now.duration_since(z.last_time).expect("can this fail?");
                    // println!("duration_since_last: {:?}", duration_since_last);
                    let interval_to_days =
                        std::time::Duration::new(24 * 60 * 60 * z.state.inter_repetition(), 0);
                    let interval_to_days = interval_to_days.saturating_sub(interval_subtract);
                    duration_since_last > interval_to_days
                })
                .collect::<Vec<_>>();

            if !questions_pending_review.is_empty() {
                return Some(
                    questions_pending_review
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .question,
                );
            }

            // After all scheduled reviews are complete, SuperMemo asks the user to re-review any cards
            // they marked with a grade less than 4 repeatedly until they give a grade ≥ 4.
            // Iterate through the questions, filter by pending re-review, pick from that.
            let questions_pending_re_review = self
                .questions
                .iter()
                .filter(|z| z.pending_re_review)
                .collect::<Vec<_>>();
            if !questions_pending_re_review.is_empty() {
                return Some(
                    questions_pending_re_review
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .question,
                );
            }

            // Reached the end of the session, no more questions to ask.
            return None;
        }

        /// Store answer to a question.
        fn store_record(&mut self, record: &Record) {
            let z = self
                .questions
                .iter_mut()
                .find(|v| v.question == record.question)
                .expect("Passed question for which we don't have a record.");
            let grade = QuestionState::score_to_grade(record.score);
            z.pending_re_review = grade < 4; // mark for re-review
            z.state.update(grade);
            z.last_time = record.time;
        }
    }
}
