// Just a simple implementation for the Recorder trait.

use crate::traits::{LearnableId, MemorizerError, Record, Recorder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MemoryRecorder {
    records: Vec<Record>,
}

impl MemoryRecorder {
    pub fn new() -> Self {
        MemoryRecorder { records: vec![] }
    }
    pub fn from(records: &[Record]) -> Self {
        MemoryRecorder {
            records: records.to_vec(),
        }
    }
}

impl Recorder for MemoryRecorder {
    fn store_record(&mut self, record: &Record) -> Result<(), MemorizerError> {
        self.records.push(*record);
        Ok(())
    }

    fn get_records_by_learnable(
        &self,
        learnable: LearnableId,
    ) -> Result<Vec<Record>, MemorizerError> {
        Ok(self
            .records
            .iter()
            .filter(|z| z.question.learnable == learnable)
            .map(|z| *z)
            .collect::<_>())
    }
}
