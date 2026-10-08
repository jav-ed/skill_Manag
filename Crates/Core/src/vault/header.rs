//! The header of a `SKILL.md`: the two keys agents read.

use serde::Deserialize;

use super::Skill;

/// The two keys agents read from the header of `SKILL.md`.
#[derive(Debug, Default, Clone, Deserialize)]
pub struct Header {
    pub name: Option<String>,
    pub description: Option<String>,
}

/// Reads the header of the skill's `SKILL.md`, or says why it cannot be read: the file is missing, the
/// header is not there, is never closed, or is not valid YAML.
pub fn read_header(skill: &Skill) -> Result<Header, String> {
    let text = std::fs::read_to_string(skill.dir.join("SKILL.md"))
        .map_err(|e| format!("cannot read SKILL.md: {e}"))?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let mut lines = text.lines();
    if lines.next().map(str::trim_end) != Some("---") {
        return Err("SKILL.md does not start with a --- header".to_string());
    }
    let mut yaml = String::new();
    let mut closed = false;
    for line in lines {
        if line.trim_end() == "---" {
            closed = true;
            break;
        }
        yaml.push_str(line);
        yaml.push('\n');
    }
    if !closed {
        return Err("the --- header of SKILL.md is never closed".to_string());
    }
    serde_saphyr::from_str::<Option<Header>>(&yaml)
        .map(Option::unwrap_or_default)
        .map_err(|e| {
            // The parser draws the offending line; the first line of its message is the reason.
            let reason = e.to_string();
            let reason = reason
                .lines()
                .next()
                .unwrap_or_default()
                .trim_start_matches("error: ");
            format!("the header is not valid YAML: {reason}")
        })
}
