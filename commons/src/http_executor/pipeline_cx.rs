// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

use super::*;

/// Contains both the current context of the pipeline, passing the request and the candidate response.
#[derive(Constructor, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct PipelineCx {
    /// Incoming request from the client (which might have been modified
    /// by a pipeline's step).
    pub request: RequestCx,

    /// Outbound response (which might be modified by children execution steps).
    /// NOTE: if there is no response set, an empty 200 response will be sent back.
    pub response: Option<ResponseCx>,
}

/// Defines how the runtime should manage the execution flow
/// when the current executor finishes.
#[derive(Default, Debug)]
pub enum PipelineAction {
    /// Continue to the next established executor.
    /// NOTE: if there is none it will finish.
    /// NOTE: if there are multiple children the first one will be executed.
    Continue(Option<ExecutionStepId>),

    /// Finish execution flow and immediately return the current response.
    #[default]
    Finish,
}
