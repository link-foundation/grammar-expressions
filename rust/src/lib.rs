use std::collections::BTreeMap;
use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    Empty,
    Literal(char),
    Any,
    Class(CharacterClass),
    Sequence(Vec<Self>),
    Choice(Vec<Self>),
    Repeat {
        expression: Box<Self>,
        min: usize,
        max: Option<usize>,
    },
    Capture {
        name: String,
        expression: Box<Self>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterClass {
    ranges: Vec<(char, char)>,
    inverted: bool,
}

impl CharacterClass {
    fn contains(&self, value: char) -> bool {
        let matched = self
            .ranges
            .iter()
            .any(|(start, end)| *start <= value && value <= *end);
        matched != self.inverted
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchResult {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub captures: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteRule {
    pub pattern: String,
    pub replacement: String,
    pub terminal: bool,
}

impl RewriteRule {
    #[must_use]
    pub fn new(pattern: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            replacement: replacement.into(),
            terminal: false,
        }
    }

    #[must_use]
    pub fn terminal(pattern: impl Into<String>, replacement: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            replacement: replacement.into(),
            terminal: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteResult {
    pub output: String,
    pub steps: usize,
    pub terminated: bool,
    pub max_steps_reached: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrammarError {
    pub message: String,
    pub position: usize,
}

impl GrammarError {
    fn new(message: impl Into<String>, position: usize) -> Self {
        Self {
            message: message.into(),
            position,
        }
    }
}

impl Display for GrammarError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at pattern position {}",
            self.message, self.position
        )
    }
}

impl Error for GrammarError {}

#[derive(Clone, Debug)]
struct State {
    pos: usize,
    captures: BTreeMap<String, String>,
}

pub fn parse(pattern: &str) -> Result<Expression, GrammarError> {
    Parser::new(pattern).parse()
}

pub fn is_match(pattern: &str, input: &str) -> Result<bool, GrammarError> {
    let expression = parse(pattern)?;
    Ok(match_full_expression(&expression, input).is_some())
}

pub fn match_full(pattern: &str, input: &str) -> Result<Option<MatchResult>, GrammarError> {
    let expression = parse(pattern)?;
    Ok(match_full_expression(&expression, input))
}

pub fn find_first(pattern: &str, input: &str) -> Result<Option<MatchResult>, GrammarError> {
    let expression = parse(pattern)?;
    Ok(find_from(&expression, input, 0))
}

pub fn find_all(pattern: &str, input: &str) -> Result<Vec<MatchResult>, GrammarError> {
    let expression = parse(pattern)?;
    Ok(find_all_expression(&expression, input))
}

pub fn replace_all(pattern: &str, replacement: &str, input: &str) -> Result<String, GrammarError> {
    let expression = parse(pattern)?;
    Ok(replace_all_expression(&expression, replacement, input))
}

pub fn rewrite(
    rules: &[RewriteRule],
    input: &str,
    max_steps: usize,
) -> Result<RewriteResult, GrammarError> {
    let compiled = rules
        .iter()
        .map(|rule| parse(&rule.pattern).map(|expression| (expression, rule)))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(rewrite_compiled(&compiled, input, max_steps))
}

#[must_use]
pub fn match_full_expression(expression: &Expression, input: &str) -> Option<MatchResult> {
    let initial = State {
        pos: 0,
        captures: BTreeMap::new(),
    };

    match_expression(expression, input, initial)
        .into_iter()
        .find(|state| state.pos == input.len())
        .map(|state| MatchResult {
            start: 0,
            end: input.len(),
            text: input.to_string(),
            captures: state.captures,
        })
}

#[must_use]
pub fn find_all_expression(expression: &Expression, input: &str) -> Vec<MatchResult> {
    let mut results = Vec::new();
    let mut cursor = 0;

    while cursor <= input.len() {
        let Some(found) = find_from(expression, input, cursor) else {
            break;
        };

        let found_end = found.end;
        let empty = found.start == found.end;
        results.push(found);

        if empty {
            if found_end == input.len() {
                break;
            }
            cursor = next_boundary(input, found_end);
        } else {
            cursor = found_end;
        }
    }

    results
}

#[must_use]
pub fn replace_all_expression(expression: &Expression, replacement: &str, input: &str) -> String {
    let mut output = String::new();
    let mut copied_until = 0;
    let mut search_start = 0;

    while search_start <= input.len() {
        let Some(found) = find_from(expression, input, search_start) else {
            break;
        };

        output.push_str(&input[copied_until..found.start]);
        output.push_str(&render_replacement(replacement, &found));
        copied_until = found.end;

        if found.start == found.end {
            if found.end == input.len() {
                search_start = input.len() + 1;
            } else {
                search_start = next_boundary(input, found.end);
            }
        } else {
            search_start = found.end;
        }
    }

    output.push_str(&input[copied_until..]);
    output
}

fn rewrite_compiled(
    rules: &[(Expression, &RewriteRule)],
    input: &str,
    max_steps: usize,
) -> RewriteResult {
    let mut output = input.to_string();
    let mut steps = 0;
    let mut terminated = false;

    while steps < max_steps {
        let mut applied = false;

        for (expression, rule) in rules {
            let Some(found) = find_from(expression, &output, 0) else {
                continue;
            };

            let replacement = render_replacement(&rule.replacement, &found);
            output = replace_match_once(&output, &found, &replacement);
            steps += 1;
            applied = true;

            if rule.terminal {
                terminated = true;
            }
            break;
        }

        if !applied || terminated {
            break;
        }
    }

    let max_steps_reached = !terminated
        && steps == max_steps
        && max_steps > 0
        && rules
            .iter()
            .any(|(expression, _)| find_from(expression, &output, 0).is_some());

    RewriteResult {
        output,
        steps,
        terminated,
        max_steps_reached,
    }
}

fn replace_match_once(input: &str, found: &MatchResult, replacement: &str) -> String {
    let mut output = String::new();
    output.push_str(&input[..found.start]);
    output.push_str(replacement);
    output.push_str(&input[found.end..]);
    output
}

fn find_from(expression: &Expression, input: &str, min_start: usize) -> Option<MatchResult> {
    boundary_indices(input)
        .into_iter()
        .filter(|start| *start >= min_start)
        .find_map(|start| {
            let initial = State {
                pos: start,
                captures: BTreeMap::new(),
            };

            match_expression(expression, input, initial)
                .into_iter()
                .next()
                .map(|state| MatchResult {
                    start,
                    end: state.pos,
                    text: input[start..state.pos].to_string(),
                    captures: state.captures,
                })
        })
}

fn match_expression(expression: &Expression, input: &str, state: State) -> Vec<State> {
    match expression {
        Expression::Empty => vec![state],
        Expression::Literal(expected) => match next_char(input, state.pos) {
            Some((actual, next)) if actual == *expected => vec![State { pos: next, ..state }],
            _ => Vec::new(),
        },
        Expression::Any => match next_char(input, state.pos) {
            Some((_, next)) => vec![State { pos: next, ..state }],
            None => Vec::new(),
        },
        Expression::Class(class) => match next_char(input, state.pos) {
            Some((actual, next)) if class.contains(actual) => vec![State { pos: next, ..state }],
            _ => Vec::new(),
        },
        Expression::Sequence(expressions) => {
            let mut states = vec![state];

            for item in expressions {
                let mut next_states = Vec::new();
                for current in states {
                    next_states.extend(match_expression(item, input, current));
                }

                if next_states.is_empty() {
                    return Vec::new();
                }

                states = next_states;
            }

            states
        }
        Expression::Choice(expressions) => {
            for item in expressions {
                let states = match_expression(item, input, state.clone());
                if !states.is_empty() {
                    return states;
                }
            }

            Vec::new()
        }
        Expression::Repeat {
            expression,
            min,
            max,
        } => {
            let mut states = Vec::new();
            collect_repetition(expression, input, state, 0, *min, *max, &mut states);
            states
        }
        Expression::Capture { name, expression } => {
            let start = state.pos;
            match_expression(expression, input, state)
                .into_iter()
                .map(|mut next| {
                    next.captures
                        .insert(name.clone(), input[start..next.pos].to_string());
                    next
                })
                .collect()
        }
    }
}

fn collect_repetition(
    expression: &Expression,
    input: &str,
    state: State,
    count: usize,
    min: usize,
    max: Option<usize>,
    states: &mut Vec<State>,
) {
    if max.map_or(true, |limit| count < limit) {
        for next in match_expression(expression, input, state.clone()) {
            if next.pos != state.pos {
                collect_repetition(expression, input, next, count + 1, min, max, states);
            }
        }
    }

    if count >= min {
        states.push(state);
    }
}

fn render_replacement(template: &str, found: &MatchResult) -> String {
    let mut output = String::new();
    let mut chars = template.chars().peekable();

    while let Some(current) = chars.next() {
        if current != '$' {
            output.push(current);
            continue;
        }

        match chars.peek().copied() {
            Some('$') => {
                chars.next();
                output.push('$');
            }
            Some('0') => {
                chars.next();
                output.push_str(&found.text);
            }
            Some('{') => {
                chars.next();
                let mut name = String::new();
                let mut closed = false;

                for value in chars.by_ref() {
                    if value == '}' {
                        closed = true;
                        break;
                    }
                    name.push(value);
                }

                if closed {
                    if let Some(value) = found.captures.get(&name) {
                        output.push_str(value);
                    }
                } else {
                    output.push_str("${");
                    output.push_str(&name);
                }
            }
            Some(value) if is_identifier_start(value) => {
                let mut name = String::new();

                while let Some(value) = chars.peek().copied() {
                    if !is_identifier_continue(value) {
                        break;
                    }

                    chars.next();
                    name.push(value);
                }

                if let Some(value) = found.captures.get(&name) {
                    output.push_str(value);
                }
            }
            _ => output.push('$'),
        }
    }

    output
}

fn boundary_indices(input: &str) -> Vec<usize> {
    input
        .char_indices()
        .map(|(index, _)| index)
        .chain(std::iter::once(input.len()))
        .collect()
}

fn next_boundary(input: &str, position: usize) -> usize {
    next_char(input, position).map_or(input.len(), |(_, next)| next)
}

fn next_char(input: &str, position: usize) -> Option<(char, usize)> {
    input[position..]
        .chars()
        .next()
        .map(|value| (value, position + value.len_utf8()))
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(pattern: &str) -> Self {
        Self {
            chars: pattern.chars().collect(),
            pos: 0,
        }
    }

    fn parse(mut self) -> Result<Expression, GrammarError> {
        let expression = self.parse_choice()?;
        if self.peek().is_some() {
            return Err(self.error("unexpected token"));
        }
        Ok(expression)
    }

    fn parse_choice(&mut self) -> Result<Expression, GrammarError> {
        let mut alternatives = vec![self.parse_sequence()?];

        while self.peek() == Some('|') {
            self.next();
            alternatives.push(self.parse_sequence()?);
        }

        Ok(if alternatives.len() == 1 {
            alternatives.remove(0)
        } else {
            Expression::Choice(alternatives)
        })
    }

    fn parse_sequence(&mut self) -> Result<Expression, GrammarError> {
        let mut expressions = Vec::new();

        while let Some(value) = self.peek() {
            if matches!(value, ')' | '}' | '|') {
                break;
            }

            expressions.push(self.parse_quantified()?);
        }

        Ok(match expressions.len() {
            0 => Expression::Empty,
            1 => expressions.remove(0),
            _ => Expression::Sequence(expressions),
        })
    }

    fn parse_quantified(&mut self) -> Result<Expression, GrammarError> {
        let mut expression = self.parse_atom()?;

        expression = match self.peek() {
            Some('*') => {
                self.next();
                Expression::Repeat {
                    expression: Box::new(expression),
                    min: 0,
                    max: None,
                }
            }
            Some('+') => {
                self.next();
                Expression::Repeat {
                    expression: Box::new(expression),
                    min: 1,
                    max: None,
                }
            }
            Some('?') => {
                self.next();
                Expression::Repeat {
                    expression: Box::new(expression),
                    min: 0,
                    max: Some(1),
                }
            }
            _ => expression,
        };

        Ok(expression)
    }

    fn parse_atom(&mut self) -> Result<Expression, GrammarError> {
        match self.next() {
            Some('(') => {
                let expression = self.parse_choice()?;
                self.expect(')')?;
                Ok(expression)
            }
            Some('{') => self.parse_capture(),
            Some('[') => self.parse_class(),
            Some('.') => Ok(Expression::Any),
            Some('\\') => self.parse_escape().map(Expression::Literal),
            Some(value) if matches!(value, '*' | '+' | '?' | ')' | '}' | '|') => Err(
                Self::error_at(format!("unexpected token `{value}`"), self.pos - 1),
            ),
            Some(value) => Ok(Expression::Literal(value)),
            None => Err(self.error("expected expression")),
        }
    }

    fn parse_capture(&mut self) -> Result<Expression, GrammarError> {
        let name_position = self.pos;
        let name = self.parse_identifier();
        if name.is_empty() {
            return Err(Self::error_at("expected capture name", name_position));
        }

        self.expect(':')?;
        let expression = self.parse_choice()?;
        self.expect('}')?;

        Ok(Expression::Capture {
            name,
            expression: Box::new(expression),
        })
    }

    fn parse_class(&mut self) -> Result<Expression, GrammarError> {
        let inverted = if self.peek() == Some('^') {
            self.next();
            true
        } else {
            false
        };
        let mut ranges = Vec::new();

        loop {
            match self.peek() {
                Some(']') => {
                    self.next();
                    break;
                }
                Some(_) => {
                    let start = self.parse_class_char()?;
                    if self.peek() == Some('-') && self.peek_offset(1) != Some(']') {
                        self.next();
                        let end = self.parse_class_char()?;
                        ranges.push((start, end));
                    } else {
                        ranges.push((start, start));
                    }
                }
                None => return Err(self.error("unterminated character class")),
            }
        }

        if ranges.is_empty() {
            return Err(self.error("empty character class"));
        }

        Ok(Expression::Class(CharacterClass { ranges, inverted }))
    }

    fn parse_class_char(&mut self) -> Result<char, GrammarError> {
        match self.next() {
            Some('\\') => self.parse_escape(),
            Some(value) => Ok(value),
            None => Err(self.error("expected character class item")),
        }
    }

    fn parse_escape(&mut self) -> Result<char, GrammarError> {
        match self.next() {
            Some('n') => Ok('\n'),
            Some('r') => Ok('\r'),
            Some('t') => Ok('\t'),
            Some(value) => Ok(value),
            None => Err(self.error("unterminated escape sequence")),
        }
    }

    fn parse_identifier(&mut self) -> String {
        let mut name = String::new();

        match self.peek() {
            Some(value) if is_identifier_start(value) => {
                name.push(value);
                self.next();
            }
            _ => return name,
        }

        while let Some(value) = self.peek() {
            if !is_identifier_continue(value) {
                break;
            }
            name.push(value);
            self.next();
        }

        name
    }

    fn expect(&mut self, expected: char) -> Result<(), GrammarError> {
        match self.next() {
            Some(actual) if actual == expected => Ok(()),
            Some(actual) => Err(Self::error_at(
                format!("expected `{expected}` but found `{actual}`"),
                self.pos - 1,
            )),
            None => Err(self.error(format!("expected `{expected}`"))),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_offset(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn next(&mut self) -> Option<char> {
        let value = self.peek()?;
        self.pos += 1;
        Some(value)
    }

    fn error(&self, message: impl Into<String>) -> GrammarError {
        Self::error_at(message, self.pos)
    }

    fn error_at(message: impl Into<String>, position: usize) -> GrammarError {
        GrammarError::new(message, position)
    }
}

const fn is_identifier_start(value: char) -> bool {
    value == '_' || value.is_ascii_alphabetic()
}

const fn is_identifier_continue(value: char) -> bool {
    is_identifier_start(value) || value.is_ascii_digit()
}
