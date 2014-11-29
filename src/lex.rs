extern crate std;

use std::io::BufferedReader;
use std::io::IoResult;

#[deriving(PartialEq, Show)]
pub enum Token<'a> {
    Tag(&'a str),
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
            TokenType::Tag => Token::Tag(s),
            TokenType::Argument => Token::Argument(s),
            TokenType::Comment => Token::Comment(s)
        }
    }
}

pub struct TokenIterator<R> {
    buffered_reader: BufferedReader<R>,
    state: State,
    buffer: String,
    token_type: Option<TokenType>
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

    fn next(&mut self) -> Option<IoResult<Token>> {
        let mut result = None;

        if self.token_type.is_some() {
            self.token_type = None;
            self.buffer.clear();
        }

        while result.is_none() {
            match self.buffered_reader.read_char() {
                Ok(c) => {
                    self.push_char(c);

                    if let Some(token_type) = self.token_type {
                        return Some(Result::Ok(Token::new(token_type,
                                                          self.buffer.as_slice())));
                    }
                }
                Err(ref e) => {
                    if e.kind == std::io::IoErrorKind::EndOfFile {
                        self.push_char('\n');

                        // TODO(bishop): deduplicate
                        if let Some(token_type) = self.token_type {
                            return Some(Result::Ok(Token::new(token_type,
                                                              self.buffer.as_slice())));
                        } else {
                            return None;
                        }

                    }
                    result = Some(Result::Err(e.clone()));
                }
            }
        }

        result
    }
}

pub fn read_obj<R: Reader>(reader: R) -> TokenIterator<R> {
    let mut iter = TokenIterator {
        buffered_reader: BufferedReader::new(reader),
        state: State::StartOfLine,
        buffer: String::new(),
        token_type: None
    };
    iter
}

fn str_reader(s: &'static str) -> std::io::BufReader {
    std::io::BufReader::new(s.as_bytes())
}

#[test]
fn test_tag() {
    let mut iter = read_obj(str_reader("a\n"));
    assert!(iter.next().unwrap().unwrap() == Token::Tag("a"));
    assert!(iter.next() == None);
}

#[test]
fn test_tag_and_arguments() {
    let mut iter = read_obj(str_reader("a b c\n"));
    assert!(iter.next().unwrap().unwrap() == Token::Tag("a"));
    assert!(iter.next().unwrap().unwrap() == Token::Argument("b"));
    assert!(iter.next().unwrap().unwrap() == Token::Argument("c"));
    assert!(iter.next() == None);
}

#[test]
fn test_tag_no_newline() {
    let mut iter = read_obj(str_reader("a"));
    assert!(iter.next().unwrap().unwrap() == Token::Tag("a"));
    assert!(iter.next() == None);
}

#[test]
fn test_line_comment() {
    let mut iter = read_obj(str_reader("# comment\n"));
    assert!(iter.next().unwrap().unwrap() == Token::Comment(" comment"));
    assert!(iter.next() == None);
}


#[test]
fn test_comment_after_tag() {
    let mut iter = read_obj(str_reader("v # comment\n"));
    assert!(iter.next().unwrap().unwrap() == Token::Tag("v"));
    assert!(iter.next().unwrap().unwrap() == Token::Comment(" comment"));
    assert!(iter.next() == None);
}
