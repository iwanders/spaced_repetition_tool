// Just a simple implementation for the Recorder trait.

use crate::traits::{LearnableId, MemorizerError, Record, Recorder};
use serde::{Deserialize, Serialize};

/// Recorder that only keeps all records in memory, but it is (de)serializable to easily allow
/// reuse of it in other recorders.
#[derive(Debug, Deserialize, Serialize, Default)]
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

/// A recorder that read and writes a yaml file.
#[derive(Debug)]
pub struct YamlRecorder {
    recorder: MemoryRecorder,
    filename: String,
}
impl YamlRecorder {
    /// Create a new yaml recorder, storing data in filename and if this file already exists it
    /// will load data from there when created.
    pub fn new(filename: &str) -> Result<Self, MemorizerError> {
        // Read from file if it exists, else empty.
        let recorder: MemoryRecorder;

        if std::path::Path::new(filename).exists() {
            let file = std::fs::File::open(filename).expect("file should be opened");
            let yaml: serde_yaml::Value = serde_yaml::from_reader(file)?;
            recorder = serde_yaml::from_value(yaml)?;
        } else {
            recorder = Default::default();
        }

        Ok(YamlRecorder {
            filename: filename.to_owned(),
            recorder,
        })
    }

    /// Write the data to the disk.
    pub fn write(&mut self) -> Result<(), MemorizerError> {
        // Flush to disk.
        use std::fs::OpenOptions;
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.filename)?;
        serde_yaml::to_writer(file, &self.recorder)?;
        Ok(())
    }
}

impl Recorder for YamlRecorder {
    fn store_record(&mut self, record: &Record) -> Result<(), MemorizerError> {
        self.recorder.store_record(record)?;
        self.write()
    }

    fn get_records_by_learnable(
        &self,
        learnable: LearnableId,
    ) -> Result<Vec<Record>, MemorizerError> {
        self.recorder.get_records_by_learnable(learnable)
    }
}
