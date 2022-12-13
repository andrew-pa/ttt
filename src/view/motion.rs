use std::ops::Range;

use anyhow::Result;
use ropey::Rope;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("incomplete command")]
    IncompleteCommand,
    #[error("invalid command")]
    InvalidCommand,
    #[error("unknown command")]
    UnknownCommand,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum CharClass {
    Whitespace,
    Punctuation,
    Regular,
}

pub trait CharClassify {
    fn class(self) -> CharClass;
}

impl CharClassify for char {
    fn class(self) -> CharClass {
        if self.is_whitespace() || self.is_ascii_whitespace() {
            CharClass::Whitespace
        } else if !self.is_alphanumeric() && self != '_' {
            CharClass::Punctuation
        } else {
            CharClass::Regular
        }
    }
}

trait RopeExt {
    fn index_of<P: Fn(char) -> bool>(&self, pred: P, start: usize) -> Option<usize>;
    fn last_index_of<P: Fn(char) -> bool>(&self, pred: P, start: usize) -> Option<usize>;
    fn dir_index_of<P: Fn(char) -> bool>(
        &self,
        pred: P,
        start: usize,
        dir: Direction,
    ) -> Option<usize> {
        match dir {
            Direction::Forward => self.index_of(pred, start),
            Direction::Backward => self.last_index_of(pred, start),
        }
    }
}

impl RopeExt for Rope {
    fn index_of<P: Fn(char) -> bool>(&self, pred: P, start: usize) -> Option<usize> {
        self.chars_at(start)
            .enumerate()
            .find(|(_, c)| pred(*c))
            .map(|(i, _)| start + i)
    }

