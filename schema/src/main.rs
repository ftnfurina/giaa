use std::{
    fs::{self, canonicalize},
    path::PathBuf,
};

use anyhow::Result;
use metadata::{ArtifactInfo, Coordinate, Rule};
use schemars::schema_for;

const VSCODE_DIR: &str = "../../../.vscode";

fn main() -> Result<()> {
    let vscode_dir = canonicalize(PathBuf::from(file!()).join(VSCODE_DIR)).unwrap();
    let artifact_info_schema = serde_json::to_string_pretty(&schema_for!(ArtifactInfo))?;
    fs::write(
        vscode_dir.join("artifact_info.schema.json"),
        artifact_info_schema,
    )?;

    let coordinate_schema = serde_json::to_string_pretty(&schema_for!(Coordinate))?;
    fs::write(vscode_dir.join("coordinate.schema.json"), coordinate_schema)?;

    let rules_schema = serde_json::to_string_pretty(&schema_for!(Vec<Rule>))?;
    fs::write(vscode_dir.join("rules.schema.json"), rules_schema)?;
    Ok(())
}
