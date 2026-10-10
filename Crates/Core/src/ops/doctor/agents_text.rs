//! The text of the AGENTS.md blocks: whether the vault's own file can be used, and whether the built-in
//! text has the skills it names.

use super::{Report, Severity};
use crate::Hint;
use crate::agents::{BUILTIN_SKILLS, Origin, load_source, vault_text_path};
use crate::vault::Vault;

const CHECK: &str = "agents-text";

pub(super) fn check(report: &mut Report, vault: &Vault) {
    report.ran(CHECK);
    let path = vault_text_path(&vault.path);
    let subject = Some(path.display().to_string());
    let source = match load_source(&vault.path) {
        Ok(source) => source,
        Err(error) => {
            report.add(
                Severity::Error,
                CHECK,
                subject,
                format!("{error}; `init` and the `agents` commands stop until this is fixed"),
                error.hint(),
            );
            return;
        }
    };
    // A text of the vault's own is the owner's business; the built-in one names three skills.
    if matches!(source.origin(), Origin::Vault(_)) {
        return;
    }
    for skill in BUILTIN_SKILLS {
        if !vault.skills.contains_key(*skill) {
            report.add(
                Severity::Warning,
                CHECK,
                subject.clone(),
                format!(
                    "the built-in text names the skill `{skill}`, which the vault does not have, so `init` stops before it makes an AGENTS.md"
                ),
                Some(format!(
                    "add the skill to the vault, or put a text of your own at {}",
                    path.display()
                )),
            );
        }
    }
}
