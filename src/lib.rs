#![feature(if_let)]
#![feature(slicing_syntax)]

use std::io::BufferedReader;
use std::num::from_int;
use std::ops::Sub;
use std::str::FromStr;

pub enum CallbackResult {
    Continue,
    Stop
}

pub enum ErrorType {
    InvalidName,
    TooManyVertexComponents,
    NotEnoughVertexComponents
}

pub struct Error<'a> {
    pub line: &'a String,
    pub line_number: uint,
    pub error: ErrorType
}

impl<'a> Error<'a> {
    fn new(line: &'a String, line_number: uint, 
           error: ErrorType) -> Error<'a> {
        Error {
            line: line,
            line_number: line_number,
            error: error
        }
    }
}

pub struct ElementIterator<'a> {
    iter: &'a mut std::iter::Filter<'a, &'a str, 
                                    std::str::CharSplits<'a, |char|:'a -> bool>>
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

fn read_obj_line<Real: FromStr, Index: FromStr>(
    line: String, importer: &mut Importer<Real, Index>, line_num: uint) {
    if line.starts_with("#") {
        importer.comment(line[1..]);
    } else {
        let mut words = line.words();

        if let Some(w) = words.next() {
            match w {
                "v" => {
                    let ox = read_real::<Real>(words.next());
                    let oy = read_real::<Real>(words.next());
                    let oz = read_real::<Real>(words.next());
                    match (ox, oy, oz) {
                        (Some(x), Some(y), Some(z)) => {
                            let ow = read_real::<Real>(words.next());
                            let junk = words.next();
                            if junk.is_some() {
                                importer.error(
                                    Error::new(&line, line_num,
                                               ErrorType::TooManyVertexComponents));
                            } else {
                                importer.v(x, y, z, ow);
                            }
                        }
                        _ => {
                            importer.error(Error::new(&line, line_num,
                                                      ErrorType::NotEnoughVertexComponents));
                        }
                    }
                }
                "f" => {
                    importer.f(ElementIterator { iter: &mut words });
                }
                _ => {
                    importer.error(Error::new(&line, line_num,
                                              ErrorType::InvalidName));
                }
            }
        }
    }
}

pub fn read_obj<R: Reader, Real: FromStr, Index: FromStr>(
    reader: R, importer: &mut Importer<Real, Index>) {
    for (line_index, line) in BufferedReader::new(reader).lines().enumerate() {
        read_obj_line(line.unwrap(), importer, line_index + 1);
    }
}
