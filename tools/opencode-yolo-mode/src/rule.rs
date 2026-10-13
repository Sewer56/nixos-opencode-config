//! One V2 permission rule line and its yolo rewrite.
//!
//! - [`RuleLine::parse`]: read a rule line.
//! - [`RuleLine::mode`]: regular `ask`, yolo rewrite, or untouched.
//! - [`RuleLine::rewrite`]: switch the line to the other mode.
//!
//! ```text
//! regular: - { action: shell, resource: "git push *", effect: ask }
//! yolo:    - { action: shell, resource: "git push *", effect: deny } # yolo
//! ```

use regex::Regex;
use std::sync::LazyLock;

static JSON_RULE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^(?P<head>\s*\{\s*"action":\s*"(?P<action>[^"]*)"\s*,\s*"resource":\s*"(?P<resource>[^"]*)"\s*,\s*"effect":\s*")(?P<effect>[a-z]+)(?P<tail>"\s*\}(?:\s*,)?)(?P<marker>\s*// yolo)?\s*$"#,
    )
    .expect("valid JSON rule regex")
});
static YAML_RULE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"^(?P<head>\s*-\s*\{\s*action:\s*(?P<action>"[^"]*"|[^,"]+?)\s*,\s*resource:\s*(?P<resource>"[^"]*"|[^,"]+?)\s*,\s*effect:\s*)(?P<effect>[a-z]+)(?P<tail>\s*\})(?P<marker>\s*# yolo)?\s*$"#,
    )
    .expect("valid YAML rule regex")
});

/// Permission state of a rule line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mode {
    /// Unmarked `ask`: waits for approval.
    Regular,

    /// Marked rewrite of a former `ask`.
    Yolo,
}

/// A parsed `{ action, resource, effect }` rule line.
pub(crate) struct RuleLine<'a> {
    head: &'a str,
    action: &'a str,
    resource: &'a str,
    effect: &'a str,
    tail: &'a str,
    marked: bool,
    syntax: Syntax,
}

/// File format of a rule line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Syntax {
    /// Agent markdown frontmatter.
    Yaml,

    /// `config/opencode.json`.
    Json,
}

impl<'a> RuleLine<'a> {
    /// Parse a line without its ending; `None` if it is not a rule.
    pub(crate) fn parse(line: &'a str, syntax: Syntax) -> Option<Self> {
        let caps = syntax.regex().captures(line)?;
        let text = |name: &str| caps.name(name).map_or("", |m| m.as_str());
        Some(Self {
            head: text("head"),
            action: text("action").trim_matches('"'),
            resource: text("resource").trim_matches('"'),
            effect: text("effect"),
            tail: text("tail"),
            marked: caps.name("marker").is_some(),
            syntax,
        })
    }

    /// Current mode; `None` for rules yolo never touches.
    pub(crate) fn mode(&self) -> Option<Mode> {
        if self.marked {
            Some(Mode::Yolo)
        } else if self.effect == "ask" {
            Some(Mode::Regular)
        } else {
            None
        }
    }

    /// The line in mode `to`; `None` if already there or never toggled.
    ///
    /// Yolo effects:
    /// - `deny`: `external_directory` or `shell` asks on a specific target.
    ///   - These guard secrets and destructive git; runs fail fast.
    /// - `allow`: every other ask.
    pub(crate) fn rewrite(&self, to: Mode) -> Option<String> {
        if self.mode()? == to {
            return None;
        }
        let (effect, marker) = match to {
            Mode::Regular => ("ask", ""),
            Mode::Yolo => (self.yolo_effect(), self.syntax.marker()),
        };
        Some(format!("{}{effect}{}{marker}", self.head, self.tail))
    }

    fn yolo_effect(&self) -> &'static str {
        let guard = matches!(self.action, "external_directory" | "shell") && self.resource != "*";
        if guard { "deny" } else { "allow" }
    }
}

impl Syntax {
    fn regex(self) -> &'static Regex {
        match self {
            Syntax::Yaml => &YAML_RULE,
            Syntax::Json => &JSON_RULE,
        }
    }

    fn marker(self) -> &'static str {
        match self {
            Syntax::Yaml => " # yolo",
            Syntax::Json => " // yolo",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // ── Rewrite ─────────────────────────────────────────────────────────────

    #[rstest]
    #[case::wildcard_path(
        r#"  - { action: external_directory, resource: "*", effect: ask }"#,
        Syntax::Yaml,
        r#"  - { action: external_directory, resource: "*", effect: allow } # yolo"#
    )]
    #[case::secret_path(
        "  - { action: external_directory, resource: /home/a/hosts.yml, effect: ask }",
        Syntax::Yaml,
        "  - { action: external_directory, resource: /home/a/hosts.yml, effect: deny } # yolo"
    )]
    #[case::destructive_git(
        r#"  - { action: shell, resource: "git push *", effect: ask }"#,
        Syntax::Yaml,
        r#"  - { action: shell, resource: "git push *", effect: deny } # yolo"#
    )]
    #[case::wildcard_shell(
        r#"  - { action: shell, resource: "*", effect: ask }"#,
        Syntax::Yaml,
        r#"  - { action: shell, resource: "*", effect: allow } # yolo"#
    )]
    #[case::artifact_edit(
        r#"  - { action: edit, resource: "artifact/**", effect: ask }"#,
        Syntax::Yaml,
        r#"  - { action: edit, resource: "artifact/**", effect: allow } # yolo"#
    )]
    #[case::json_secret(
        r#"    { "action": "external_directory", "resource": "/a/*.json", "effect": "ask" },"#,
        Syntax::Json,
        r#"    { "action": "external_directory", "resource": "/a/*.json", "effect": "deny" }, // yolo"#
    )]
    #[case::json_last_entry(
        r#"    { "action": "external_directory", "resource": "/a/*.json", "effect": "ask" }"#,
        Syntax::Json,
        r#"    { "action": "external_directory", "resource": "/a/*.json", "effect": "deny" } // yolo"#
    )]
    fn rewrite_should_apply_yolo_policy_and_restore_original(
        #[case] line: &str,
        #[case] syntax: Syntax,
        #[case] expected: &str,
    ) {
        // Act
        let yolo = RuleLine::parse(line, syntax)
            .unwrap()
            .rewrite(Mode::Yolo)
            .unwrap();
        let regular = RuleLine::parse(&yolo, syntax)
            .unwrap()
            .rewrite(Mode::Regular);

        // Assert
        assert_eq!(yolo, expected);
        assert_eq!(regular.as_deref(), Some(line));
    }

    // ── Edge cases ──────────────────────────────────────────────────────────

    #[rstest]
    #[case::unmarked_allow(r#"  - { action: read, resource: "*", effect: allow }"#, Mode::Yolo)]
    #[case::already_regular(r#"  - { action: shell, resource: "*", effect: ask }"#, Mode::Regular)]
    #[case::already_yolo(
        r#"  - { action: shell, resource: "*", effect: allow } # yolo"#,
        Mode::Yolo
    )]
    fn rewrite_should_skip_when_not_toggleable(#[case] line: &str, #[case] to: Mode) {
        // Act
        let rewritten = RuleLine::parse(line, Syntax::Yaml).unwrap().rewrite(to);

        // Assert
        assert_eq!(rewritten, None);
    }
}
