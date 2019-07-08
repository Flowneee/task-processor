use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TaskInput {
    pub module: String,
    pub data: serde_json::Value,
}

pub(crate) type TaskResult = Result<serde_json::Value, String>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct TaskOutput {
    pub input: TaskInput,
    pub result: TaskResult,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) enum TaskState {
    New(TaskInput),
    InProgress(TaskInput),
    Done(TaskOutput),
}
