// Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved
//

use std::{fmt, result};

/// A custom error type for the MeldTool.
///
// (until NoneOption stabilises, we have to use this rather silly type-ersing
// trick to be able to use ? on Option.)
pub struct ToolError(Box<dyn fmt::Debug>);

impl<T: fmt::Debug + 'static> From<T> for ToolError {
    fn from(error: T) -> Self {
        Self(Box::new(error))
    }
}

impl fmt::Display for ToolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

/// Convenience type for a Result using our Error.
pub type Result<T> = result::Result<T, ToolError>;
