#![allow(clippy::literal_string_with_formatting_args)]

use grammar_expressions::{find_first, is_match, replace_all, rewrite, RewriteRule};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    assert!(is_match("{word:[A-Za-z]+}", "grammar")?);

    let found = find_first("{key:[A-Za-z]+}: {value:[0-9]+}", "width: 42")?
        .expect("expected key/value pair");
    println!("{} -> {:?}", found.text, found.captures);

    let rewritten = replace_all(
        "{key:[A-Za-z]+}: {value:[0-9]+}",
        "$key=$value",
        "width: 42 height: 7",
    )?;
    println!("{rewritten}");

    let result = rewrite(
        &[
            RewriteRule::new("{left:a}{right:b}", "$right$left"),
            RewriteRule::terminal("ba", "done"),
        ],
        "ab",
        10,
    )?;
    println!("{}", result.output);

    Ok(())
}
