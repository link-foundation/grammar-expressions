#![allow(clippy::literal_string_with_formatting_args)]

use std::collections::BTreeMap;

use grammar_expressions::{find_all, find_first, is_match, replace_all, rewrite, RewriteRule};

#[test]
fn matches_literals_sequences_choices_and_repetition() {
    assert!(is_match("hello", "hello").unwrap());
    assert!(is_match("h(i|ello)", "hello").unwrap());
    assert!(is_match("ab+c", "abbbc").unwrap());
    assert!(is_match("colou?r", "color").unwrap());
    assert!(is_match("colou?r", "colour").unwrap());
    assert!(!is_match("ab+c", "ac").unwrap());
}

#[test]
fn captures_named_substrings_for_replacement() {
    let output = replace_all(
        "{name:[A-Za-z]+}: {value:[0-9]+}",
        "$name=$value",
        "width: 42 height: 7",
    )
    .unwrap();

    assert_eq!(output, "width=42 height=7");
}

#[test]
fn finds_unicode_text_at_character_boundaries() {
    let first = find_first("{word:[A-Za-z]+}={value:.+}", "alpha=βeta")
        .unwrap()
        .expect("expected a match");
    let captures: BTreeMap<_, _> = first.captures.into_iter().collect();

    assert_eq!(first.text, "alpha=βeta");
    assert_eq!(captures.get("word"), Some(&"alpha".to_string()));
    assert_eq!(captures.get("value"), Some(&"βeta".to_string()));
}

#[test]
fn find_all_advances_after_zero_width_matches() {
    let matches = find_all("a?", "baa").unwrap();
    let texts: Vec<_> = matches.into_iter().map(|found| found.text).collect();

    assert_eq!(texts, vec!["", "a", "a", ""]);
}

#[test]
fn rewrite_applies_ordered_rules_until_terminal_rule_matches() {
    let rules = vec![
        RewriteRule::new("{left:a}{right:b}", "$right$left"),
        RewriteRule::terminal("ba", "done"),
    ];

    let result = rewrite(&rules, "ab", 10).unwrap();

    assert_eq!(result.output, "done");
    assert_eq!(result.steps, 2);
    assert!(result.terminated);
    assert!(!result.max_steps_reached);
}

#[test]
fn rewrite_reports_when_step_limit_stops_applicable_rules() {
    let rules = vec![RewriteRule::new("a", "aa")];

    let result = rewrite(&rules, "a", 3).unwrap();

    assert_eq!(result.output, "aaaa");
    assert_eq!(result.steps, 3);
    assert!(!result.terminated);
    assert!(result.max_steps_reached);
}
