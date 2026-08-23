use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use jsonc_parser::parse_to_serde_value;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::SidecarError;

/// Inputs required to resolve one launch configuration.
#[derive(Clone, Copy, Debug)]
pub struct LaunchRequest<'a> {
    /// Source file the caller wants to debug.
    pub file_path: &'a Path,
    /// Declared project root containing `.vscode/launch.json`.
    pub working_directory: &'a Path,
    /// Exact configuration name.
    pub configuration_name: &'a str,
    /// Whether files outside the project root are allowed.
    pub allow_external_files: bool,
}

/// Parsed VS Code launch file subset.
#[derive(Debug, Deserialize)]
struct LaunchFile {
    /// Optional VS Code schema version.
    #[serde(rename = "version")]
    _version: Option<String>,
    /// Declared debug configurations.
    configurations: Vec<LaunchConfiguration>,
}

/// One preserved launch or attach configuration.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct LaunchConfiguration {
    /// User-facing configuration name.
    name: String,
    /// DAP request kind.
    request: String,
    /// Adapter type key consumed by nvim-dap.
    #[serde(rename = "type")]
    adapter_type: String,
    /// Adapter-specific fields preserved without interpretation.
    #[serde(flatten)]
    fields: BTreeMap<String, Value>,
}

/// Reads, validates, and selects one named launch configuration.
pub fn select_configuration(request: LaunchRequest<'_>) -> Result<Value, SidecarError> {
    validate_workspace_boundary(request)?;
    let launch_path = request.working_directory.join(".vscode/launch.json");
    let contents = fs::read_to_string(&launch_path).map_err(|source| SidecarError::FileRead {
        path: launch_path.clone(),
        source,
    })?;
    let launch_file: LaunchFile =
        parse_to_serde_value(&contents, &Default::default()).map_err(|error| {
            SidecarError::InvalidLaunchJson {
                message: error.to_string(),
            }
        })?;

    let available = launch_file
        .configurations
        .iter()
        .map(|configuration| configuration.name.clone())
        .collect::<Vec<_>>();
    let matches = launch_file
        .configurations
        .into_iter()
        .filter(|configuration| configuration.name == request.configuration_name)
        .collect::<Vec<_>>();
    let configuration = match matches.as_slice() {
        [] => {
            return Err(SidecarError::LaunchConfigurationMissing {
                name: request.configuration_name.to_owned(),
                available,
            });
        }
        [_one, _two, ..] => {
            return Err(SidecarError::LaunchConfigurationDuplicate {
                name: request.configuration_name.to_owned(),
            });
        }
        [one] => one,
    };
    if !matches!(configuration.request.as_str(), "launch" | "attach") {
        return Err(SidecarError::UnsupportedLaunchRequest {
            name: configuration.name.clone(),
        });
    }
    let mut selected =
        serde_json::to_value(configuration).map_err(|error| SidecarError::InvalidLaunchJson {
            message: error.to_string(),
        })?;
    expand_workspace_folder(&mut selected, request.working_directory);
    Ok(selected)
}

