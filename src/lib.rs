/* This program assumes that the strings are valid ascii sequences. Hence u8 is used instead of char. */

pub struct Matcher<'a> {
    input: &'a [u8],
    // TODO: use &[u8] instead of String for better performance (lifetimes get difficult with recursion).
    backrefs: Vec<String>,
    backref_idx: usize,
}

impl<'a> Matcher<'a> {
    pub fn new(input: &'a str) -> Matcher<'a> {
        Matcher {
            input: input.as_bytes(),
            backrefs: vec![String::new(); 9],
            backref_idx: 0,
        }
    }

    fn clear_backrefs(&mut self) {
        self.backref_idx = 0;
        for backref in self.backrefs.iter_mut() {
            *backref = String::new();
        }
    }

    // TODO: apply interior mutability, I suppose this function does not have to mutably borrow self
    pub fn match_pattern(&mut self, pattern: &str) -> bool {
        for i in 0..self.input.len() {
            println!("new idx: {}", i);
            if let Some(idx) = self.match_starting_from(pattern, i) {
                println!("input index: {}", idx);
                return true;
            }
            self.clear_backrefs();
        }
        false
    }

    // if match successful, returns current index of input
    fn match_starting_from<'b>(&mut self, pattern: &'b str, start_idx: usize) -> Option<usize> {
        self.match_helper(pattern.as_bytes(), 0, start_idx)
    }

