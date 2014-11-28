#![feature(if_let)]
#![feature(slicing_syntax)]

use std::io::BufferedReader;
use std::num::from_int;
use std::ops::Sub;
use std::str::FromStr;

mod lex;

pub enum CallbackResult {
    Continue,
    Stop
}

pub enum ErrorType {
    InvalidName,
    TooManyVertexComponents,
    NotEnoughVertexComponents
}

pub struct Line<'a> {
    pub text: &'a String,
    pub number: uint
}

pub struct Error<'a> {
    pub line: Line<'a>,
    pub error: ErrorType
}

impl<'a> Error<'a> {
    fn new(line: Line<'a>, error: ErrorType) -> Error<'a> {
        Error {
            line: line,
            error: error
        }
    }
}

type Words<'a> = std::iter::Filter<'a, &'a str, 
                                   std::str::CharSplits<'a, |char|:'a -> bool>>;

pub struct ElementIterator<'a> {
    iter: Words<'a>
}

impl<'a, Index: FromStr + 
                Sub<Index, Index> + 
                FromPrimitive> Iterator<Index> for ElementIterator<'a> {
    fn next(&mut self) -> Option<Index> {
        if let Some(word) = self.iter.next() {
            let op_val : Option<Index> = from_str(word);
            if let Some(val) = op_val {
                let op_one : Option<Index> = from_int(1);
                if let Some(one) = op_one {
                    Some(val.sub(&one))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub trait Importer<Real, Index> {
    fn comment(&mut self, _line: &str) -> CallbackResult {
        CallbackResult::Continue
    }

    fn error(&mut self, _error: Error) -> CallbackResult {
        CallbackResult::Continue
    }

    fn v(&mut self, _x: Real, _y: Real, _z: Real,
         _w: Option<Real>) -> CallbackResult {
        CallbackResult::Continue
    }

    fn vt(&mut self, _u: Real, _v: Real, _w: Option<Real>) -> CallbackResult {
        CallbackResult::Continue
    }

    fn f(&mut self, _iter: ElementIterator) -> CallbackResult {
        CallbackResult::Continue
    }
}

fn read_real<Real: FromStr>(word: Option<&str>) -> Option<Real> {
    match word {
        Some(val) => from_str(val),
        None => None
    }
}

fn read_obj_v<Real: FromStr, Index>(mut words: Words,
                                    importer: &mut Importer<Real, Index>,
                                    line: Line) {
    let ox = read_real::<Real>(words.next());
    let oy = read_real::<Real>(words.next());
    let oz = read_real::<Real>(words.next());
    match (ox, oy, oz) {
        (Some(x), Some(y), Some(z)) => {
            let ow = read_real::<Real>(words.next());
            let junk = words.next();
            if junk.is_some() {
                importer.error(
                    Error::new(line, ErrorType::TooManyVertexComponents));
            } else {
                importer.v(x, y, z, ow);
            }
        }
        _ => {
            importer.error(Error::new(line,
                                      ErrorType::NotEnoughVertexComponents));
        }
    }
}

fn read_obj_vt<Real: FromStr, Index>(mut words: Words,
                                    importer: &mut Importer<Real, Index>,
                                    line: Line) {
    let ox = read_real::<Real>(words.next());
    let oy = read_real::<Real>(words.next());
    match (ox, oy) {
        (Some(x), Some(y)) => {
            let ow = read_real::<Real>(words.next());
            let junk = words.next();
            if junk.is_some() {
                importer.error(
                    Error::new(line, ErrorType::TooManyVertexComponents));
            } else {
                importer.vt(x, y, ow);
            }
        }
        _ => {
            importer.error(Error::new(line,
                                      ErrorType::NotEnoughVertexComponents));
        }
    }
}

fn read_obj_line<Real: FromStr, Index: FromStr>(
    importer: &mut Importer<Real, Index>, line: Line) {
    if line.text.starts_with("#") {
        importer.comment(line.text[1..]);
    } else {
        let mut words = line.text.words();

        if let Some(w) = words.next() {
            match w {
                "v" => {
                    read_obj_v(words, importer, line);
                }
                "vt" => {
                    read_obj_vt(words, importer, line);
                }
                "f" => {
                    importer.f(ElementIterator { iter: words });
                }
                _ => {
                    importer.error(Error::new(line, ErrorType::InvalidName));
                }
            }
        }
    }
}

pub fn read_obj<R: Reader, Real: FromStr, Index: FromStr>(
    reader: R, importer: &mut Importer<Real, Index>) {
    for (line_index, line) in BufferedReader::new(reader).lines().enumerate() {
        read_obj_line(importer,
                      Line { text: &line.unwrap(), number: line_index + 1 });
    }
}
