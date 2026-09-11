// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

/// Execution atom (node) for composing the tree-like structure.
#[serde_as]
#[derive(Clone, Constructor, Serialize, Deserialize, Getters, MutGetters, Debug)]
#[getset(get = "pub", get_mut = "pub")]
pub struct ExecutionStep {
    /// Unique identifier for each executor.
    /// NOTE: if the id is not set the parent node won't be able to select this step if it has multiple children.
    #[serde(default, skip_serializing_if = "should_skip_option")]
    id: Option<ExecutionStepId>,

    /// Establishes the execute handler.
    #[serde_as(as = "IfIsHumanReadable<_, JsonString>")] // Explore müsli to avoid this.
    executor: Arc<dyn AnyHttpExecutor>,

    /// All children of the current node.
    children: CheapVec<Arc<ExecutionStep>>,
}

impl<T: AnyHttpExecutor> From<Arc<T>> for ExecutionStep {
    fn from(value: Arc<T>) -> Self {
        ExecutionStep::new(None, value, CheapVec::new_const())
    }
}
