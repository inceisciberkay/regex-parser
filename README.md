**Some examples (Backreferences and Nested Backreferences)**:
<br>**\$** echo -n "abc-def is abc-def, not efg, abc, or def" | ./target/debug/regex-parser -E "(([abc]+)-([def]+)) is \1, not ([^xyz]+), \2, or \3"
<br>**\$** echo -n "'cat and cat' is the same as 'cat and cat'" | ./target/debug/regex-parser -E "('(cat) and \2') is the same as \1"
