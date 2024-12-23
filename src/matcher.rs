use std::cell::{Cell, RefCell};

use crate::{parse_pattern, Pattern};

pub struct Matcher {
    captured: RefCell<Vec<String>>,
    capture_count: Cell<usize>,
}

impl Matcher {
    pub fn new() -> Self {
        Self {
            captured: RefCell::new(vec!["".to_string(); 9]),
            capture_count: Cell::new(0),
        }
    }

    pub fn match_pattern(&self, input: &str, pattern: &str) -> bool {
        let input_chars: Vec<char> = input.chars().collect();
        let pattern = parse_pattern(pattern);

        for i in 0..input_chars.len() {
            self.capture_count.set(0);
            self.captured
                .borrow_mut()
                .iter_mut()
                .for_each(|s| s.clear());
            if let Some(_) = self.match_pattern_helper(&input_chars, i, &pattern, 0) {
                return true;
            }
        }
        false
    }

    // recursive match function, return value is the position where match ends
    fn match_pattern_helper(
        &self,
        input_chars: &Vec<char>,
        input_idx: usize,
        pattern: &Vec<Pattern>,
        pattern_idx: usize,
    ) -> Option<usize> {
        if pattern_idx >= pattern.len() {
            Some(input_idx)
        } else if input_idx >= input_chars.len() {
            match pattern.get(pattern_idx) {
                Some(Pattern::LineEnd) => Some(input_idx),
                Some(Pattern::Capture(..)) if pattern_idx == pattern.len() - 1 => Some(input_idx),
                _ => None,
            }
        } else {
            match pattern[pattern_idx] {
                Pattern::Character('.') => {
                    self.match_pattern_helper(input_chars, input_idx + 1, pattern, pattern_idx + 1)
                }
                Pattern::Character(c) => {
                    if input_chars[input_idx] == c {
                        self.match_pattern_helper(
                            input_chars,
                            input_idx + 1,
                            pattern,
                            pattern_idx + 1,
                        )
                    } else {
                        None
                    }
                }
                Pattern::Digit => {
                    if input_chars[input_idx].is_digit(10) {
                        self.match_pattern_helper(
                            input_chars,
                            input_idx + 1,
                            pattern,
                            pattern_idx + 1,
                        )
                    } else {
                        None
                    }
                }
                Pattern::AlphaNumeric => {
                    if input_chars[input_idx].is_alphanumeric() {
                        self.match_pattern_helper(
                            input_chars,
                            input_idx + 1,
                            pattern,
                            pattern_idx + 1,
                        )
                    } else {
                        None
                    }
                }
                Pattern::AnyCharacter(ref set) => {
                    if set.contains(&input_chars[input_idx]) {
                        self.match_pattern_helper(
                            input_chars,
                            input_idx + 1,
                            pattern,
                            pattern_idx + 1,
                        )
                    } else {
                        None
                    }
                }
                Pattern::NoneCharacter(ref set) => {
                    if !set.contains(&input_chars[input_idx]) {
                        self.match_pattern_helper(
                            input_chars,
                            input_idx + 1,
                            pattern,
                            pattern_idx + 1,
                        )
                    } else {
                        None
                    }
                }
                // TODO: LineStart and LineEnd do not take the possibility of special characters being in the middle of the pattern into account
                Pattern::LineStart => {
                    if input_idx == 0 {
                        self.match_pattern_helper(input_chars, input_idx, pattern, pattern_idx + 1)
                    } else {
                        None
                    }
                }
                Pattern::LineEnd => None,
                Pattern::OneOrMore(ref single_pattern) => {
                    let single_pattern = vec![(**single_pattern).clone()];
                    let matcher = Matcher::new();

                    let mut curr_idx = input_idx;
                    while let Some(input_idx) =
                        matcher.match_pattern_helper(input_chars, curr_idx, &single_pattern, 0)
                    {
                        if let Some(input_idx) = self.match_pattern_helper(
                            input_chars,
                            input_idx,
                            pattern,
                            pattern_idx + 1,
                        ) {
                            return Some(input_idx);
                        }
                        curr_idx += 1;
                    }

                    None
                }
                Pattern::ZeroOrOne(ref single_pattern) => {
                    let single_pattern = vec![(**single_pattern).clone()];
                    let matcher = Matcher::new();

                    // try matching once
                    if let Some(input_idx) =
                        matcher.match_pattern_helper(input_chars, input_idx, &single_pattern, 0)
                    {
                        if let Some(input_idx) = self.match_pattern_helper(
                            input_chars,
                            input_idx,
                            pattern,
                            pattern_idx + 1,
                        ) {
                            return Some(input_idx);
                        }
                    }

                    // match zero times
                    self.match_pattern_helper(input_chars, input_idx, pattern, pattern_idx + 1)
                }
                Pattern::Group(ref group) => {
                    self.capture_count.set(self.capture_count.get() + 1);
                    for single_pattern in group {
                        let mut new_pattern = (*single_pattern).clone();
                        new_pattern.push(Pattern::Capture(input_idx, self.capture_count.get() - 1));
                        new_pattern.extend_from_slice(&pattern[pattern_idx + 1..]);
                        if let Some(input_idx) =
                            self.match_pattern_helper(input_chars, input_idx, &new_pattern, 0)
                        {
                            return Some(input_idx);
                        }
                    }
                    None
                }
                Pattern::Backreference(idx) => {
                    let matcher = Matcher::new();
                    let backreference_pattern = parse_pattern(&self.captured.borrow()[idx]);
                    if let Some(input_idx) = matcher.match_pattern_helper(
                        input_chars,
                        input_idx,
                        &backreference_pattern,
                        0,
                    ) {
                        self.match_pattern_helper(input_chars, input_idx, pattern, pattern_idx + 1)
                    } else {
                        None
                    }
                }
                Pattern::Capture(start_idx, captured_idx) => {
                    self.captured.borrow_mut()[captured_idx] =
                        String::from_iter(&input_chars[start_idx..input_idx]);
                    let ret =
                        self.match_pattern_helper(input_chars, input_idx, pattern, pattern_idx + 1);
                    if ret.is_none() {
                        self.captured.borrow_mut()[captured_idx] = "".to_string();
                    }
                    ret
                }
            }
        }
    }
}
