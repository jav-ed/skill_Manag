//! What `init` sets up around the skills (the git repository and the project scaffold): one section after
//! the skills, and one field of the `--json` document.

use serde::Serialize;

use super::style::{ERROR, HEADER, MUTED, SUCCESS};

/// The setup of a new project, as planned and as it came out.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct SetupRow {
    /// One line per step that is taken: the repository, the scaffold.
    pub(crate) steps: Vec<String>,
    /// `planned`, `done` or `failed`.
    pub(crate) state: &'static str,
    /// What the scaffold script printed.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) script: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,
}

impl SetupRow {
    pub(crate) fn planned(steps: Vec<String>) -> Self {
        Self {
            steps,
            state: "planned",
            script: Vec::new(),
            error: None,
        }
    }

    pub(crate) fn done(&self, script: &[String]) -> Self {
        Self {
            state: "done",
            // The script indents its lines for the terminal; a reader of the JSON does not need that.
            script: script.iter().map(|l| l.trim_start().to_string()).collect(),
            ..self.clone()
        }
    }

    pub(crate) fn failed(&self, error: String) -> Self {
        Self {
            state: "failed",
            error: Some(error),
            ..self.clone()
        }
    }
}

pub(crate) fn render_setup(row: &SetupRow) -> String {
    let mut out = String::new();
    putln!(out, "\n{HEADER}● Project setup{HEADER:#}");
    for step in &row.steps {
        putln!(out, "  {SUCCESS}+{SUCCESS:#} {step}");
    }
    for line in &row.script {
        putln!(out, "    {MUTED}{line}{MUTED:#}");
    }
    if let Some(error) = &row.error {
        putln!(out, "  {ERROR}✗ {error}{ERROR:#}");
    }
    out
}
