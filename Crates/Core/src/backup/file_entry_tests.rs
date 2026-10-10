//! Notes about a file next to the notes about skill folders: old notes must stay readable.

use super::*;

#[test]
fn a_note_written_before_files_had_entries_is_about_a_skill() {
    let old = r#"{"kind":"sync","project":"/p","skill":"coding","change":"updated"}"#;

    let entry: Entry = serde_json::from_str(old).unwrap();

    assert_eq!(entry.subject, Subject::Skill);
    assert_eq!(entry.skill, "coding");
}

#[test]
fn a_skill_note_is_written_without_a_subject_so_older_builds_read_it_unchanged() {
    let entry = Entry {
        kind: RunKind::Sync,
        project: "/p".into(),
        skill: "coding".to_string(),
        change: Change::Updated,
        subject: Subject::Skill,
    };

    let text = serde_json::to_string(&entry).unwrap();

    assert!(!text.contains("subject"), "{text}");
}

#[test]
fn a_file_note_says_so_and_round_trips() {
    let entry = Entry {
        kind: RunKind::Add,
        project: "/p".into(),
        skill: "AGENTS.md".to_string(),
        change: Change::Created,
        subject: Subject::Agents,
    };

    let text = serde_json::to_string(&entry).unwrap();

    assert!(text.contains(r#""subject":"agents""#), "{text}");
    assert_eq!(serde_json::from_str::<Entry>(&text).unwrap(), entry);
}
