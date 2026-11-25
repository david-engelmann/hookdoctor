//! The pins of slice 49, the matcher engine grammar
//! standings, over pattern texts and texts the pins
//! hand to the public engine doors directly.
//! PORT-4 states the matcher reading at the port
//! grain and enumerates the grammar in one
//! sentence, and the RFIX obligations amend that
//! sentence at single seats, and the pins below
//! state the grammar whole at its own grain, the
//! structure and the groups and the repetition
//! forms and their refusal seats and the classes
//! and their refusal seats and the dot and the
//! anchors and the boundaries and the character
//! readings and the escapes and the doors with
//! the one refusal and the guard at its boundary.
//! Every pin stands as a fence on the landed code,
//! this slice moving no production code.

use hookdoctor::regexlite::{compile, fullmatch, pattern_compiles, InvalidPattern};

fn m(pat: &str, text: &str) -> bool {
    fullmatch(pat, text).expect("the pattern compiles")
}

fn bad(pat: &str) -> bool {
    compile(pat).is_err()
}

#[test]
fn test_a_pattern_reads_as_an_alternation_of_concatenations_of_repeated_atoms_and_a_match_covers_the_whole_text(
) {
    // Alternation binds last, each branch standing as its
    // concatenation whole.
    assert!(m("ab|cd", "ab"));
    assert!(m("ab|cd", "cd"));
    assert!(!m("ab|cd", "abcd"));
    assert!(!m("ab|cd", "ad"));
    assert!(m("a|bc", "bc"));
    assert!(!m("a|bc", "abc"));
    // Repetition binds to the one atom standing ahead of it.
    assert!(m("ab*", "a"));
    assert!(m("ab*", "abbb"));
    assert!(!m("ab*", "abab"));
    // A literal character matches itself exactly, the case of
    // a letter standing as written with no folding.
    assert!(m("abc", "abc"));
    assert!(!m("Abc", "abc"));
    assert!(!m("abc", "Abc"));
    // The empty standings at their seats, an empty pattern and
    // an empty branch and an empty group each matching the
    // empty text alone.
    assert!(m("", ""));
    assert!(!m("", "a"));
    assert!(m("a|", ""));
    assert!(m("a|", "a"));
    assert!(m("|", ""));
    assert!(m("()", ""));
    assert!(m("(a|)", ""));
    // The whole text standings, a match covering neither a
    // prefix nor a suffix of the text.
    assert!(!m("abc", "ab"));
    assert!(!m("abc", "xabc"));
    assert!(!m("a{2,3}", "aaaa"));
}

#[test]
fn test_capturing_and_noncapturing_groups_compile_to_the_one_reading_and_the_group_family_refusals_stand(
) {
    // The capturing and noncapturing forms read alike, the
    // engine carrying no capture surface.
    assert!(m("(ab)+", "abab"));
    assert!(m("(?:ab)+", "abab"));
    assert!(m("((a))", "a"));
    assert!(m("(a|b)*", "abba"));
    assert!(m("(a*)*", "aaa"));
    assert!(m("(a*)*", ""));
    // A group may stand empty and may carry an alternation,
    // and a repetition after a group binds to the group whole.
    assert!(m("()", ""));
    assert!(m("(a|)", ""));
    assert!(m("(ab){2}", "abab"));
    assert!(!m("(ab){2}", "ab"));
    // A group standing unclosed refuses, and a closing paren
    // standing outside a group refuses.
    assert!(bad("(a"));
    assert!(bad("(?:a"));
    assert!(bad("a)"));
    // The question mark family behind a group opener refuses
    // at every spelling but the colon.
    assert!(bad("(?)"));
    assert!(bad("(?P<n>a)"));
    assert!(bad("(?=a)"));
    assert!(bad("(?!a)"));
    assert!(bad("(?<=a)"));
    assert!(bad("(?<!a)"));
    assert!(bad("(?i)a"));
    assert!(bad("(?#c)a"));
}

