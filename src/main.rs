mod matcher;
mod pattern;

use matcher::*;
use pattern::*;

use std::env;
use std::io;
use std::process;

fn match_pattern(input: &str, pattern: &str) -> bool {
    let matcher = Matcher::new();
    matcher.match_pattern(input, pattern)
}

// Usage: echo <input_text> | ./regex-parser -E <pattern>
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        println!("Expected first argument to be '-E'");
        process::exit(1);
    }

    let pattern = env::args().nth(2).unwrap();
    let mut input_line = String::new();

    io::stdin().read_line(&mut input_line).unwrap();

    if match_pattern(&input_line, &pattern) {
        println!("Match successful");
        process::exit(0)
    } else {
        println!("Match failed");
        process::exit(1)
    }
}
