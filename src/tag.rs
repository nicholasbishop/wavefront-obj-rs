#[deriving(PartialEq, Show)]
pub enum Tag<'a> {
    F,
    V,
    Vn,
    Vt,

    Unknown(&'a str)
}

impl<'a> Tag<'a> {
    pub fn from_str(s: &'a str) -> Tag<'a> {
        // TODO(bishop): probably a less repetitive way to write this
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