#[test]
fn test_the_repetition_forms_read_at_their_bounds_with_the_lazy_suffix_as_the_plain_form() {
    // The three marks at their bounds.
    assert!(m("a*", ""));
    assert!(m("a*", "aaa"));
    assert!(!m("a+", ""));
    assert!(m("a+", "aaa"));
    assert!(m("a?", ""));
    assert!(m("a?", "a"));
    assert!(!m("a?b", "aab"));
    assert!(m("a?b", "ab"));
    assert!(m("a?b", "b"));
    // The counted shapes, a lone count as both bounds, a low
    // count and a comma as the low bound with no high bound,
    // two counts as the two bounds, and a comma and a high
    // count as zero for the low bound with that high bound.
    assert!(m("a{2}", "aa"));
    assert!(!m("a{2}", "a"));
    assert!(m("a{2,}", "aaaa"));
    assert!(!m("a{2,}", "a"));
    assert!(m("a{2,3}", "aa"));
    assert!(m("a{2,3}", "aaa"));
    assert!(!m("a{2,3}", "a"));
    assert!(m("a{,3}", ""));
    assert!(m("a{,3}", "aaa"));
    // A counted form stands only where its closing brace
    // stands, a brace opening no counted spec reading as a
    // literal brace at this seat as MG-5 states it whole.
    assert!(m("a{2", "a{2"));
    // The bare comma form reads as no bound and not as a
    // literal, matching the empty text and any run.
    assert!(m("a{,}", ""));
    assert!(m("a{,}", "aaa"));
    assert!(m("a{0}b", "b"));
    assert!(!m("a{0}b", "ab"));
    // The lazy suffix reads as the plain form at the marks
    // and at a counted form alike.
    assert!(m("a*?", "aaa"));
    assert!(m("a+?", "a"));
    assert!(m("a??", ""));
    assert!(m("a{2}?", "aa"));
    assert!(m("a{2,}?", "aaa"));
}

#[test]
fn test_a_quantifier_with_no_atom_ahead_of_it_refuses_at_every_seat_and_a_brace_opening_no_counted_spec_reads_as_a_literal(
) {
    // A quantifier standing where no atom precedes it refuses,
    // at the head and behind a bar and behind an opener.
    assert!(bad("*a"));
    assert!(bad("+a"));
    assert!(bad("?a"));
    assert!(bad("a|*"));
    assert!(bad("(*a)"));
    assert!(bad("{2}a"));
    // An anchor and a boundary assertion carry no repetition.
    assert!(bad("^*"));
    assert!(bad("$+"));
    assert!(bad("\\b*"));
    assert!(bad("\\B?"));
    assert!(bad("\\A+"));
    // A quantifier behind a quantified atom refuses, and the
    // possessive form refuses, and a counted form whose low
    // passes its high refuses.
    assert!(bad("a**"));
    assert!(bad("a?*"));
    assert!(bad("a*+"));
    assert!(bad("a{2}+"));
    assert!(bad("a*{2}"));
    assert!(bad("a{2}{3}"));
    assert!(bad("a{3,2}"));
    // A brace that opens no counted spec reads as a literal
    // brace character whole, at the head and in the body.
    assert!(m("{a}", "{a}"));
    assert!(m("a{", "a{"));
    assert!(m("a{2", "a{2"));
    assert!(m("a{x}", "a{x}"));
    assert!(m("a{}", "a{}"));
    assert!(m("}", "}"));
    assert!(m("a}", "a}"));
}

