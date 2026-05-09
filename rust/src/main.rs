use std::env;
use std::process::ExitCode;

use grammar_expressions::{find_first, is_match, replace_all, rewrite, RewriteRule};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(String::as_str);

    match command {
        Some("match") => {
            let (pattern, input) = parse_two_args(&args)?;
            println!(
                "{}",
                is_match(pattern, input).map_err(|error| error.to_string())?
            );
        }
        Some("find") => {
            let (pattern, input) = parse_two_args(&args)?;
            if let Some(found) = find_first(pattern, input).map_err(|error| error.to_string())? {
                println!("{}..{} {}", found.start, found.end, found.text);
                for (name, value) in found.captures {
                    println!("{name}={value}");
                }
            }
        }
        Some("replace") => {
            let pattern = args
                .get(2)
                .ok_or_else(|| usage("replace requires PATTERN REPLACEMENT INPUT"))?;
            let replacement = args
                .get(3)
                .ok_or_else(|| usage("replace requires PATTERN REPLACEMENT INPUT"))?;
            let input = args
                .get(4)
                .ok_or_else(|| usage("replace requires PATTERN REPLACEMENT INPUT"))?;
            println!(
                "{}",
                replace_all(pattern, replacement, input).map_err(|error| error.to_string())?
            );
        }
        Some("rewrite") => {
            let pattern = args
                .get(2)
                .ok_or_else(|| usage("rewrite requires PATTERN REPLACEMENT INPUT [MAX_STEPS]"))?;
            let replacement = args
                .get(3)
                .ok_or_else(|| usage("rewrite requires PATTERN REPLACEMENT INPUT [MAX_STEPS]"))?;
            let input = args
                .get(4)
                .ok_or_else(|| usage("rewrite requires PATTERN REPLACEMENT INPUT [MAX_STEPS]"))?;
            let max_steps = parse_max_steps(args.get(5))?;
            let rule = RewriteRule::new(pattern.as_str(), replacement.as_str());
            let result = rewrite(&[rule], input, max_steps).map_err(|error| error.to_string())?;
            println!("{}", result.output);
        }
        _ => return Err(usage("expected command: match, find, replace, or rewrite")),
    }

    Ok(())
}

fn parse_two_args(args: &[String]) -> Result<(&str, &str), String> {
    let pattern = args
        .get(2)
        .ok_or_else(|| usage("command requires PATTERN INPUT"))?;
    let input = args
        .get(3)
        .ok_or_else(|| usage("command requires PATTERN INPUT"))?;
    Ok((pattern, input))
}

fn parse_max_steps(value: Option<&String>) -> Result<usize, String> {
    value.map_or_else(
        || Ok(1000),
        |value| {
            value
                .parse()
                .map_err(|_| usage("MAX_STEPS must be a non-negative integer"))
        },
    )
}

fn usage(message: &str) -> String {
    format!(
        "{message}\n\nUsage:\n  grammar-expressions match PATTERN INPUT\n  grammar-expressions find PATTERN INPUT\n  grammar-expressions replace PATTERN REPLACEMENT INPUT\n  grammar-expressions rewrite PATTERN REPLACEMENT INPUT [MAX_STEPS]"
    )
}