/// Expands VS Code's stable workspaceFolder variable in all string fields.
fn expand_workspace_folder(value: &mut Value, working_directory: &Path) {
    match value {
        Value::String(text) => {
            *text = text.replace("${workspaceFolder}", &working_directory.to_string_lossy());
        }
        Value::Array(values) => {
            for value in values {
                expand_workspace_folder(value, working_directory);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                expand_workspace_folder(value, working_directory);
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

/// Ensures the target file is within the declared workspace unless explicitly allowed.
fn validate_workspace_boundary(request: LaunchRequest<'_>) -> Result<(), SidecarError> {
    if request.allow_external_files {
        return Ok(());
    }
    let workspace = canonicalize(request.working_directory)?;
    let file = canonicalize(request.file_path)?;
    if !file.starts_with(&workspace) {
        return Err(SidecarError::FileOutsideWorkspace { file, workspace });
    }
    Ok(())
}

/// Canonicalizes one path while preserving the failing path in the error.
fn canonicalize(path: &Path) -> Result<PathBuf, SidecarError> {
    fs::canonicalize(path).map_err(|source| SidecarError::FileRead {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use tempfile::TempDir;

    use super::{LaunchRequest, select_configuration};

    /// Builds a workspace containing a source file and launch.json text.
    fn workspace(launch_json: &str) -> (TempDir, std::path::PathBuf) {
        let directory = tempfile::tempdir().expect("temporary workspace");
        fs::create_dir_all(directory.path().join(".vscode")).expect("create .vscode");
        fs::write(directory.path().join(".vscode/launch.json"), launch_json)
            .expect("write launch.json");
        let source = directory.path().join("src/main.rs");
        fs::create_dir_all(source.parent().expect("source parent")).expect("create src");
        fs::write(&source, "fn main() {}\n").expect("write source");
        (directory, source)
    }

    /// Creates a request for one fixture workspace.
    fn request<'a>(root: &'a Path, source: &'a Path, name: &'a str) -> LaunchRequest<'a> {
        LaunchRequest {
            file_path: source,
            working_directory: root,
            configuration_name: name,
            allow_external_files: false,
        }
    }

    /// Parses comments and trailing commas while preserving adapter fields.
    #[test]
    fn selects_jsonc_launch_configuration() {
        let (directory, source) = workspace(
            r#"{
              // fixture
              "version": "0.2.0",
              "configurations": [
                { "name": "Launch app", "type": "codelldb", "request": "launch", "program": "${workspaceFolder}/target/app", "cargo": {}, },
              ],
            }"#,
        );
        let selected = select_configuration(request(directory.path(), &source, "Launch app"))
            .expect("configuration must resolve");
        assert_eq!(selected["type"], "codelldb");
        assert_eq!(selected["request"], "launch");
        assert_eq!(
            selected["program"],
            format!("{}/target/app", directory.path().display())
        );
    }

    /// Reports candidates when a configuration name is missing.
    #[test]
    fn reports_missing_configuration_candidates() {
        let (directory, source) =
            workspace(r#"{"configurations":[{"name":"Attach","type":"x","request":"attach"}]}"#);
        let error = select_configuration(request(directory.path(), &source, "Missing"))
            .expect_err("missing name must fail");
        assert_eq!(error.code(), "LAUNCH_CONFIGURATION_MISSING");
        assert!(error.to_string().contains("Attach"));
    }

    /// Rejects duplicate configuration names.
    #[test]
    fn rejects_duplicate_configuration_names() {
        let (directory, source) = workspace(
            r#"{"configurations":[
              {"name":"Same","type":"x","request":"launch"},
              {"name":"Same","type":"x","request":"attach"}
            ]}"#,
        );
        let error = select_configuration(request(directory.path(), &source, "Same"))
            .expect_err("duplicate name must fail");
        assert_eq!(error.code(), "LAUNCH_CONFIGURATION_DUPLICATE");
    }

    /// Rejects source files outside the declared workspace by default.
    #[test]
    fn rejects_external_source_file() {
        let (directory, _) = workspace(r#"{"configurations":[]}"#);
        let external = tempfile::NamedTempFile::new().expect("external source");
        let error = select_configuration(request(directory.path(), external.path(), "Missing"))
            .expect_err("external source must fail first");
        assert_eq!(error.code(), "FILE_OUTSIDE_WORKSPACE");
    }

    /// Allows an external source only when the runtime policy opts in.
    #[test]
    fn allows_external_source_when_configured() {
        let (directory, _) = workspace(
            r#"{"configurations":[{"name":"Attach","type":"codelldb","request":"attach","pid":1}]}"#,
        );
        let external = tempfile::NamedTempFile::new().expect("external source");
        let selected = select_configuration(LaunchRequest {
            file_path: external.path(),
            working_directory: directory.path(),
            configuration_name: "Attach",
            allow_external_files: true,
        })
        .expect("explicit external policy");
        assert_eq!(selected["request"], "attach");
    }

    /// Rejects malformed JSONC with a stable parser error.
    #[test]
    fn rejects_invalid_jsonc() {
        let (directory, source) = workspace("{ invalid }");
        let error = select_configuration(request(directory.path(), &source, "Missing"))
            .expect_err("invalid JSONC must fail");
        assert_eq!(error.code(), "INVALID_LAUNCH_JSON");
    }
}