#[test]
fn test_a_class_reads_by_its_items_and_ranges_and_negation_with_the_dash_and_the_bracket_at_their_literal_seats(
) {
    assert!(m("[abc]", "b"));
    assert!(!m("[abc]", "z"));
    assert!(!m("[abc]", ""));
    // The negation standing first turns the reading.
    assert!(m("[^abc]", "z"));
    assert!(!m("[^abc]", "b"));
    // A range reads between its two characters, the case
    // standing as written.
    assert!(m("[a-z]", "m"));
    assert!(!m("[a-z]", "M"));
    assert!(m("[a-cx-z]", "y"));
    // A closing bracket standing first reads as a literal
    // bracket item, under the negation as without it.
    assert!(m("[]]", "]"));
    assert!(m("[^]]", "a"));
    assert!(!m("[^]]", "]"));
    // A dash at an edge reads as a literal dash, and a bar
    // inside a class stands as a literal character.
    assert!(m("[a-]", "-"));
    assert!(m("[-a]", "-"));
    assert!(m("[a|b]", "|"));
    // The class escapes stand inside a class.
    assert!(m("[\\d]", "5"));
    assert!(!m("[\\d]", "x"));
    assert!(m("[\\s]", "\u{1c}"));
    assert!(m("[\\]]", "]"));
    assert!(m("[\\\\]", "\\"));
    assert!(m("[\\-]", "-"));
}

#[test]
fn test_a_class_refuses_at_the_reversed_range_and_the_escape_range_ends_and_the_unclosed_spellings_and_reads_the_letter_b_as_backspace(
) {
    // A range whose low character passes its high refuses.
    assert!(bad("[z-a]"));
    // A range with a class escape at either end refuses.
    assert!(bad("[a-\\d]"));
    assert!(bad("[\\d-z]"));
    // A class standing unclosed refuses, the empty class
    // spelling standing unclosed whole by the first bracket
    // reading.
    assert!(bad("["));
    assert!(bad("[a"));
    assert!(bad("[]"));
    assert!(bad("[^]"));
    // The digits eight and nine after a backslash inside a
    // class refuse.
    assert!(bad("[\\8]"));
    assert!(bad("[\\9]"));
    // Inside a class the letter b reads as the backspace
    // character and not as a boundary assertion.
    assert!(m("[\\b]", "\u{8}"));
    assert!(!m("[\\b]", "a"));
}

#[test]
fn test_the_dot_takes_any_character_but_a_newline_and_the_anchors_and_boundaries_hold_at_the_whole_text_seats(
) {
    // The dot takes any one character but a newline, a
    // carriage return standing inside its reading.
    assert!(m(".", "a"));
    assert!(!m(".", "\n"));
    assert!(m(".", "\r"));
    assert!(!m(".", ""));
    // The caret and the dollar stand as the start and end
    // assertions of the whole text at any seat.
    assert!(m("^a", "a"));
    assert!(m("a$", "a"));
    assert!(m("^$", ""));
    assert!(m("^", ""));
    assert!(m("$", ""));
    // An anchor standing mid pattern holds only where the
    // whole text standing holds there, so these readings no
    // text satisfies through that seat.
    assert!(!m("a^b", "ab"));
    assert!(!m("a^b", "a^b"));
    assert!(!m("a$b", "ab"));
    assert!(!m("a^", "a"));
    // The letter anchors stand at the same seats.
    assert!(m("\\Aab", "ab"));
    assert!(m("ab\\Z", "ab"));
    // The boundary assertion and its negation hold by the
    // word reading over the characters behind and ahead.
    assert!(m("\\bfoo\\b", "foo"));
    assert!(!m("\\bfoo\\b", "afoo"));
    assert!(m("x\\Bfoo", "xfoo"));
    assert!(m("a\\b ", "a "));
    assert!(!m("a\\B ", "a "));
    assert!(m("\\B", ""));
    assert!(!m("\\b", ""));
}

