extern crate std;

use std::io::BufferedReader;
use std::io::IoResult;

#[deriving(PartialEq, Show)]
pub enum Token {
    Tag,
    Argument,
    Comment
}

#[deriving(PartialEq)]
enum State {
    StartOfLine,
    Tag,
    Argument,
    Comment,
    EndOfFile
}

pub struct TokenIterator<'a, R: 'a> {
    buffered_reader: BufferedReader<R>,
    state: State,
    buffer: String
    //line_iter: std::iter::Enumerate<std::io::Lines<'a, std::io::buffered::BufferedReader<R>>>,

    // line_iter: Option<std::io::Lines<'a, BufferedReader<R>>>,
    // line: Option<String>,
    // word_iter: Option<WordIterator<'a>>
}

fn comment_char(c: char) -> bool {
    c == '#'
}

impl<'a, R: Reader> TokenIterator<'a, R> {
//     fn init_iters(&'a mut self) {
//         self.line_iter = Some(self.buffered_reader.lines());
//         self.line = self.line_iter.unwrap().next();
//         self.word_iter = self.line.unwrap().words();
//     }

    fn handle_char(&mut self, c: char) -> Option<Token> {
        let mut result = None;

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
                        self.buffer.clear();
                        result = Some(Token::Tag);
                    } else {
                        self.state = State::Argument;
                        self.buffer.clear();
                        result = Some(Token::Argument);
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
                        self.buffer.clear();
                        result = Some(Token::Argument);
                    } else {
                        self.state = State::Argument;
                        self.buffer.clear();
                        result = Some(Token::Argument);
                    }
                } else {
                    self.buffer.push(c);
                }
            }
            State::Comment => {
                if c == '\n' {
                    self.state = State::StartOfLine;
                    self.buffer.clear();
                    result = Some(Token::Comment);
                } else {
                    self.buffer.push(c);
                }
            }
            State::EndOfFile => {
                
            }
        }

        result
    }
}

impl<'a, R: Reader> Iterator<IoResult<Token>> for TokenIterator<'a, R> {
    fn next(&mut self) -> Option<IoResult<Token>> {
        let mut result = None;

        while result.is_none() {
            // TODO
            let ioc = self.buffered_reader.read_char();
            match ioc {
                Ok(c) => {
                    if let Some(token) = self.handle_char(c) {
                        result = Some(Result::Ok(token));
                    }
                }
                Err(ref e) => {
                    // println!("{}", e);
                    // panic!("blah");
                    // TODO
                    if e.kind == std::io::IoErrorKind::EndOfFile {
                        return None;
                    }
                    result = Some(Result::Err(e.clone()));
                }
            }
        }

        result
    }
}

// impl<'a, LineIter> TokenIterator<'a, LineIter> {
//     fn new(line_iter: LineIter) -> TokenIterator<'a, LineIter> {
//         TokenIterator { line_iter: line_iter }
//     }
// }

// impl<'a, R: Reader> Iterator<Token> for TokenIterator<'a, R> {
//     fn next(&mut self) -> Option<Token> {
//         // if let Some(result) = self.reader.next() {
            
//         // }
//     }
// }

pub fn read_obj<'a, R: Reader>(reader: R) -> TokenIterator<'a, R> {
    let mut iter = TokenIterator {
        buffered_reader: BufferedReader::new(reader),
        state: State::StartOfLine,
        buffer: String::new()
    };
    iter
}

fn str_reader(s: &'static str) -> std::io::BufReader {
    std::io::BufReader::new(s.as_bytes())
}

#[test]
fn test_tag() {
    let mut iter = read_obj(str_reader("a\n"));
    assert!(iter.next().unwrap().unwrap() == Token::Tag);
    assert!(iter.next() == None);
}

#[test]
fn test_tag_and_argument() {
    let mut iter = read_obj(str_reader("a b\n"));
    assert!(iter.next().unwrap().unwrap() == Token::Tag);
    assert!(iter.next().unwrap().unwrap() == Token::Argument);
    assert!(iter.next() == None);
}

#[test]
fn test_tag_no_newline() {
    let mut iter = read_obj(str_reader("a"));
    assert!(iter.next().unwrap().unwrap() == Token::Tag);
    assert!(iter.next() == None);
}
