extern crate "wavefront_obj" as obj;

use std::io::BufReader;
use obj::CallbackResult;

struct TestImporter {
    panic_on_error: bool,

    comments: Vec<String>,
    errors: Vec<uint>,

    verts: Vec<(f32, f32, f32, Option<f32>)>,
    uvs: Vec<(f32, f32, Option<f32>)>,

    faces: Vec<Vec<u32>>
}

impl TestImporter {
    fn new(panic_on_error: bool) -> TestImporter {
        TestImporter {
            panic_on_error: panic_on_error,
            comments: Vec::new(),
            errors: Vec::new(),
            verts: Vec::new(),
            uvs: Vec::new(),
            faces: Vec::new()
        }
    }
}

impl obj::Importer<f32, u32> for TestImporter  {
    fn comment(&mut self, line: &str) -> CallbackResult {
        self.comments.push(line.to_string());
        obj::Continue
    }

    fn error(&mut self, error: obj::Error) -> CallbackResult {
        if self.panic_on_error {
            panic!()
        } else {
            self.errors.push(error.line.number);
            obj::Continue
        }
    }

    fn v(&mut self, x: f32, y: f32, z: f32, w: Option<f32>) -> CallbackResult {
        self.verts.push((x, y, z, w));
        obj::Continue
    }

    fn vt(&mut self, u: f32, v: f32, w: Option<f32>) -> CallbackResult {
        self.uvs.push((u, v, w));
        obj::Continue
    }

    fn f(&mut self, mut iter: obj::ElementIterator) -> CallbackResult {
        let mut face = Vec::new();
        for v in iter {
            face.push(v);
        }
        self.faces.push(face);
        obj::Continue
    }
}

fn str_reader(string: &str) -> BufReader {
    return BufReader::new(string.as_bytes());
}

#[test]
fn triangle() {
    let input = r"
v 0 0 0
v 1 0 0
v 0 1 0
f 1 2 3
";
    let mut importer = TestImporter::new(true);
    obj::read_obj(str_reader(input), &mut importer);
    assert!(importer.verts == 
            vec!((0.0, 0.0, 0.0, None),
                 (1.0, 0.0, 0.0, None),
                 (0.0, 1.0, 0.0, None)));
    assert!(importer.faces == vec!(vec!(0, 1, 2)));
}

#[test]
fn errors() {
    let input = r"invalid
invalid
";
    let mut importer = TestImporter::new(false);
    obj::read_obj(str_reader(input), &mut importer);
    assert!(importer.errors == vec!(1, 2));
}

#[test]
fn comment() {
    let mut importer = TestImporter::new(true);
    obj::read_obj(str_reader("#comment"), &mut importer);
    assert!(importer.comments.len() == 1);
    assert!(importer.comments[0].as_slice() == "comment");
}

#[test]
fn test_invalid_vert() {
    let mut importer = TestImporter::new(false);
    obj::read_obj(str_reader("v 0 0"), &mut importer);
    assert!(importer.errors.len() == 1);
    assert!(importer.verts.is_empty());
}

#[test]
fn test_vt() {
    let mut importer = TestImporter::new(true);
    obj::read_obj(str_reader("vt 1 2"), &mut importer);
    assert!(importer.uvs == vec!((1.0, 2.0, None)));
}
