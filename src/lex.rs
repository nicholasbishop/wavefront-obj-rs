extern crate std;

use std::io::BufferedReader;
use std::io::IoErrorKind;
use std::io::IoResult;

#[deriving(PartialEq, Show)]
pub enum Tag<'a> {
    F,
    V,
    Vn,
    Vt,

    Unknown(&'a str)
}

impl<'a> Tag<'a> {
    // TODO(bishop): probably a better way to do this
    fn from_str(s: &'a str) -> Tag<'a> {
        if s == "f" {
            Tag::F
        } else if s == "v" {
            Tag::V
        } else if s == "vn" {
            Tag::Vn
        } else if s == "vt" {
            Tag::Vt
        } else {
            Tag::Unknown(s)
        }
    }
}

#[deriving(PartialEq, Show)]
pub enum Token<'a> {
    Tag(Tag<'a>),
    Argument(&'a str),
    Comment(&'a str)
}

#[deriving(PartialEq)]
enum State {
    StartOfLine,
    Tag,
    Argument,
    Comment,
    EndOfFile
}

#[deriving(PartialEq)]
enum TokenType {
    Tag,
    Argument,
    Comment
}

impl<'a> Token<'a> {
    fn new(token_type: TokenType, s: &'a str) -> Token<'a> {
        match token_type {
            TokenType::Tag => Token::Tag(Tag::from_str(s)),
            TokenType::Argument => Token::Argument(s),
            TokenType::Comment => Token::Comment(s)
        }
    }
}

pub struct TokenIterator<R> {
    buffered_reader: BufferedReader<R>,
    state: State,
    buffer: String,
    token_type: Option<TokenType>,
    err: Option<std::io::IoError>
}

fn comment_char(c: char) -> bool {
    c == '#'
}

impl<R: Reader> TokenIterator<R> {
    /// Read a character, updating the iterator state accordingly
    fn push_char(&mut self, c: char) {
        assert!(self.token_type.is_none());

        match self.state {
            State::StartOfLine => {
                if comment_char(c) {
                    self.state = State::Comment;
                } else if !c.is_whitespace() {
                    self.buffer.push(c);
                    self.state = State::Tag;
                }
            }
            State::Tag => {
                if comment_char(c) {
                    self.state = State::Comment;
                } else if c.is_whitespace() {
                    if c == '\n' {
                        self.state = State::StartOfLine;
                        self.token_type = Some(TokenType::Tag);
                    } else {
                        self.state = State::Argument;
                        self.token_type = Some(TokenType::Tag);
                    }
                } else {
                    self.buffer.push(c);
                }
            }
            State::Argument => {
                // TODO: swallow extra whitespace somewhere here

                if comment_char(c) {
                    self.state = State::Comment;
                } else if c.is_whitespace() {
                    if c == '\n' {
                        self.state = State::StartOfLine;
                        self.token_type = Some(TokenType::Argument);
                    } else {
                        self.state = State::Argument;
                        self.token_type = Some(TokenType::Argument);
                    }
                } else {
                    self.buffer.push(c);
                }
            }
            State::Comment => {
                if c == '\n' {
                    self.state = State::StartOfLine;
                    self.token_type = Some(TokenType::Comment);
                } else {
                    self.buffer.push(c);
                }
            }
            State::EndOfFile => {
                
            }
        }
    }

    fn next(&mut self) -> IoResult<Token> {
        if self.token_type.is_some() {
            self.token_type = None;
            self.buffer.clear();
        }

        if self.state == State::EndOfFile {
            // TODO(bishop)
            return Result::Err(self.err.as_ref().unwrap().clone());
        }

        loop {
            match self.buffered_reader.read_char() {
                Ok(c) => {
                    self.push_char(c);

                    if let Some(token_type) = self.token_type {
                        return Result::Ok(Token::new(token_type,
                                                     self.buffer.as_slice()));
                    }
                }
                Err(ref e) => {
                    self.err = Some(e.clone());
                    if e.kind == std::io::IoErrorKind::EndOfFile {
                        self.push_char('\n');

                        // TODO(bishop): deduplicate
                        if let Some(token_type) = self.token_type {
                            self.state = State::EndOfFile;
                            return Result::Ok(Token::new(token_type,
                                                         self.buffer.as_slice()));
                        } else {
                            return Result::Err(e.clone());
                        }

                    }
                    return Result::Err(e.clone());
                }
            }
        }
    }
}

pub fn read_obj<R: Reader>(reader: R) -> TokenIterator<R> {
    let mut iter = TokenIterator {
        buffered_reader: BufferedReader::new(reader),
        state: State::StartOfLine,
        buffer: String::new(),
        token_type: None,
        err: None
    };
    iter
}

fn str_reader(s: &'static str) -> std::io::BufReader {
    std::io::BufReader::new(s.as_bytes())
}

// TODO
fn iter_eof<R: Reader>(iter: &mut TokenIterator<R>) -> bool {
    if let Result::Err(e) = iter.next() {
        e.kind == IoErrorKind::EndOfFile
    } else {
        false
    }
}

#[test]
fn test_tag() {
    let mut iter = read_obj(str_reader("v\n"));
    assert!(iter.next().unwrap() == Token::Tag(Tag::V));
    assert!(iter_eof(&mut iter));
}

#[test]
fn test_unknown_tag() {
    let mut iter = read_obj(str_reader("foo\n"));
    assert!(iter.next().unwrap() == Token::Tag(Tag::Unknown("foo")));
    assert!(iter_eof(&mut iter));
}

#[test]
fn test_tag_and_arguments() {
    let mut iter = read_obj(str_reader("v b c\n"));
    assert!(iter.next().unwrap() == Token::Tag(Tag::V));
    assert!(iter.next().unwrap() == Token::Argument("b"));
    assert!(iter.next().unwrap() == Token::Argument("c"));
    assert!(iter_eof(&mut iter));
}

#[test]
fn test_tag_no_newline() {
    let mut iter = read_obj(str_reader("v"));
    assert!(iter.next().unwrap() == Token::Tag(Tag::V));
    assert!(iter_eof(&mut iter));
}

#[test]
fn test_line_comment() {
    let mut iter = read_obj(str_reader("# comment\n"));
    assert!(iter.next().unwrap() == Token::Comment(" comment"));
    assert!(iter_eof(&mut iter));
}


#[test]
fn test_comment_after_tag() {
    let mut iter = read_obj(str_reader("v # comment\n"));
    assert!(iter.next().unwrap() == Token::Tag(Tag::V));
    assert!(iter.next().unwrap() == Token::Comment(" comment"));
    assert!(iter_eof(&mut iter));
}
