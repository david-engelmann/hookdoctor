//! A small pattern engine carrying the matcher grammar of
//! obligation PORT-4.
//!
//! The grammar covers literal characters and alternation and
//! grouping with capturing and noncapturing groups and character
//! classes with ranges and negation and the dot and the repetition
//! forms star and plus and question and counted repetition, with
//! lazy forms read as their plain forms, and the anchors and the
//! character escapes for digits and word characters and whitespace
//! and their negations. Patterns standing outside that grammar,
//! among them backreferences and possessive quantifiers and
//! lookahead and lookbehind and named groups and inline flags and
//! conditionals, do not compile here. Matching is a Thompson state simulation
//! and a match must cover the whole text, as the Python fullmatch
//! reading requires.

/// The Python whitespace reading for the space escape.
pub fn py_is_space(ch: char) -> bool {
    ch.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&ch)
}

/// The Python word character reading.
pub fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// The merged code point ranges standing in the Unicode decimal
/// number category. The table was generated from the unicodedata
/// of the Python runtime the landed spec runs on, CPython 3.12
/// over Unicode 15.0.0, by the generator kept in the Round 2
/// scratch, and it is committed here as source data.
static ND_RANGES: &[(u32, u32)] = &[
    (0x0030, 0x0039),
    (0x0660, 0x0669),
    (0x06f0, 0x06f9),
    (0x07c0, 0x07c9),
    (0x0966, 0x096f),
    (0x09e6, 0x09ef),
    (0x0a66, 0x0a6f),
    (0x0ae6, 0x0aef),
    (0x0b66, 0x0b6f),
    (0x0be6, 0x0bef),
    (0x0c66, 0x0c6f),
    (0x0ce6, 0x0cef),
    (0x0d66, 0x0d6f),
    (0x0de6, 0x0def),
    (0x0e50, 0x0e59),
    (0x0ed0, 0x0ed9),
    (0x0f20, 0x0f29),
    (0x1040, 0x1049),
    (0x1090, 0x1099),
    (0x17e0, 0x17e9),
    (0x1810, 0x1819),
    (0x1946, 0x194f),
    (0x19d0, 0x19d9),
    (0x1a80, 0x1a89),
    (0x1a90, 0x1a99),
    (0x1b50, 0x1b59),
    (0x1bb0, 0x1bb9),
    (0x1c40, 0x1c49),
    (0x1c50, 0x1c59),
    (0xa620, 0xa629),
    (0xa8d0, 0xa8d9),
    (0xa900, 0xa909),
    (0xa9d0, 0xa9d9),
    (0xa9f0, 0xa9f9),
    (0xaa50, 0xaa59),
    (0xabf0, 0xabf9),
    (0xff10, 0xff19),
    (0x104a0, 0x104a9),
    (0x10d30, 0x10d39),
    (0x11066, 0x1106f),
    (0x110f0, 0x110f9),
    (0x11136, 0x1113f),
    (0x111d0, 0x111d9),
    (0x112f0, 0x112f9),
    (0x11450, 0x11459),
    (0x114d0, 0x114d9),
    (0x11650, 0x11659),
    (0x116c0, 0x116c9),
    (0x11730, 0x11739),
    (0x118e0, 0x118e9),
    (0x11950, 0x11959),
    (0x11c50, 0x11c59),
    (0x11d50, 0x11d59),
    (0x11da0, 0x11da9),
    (0x11f50, 0x11f59),
    (0x16a60, 0x16a69),
    (0x16ac0, 0x16ac9),
    (0x16b50, 0x16b59),
    (0x1d7ce, 0x1d7ff),
    (0x1e140, 0x1e149),
    (0x1e2f0, 0x1e2f9),
    (0x1e4f0, 0x1e4f9),
    (0x1e950, 0x1e959),
    (0x1fbf0, 0x1fbf9),
];

