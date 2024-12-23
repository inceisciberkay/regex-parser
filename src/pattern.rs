use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone)]
pub enum Pattern {
    Character(char),
    Digit,
    AlphaNumeric,
    AnyCharacter(HashSet<char>),
    NoneCharacter(HashSet<char>),
    LineStart,
    LineEnd,
    OneOrMore(Box<Pattern>),
    ZeroOrOne(Box<Pattern>),
    Group(Vec<Vec<Pattern>>),
    Backreference(usize),
    Capture(usize, usize), // start_idx, captured_idx
}

pub fn parse_pattern(pattern: &str) -> Vec<Pattern> {
    let mut patterns = Vec::new();
    let mut chars = pattern.chars().enumerate().peekable();

    while let Some(c) = chars.next() {
        match c {
            (_, '\\') => {
                if let Some(next_char) = chars.next() {
                    match next_char.1 {
                        'd' => patterns.push(Pattern::Digit),
                        'w' => patterns.push(Pattern::AlphaNumeric),
                        '\\' => patterns.push(Pattern::Character('\\')),
                        '1'..'9' => patterns.push(
                            Pattern::Backreference(next_char.1.to_digit(10).unwrap() as usize - 1), // 0 based indexing
                        ),
                        _ => panic!("Unsupported escape sequence: \\{}", next_char.1),
                    }
                }
            }
            (_, '[') => {
                let mut set = HashSet::new();
                let mut closed = false;
                let mut negative = false;

                if let Some((_, '^')) = chars.peek() {
                    chars.next();
                    negative = true;
                }

                while let Some((_, c)) = chars.next() {
                    if c == ']' {
                        closed = true;
                        break;
                    }
                    set.insert(c);
                }

                if !closed {
                    panic!("[ has no matching ]");
                }

                if negative {
                    patterns.push(Pattern::NoneCharacter(set));
                } else {
                    patterns.push(Pattern::AnyCharacter(set));
                }
            }
            (l_paren_idx, '(') => {
                let mut nesting_level = 1;
                let mut r_paren_idx = l_paren_idx;

                while nesting_level > 0 {
                    if let Some((idx, c)) = chars.next() {
                        r_paren_idx = idx;
                        match c {
                            '(' => nesting_level += 1,
                            ')' => nesting_level -= 1,
                            _ => {}
                        }
                    } else {
                        panic!("( has no matching )");
                    }
                }

                let group_content = &pattern[l_paren_idx + 1..r_paren_idx];
                let mut group = Vec::new();

                // Split on top-level | only
                let mut current_start = 0;
                let mut local_nesting = 0;

                for (idx, c) in group_content.chars().enumerate() {
                    match c {
                        '(' => local_nesting += 1,
                        ')' => local_nesting -= 1,
                        '|' if local_nesting == 0 => {
                            group.push(parse_pattern(&group_content[current_start..idx]));
                            current_start = idx + 1;
                        }
                        _ => {}
                    }
                }

                // Push the last group
                if current_start < group_content.len() {
                    group.push(parse_pattern(&group_content[current_start..]));
                }

                patterns.push(Pattern::Group(group));
            }
            (_, '^') => {
                patterns.push(Pattern::LineStart);
            }
            (_, '$') => {
                patterns.push(Pattern::LineEnd);
            }
            (_, '+') => {
                if patterns.is_empty() {
                    panic!("+ must have preceding pattern");
                }

                let last_pattern = patterns.pop().unwrap();
                patterns.push(Pattern::OneOrMore(Box::new(last_pattern)));
            }
            (_, '?') => {
                if patterns.is_empty() {
                    panic!("? must have preceding pattern");
                }

                let last_pattern = patterns.pop().unwrap();
                patterns.push(Pattern::ZeroOrOne(Box::new(last_pattern)));
            }
            (_, c) => {
                patterns.push(Pattern::Character(c));
            }
        }
    }

    patterns
}
