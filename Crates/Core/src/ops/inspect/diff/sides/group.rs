//! Putting the targets that have the same change together.

use super::rows::side_file;
use super::{ChangeGroup, RawFile};
use crate::scan::Target;

/// Groups the targets of one skill whose files are exactly alike (the same files, with the same old and
/// new text). The rows are made once per group, not once per target.
pub(super) fn groups(entries: Vec<(Target, Vec<RawFile>)>) -> Vec<ChangeGroup> {
    let mut found: Vec<(String, Vec<RawFile>, Vec<Target>)> = Vec::new();
    for (target, files) in entries {
        // A linear search: a scan has dozens of groups at most, and most targets match the first one.
        match found
            .iter_mut()
            .find(|(skill, same, _)| *skill == target.skill && *same == files)
        {
            Some((_, _, targets)) => targets.push(target),
            None => found.push((target.skill.clone(), files, vec![target])),
        }
    }
    // The skills in name order; of one skill the change most projects have comes first, since that is the
    // usual one and the rest are what makes a project different.
    found.sort_by(|a, b| a.0.cmp(&b.0).then(b.2.len().cmp(&a.2.len())));
    found
        .into_iter()
        .map(|(skill, files, targets)| ChangeGroup {
            skill,
            targets,
            files: files.into_iter().map(side_file).collect(),
        })
        .collect()
}