/// The Python digit reading, the decimal number category and
/// nothing wider, as obligation RFIX-5 states.
fn is_python_digit(ch: char) -> bool {
    let point = ch as u32;
    ND_RANGES
        .binary_search_by(|&(low, high)| {
            if point < low {
                std::cmp::Ordering::Greater
            } else if point > high {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

#[derive(Debug, Clone)]
enum ClassItem {
    Char(char),
    Range(char, char),
    Digit(bool),
    Word(bool),
    Space(bool),
}

impl ClassItem {
    fn matches(&self, ch: char) -> bool {
        match self {
            ClassItem::Char(expected) => ch == *expected,
            ClassItem::Range(low, high) => *low <= ch && ch <= *high,
            ClassItem::Digit(want) => is_python_digit(ch) == *want,
            ClassItem::Word(want) => is_word_char(ch) == *want,
            ClassItem::Space(want) => py_is_space(ch) == *want,
        }
    }
}

#[derive(Debug, Clone)]
enum CharPred {
    Single(char),
    Any,
    Class {
        negated: bool,
        items: Vec<ClassItem>,
    },
}

impl CharPred {
    fn matches(&self, ch: char) -> bool {
        match self {
            CharPred::Single(expected) => ch == *expected,
            CharPred::Any => ch != '\n',
            CharPred::Class { negated, items } => {
                let hit = items.iter().any(|item| item.matches(ch));
                hit != *negated
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Assertion {
    Start,
    End,
    WordBoundary(bool),
}

impl Assertion {
    fn holds(&self, chars: &[char], pos: usize) -> bool {
        match self {
            Assertion::Start => pos == 0,
            Assertion::End => pos == chars.len(),
            Assertion::WordBoundary(want) => {
                let before = pos > 0 && is_word_char(chars[pos - 1]);
                let after = pos < chars.len() && is_word_char(chars[pos]);
                (before != after) == *want
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Node {
    Empty,
    Char(CharPred),
    Concat(Vec<Node>),
    Alt(Vec<Node>),
    Repeat {
        inner: Box<Node>,
        min: u32,
        max: Option<u32>,
    },
    Assert(Assertion),
}

/// A pattern outside the grammar this engine reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidPattern;

/// A counted repetition spec, the lowest count, the highest
/// count where the spec carries one, and the cursor position just
/// past the closing brace.
type BraceSpec = (u32, Option<u32>, usize);

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

const MAX_REPEAT: u32 = 10_000;

/// The expansion ceiling of obligation RFIX-3, counted in the
/// instructions a compilation emits.
const MAX_PROGRAM: u64 = 1_000_000;

/// The expansion of a pattern, counted exactly as the compilation
/// emits it, as obligation RFIX-3 states the count. The count
/// saturates, because its only use is the comparison against the
/// ceiling.
fn expansion(node: &Node) -> u64 {
    match node {
        Node::Empty => 0,
        Node::Char(_) | Node::Assert(_) => 1,
        Node::Concat(pieces) => pieces
            .iter()
            .fold(0u64, |total, piece| total.saturating_add(expansion(piece))),
        Node::Alt(branches) => {
            let bodies = branches.iter().fold(0u64, |total, branch| {
                total.saturating_add(expansion(branch))
            });
            bodies.saturating_add(2u64.saturating_mul(branches.len().saturating_sub(1) as u64))
        }
        Node::Repeat { inner, min, max } => {
            let body = expansion(inner);
            let lowest = body.saturating_mul(u64::from(*min));
            match max {
                None => lowest.saturating_add(body).saturating_add(2),
                Some(limit) => lowest.saturating_add(
                    body.saturating_add(1)
                        .saturating_mul(u64::from(limit.saturating_sub(*min))),
                ),
            }
        }
    }
}

impl Parser {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.chars.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    /// Read a counted repetition spec standing at the cursor, in
    /// the shape of braces holding digits and at most one comma.
    /// A brace that opens no such spec is a literal character in
    /// the Python reading and reads as None here.
    fn brace_spec(&self) -> Option<Result<BraceSpec, InvalidPattern>> {
        let mut index = self.pos + 1;
        let mut low_text = String::new();
        while let Some(ch) = self.chars.get(index).copied() {
            if ch.is_ascii_digit() {
                low_text.push(ch);
                index += 1;
            } else {
                break;
            }
        }
        let mut comma = false;
        let mut high_text = String::new();
        if self.chars.get(index) == Some(&',') {
            comma = true;
            index += 1;
            while let Some(ch) = self.chars.get(index).copied() {
                if ch.is_ascii_digit() {
                    high_text.push(ch);
                    index += 1;
                } else {
                    break;
                }
            }
        }
        if self.chars.get(index) != Some(&'}') {
            return None;
        }
        if low_text.is_empty() && high_text.is_empty() && !comma {
            return None;
        }
        let parse_count = |text: &str| -> Result<u32, InvalidPattern> {
            if text.len() > 6 {
                return Err(InvalidPattern);
            }
            let value: u32 = text.parse().map_err(|_| InvalidPattern)?;
            if value > MAX_REPEAT {
                return Err(InvalidPattern);
            }
            Ok(value)
        };
        let low = if low_text.is_empty() {
            0
        } else {
            match parse_count(&low_text) {
                Ok(value) => value,
                Err(error) => return Some(Err(error)),
            }
        };
        let high = if !comma {
            Some(low)
        } else if high_text.is_empty() {
            None
        } else {
            match parse_count(&high_text) {
                Ok(value) => Some(value),
                Err(error) => return Some(Err(error)),
            }
        };
        if let Some(high_value) = high {
            if low > high_value {
                return Some(Err(InvalidPattern));
            }
        }
        Some(Ok((low, high, index + 1)))
    }

    fn parse_alt(&mut self, in_group: bool) -> Result<Node, InvalidPattern> {
        let mut branches = vec![self.parse_concat()?];
        while self.peek() == Some('|') {
            self.pos += 1;
            branches.push(self.parse_concat()?);
        }
        if in_group && self.peek() != Some(')') {
            return Err(InvalidPattern);
        }
        if branches.len() == 1 {
            Ok(branches.pop().expect("one branch stands"))
        } else {
            Ok(Node::Alt(branches))
        }
    }

    fn parse_concat(&mut self) -> Result<Node, InvalidPattern> {
        let mut pieces: Vec<Node> = Vec::new();
        loop {
            match self.peek() {
                None | Some('|') | Some(')') => break,
                Some('*') | Some('+') | Some('?') => return Err(InvalidPattern),
                Some('{') => {
                    if self.brace_spec().is_some() {
                        return Err(InvalidPattern);
                    }
                    let (node, _) = self.parse_qualified_atom()?;
                    pieces.push(node);
                }
                _ => {
                    let (node, _) = self.parse_qualified_atom()?;
                    pieces.push(node);
                }
            }
        }
        match pieces.len() {
            0 => Ok(Node::Empty),
            1 => Ok(pieces.pop().expect("one piece stands")),
            _ => Ok(Node::Concat(pieces)),
        }
    }

    /// Read one atom and the repetition that may stand after it.
    fn parse_qualified_atom(&mut self) -> Result<(Node, ()), InvalidPattern> {
        let (node, repeatable) = self.parse_atom()?;
        let mut node = node;
        let mut quantified = false;
        match self.peek() {
            Some('*') | Some('+') | Some('?') => {
                if !repeatable {
                    return Err(InvalidPattern);
                }
                let mark = self.advance().expect("quantifier stands");
                let (min, max) = match mark {
                    '*' => (0, None),
                    '+' => (1, None),
                    _ => (0, Some(1)),
                };
                node = Node::Repeat {
                    inner: Box::new(node),
                    min,
                    max,
                };
                quantified = true;
            }
            Some('{') => {
                if let Some(spec) = self.brace_spec() {
                    if !repeatable {
                        return Err(InvalidPattern);
                    }
                    let (min, max, next) = spec?;
                    self.pos = next;
                    node = Node::Repeat {
                        inner: Box::new(node),
                        min,
                        max,
                    };
                    quantified = true;
                }
            }
            _ => {}
        }
        if quantified {
            // One suffix marker may follow. A lazy question mark
            // reads as the plain form here. A plus sign is a
            // possessive form, and a possessive form stands outside
            // the grammar, as obligation RFIX-2 states.
            match self.peek() {
                Some('?') => {
                    self.pos += 1;
                }
                Some('+') => return Err(InvalidPattern),
                _ => {}
            }
            match self.peek() {
                Some('*') | Some('+') | Some('?') => return Err(InvalidPattern),
                Some('{') => {
                    if self.brace_spec().is_some() {
                        return Err(InvalidPattern);
                    }
                }
                _ => {}
            }
        }
        Ok((node, ()))
    }

    fn parse_atom(&mut self) -> Result<(Node, bool), InvalidPattern> {
        let ch = self.advance().ok_or(InvalidPattern)?;
        match ch {
            '(' => {
                if self.peek() == Some('?') {
                    self.pos += 1;
                    match self.peek() {
                        Some(':') => {
                            self.pos += 1;
                        }
                        // Named groups and lookahead and lookbehind
                        // and comments and inline flags stand
                        // outside the grammar.
                        _ => return Err(InvalidPattern),
                    }
                }
                let inner = self.parse_alt(true)?;
                if self.advance() != Some(')') {
                    return Err(InvalidPattern);
                }
                Ok((inner, true))
            }
            '[' => {
                let class = self.parse_class()?;
                Ok((Node::Char(class), true))
            }
            '.' => Ok((Node::Char(CharPred::Any), true)),
            '^' => Ok((Node::Assert(Assertion::Start), false)),
            '$' => Ok((Node::Assert(Assertion::End), false)),
            '\\' => self.parse_escape(),
            _ => Ok((Node::Char(CharPred::Single(ch)), true)),
        }
    }

    fn parse_escape(&mut self) -> Result<(Node, bool), InvalidPattern> {
        let ch = self.advance().ok_or(InvalidPattern)?;
        let class = |item: ClassItem| {
            (
                Node::Char(CharPred::Class {
                    negated: false,
                    items: vec![item],
                }),
                true,
            )
        };
        match ch {
            'd' => Ok(class(ClassItem::Digit(true))),
            'D' => Ok(class(ClassItem::Digit(false))),
            'w' => Ok(class(ClassItem::Word(true))),
            'W' => Ok(class(ClassItem::Word(false))),
            's' => Ok(class(ClassItem::Space(true))),
            'S' => Ok(class(ClassItem::Space(false))),
            'b' => Ok((Node::Assert(Assertion::WordBoundary(true)), false)),
            'B' => Ok((Node::Assert(Assertion::WordBoundary(false)), false)),
            'A' => Ok((Node::Assert(Assertion::Start), false)),
            'Z' => Ok((Node::Assert(Assertion::End), false)),
            'n' => Ok((Node::Char(CharPred::Single('\n')), true)),
            'r' => Ok((Node::Char(CharPred::Single('\r')), true)),
            't' => Ok((Node::Char(CharPred::Single('\t')), true)),
            'f' => Ok((Node::Char(CharPred::Single('\u{000C}')), true)),
            'v' => Ok((Node::Char(CharPred::Single('\u{000B}')), true)),
            'a' => Ok((Node::Char(CharPred::Single('\u{0007}')), true)),
            '0' => {
                let ch = self.read_octal(0)?;
                Ok((Node::Char(CharPred::Single(ch)), true))
            }
            '1'..='9' => {
                // A nonzero first digit followed by two octal
                // digits is an octal escape of three digits, as
                // obligation RFIX-7 states. Any other digit run is
                // a group reference, and group references stand
                // outside the grammar.
                let octal =
                    |candidate: Option<char>| candidate.filter(|digit| ('0'..='7').contains(digit));
                match (octal(self.peek()), octal(self.peek_at(1))) {
                    (Some(second), Some(third)) => {
                        let value = (ch as u32 - '0' as u32) * 64
                            + (second as u32 - '0' as u32) * 8
                            + (third as u32 - '0' as u32);
                        if value > 0o377 {
                            return Err(InvalidPattern);
                        }
                        self.pos += 2;
                        let ch = char::from_u32(value).ok_or(InvalidPattern)?;
                        Ok((Node::Char(CharPred::Single(ch)), true))
                    }
                    _ => Err(InvalidPattern),
                }
            }
            'x' => {
                let ch = self.parse_hex(2)?;
                Ok((Node::Char(CharPred::Single(ch)), true))
            }
            'u' => {
                let ch = self.parse_hex(4)?;
                Ok((Node::Char(CharPred::Single(ch)), true))
            }
            'U' => {
                let ch = self.parse_hex(8)?;
                Ok((Node::Char(CharPred::Single(ch)), true))
            }
            _ if ch.is_alphabetic() => Err(InvalidPattern),
            _ => Ok((Node::Char(CharPred::Single(ch)), true)),
        }
    }

    /// Read the rest of an octal escape whose first digit is
    /// already consumed, reading up to two further octal digits
    /// and refusing a value past the three digit octal bound, in
    /// the shape of the Python reading of obligation RFIX-7.
    fn read_octal(&mut self, first: u32) -> Result<char, InvalidPattern> {
        let mut value = first;
        for _ in 0..2 {
            match self.peek() {
                Some(digit @ '0'..='7') => {
                    value = value * 8 + (digit as u32 - '0' as u32);
                    self.pos += 1;
                }
                _ => break,
            }
        }
        if value > 0o377 {
            return Err(InvalidPattern);
        }
        char::from_u32(value).ok_or(InvalidPattern)
    }

    fn parse_hex(&mut self, count: usize) -> Result<char, InvalidPattern> {
        let mut value: u32 = 0;
        for _ in 0..count {
            let ch = self.advance().ok_or(InvalidPattern)?;
            let digit = ch.to_digit(16).ok_or(InvalidPattern)?;
            value = value * 16 + digit;
        }
        char::from_u32(value).ok_or(InvalidPattern)
    }

    fn parse_class_escape(&mut self) -> Result<ClassItem, InvalidPattern> {
        let ch = self.advance().ok_or(InvalidPattern)?;
        match ch {
            'd' => Ok(ClassItem::Digit(true)),
            'D' => Ok(ClassItem::Digit(false)),
            'w' => Ok(ClassItem::Word(true)),
            'W' => Ok(ClassItem::Word(false)),
            's' => Ok(ClassItem::Space(true)),
            'S' => Ok(ClassItem::Space(false)),
            'b' => Ok(ClassItem::Char('\u{0008}')),
            'n' => Ok(ClassItem::Char('\n')),
            'r' => Ok(ClassItem::Char('\r')),
            't' => Ok(ClassItem::Char('\t')),
            'f' => Ok(ClassItem::Char('\u{000C}')),
            'v' => Ok(ClassItem::Char('\u{000B}')),
            'a' => Ok(ClassItem::Char('\u{0007}')),
            '0'..='7' => Ok(ClassItem::Char(self.read_octal(ch as u32 - '0' as u32)?)),
            '8' | '9' => Err(InvalidPattern),
            'x' => Ok(ClassItem::Char(self.parse_hex(2)?)),
            'u' => Ok(ClassItem::Char(self.parse_hex(4)?)),
            'U' => Ok(ClassItem::Char(self.parse_hex(8)?)),
            _ if ch.is_alphabetic() => Err(InvalidPattern),
            _ => Ok(ClassItem::Char(ch)),
        }
    }

    fn parse_class(&mut self) -> Result<CharPred, InvalidPattern> {
        let mut negated = false;
        if self.peek() == Some('^') {
            negated = true;
            self.pos += 1;
        }
        let mut items: Vec<ClassItem> = Vec::new();
        // A bracket standing first is a literal bracket.
        if self.peek() == Some(']') {
            items.push(ClassItem::Char(']'));
            self.pos += 1;
        }
        loop {
            let ch = self.peek().ok_or(InvalidPattern)?;
            if ch == ']' {
                self.pos += 1;
                return Ok(CharPred::Class { negated, items });
            }
            let low = if ch == '\\' {
                self.pos += 1;
                self.parse_class_escape()?
            } else {
                self.pos += 1;
                ClassItem::Char(ch)
            };
            if self.peek() == Some('-') && self.peek_at(1).is_some() && self.peek_at(1) != Some(']')
            {
                self.pos += 1;
                let high_ch = self.peek().ok_or(InvalidPattern)?;
                let high = if high_ch == '\\' {
                    self.pos += 1;
                    self.parse_class_escape()?
                } else {
                    self.pos += 1;
                    ClassItem::Char(high_ch)
                };
                match (low, high) {
                    (ClassItem::Char(low_char), ClassItem::Char(high_char)) => {
                        if low_char > high_char {
                            return Err(InvalidPattern);
                        }
                        items.push(ClassItem::Range(low_char, high_char));
                    }
                    _ => return Err(InvalidPattern),
                }
            } else {
                items.push(low);
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Inst {
    Char(CharPred),
    Split(usize, usize),
    Jmp(usize),
    Assert(Assertion),
    Match,
}

fn emit(node: &Node, program: &mut Vec<Inst>) {
    match node {
        Node::Empty => {}
        Node::Char(pred) => program.push(Inst::Char(pred.clone())),
        Node::Assert(assertion) => program.push(Inst::Assert(*assertion)),
        Node::Concat(pieces) => {
            for piece in pieces {
                emit(piece, program);
            }
        }
        Node::Alt(branches) => {
            let mut jmp_holes: Vec<usize> = Vec::new();
            for (index, branch) in branches.iter().enumerate() {
                if index + 1 == branches.len() {
                    emit(branch, program);
                } else {
                    let split_at = program.len();
                    program.push(Inst::Split(0, 0));
                    let body_at = program.len();
                    emit(branch, program);
                    jmp_holes.push(program.len());
                    program.push(Inst::Jmp(0));
                    let next_at = program.len();
                    program[split_at] = Inst::Split(body_at, next_at);
                }
            }
            let end = program.len();
            for hole in jmp_holes {
                program[hole] = Inst::Jmp(end);
            }
        }
        Node::Repeat { inner, min, max } => {
            for _ in 0..*min {
                emit(inner, program);
            }
            match max {
                None => {
                    // A star over one more copy of the body.
                    let split_at = program.len();
                    program.push(Inst::Split(0, 0));
                    let body_at = program.len();
                    emit(inner, program);
                    program.push(Inst::Jmp(split_at));
                    let end = program.len();
                    program[split_at] = Inst::Split(body_at, end);
                }
                Some(limit) => {
                    for _ in *min..*limit {
                        let split_at = program.len();
                        program.push(Inst::Split(0, 0));
                        let body_at = program.len();
                        emit(inner, program);
                        let end = program.len();
                        program[split_at] = Inst::Split(body_at, end);
                    }
                }
            }
        }
    }
}

/// A compiled pattern.
#[derive(Debug, Clone)]
pub struct Pattern {
    program: Vec<Inst>,
}

/// Compile a pattern under the grammar this engine reads.
pub fn compile(text: &str) -> Result<Pattern, InvalidPattern> {
    let mut parser = Parser {
        chars: text.chars().collect(),
        pos: 0,
    };
    let node = parser.parse_alt(false)?;
    if parser.pos != parser.chars.len() {
        return Err(InvalidPattern);
    }
    if expansion(&node) > MAX_PROGRAM {
        return Err(InvalidPattern);
    }
    let mut program = Vec::new();
    emit(&node, &mut program);
    program.push(Inst::Match);
    Ok(Pattern { program })
}

/// Whether a pattern compiles under the grammar.
pub fn pattern_compiles(text: &str) -> bool {
    compile(text).is_ok()
}

impl Pattern {
    /// Whether the pattern matches the whole text.
    pub fn fullmatch(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let mut states = self.closure(&[0], &chars, 0);
        for (index, ch) in chars.iter().enumerate() {
            let mut next: Vec<usize> = Vec::new();
            for state in &states {
                if let Inst::Char(pred) = &self.program[*state] {
                    if pred.matches(*ch) {
                        next.push(state + 1);
                    }
                }
            }
            if next.is_empty() {
                return false;
            }
            states = self.closure(&next, &chars, index + 1);
        }
        let match_index = self.program.len() - 1;
        states.contains(&match_index)
    }

    fn closure(&self, seeds: &[usize], chars: &[char], pos: usize) -> Vec<usize> {
        let mut seen: Vec<usize> = Vec::new();
        let mut stack: Vec<usize> = seeds.to_vec();
        while let Some(index) = stack.pop() {
            if seen.contains(&index) {
                continue;
            }
            seen.push(index);
            match &self.program[index] {
                Inst::Split(first, second) => {
                    stack.push(*first);
                    stack.push(*second);
                }
                Inst::Jmp(target) => {
                    stack.push(*target);
                }
                Inst::Assert(assertion) => {
                    if assertion.holds(chars, pos) {
                        stack.push(index + 1);
                    }
                }
                _ => {}
            }
        }
        seen
    }
}

/// Compile and match in one call, as the matcher reading does.
pub fn fullmatch(text_pattern: &str, text: &str) -> Result<bool, InvalidPattern> {
    Ok(compile(text_pattern)?.fullmatch(text))
}