    fn match_helper<'b>(
        &mut self,
        pattern: &'b [u8],
        pattern_cur: usize,
        input_cur: usize,
    ) -> Option<usize> {
        println!(
            "match helper: pattern: {:?}, pattern_cur: {}, input_cur: {}",
            std::str::from_utf8(pattern).unwrap(),
            pattern_cur,
            input_cur
        );
        if let Some(c_pattern) = peek(pattern, pattern_cur) {
            match c_pattern {
                b'^' => {
                    if input_cur == 0 || peek(self.input, input_cur - 1) == Some(b'\n') {
                        self.match_helper(pattern, pattern_cur + 1, input_cur)
                    } else {
                        None
                    }
                }
                b'$' => {
                    if peek(self.input, input_cur) == None {
                        self.match_helper(pattern, pattern_cur + 1, input_cur)
                    } else if peek(self.input, input_cur) == Some(b'\n') {
                        self.match_helper(pattern, pattern_cur + 1, input_cur + 1)
                    } else {
                        None
                    }
                }
                b'\\' => self.handle_escape_character(pattern, pattern_cur, input_cur),
                b'[' => self.handle_character_group(pattern, pattern_cur, input_cur),
                b'(' => self.handle_pattern_group(pattern, pattern_cur, input_cur),
                _ => self.handle_default_character(pattern, pattern_cur, input_cur, c_pattern),
            }
        } else {
            Some(input_cur)
        }
    }

    fn handle_escape_character<'b>(
        &mut self,
        pattern: &'b [u8],
        pattern_cur: usize,
        input_cur: usize,
    ) -> Option<usize> {
        match peek(pattern, pattern_cur + 1) {
            Some(b'd') => {
                let check = |c| match c {
                    b'0'..=b'9' => true,
                    _ => false,
                };
                match peek(pattern, pattern_cur + 2) {
                    Some(b'+') => {
                        if let Some(input_cur) = self.handle_plus(input_cur, check) {
                            self.match_helper(pattern, pattern_cur + 3, input_cur)
                        } else {
                            None
                        }
                    }
                    Some(b'?') => {
                        let input_cur = self.handle_question(input_cur, check);
                        self.match_helper(pattern, pattern_cur + 3, input_cur)
                    }
                    _ => {
                        if let Some(input_cur) = self.handle_single_match(input_cur, check) {
                            self.match_helper(pattern, pattern_cur + 2, input_cur)
                        } else {
                            None
                        }
                    }
                }
            }
            Some(b'w') => {
                let check = |c| match c {
                    b'a'..=b'z' | b'A'..=b'Z' | b'_' => true,
                    _ => false,
                };
                match peek(pattern, pattern_cur + 2) {
                    Some(b'+') => {
                        if let Some(input_cur) = self.handle_plus(input_cur, check) {
                            self.match_helper(pattern, pattern_cur + 3, input_cur)
                        } else {
                            None
                        }
                    }
                    Some(b'?') => {
                        let input_cur = self.handle_question(input_cur, check);
                        self.match_helper(pattern, pattern_cur + 3, input_cur)
                    }
                    _ => {
                        if let Some(input_cur) = self.handle_single_match(input_cur, check) {
                            self.match_helper(pattern, pattern_cur + 2, input_cur)
                        } else {
                            None
                        }
                    }
                }
            }
            Some(b'\\') => {
                let check = |c| match c {
                    b'\\' => true,
                    _ => false,
                };
                match peek(pattern, pattern_cur + 2) {
                    Some(b'+') => {
                        if let Some(input_cur) = self.handle_plus(input_cur, check) {
                            self.match_helper(pattern, pattern_cur + 3, input_cur)
                        } else {
                            None
                        }
                    }
                    Some(b'?') => {
                        let input_cur = self.handle_question(input_cur, check);
                        self.match_helper(pattern, pattern_cur + 3, input_cur)
                    }
                    _ => {
                        if let Some(input_cur) = self.handle_single_match(input_cur, check) {
                            self.match_helper(pattern, pattern_cur + 2, input_cur)
                        } else {
                            None
                        }
                    }
                }
            }
            Some(c @ b'1'..=b'9') => {
                println!("hjere");
                let backref_idx = c as usize - b'0' as usize - 1;
                println!("backref idx: {}", backref_idx);
                if backref_idx >= self.backref_idx {
                    None
                } else {
                    let backref = self.backrefs[backref_idx].clone();
                    if let Some(input_cur) = self.match_starting_from(&backref, input_cur) {
                        println!(
                            "found backreference, continuing on input idx: {}!",
                            input_cur
                        );
                        self.match_helper(pattern, pattern_cur + 2, input_cur)
                    } else {
                        None
                    }
                }
            }
            Some(_) => panic!("Unknown escape character"),
            None => None,
        }
    }

    fn get_character_group<'b>(pattern: &'b [u8], pattern_cur: usize) -> Option<(Vec<u8>, usize)> {
        let mut group = Vec::new();
        let mut closed = false;

        let mut pattern_cur = pattern_cur;
        while let Some(c) = peek(pattern, pattern_cur) {
            pattern_cur += 1;
            if c == b']' {
                closed = true;
                break;
            }
            group.push(c);
        }

        if closed {
            Some((group, pattern_cur))
        } else {
            None
        }
    }

    fn handle_character_group<'b>(
        &mut self,
        pattern: &'b [u8],
        pattern_cur: usize,
        input_cur: usize,
    ) -> Option<usize> {
        match peek(pattern, pattern_cur + 1) {
            Some(b'^') => {
                // negative character group
                let Some((group, pattern_cur)) =
                    Self::get_character_group(pattern, pattern_cur + 2)
                else {
                    return None;
                };
                let check = |c| !group.contains(&c);
                match peek(pattern, pattern_cur) {
                    Some(b'+') => {
                        if let Some(input_cur) = self.handle_plus(input_cur, check) {
                            self.match_helper(pattern, pattern_cur + 1, input_cur)
                        } else {
                            None
                        }
                    }
                    Some(b'?') => {
                        let input_cur = self.handle_question(input_cur, check);
                        self.match_helper(pattern, pattern_cur + 1, input_cur)
                    }
                    _ => {
                        if let Some(input_cur) = self.handle_single_match(input_cur, check) {
                            self.match_helper(pattern, pattern_cur, input_cur)
                        } else {
                            None
                        }
                    }
                }
            }
            Some(_) => {
                // positive character group
                let Some((group, pattern_cur)) =
                    Self::get_character_group(pattern, pattern_cur + 1)
                else {
                    return None;
                };
                let check = |c| group.contains(&c);
                match peek(pattern, pattern_cur) {
                    Some(b'+') => {
                        if let Some(input_cur) = self.handle_plus(input_cur, check) {
                            self.match_helper(pattern, pattern_cur + 1, input_cur)
                        } else {
                            None
                        }
                    }
                    Some(b'?') => {
                        let input_cur = self.handle_question(input_cur, check);
                        self.match_helper(pattern, pattern_cur + 1, input_cur)
                    }
                    _ => {
                        if let Some(input_cur) = self.handle_single_match(input_cur, check) {
                            self.match_helper(pattern, pattern_cur, input_cur)
                        } else {
                            None
                        }
                    }
                }
            }
            None => None,
        }
    }

    fn get_pattern_group<'b>(
        pattern: &'b [u8],
        pattern_cur: usize,
    ) -> Option<(Vec<String>, usize)> {
        let mut group = Vec::new();
        let mut closed = false;
        let mut n_opening = 0u32;

        let mut pattern_start_idx = pattern_cur;
        let mut pattern_cur = pattern_cur;
        while let Some(c) = peek(pattern, pattern_cur) {
            pattern_cur += 1;
            match c {
                b'(' => n_opening += 1,
                b')' => {
                    if n_opening == 0 {
                        // matching paranthesis is found
                        closed = true;
                        break;
                    }
                    n_opening -= 1;
                }
                b'|' => {
                    let pattern_end_idx = pattern_cur - 2;
                    if pattern_end_idx >= pattern_start_idx {
                        group.push(
                            String::from_utf8(
                                (&pattern[pattern_start_idx..=pattern_end_idx]).to_vec(),
                            )
                            .unwrap(),
                        );
                    }
                    pattern_start_idx = pattern_cur;
                    continue;
                }
                _ => (),
            }
        }

        let pattern_end_idx = pattern_cur - 2;
        if pattern_end_idx >= pattern_start_idx {
            group.push(
                String::from_utf8((&pattern[pattern_start_idx..=pattern_end_idx]).to_vec())
                    .unwrap(),
            );
        }
        if closed {
            Some((group, pattern_cur))
        } else {
            None
        }
    }

    fn handle_pattern_group<'b>(
        &mut self,
        pattern: &'b [u8],
        pattern_cur: usize,
        input_cur: usize,
    ) -> Option<usize> {
        println!("getting pattern group, pattern_cur: {}", pattern_cur);
        let Some((group, pattern_cur)) = Self::get_pattern_group(pattern, pattern_cur + 1) else {
            return None;
        };

        println!(
            "group: {:?}, pattern_cur: {}, input_cur: {}",
            group, pattern_cur, input_cur
        );
        let backref_idx = self.backref_idx;
        self.backref_idx += 1;

        let input_start_idx = input_cur;
        for group_pattern in group {
            println!("calling match pattern with input cur: {}", input_cur);
            if let Some(input_cur) = self.match_starting_from(&group_pattern, input_cur) {
                println!(
                    "found, cur idx: {}, continuing on pattern idx: {:?}",
                    input_cur, pattern_cur
                );
                self.backrefs[backref_idx] =
                    String::from_utf8(self.input[input_start_idx..input_cur].to_vec()).unwrap();
                println!("backrefs: {:?}", self.backrefs);
                return self.match_helper(pattern, pattern_cur, input_cur);
            }
        }
        println!("after handle pattern group");

        None
    }

    fn handle_default_character<'b>(
        &mut self,
        pattern: &'b [u8],
        pattern_cur: usize,
        input_cur: usize,
        c_pattern: u8,
    ) -> Option<usize> {
        let check = |c| c_pattern == b'.' || c == c_pattern;
        match peek(pattern, pattern_cur + 1) {
            Some(b'+') => {
                if let Some(input_cur) = self.handle_plus(input_cur, check) {
                    self.match_helper(pattern, pattern_cur + 2, input_cur)
                } else {
                    None
                }
            }
            Some(b'?') => {
                let input_cur = self.handle_question(input_cur, check);
                self.match_helper(pattern, pattern_cur + 2, input_cur)
            }
            _ => {
                if let Some(input_cur) = self.handle_single_match(input_cur, check) {
                    self.match_helper(pattern, pattern_cur + 1, input_cur)
                } else {
                    println!("unmatched!");
                    None
                }
            }
        }
    }

    fn handle_single_match<F>(&self, input_cur: usize, check: F) -> Option<usize>
    where
        F: Fn(u8) -> bool,
    {
        if let Some(c_input) = peek(self.input, input_cur) {
            if check(c_input) {
                return Some(input_cur + 1);
            }
        }
        None
    }

    // TODO: It should check the pattern against remaining input at each loop iteration
    // The following should return 0: echo -n "abc-def is abc-def, not efg, abc, or def" | ./regex-parser -E "(([abc]+)-([def]+)) is \1, not ([^xyz]+), \2, or \3"
    fn handle_plus<F>(&self, input_cur: usize, check: F) -> Option<usize>
    where
        F: Fn(u8) -> bool,
    {
        if let Some(c_input) = peek(self.input, input_cur) {
            if !check(c_input) {
                None
            } else {
                let mut input_cur = input_cur + 1;
                while let Some(c_input) = peek(self.input, input_cur) {
                    if !check(c_input) {
                        break;
                    }
                    input_cur += 1;
                }
                Some(input_cur)
            }
        } else {
            None
        }
    }

    fn handle_question<F>(&self, input_cur: usize, check: F) -> usize
    where
        F: Fn(u8) -> bool,
    {
        if let Some(c_input) = peek(self.input, input_cur) {
            if check(c_input) {
                return input_cur + 1;
            }
        }
        input_cur
    }

    // TODO: It should check the pattern against remaining input at each loop iteration
    #[allow(dead_code)]
    fn handle_asterisk<F>(&self, input_cur: usize, check: F) -> usize
    where
        F: Fn(u8) -> bool,
    {
        let mut input_cur = input_cur;
        while let Some(c_input) = peek(self.input, input_cur) {
            if !check(c_input) {
                break;
            }
            input_cur += 1;
        }
        input_cur
    }
}

fn peek(input: &[u8], idx: usize) -> Option<u8> {
    if idx >= input.len() {
        None
    } else {
        Some(input[idx])
    }
}
