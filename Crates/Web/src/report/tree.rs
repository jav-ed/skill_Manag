//! The vault as a tree of groups and skills.

use std::collections::BTreeMap;

use maud::{Markup, html};
use skillmirror_core::ops::{ReportData, SkillInfo};

use super::find_text;

/// One group: the skills directly in it and the groups below it.
#[derive(Default)]
struct Node<'a> {
    skills: Vec<(usize, &'a SkillInfo)>,
    groups: BTreeMap<&'a str, Node<'a>>,
}

impl<'a> Node<'a> {
    fn insert(&mut self, path: &'a [String], number: usize, skill: &'a SkillInfo) {
        match path.split_first() {
            None => self.skills.push((number, skill)),
            Some((first, rest)) => self
                .groups
                .entry(first.as_str())
                .or_default()
                .insert(rest, number, skill),
        }
    }

    /// Everything a filter may match inside this group.
    fn find(&self) -> String {
        let mut parts: Vec<String> = self.skills.iter().map(|(_, s)| find_of(s)).collect();
        for (name, group) in &self.groups {
            parts.push((*name).to_lowercase());
            parts.push(group.find());
        }
        parts.join(" ")
    }
}

fn find_of(skill: &SkillInfo) -> String {
    find_text(&[
        &skill.name,
        &skill.group.join("/"),
        skill.description.as_deref().unwrap_or_default(),
    ])
}

pub(super) fn section(data: &ReportData) -> Markup {
    let mut root = Node::default();
    for (number, skill) in data.skills.iter().enumerate() {
        root.insert(&skill.group, number, skill);
    }
    html! {
        h2 { "Vault" }
        div.tree { (list(&root, data)) }
    }
}

fn list(node: &Node<'_>, data: &ReportData) -> Markup {
    html! {
        ul {
            @for (number, skill) in &node.skills {
                li data-find=(find_of(skill)) {
                    a href={ "#skill-" (number) } { (skill.name) }
                    @if skill.mandatory { span.badge { "mandatory" } }
                    span.badge { (installed_in(data, &skill.name)) " projects" }
                    @if let Some(description) = &skill.description {
                        span.desc { (description) }
                    } @else if let Some(problem) = &skill.header_problem {
                        span.desc.warn { (problem) }
                    }
                }
            }
            @for (name, group) in &node.groups {
                li data-find=(group.find()) {
                    details open {
                        summary { (name) "/" }
                        (list(group, data))
                    }
                }
            }
        }
    }
}

/// In how many projects the skill is installed.
fn installed_in(data: &ReportData, name: &str) -> usize {
    (0..data.projects.len())
        .filter(|index| data.cells.contains_key(&(name.to_string(), *index)))
        .count()
}
