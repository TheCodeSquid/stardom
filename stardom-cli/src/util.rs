use std::process::ExitStatus;

use cargo_metadata::{Metadata, Package};
use thiserror::Error;

use crate::project::WatchAbortError;

pub fn is_error_silent(err: &anyhow::Error) -> bool {
    err.is::<WatchAbortError>() || err.is::<ExitStatusError>()
}

pub trait MetadataExt {
    fn primary_package(&self) -> Option<&Package>;
}
impl MetadataExt for Metadata {
    fn primary_package(&self) -> Option<&Package> {
        if self.workspace_default_members.len() == 1 {
            let id = &self.workspace_default_members[0];
            self.packages.iter().find(|p| &p.id == id)
        } else {
            None
        }
    }
}

/// Minimal stable replacement for [`exit_status_error`](https://github.com/rust-lang/rust/issues/84908)
#[derive(Clone, Copy, Error, Debug)]
#[error("process exited unsuccessfully: {0}")]
pub struct ExitStatusError(ExitStatus);

pub trait ExitStatusExt {
    fn exit_ok(&self) -> Result<(), ExitStatusError>;
}

impl ExitStatusExt for ExitStatus {
    fn exit_ok(&self) -> Result<(), ExitStatusError> {
        if self.success() {
            Ok(())
        } else {
            Err(ExitStatusError(*self))
        }
    }
}