    fn last_index_of<P: Fn(char) -> bool>(&self, pred: P, start: usize) -> Option<usize> {
        self.chars_at(start)
            .reversed()
            .enumerate()
            .find(|(_, c)| pred(*c))
            .map(|(i, _)| start - i - 1)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TextObject {
    Word,
    BigWord,
    Paragraph,
    Block(char),
}

fn matching_block_char(c: char) -> char {
    match c {
        '{' => '}',
        '(' => ')',
        '[' => ']',
        '<' => '>',
        '"' => '"',
        '\'' => '\'',
        _ => panic!("no matching block char for {}", c),
    }
}

impl TextObject {
    fn range(&self, buf: &Rope, cursor_index: usize, count: usize, include: bool) -> Range<usize> {
        // include = true->An, false->Inner
        match self {
            TextObject::Word | TextObject::BigWord => {
                let bigword = *self == TextObject::BigWord;
                let mut range = cursor_index..cursor_index;
                // find start of range
                let mut chars = buf
                    .chars_at(range.start + 1)
                    .reversed()
                    // .inspect(|i| println!("b-{}", i))
                    .map(CharClassify::class)
                    .map(|cc| {
                        if bigword && cc == CharClass::Punctuation {
                            CharClass::Regular
                        } else {
                            cc
                        }
                    })
                    .peekable();
                let starting_class = chars.next().unwrap();
                while chars
                    .peek()
                    .map(|cc| *cc == starting_class)
                    .unwrap_or(false)
                {
                    range.start -= 1;
                    chars.next();
                }
                // find end of range
                if !include && starting_class == CharClass::Whitespace {
                    return range;
                }
                range.end = range.start + 1;
                let mut chars = buf
                    .chars_at(range.end)
                    // .inspect(|i| println!("f-{}", i))
                    .map(CharClassify::class)
                    .map(|cc| {
                        if bigword && cc == CharClass::Punctuation {
                            CharClass::Regular
                        } else {
                            cc
                        }
                    })
                    .peekable();
                for i in 0..count {
                    while chars
                        .peek()
                        .map(|cc| *cc == starting_class)
                        .unwrap_or(false)
                    {
                        range.end += 1;
                        chars.next();
                    }
                    if i > 0 || include {
                        let class = if starting_class == CharClass::Whitespace {
                            *chars.peek().unwrap()
                        } else {
                            CharClass::Whitespace
                        };
                        while chars.peek().map(|cc| *cc == class).unwrap_or(false) {
                            range.end += 1;
                            chars.next();
                        }
                    }
                }
                range.end -= 1;
                range
            }
            TextObject::Block(open_char) => {
                println!("---");
                let mut range = cursor_index..cursor_index;

                let mut chars = buf.chars_at(range.start + 1).reversed().peekable();
                while chars.peek().map(|cc| cc != open_char).unwrap_or(false) {
                    chars.next();
                    range.start -= 1;
                }
                if !include {
                    range.start += 1;
                }
                range.end = range.start;

                let close_char = matching_block_char(*open_char);
                let mut chars = buf.chars_at(range.end).peekable();
                let mut count = 0;
                loop {
                    match chars.peek() {
                        Some(ch) if *ch == *open_char => {
                            count += 1;
                            println!("+ {count}");
                        }
                        Some(ch) if *ch == close_char => {
                            if count <= 1 {
                                println!("x {count}");
                                break;
                            } else {
                                count -= 1;
                                println!("- {count}");
                            }
                        }
                        Some(_) => {}
                        None => break,
                    }
                    range.end += 1;
                    println!("{:?} {}", chars.next(), range.end);
                }
                /*while chars.peek().map(|cc| *cc != mc).unwrap_or(false) {
                chars.next();
                range.end += 1;
                }*/
                if !include {
                    range.end -= 1;
                }

                range
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Direction {
    Forward,
    Backward,
}
impl Direction {
    fn reverse(&self) -> Direction {
        match self {
            Direction::Forward => Direction::Backward,
            Direction::Backward => Direction::Forward,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum MotionType {
    Char(Direction),
    Word(Direction),    // words
    BigWord(Direction), // WORDS
    EndOfWord(Direction),
    EndOfBigWord(Direction),
    NextChar {
        c: char,
        place_before: bool,
        direction: Direction,
    },
    RepeatNextChar {
        opposite: bool, // true -> reverse direction
    },
    WholeLine,
    Line(Direction),
    StartOfLine,
    EndOfLine,
    Paragraph,
    An(TextObject),
    Inner(TextObject),
    NextSearchMatch(Direction),
    Passthrough(usize, usize),
}

impl MotionType {
    pub fn inclusive(&self) -> bool {
        match self {
            MotionType::NextChar { .. } | MotionType::RepeatNextChar { .. } => true,
            MotionType::An(_) | MotionType::Inner(_) => true,
            MotionType::Passthrough(_, _) => true,
            MotionType::EndOfLine => true,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Motion {
    pub count: usize,
    pub mo: MotionType,
}

impl Motion {
    pub fn passthrough(r: &Range<usize>) -> Motion {
        Motion {
            count: 1,
            mo: MotionType::Passthrough(r.start, r.end),
        }
    }

    pub fn parse(
        c: &mut impl Iterator<Item = char>,
        opchar: Option<char>,
    ) -> Result<Motion, ParseError> {
        let first = c.next();
        if first.is_none() {
            return Err(ParseError::IncompleteCommand);
        }
        let mut ch = first.unwrap();

        let count = if ch.is_digit(10) {
            let mut num = ch.to_digit(10).unwrap() as usize;
            while let Some(dch) = c.next() {
                if let Some(d) = dch.to_digit(10) {
                    num = num * 10 + d as usize;
                } else {
                    ch = dch;
                    break;
                }
            }
            Some(num)
        } else {
            None
        };

        // println!("m {count:?} {first:?}");

        let txo = match ch {
            'h' => MotionType::Char(Direction::Backward),
            'j' => MotionType::Line(Direction::Forward),
            'k' => MotionType::Line(Direction::Backward),
            'l' => MotionType::Char(Direction::Forward),
            'w' => MotionType::Word(Direction::Forward),
            'b' => MotionType::Word(Direction::Backward),
            'W' => MotionType::BigWord(Direction::Forward),
            'B' => MotionType::BigWord(Direction::Backward),
            'e' => MotionType::EndOfWord(Direction::Forward),
            'E' => MotionType::EndOfBigWord(Direction::Forward),
            'n' => MotionType::NextSearchMatch(Direction::Forward),
            'N' => MotionType::NextSearchMatch(Direction::Backward),
            'g' => match c.next() {
                Some('e') => MotionType::EndOfWord(Direction::Backward),
                Some('E') => MotionType::EndOfBigWord(Direction::Backward),
                Some(';') => MotionType::RepeatNextChar { opposite: true },
                Some(_) => return Err(ParseError::UnknownCommand),
                None => return Err(ParseError::IncompleteCommand),
            },
            '^' => MotionType::StartOfLine,
            '$' => MotionType::EndOfLine,
            '_' => MotionType::WholeLine,
            tc if tc == 'f' || tc == 'F' || tc == 't' || tc == 'T' => MotionType::NextChar {
                c: c.next().ok_or(ParseError::IncompleteCommand)?,
                place_before: match tc {
                    'f' => false,
                    'F' => false,
                    't' => true,
                    'T' => true,
                    _ => unreachable!(),
                },
                direction: match tc {
                    'f' => Direction::Forward,
                    'F' => Direction::Backward,
                    't' => Direction::Forward,
                    'T' => Direction::Backward,
                    _ => unreachable!(),
                },
            },
            t @ ('i' | 'a') if opchar.is_some() => {
                let obj = match c.next() {
                    Some('w') => TextObject::Word,
                    Some('W') => TextObject::BigWord,
                    Some('p') => TextObject::Paragraph,
                    Some('{') | Some('}') => TextObject::Block('{'),
                    Some('(') | Some(')') => TextObject::Block('('),
                    Some('[') | Some(']') => TextObject::Block('['),
                    Some('<') | Some('>') => TextObject::Block('<'),
                    Some('"') => TextObject::Block('"'),
                    Some('\'') => TextObject::Block('\''),
                    Some(_) => return Err(ParseError::UnknownCommand),
                    None => return Err(ParseError::IncompleteCommand),
                };
                match t {
                    'i' => MotionType::Inner(obj),
                    'a' => MotionType::An(obj),
                    _ => unreachable!(),
                }
            }
            ';' => MotionType::RepeatNextChar { opposite: false },
            ',' => MotionType::RepeatNextChar { opposite: true },
            c if opchar.map(|opc| opc == c).unwrap_or(false) => MotionType::WholeLine,
            _ => return Err(ParseError::UnknownCommand),
        };
        Ok(Motion {
            count: count.unwrap_or(1),
            mo: txo,
        })
    }

    #[cfg(test)]
    fn range_without_find(
        &self,
        buf: &Rope,
        cursor_index: usize,
        multiplier: usize,
    ) -> Range<usize> {
        self.range(buf, cursor_index, multiplier, &mut None)
    }

    pub fn range(
        &self,
        buf: &Rope,
        cursor_index: usize,
        multiplier: usize,
        last_char_query: &mut Option<(char, bool, Direction)>,
    ) -> Range<usize> {
        match &self.mo {
            MotionType::Passthrough(s, e) => return *s..*e,
            MotionType::An(obj) => {
                return obj.range(buf, cursor_index, self.count * multiplier, true);
            }
            MotionType::Inner(obj) => {
                return obj.range(buf, cursor_index, self.count * multiplier, false);
            }
            _ => {}
        };
        let mut range = cursor_index..cursor_index;
        for _ in 0..(self.count * multiplier) {
            match &self.mo {
                MotionType::Char(Direction::Forward) => {
                    range.end = (range.end + 1).min(buf.len_chars());
                }

                MotionType::Char(Direction::Backward) => {
                    range.end = range.end.saturating_sub(1);
                }

                MotionType::Line(direction) => {
                    let cur_line_index = buf.char_to_line(range.end);
                    let new_line_index = match direction {
                        Direction::Forward => (cur_line_index + 1).min(buf.len_lines()),
                        Direction::Backward => cur_line_index.saturating_sub(1),
                    };
                    let start_of_new_line = buf.line_to_char(new_line_index);
                    let start_of_cur_line = buf.line_to_char(cur_line_index);
                    let new_line = buf.line(new_line_index);
                    // preserve column position if possible
                    range.end = start_of_new_line
                        + (range.end - start_of_cur_line).min(new_line.len_chars());
                }

                MotionType::StartOfLine => {
                    let cur_line_index = buf.char_to_line(range.end);
                    range.end = buf.line_to_char(cur_line_index);
                    let mut chars = buf
                        .line(cur_line_index)
                        .chars()
                        .map(CharClassify::class)
                        .peekable();
                    while chars
                        .peek()
                        .map_or(false, |cc| *cc == CharClass::Whitespace)
                    {
                        range.end += 1;
                        chars.next();
                    }
                }

                MotionType::EndOfLine => {
                    let cur_line_index = buf.char_to_line(range.end);
                    let cur_line = buf.line(cur_line_index);
                    range.end = buf.line_to_char(cur_line_index) + cur_line.len_chars() - 1;
                }

                MotionType::Word(Direction::Forward) => {
                    // is the character under the cursor alphanumeric+ or a 'other non-blank'?
                    if buf
                        .get_char(range.end)
                        .map(|c| c.is_alphanumeric() || c == '_')
                        .unwrap_or(false)
                    {
                        // find the next whitespace or non-blank char
                        let f = buf
                            .index_of(|sc| !(sc.is_alphanumeric() || sc == '_'), range.end)
                            .unwrap_or(range.end);
                        // the next word starts at either `f` or if `f` is whitespace, the next
                        // non-blank after `f`
                        range.end = if buf
                            .get_char(f)
                            .map(|c| c.is_ascii_whitespace())
                            .unwrap_or(false)
                        {
                            // println!("G");
                            buf.index_of(|sc| !sc.is_ascii_whitespace(), f).unwrap_or(f)
                        } else {
                            f
                        };
                    } else {
                        // "a sequence of other non-blank characters"
                        // find the next blank or alphanumeric+ char
                        let f = buf
                            .index_of(
                                |sc| sc.is_ascii_whitespace() || sc.is_alphanumeric() || sc == '_',
                                range.end + 1,
                            )
                            .unwrap_or(range.end);
                        // the next word starts at `f` or if `f` is whitespace, at the next
                        // non-blank char after `f`
                        range.end = if buf
                            .get_char(f)
                            .map(|c| c.is_ascii_whitespace())
                            .unwrap_or(false)
                        {
                            // println!("G");
                            buf.index_of(|sc| !sc.is_ascii_whitespace(), f).unwrap_or(f)
                        } else {
                            f
                        };
                    }
                }

                MotionType::Word(Direction::Backward) => {
                    let mut chars = buf
                        .chars_at((range.end + 1).min(buf.len_chars()))
                        .reversed()
                        .inspect(|i| println!("{i} {:?}", i.class()))
                        .map(CharClassify::class)
                        .peekable();
                    if let Some(_) = chars.next() {
                        range.end -= 1;
                        while let Some(CharClass::Whitespace) = chars.peek() {
                            chars.next();
                            range.end -= 1;
                        }
                        let scls = chars.peek().cloned().unwrap();
                        while range.end > 0 && chars.next().map_or(false, |x| x == scls) {
                            range.end -= 1;
                        }
                        if range.end > 0 {
                            range.end += 1;
                        }
                    } else {
                        range.end = 0;
                    }
                }

                MotionType::BigWord(direction) => {
                    let next_blank = buf
                        .dir_index_of(|sc| sc.is_ascii_whitespace(), range.start, *direction)
                        .unwrap_or(range.start);
                    range.end = match *direction {
                        Direction::Forward => buf
                            .index_of(|sc| !sc.is_ascii_whitespace(), next_blank)
                            .unwrap_or(next_blank),
                        Direction::Backward => buf
                            .last_index_of(|sc| sc.is_ascii_whitespace(), next_blank)
                            .map(|i| i + 1)
                            .unwrap_or(0),
                    };
                }

                MotionType::EndOfWord(Direction::Forward)
                | MotionType::EndOfBigWord(Direction::Forward) => {
                    let mut chars: Box<dyn Iterator<Item = CharClass>> =
                        Box::new(buf.chars_at(range.end).map(CharClassify::class));
                    if let MotionType::EndOfBigWord(_) = self.mo {
                        chars = Box::new(chars.map(|c| match c {
                            CharClass::Punctuation => CharClass::Regular,
                            _ => c,
                        }));
                    }
                    let mut chars = chars.peekable();
                    if let Some(starting_class) = chars.next() {
                        range.end += 1;
                        if starting_class != CharClass::Whitespace
                            && chars
                                .peek()
                                .map(|cc| *cc == starting_class)
                                .unwrap_or(false)
                        {
                            while chars.next().map_or(false, |cc| cc == starting_class) {
                                range.end += 1;
                            }
                        } else {
                            while let Some(CharClass::Whitespace) = chars.peek() {
                                chars.next();
                                range.end += 1;
                            }
                            let scls = chars.peek().cloned().unwrap();
                            while range.end < buf.len_chars()
                                && chars.next().map_or(false, |x| x == scls)
                            {
                                range.end += 1;
                            }
                        }
                        range.end -= 1;
                    } else {
                        range.end = 0;
                    }
                }

                // of course, the most arcane is the simplest
                MotionType::EndOfWord(Direction::Backward)
                | MotionType::EndOfBigWord(Direction::Backward) => {
                    let mut chars: Box<dyn Iterator<Item = CharClass>> = Box::new(
                        buf.chars_at(range.end + 1)
                            .reversed()
                            .map(CharClassify::class),
                    );
                    if let MotionType::EndOfBigWord(_) = self.mo {
                        chars = Box::new(chars.map(|c| match c {
                            CharClass::Punctuation => CharClass::Regular,
                            _ => c,
                        }));
                    }
                    let mut chars = chars.peekable();
                    if let Some(starting_class) = chars.next() {
                        range.end -= 1;
                        if starting_class != CharClass::Whitespace {
                            while chars.peek().map_or(false, |cc| *cc == starting_class) {
                                chars.next();
                                range.end -= 1;
                            }
                        }

                        while chars
                            .peek()
                            .map_or(false, |cc| *cc == CharClass::Whitespace)
                        {
                            chars.next();
                            range.end -= 1;
                        }
                    } else {
                        range.end = 0;
                    }
                }

                MotionType::NextChar {
                    c,
                    place_before,
                    direction,
                } => {
                    *last_char_query = Some((*c, *place_before, *direction));
                    range.end = buf
                        .dir_index_of(
                            |cc| cc == *c,
                            match direction {
                                Direction::Forward => range.end + 1,
                                Direction::Backward => range.end - 1,
                            },
                            *direction,
                        )
                        .unwrap_or(range.end);
                    if *place_before {
                        match direction {
                            Direction::Forward => range.end -= 1,
                            Direction::Backward => range.end += 1,
                        }
                    }
                }

                MotionType::RepeatNextChar { opposite } => {
                    if let Some((c, place_before, direction)) = last_char_query.as_ref() {
                        range = (Motion {
                            count: self.count,
                            mo: MotionType::NextChar {
                                c: *c,
                                place_before: *place_before,
                                direction: if *opposite {
                                    direction.reverse()
                                } else {
                                    *direction
                                },
                            },
                        })
                        .range(
                            buf,
                            cursor_index,
                            multiplier,
                            last_char_query,
                        );
                        //*last_char_query = Some((c, place_before, direction));
                    } else {
                        // can't fail but probably the user would like to know what's going on
                    }
                }

                MotionType::WholeLine => {
                    let cur_line_index = buf.char_to_line(range.end);
                    let cur_line = buf.line(cur_line_index);
                    range.start = buf.line_to_char(cur_line_index);
                    range.end = range.start + cur_line.len_chars();
                }

                // MotionType::NextSearchMatch(direction) => {
                //     range.end = buf
                //         .next_query_index(range.start + 1, *direction, true)
                //         .unwrap_or(range.start);
                // }
                _ => unimplemented!(),
            }
        }
        range
    }
}

#[derive(Debug)]
pub enum Command {
    Move(Motion),
    Insert { at: Option<Motion> },
    ReplaceChar(char),
    Change(Motion),
    Delete(Motion),
    Copy(Motion),
    Put { consume: bool },
}

impl Command {
    pub fn parse(cmd: &str) -> Result<Command, ParseError> {
        match cmd.chars().next() {
            Some('i') => Ok(Command::Insert { at: None }),
            Some('a') => Ok(Command::Insert {
                at: Some(Motion {
                    count: 1,
                    mo: MotionType::Char(Direction::Forward),
                }),
            }),
            Some('I') => Ok(Command::Insert {
                at: Some(Motion {
                    count: 1,
                    mo: MotionType::StartOfLine,
                }),
            }),
            Some('A') => Ok(Command::Insert {
                at: Some(Motion {
                    count: 1,
                    mo: MotionType::EndOfLine,
                }),
            }),
            Some('p') => Ok(Command::Put { consume: true }),
            Some('P') => Ok(Command::Put { consume: false }),
            Some('r') => Ok(Command::ReplaceChar(
                cmd.chars().nth(1).ok_or(ParseError::IncompleteCommand)?,
            )),
            Some('x') => Ok(Command::Delete(Motion {
                count: 1,
                mo: MotionType::Char(Direction::Forward),
            })),
            Some('o') => todo!(),
            Some(op @ ('d' | 'c' | 'y')) => {
                let m = Motion::parse(&mut cmd.chars().skip(1), Some(op))?;
                match op {
                    'd' => Ok(Command::Delete(m)),
                    'c' => Ok(Command::Change(m)),
                    'y' => Ok(Command::Copy(m)),
                    _ => Err(ParseError::UnknownCommand),
                }
            }
            Some(_) => Ok(Command::Move(Motion::parse(&mut cmd.chars(), None)?)),
            None => Err(ParseError::IncompleteCommand),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_line_test_buffer() -> Rope {
        let mut b = Rope::from_str("abc\ndef\nghi\n");
        b
    }
    fn create_word_test_buffer() -> Rope {
        Rope::from_str("word\nw0rd w##d ++++ word\n")
    }

    #[test]
    fn txo_char() {
        let b = create_line_test_buffer();
        let mo = Motion {
            mo: MotionType::Char(Direction::Forward),
            count: 1,
        };
        assert_eq!(mo.range_without_find(&b, 4, 1), 4..5);
    }

    #[test]
    fn txo_line() {
        let b = create_line_test_buffer();
        let mo = Motion {
            mo: MotionType::Line(Direction::Forward),
            count: 1,
        };
        assert_eq!(mo.range_without_find(&b, 4, 1), 4..8);
    }

    #[test]
    fn txo_start_of_line() {
        let b = create_line_test_buffer();
        let mo = Motion {
            mo: MotionType::StartOfLine,
            count: 1,
        };
        assert_eq!(mo.range_without_find(&b, 4, 1), 4..4);
    }

    #[test]
    fn txo_end_of_line() {
        let b = create_line_test_buffer();
        let mo = Motion {
            mo: MotionType::EndOfLine,
            count: 1,
        };
        assert_eq!(mo.range_without_find(&b, 4, 1), 4..7);
    }

    #[test]
    fn txo_line_backward() {
        let b = create_line_test_buffer();
        let mo = Motion {
            mo: MotionType::Line(Direction::Backward),
            count: 1,
        };
        assert_eq!(mo.range_without_find(&b, 4, 1), 4..0);
    }

    fn run_repeated_test<'a>(
        b: &Rope,
        cursor_index: &mut usize,
        mo: &Motion,
        correct_ends: impl Iterator<Item = &'a usize>,
        assert_msg: &str,
    ) {
        for (i, cwb) in correct_ends.enumerate() {
            let r = mo.range_without_find(&b, *cursor_index, 1);
            println!("actual:");
            println!("{}", b.to_string().escape_debug());
            for c in b.chars().enumerate() {
                if c.0 == r.end {
                    break;
                } else {
                    if c.1.is_control() {
                        print!("\\\\");
                    } else {
                        print!("-");
                    }
                }
            }
            println!("^ {}", r.end);

            if r.end != *cwb {
                println!("expected:");
                println!("{}", b.to_string().escape_debug());
                for c in b.chars().enumerate() {
                    if c.0 == *cwb {
                        break;
                    } else {
                        if c.1.is_control() {
                            print!("\\\\");
                        } else {
                            print!("-");
                        }
                    }
                }
                println!("^ {}", *cwb);
                panic!("{} i={} ci={}", assert_msg, i, *cursor_index);
            }
            *cursor_index = r.end;
        }
    }

    fn run_repeated_test_then_offset<'a>(
        b: &mut Rope,
        cursor_index: &mut usize,
        mo: &Motion,
        correct_ends: impl Iterator<Item = &'a usize>,
        offset: isize,
        assert_msg: &str,
    ) {
        for (i, cwb) in correct_ends.enumerate() {
            let r = mo.range_without_find(&b, *cursor_index, 1);
            assert_eq!(r.end, *cwb, "{} i={}", assert_msg, i);
            *cursor_index = (r.end as isize + offset) as usize;
        }
    }

    #[test]
    fn txo_word_no_spaces() {
        let mut b = Rope::from_str("word+++word+++ +ope");
        let mo = Motion {
            mo: MotionType::Word(Direction::Forward),
            count: 1,
        };
        let mut ci = 0;
        run_repeated_test(&mut b, &mut ci, &mo, [4, 7, 11, 15].iter(), "forward");
        let mo = Motion {
            mo: MotionType::Word(Direction::Backward),
            count: 1,
        };
        run_repeated_test(&mut b, &mut ci, &mo, [11, 7, 4, 0].iter(), "backward");
    }

    #[test]
    fn txo_word() {
        let mut b = create_word_test_buffer();
        let mo = Motion {
            mo: MotionType::Word(Direction::Forward),
            count: 1,
        };
        let mut ci = 0;
        run_repeated_test(
            &mut b,
            &mut ci,
            &mo,
            [5, 10, 11, 13, 15, 20].iter(),
            "forward",
        );

        let mo = Motion {
            mo: MotionType::Word(Direction::Backward),
            count: 1,
        };
        run_repeated_test(
            &mut b,
            &mut ci,
            &mo,
            [15, 13, 11, 10, 5, 0].iter(),
            "backward",
        );
    }

    #[test]
    fn txo_big_word() {
        let mut b = create_word_test_buffer();
        let mo = Motion {
            mo: MotionType::BigWord(Direction::Forward),
            count: 1,
        };
        let mut ci = 0;
        run_repeated_test(&mut b, &mut ci, &mo, [5, 10, 15].iter(), "forward");

        let mo = Motion {
            mo: MotionType::BigWord(Direction::Backward),
            count: 1,
        };
        run_repeated_test(&mut b, &mut ci, &mo, [10, 5, 0].iter(), "backward");
    }

    #[test]
    fn txo_end_word() {
        let mut b = create_word_test_buffer();
        let mo = Motion {
            mo: MotionType::EndOfWord(Direction::Forward),
            count: 1,
        };
        let mut ci = 0;
        run_repeated_test(
            &mut b,
            &mut ci,
            &mo,
            [3, 8, 10, 12, 13, 18, 23].iter(),
            "forward",
        );

        let mo = Motion {
            mo: MotionType::EndOfWord(Direction::Backward),
            count: 1,
        };
        run_repeated_test(
            &mut b,
            &mut ci,
            &mo,
            [18, 13, 12, 10, 8, 3].iter(),
            "backward",
        );
    }

    #[test]
    fn txo_end_big_word() {
        let mut b = create_word_test_buffer();
        let mo = Motion {
            mo: MotionType::EndOfBigWord(Direction::Forward),
            count: 1,
        };
        let mut ci = 0;
        run_repeated_test(&mut b, &mut ci, &mo, [3, 8, 13, 18, 23].iter(), "forward");

        let mo = Motion {
            mo: MotionType::EndOfBigWord(Direction::Backward),
            count: 1,
        };
        run_repeated_test(&mut b, &mut ci, &mo, [18, 13, 8, 3].iter(), "backward");
    }

    #[test]
    fn txo_find_next_on() {
        let mut b = Rope::from_str("so!me s!ample tex!t");
        let correct = [2, 7, 17];
        let mo = Motion {
            mo: MotionType::NextChar {
                c: '!',
                place_before: false,
                direction: Direction::Forward,
            },
            count: 1,
        };
        let mut ci = 0;
        run_repeated_test(&mut b, &mut ci, &mo, correct.iter(), "forward, place on");

        let mo = Motion {
            mo: MotionType::NextChar {
                c: '!',
                place_before: false,
                direction: Direction::Backward,
            },
            count: 1,
        };
        run_repeated_test(
            &mut b,
            &mut ci,
            &mo,
            correct.iter().rev().skip(1),
            "backward, place on",
        );
    }

    #[test]
    fn txo_find_next_before() {
        let mut b = Rope::from_str("so!me s!ample tex!t");
        let mo = Motion {
            mo: MotionType::NextChar {
                c: '!',
                place_before: true,
                direction: Direction::Forward,
            },
            count: 1,
        };
        let mut ci = 0;
        run_repeated_test_then_offset(
            &mut b,
            &mut ci,
            &mo,
            [1, 6, 16].iter(),
            1,
            "forward, place before",
        );

        let mo = Motion {
            mo: MotionType::NextChar {
                c: '!',
                place_before: true,
                direction: Direction::Backward,
            },
            count: 1,
        };
        run_repeated_test_then_offset(
            &mut b,
            &mut ci,
            &mo,
            [8, 3].iter(),
            -1,
            "backward, place before",
        );
    }

    #[test]
    fn txo_object_a_word() {
        let mut b = Rope::from_str(" word   w0rd wr+d");
        let mut mo = Motion {
            mo: MotionType::An(TextObject::Word),
            count: 1,
        };
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..7);
        mo.count += 1;
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..12);
        mo.count += 1;
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..14);

        mo.count = 1;
        assert_eq!(mo.range_without_find(&b, 6, 1), 5..11);
    }

    #[test]
    fn txo_object_inner_word() {
        let mut b = Rope::from_str(" word  word+ ");
        let mut mo = Motion {
            mo: MotionType::Inner(TextObject::Word),
            count: 1,
        };
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..4);
        mo.count += 1;
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..6);
        mo.count += 1;
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..10);

        mo.count = 1;
        assert_eq!(mo.range_without_find(&b, 6, 1), 5..6);
    }

    #[test]
    fn txo_object_a_bigword() {
        let mut b = Rope::from_str(" wor+   w0rd wr+d");
        let mut mo = Motion {
            mo: MotionType::An(TextObject::BigWord),
            count: 1,
        };
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..7);
        mo.count += 1;
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..12);
        mo.count += 1;
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..16);

        mo.count = 1;
        assert_eq!(mo.range_without_find(&b, 6, 1), 5..11);
    }

    #[test]
    fn txo_object_inner_bigword() {
        let mut b = Rope::from_str(" w--d  w--d+ ");
        let mut mo = Motion {
            mo: MotionType::Inner(TextObject::BigWord),
            count: 1,
        };
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..4);
        mo.count += 1;
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..6);
        mo.count += 1;
        assert_eq!(mo.range_without_find(&b, 3, 1), 1..12); // this doesn't quite agree with Vim, but it seems questionable either way

        mo.count = 1;
        assert_eq!(mo.range_without_find(&b, 6, 1), 5..6);
    }

    #[test]
    fn txo_object_a_block() {
        let mut b = Rope::from_str("<(bl(o)ck) {\nblock\n}>");
        let mut mo = Motion {
            mo: MotionType::An(TextObject::Block('<')),
            count: 1,
        };

        let mut cursor_index = 0;
        assert_eq!(mo.range_without_find(&b, cursor_index, 1), 0..20, "on <");
        cursor_index += 3;
        assert_eq!(mo.range_without_find(&b, cursor_index, 1), 0..20, "in <");

        cursor_index = 1;
        mo.mo = MotionType::An(TextObject::Block('('));
        assert_eq!(
            mo.range_without_find(&b, cursor_index, 1),
            1..9,
            "on first ("
        );
        cursor_index += 2;
        assert_eq!(
            mo.range_without_find(&b, cursor_index, 1),
            1..9,
            "in first ("
        );

        cursor_index += 2;
        assert_eq!(
            mo.range_without_find(&b, cursor_index, 1),
            4..6,
            "in nested ("
        );

        cursor_index = 15;
        mo.mo = MotionType::An(TextObject::Block('{'));
        assert_eq!(mo.range_without_find(&b, cursor_index, 1), 11..19, "in {{");
    }

    #[test]
    fn txo_object_inner_block() {
        let mut b = Rope::from_str("<(bl(o)ck) {\nblock\n}>");
        let mut mo = Motion {
            mo: MotionType::Inner(TextObject::Block('<')),
            count: 1,
        };

        let mut cursor_index = 0;
        assert_eq!(mo.range_without_find(&b, cursor_index, 1), 1..19, "on <");
        cursor_index += 3;
        assert_eq!(mo.range_without_find(&b, cursor_index, 1), 1..19, "in <");

        cursor_index = 1;
        mo.mo = MotionType::Inner(TextObject::Block('('));
        assert_eq!(
            mo.range_without_find(&b, cursor_index, 1),
            2..8,
            "on first ("
        );
        cursor_index += 2;
        assert_eq!(
            mo.range_without_find(&b, cursor_index, 1),
            2..8,
            "in first ("
        );

        cursor_index += 2;
        assert_eq!(
            mo.range_without_find(&b, cursor_index, 1),
            5..5,
            "in nested ("
        );

        cursor_index = 15;
        mo.mo = MotionType::Inner(TextObject::Block('{'));
        assert_eq!(mo.range_without_find(&b, cursor_index, 1), 12..18, "in {{");
    }
}