#[test]
fn test_the_digit_and_word_and_space_readings_and_the_escape_family_stand_at_their_values_and_their_refusals(
) {
    // The digit reading of RFIX-5 at its discriminating
    // characters, the Arabic Indic digit standing inside it
    // and the superscript two standing outside it.
    assert!(m("\\d", "\u{663}"));
    assert!(!m("\\d", "\u{b2}"));
    assert!(m("\\D", "\u{b2}"));
    assert!(m("\\d", "5"));
    // The word reading, alphanumeric over Unicode or the
    // underscore.
    assert!(m("\\w", "é"));
    assert!(m("\\w", "_"));
    assert!(!m("\\w", "-"));
    assert!(m("\\W", "-"));
    assert!(!m("\\W", "é"));
    // The space reading, the whitespace reading with the
    // control characters of its range standing inside it.
    assert!(m("\\s", "\u{1c}"));
    assert!(m("\\s", "\u{a0}"));
    assert!(m("\\s", " "));
    assert!(!m("\\S", "\u{1c}"));
    assert!(!m("\\s", "x"));
    // The named control escapes read as the characters they
    // name.
    assert!(m("\\n", "\n"));
    assert!(m("\\r", "\r"));
    assert!(m("\\t", "\t"));
    assert!(m("\\f", "\u{c}"));
    assert!(m("\\v", "\u{b}"));
    assert!(m("\\a", "\u{7}"));
    // The hex forms read at two and four and eight digits,
    // refusing at a short form and a non digit and a value
    // naming no character.
    assert!(m("\\x41", "A"));
    assert!(m("\\u0041", "A"));
    assert!(m("\\U0001F600", "\u{1F600}"));
    assert!(bad("\\x4"));
    assert!(bad("\\xZZ"));
    assert!(bad("\\uD800"));
    assert!(bad("\\U00110000"));
    // The octal family at the standings of RFIX-7.
    assert!(m("\\0", "\u{0}"));
    assert!(m("\\01", "\u{1}"));
    assert!(m("\\07", "\u{7}"));
    assert!(m("\\101", "A"));
    assert!(bad("\\400"));
    assert!(bad("\\47"));
    assert!(bad("\\8"));
    // An alphabetic escape outside the named set refuses,
    // and a non alphabetic escape reads as the literal
    // character, and a trailing backslash refuses.
    assert!(bad("\\e"));
    assert!(bad("\\q"));
    assert!(bad("\\z"));
    assert!(m("\\ ", " "));
    assert!(m("\\.", "."));
    assert!(m("\\\\", "\\"));
    assert!(m("\\-", "-"));
    assert!(bad("\\"));
}

#[test]
fn test_the_engine_stands_behind_its_four_doors_with_the_one_refusal_and_the_repetition_guard_at_its_boundary(
) {
    // The pattern_compiles door stands as the boolean shadow
    // of the compile door.
    assert!(pattern_compiles("a+"));
    assert!(!pattern_compiles("a**"));
    assert!(pattern_compiles(""));
    // The free fullmatch door compiles and matches in one
    // call, and the compiled pattern carries its own
    // fullmatch method, a match at both doors covering the
    // whole text.
    assert!(fullmatch("ab|cd", "cd").expect("the pattern compiles"));
    assert!(!fullmatch("ab", "abc").expect("the pattern compiles"));
    assert!(fullmatch("a**", "a").is_err());
    let pattern = compile("ab|cd").expect("the pattern compiles");
    assert!(pattern.fullmatch("cd"));
    assert!(!pattern.fullmatch("abc"));
    // Every refusal of the engine stands as the one
    // InvalidPattern, equal at every refusal seat of the
    // pins above, carrying no position and no reason.
    let seats = [
        "(a",
        "a**",
        "{2}a",
        "[z-a]",
        "[\\8]",
        "\\e",
        "\\",
        "a{3,2}",
        "a{10001}",
        "(a{10000}){10000}",
    ];
    for seat in seats {
        assert_eq!(compile(seat).unwrap_err(), InvalidPattern, "seat {seat}");
    }
    // The count caps of RFIX-3 at their edges, a count at the
    // cap compiling and a count past it refusing and a count
    // written in seven digits refusing at its value of one.
    assert!(pattern_compiles("a{10000}"));
    assert!(bad("a{10001}"));
    assert!(bad("a{1000000}"));
    assert!(bad("a{0000001}"));
    // The expansion ceiling at its boundary pair, the nested
    // counted form expanding to exactly the ceiling compiling
    // and the example form of RFIX-3 refusing.
    assert!(pattern_compiles("(a{1000}){1000}"));
    assert!(bad("(a{10000}){10000}"));
}
