use regex_parser::Matcher;
use std::env;
use std::io;
use std::process;

fn usage() -> &'static str {
    "Usage: echo <input_text> | regex_parser -E <pattern>"
}
fn main() {
    if env::args().nth(1).unwrap() != "-E" {
        eprintln!("{}", usage());
        process::exit(1);
    }

    let pattern = env::args().nth(2).expect(usage());
    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap();

    let mut matcher = Matcher::new(&input);

    if !matcher.match_pattern(&pattern) {
        println!("Not matched");
        process::exit(1)
    }
}
