// Waveless
// Copyright (C) 2026 Oscar Alvarez Gonzalez

use crate::*;

/// Workaround of https://github.com/serde-rs/serde/issues/1732 for `bool`
pub fn should_skip(data: &bool) -> bool {
    if !BINARY_MODE.with(|var| var.get()) {
        return *data;
    }
    false
}

/// Workaround of https://github.com/serde-rs/serde/issues/1732 for `Option<T>`
pub fn should_skip_option<T>(data: &Option<T>) -> bool {
    if !BINARY_MODE.with(|var| var.get()) {
        if data.is_none() {
            return true;
        }
    }
    false
}

/// Workaround of https://github.com/serde-rs/serde/issues/1732 for `CheapVec<T>`
pub fn should_skip_cheapvec<T>(data: &CheapVec<T>) -> bool {
    if !BINARY_MODE.with(|var| var.get()) {
        if data.is_empty() {
            return true;
        }
    }
    false
}
