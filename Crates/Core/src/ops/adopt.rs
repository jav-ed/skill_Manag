//! Adopting: a skill folder that a project has becomes a skill of the vault. The vault gets a copy of the
//! regular files, staged and not committed; the project is not touched, and once the copy is committed
//! the project's folder is in sync with it.

use std::path::{Path, PathBuf};

use super::author::{AuthorError, Authored, check_free, group_parts};
use super::created::{Created, stage};
use crate::scan::first_link_above;
use crate::vault::Vault;

/// What adopting would do.
#[derive(Debug, Clone)]
pub struct AdoptPlan {
    pub name: String,
    /// The project's skill folder.
    pub source: PathBuf,
    /// Where it lands in the vault.
    pub dest: PathBuf,
    /// The files to copy, relative to the folder, sorted.
    pub files: Vec<PathBuf>,
    parts: Vec<String>,
}

/// Reads the project's folder and checks the vault side; writes nothing. A folder is adopted only when it
/// is a real folder holding a `SKILL.md` and nothing but regular files and folders.
pub fn plan_adopt(
    vault: &Vault,
    project: &Path,
    name: &str,
    group: &str,
) -> Result<AdoptPlan, AuthorError> {
    super::validate_name(name).map_err(|_| AuthorError::InvalidName {
        name: name.to_string(),
    })?;
    check_free(vault, name)?;
    let parts = group_parts(vault, group)?;
    let source = project.join(".agents").join("skills").join(name);
    if let Some(link) = first_link_above(&source)? {
        return Err(AuthorError::LinkedParent { path: link });
    }
    match fs_err::symlink_metadata(&source) {
        Ok(meta) if meta.is_dir() => {}
        Ok(_) => return Err(AuthorError::NotARealFolder { path: source }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(AuthorError::NotInProject {
                project: project.to_path_buf(),
                name: name.to_string(),
            });
        }
        Err(e) => return Err(e.into()),
    }
    let mut files = Vec::new();
    collect(&source, Path::new(""), &mut files)?;
    files.sort();
    if !files.iter().any(|f| f == Path::new("SKILL.md")) {
        return Err(AuthorError::NoSkillFile { path: source });
    }
    let dest: PathBuf = std::iter::once(vault.path.clone())
        .chain(parts.iter().map(PathBuf::from))
        .chain(std::iter::once(PathBuf::from(name)))
        .collect();
    if fs_err::symlink_metadata(&dest).is_ok() {
        return Err(AuthorError::Occupied { path: dest });
    }
    Ok(AdoptPlan {
        name: name.to_string(),
        source,
        dest,
        files,
        parts,
    })
}

fn collect(base: &Path, rel: &Path, files: &mut Vec<PathBuf>) -> Result<(), AuthorError> {
    for entry in fs_err::read_dir(base.join(rel))? {
        let entry = entry?;
        let path = rel.join(entry.file_name());
        let kind = entry.file_type()?;
        if entry.file_name() == ".git" {
            return Err(AuthorError::UnsupportedEntry {
                path: base.join(&path),
                what: "git repository",
            });
        }
        if kind.is_dir() {
            collect(base, &path, files)?;
        } else if kind.is_file() {
            files.push(path);
        } else {
            return Err(AuthorError::UnsupportedEntry {
                path: base.join(&path),
                what: if kind.is_symlink() {
                    "symbolic link"
                } else {
                    "special file"
                },
            });
        }
    }
    Ok(())
}

/// Copies the folder into the vault and stages it. Nothing in the vault is overwritten and nothing is
/// committed; a failure leaves the vault as it was.
pub fn adopt(vault: &Vault, plan: &AdoptPlan) -> Result<Authored, AuthorError> {
    let mut created = Created::default();
    let outcome = (|| {
        created.make_dirs(&vault.path, &plan.parts, &plan.name)?;
        for rel in &plan.files {
            let target = plan.dest.join(rel);
            if let Some(parent) = target.parent()
                && parent != plan.dest
            {
                created.make_subdirs(&plan.dest, parent)?;
            }
            let mut file = created.create_file(&target)?;
            let mut source = fs_err::File::open(plan.source.join(rel))?;
            std::io::copy(&mut source, &mut file)?;
            file.set_permissions(source.metadata()?.permissions())?;
        }
        stage(vault, &plan.dest, &plan.files)
    })();
    match outcome {
        Ok(left_out) => Ok(Authored {
            dir: plan.dest.clone(),
            files: plan.files.clone(),
            left_out,
        }),
        Err(e) => {
            created.undo();
            Err(e)
        }
    }
}
