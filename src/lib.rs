#![feature(if_let)]
#![feature(slicing_syntax)]

use std::io::BufferedReader;
use std::from_str::FromStr;

pub enum CallbackResult {
    Continue,
    Stop
}

pub struct Error<'a> {
    pub line: &'a String,
    pub line_number: uint,
    pub message: &'static str
}

impl<'a> Error<'a> {
    fn new(line: &'a String, line_number: uint,
           message: &'static str) -> Error<'a> {
        Error {
            line: line,
            line_number: line_number,
            message: message
        }
    }
}

pub trait Importer<Real> {
    fn comment(&mut self, _line: &str) -> CallbackResult {
        Continue
    }

    fn error(&mut self, _error: Error) -> CallbackResult {
        Continue
    }

    fn v(&mut self, _x: Real, _y: Real, _z: Real,
         _w: Option<Real>) -> CallbackResult {
        Continue
    }
}

fn read_real<Real: FromStr>(word: Option<&str>) -> Option<Real> {
    match word {
        Some(val) => from_str(val),
        None => None
    }
}

fn read_obj_line<Real: FromStr>(line: String, importer: &mut Importer<Real>,
                       line_num: uint) {
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
                                               "junk at end of line"));
                            } else {
                                importer.v(x, y, z, ow);
                            }
                        }
                        _ => {
                            importer.error(Error::new(&line, line_num,
                                                      "not enough components"));
                        }
                    }
                }
                _ => {
                    importer.error(Error::new(&line, line_num,
                                              "invalid name"));
                }
            }
        }
    }
}

pub fn read_obj<R: Reader, Real: FromStr>(reader: R,
                                          importer: &mut Importer<Real>) {
    for (line_index, line) in BufferedReader::new(reader).lines().enumerate() {
        read_obj_line(line.unwrap(), importer, line_index + 1);
    }
}
