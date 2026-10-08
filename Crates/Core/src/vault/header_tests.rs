//! The header of `SKILL.md`: what is read, and every way it can be wrong.

use super::*;
use crate::testutil::TempTree;

fn skill_with(tree: &TempTree, text: Option<&str>) -> Skill {
    if let Some(text) = text {
        tree.write("vault/s/SKILL.md", text);
    } else {
        tree.write("vault/s/other.md", "x");
    }
    let dir = tree.path().join("vault/s");
    Skill {
        name: "s".to_string(),
        group: Vec::new(),
        rel: "s".into(),
        dir,
    }
}

fn read(text: &str) -> Result<Header, String> {
    let tree = TempTree::new();
    read_header(&skill_with(&tree, Some(text)))
}

#[test]
fn name_and_description_are_read() {
    let header = read("---\nname: s\ndescription: Does things\n---\nbody\n").unwrap();

    assert_eq!(header.name.as_deref(), Some("s"));
    assert_eq!(header.description.as_deref(), Some("Does things"));
}

#[test]
fn a_byte_order_mark_and_trailing_blanks_on_the_fences_are_fine() {
    let header = read("\u{feff}---  \nname: s\n---\t\n").unwrap();

    assert_eq!(header.name.as_deref(), Some("s"));
    assert!(header.description.is_none());
}

#[test]
fn an_empty_header_is_empty_not_wrong() {
    let header = read("---\n---\nbody\n").unwrap();

    assert!(header.name.is_none() && header.description.is_none());
}

#[test]
fn every_way_a_header_can_be_wrong_is_named() {
    assert!(read("body only\n").unwrap_err().contains("does not start"));
    assert!(read("---\nname: s\n").unwrap_err().contains("never closed"));
    let invalid = read("---\ndescription: Use: carefully\n---\n").unwrap_err();
    assert!(invalid.contains("not valid YAML"), "{invalid}");
    let tree = TempTree::new();
    let missing = read_header(&skill_with(&tree, None)).unwrap_err();
    assert!(missing.contains("cannot read SKILL.md"), "{missing}");
}
