use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Closed release CLI parse failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbeCliErrorKindV1 {
    /// No artifact output path was supplied.
    MissingArtifactPath,
    /// More than one artifact output path was supplied.
    ExtraArgument,
    /// The supplied value does not name a file path.
    InvalidArtifactPath,
}

/// Validated command-line request for one interactive probe run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProbeCliV1 {
    artifact_path: PathBuf,
}

impl ProbeCliV1 {
    /// Returns the caller-selected artifact output path.
    #[must_use]
    pub fn artifact_path(&self) -> &Path {
        &self.artifact_path
    }
}

/// Parses an executable name followed by exactly one artifact output path.
#[must_use = "CLI parse failures must be handled"]
pub fn parse_probe_cli_v1(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<ProbeCliV1, ProbeCliErrorKindV1> {
    let mut arguments = arguments.into_iter();
    let _program = arguments.next();
    let path = PathBuf::from(
        arguments
            .next()
            .ok_or(ProbeCliErrorKindV1::MissingArtifactPath)?,
    );
    if arguments.next().is_some() {
        return Err(ProbeCliErrorKindV1::ExtraArgument);
    }
    if path.as_os_str().is_empty() || path.file_name().is_none() {
        return Err(ProbeCliErrorKindV1::InvalidArtifactPath);
    }
    Ok(ProbeCliV1 {
        artifact_path: path,
    })
}
