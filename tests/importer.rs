extern crate "wavefront_obj" as obj;

use std::io::BufReader;
use obj::CallbackResult;

struct TestImporter {
    comments: Vec<String>,
    errors: Vec<uint>,

    verts: Vec<(f32, f32, f32, Option<f32>)>
}

impl TestImporter {
    fn new() -> TestImporter {
        TestImporter {
            comments: Vec::new(),
            errors: Vec::new(),
            verts: Vec::new()
        }
    }
}

impl obj::Importer<f32> for TestImporter  {
    fn comment(&mut self, line: &str) -> CallbackResult {
        self.comments.push(line.to_string());
        obj::Continue
    }

    fn error(&mut self, error: obj::Error) -> CallbackResult {
        self.errors.push(error.line_number);
        obj::Continue
    }

    fn v(&mut self, x: f32, y: f32, z: f32, w: Option<f32>) -> CallbackResult {
        self.verts.push((x, y, z, w));
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
    let mut importer = TestImporter::new();
    obj::read_obj(str_reader(input), &mut importer);
    assert!(importer.verts.len() == 3);
    assert!(importer.verts[0] == (0.0, 0.0, 0.0, None));
    assert!(importer.verts[1] == (1.0, 0.0, 0.0, None));
    assert!(importer.verts[2] == (0.0, 1.0, 0.0, None));
}

#[test]
fn errors() {
    let input = r"invalid
invalid
";
    let mut importer = TestImporter::new();
    obj::read_obj(str_reader(input), &mut importer);
    assert!(importer.errors.len() == 2);
    assert!(importer.errors[0] == 1);
    assert!(importer.errors[1] == 2);
}

#[test]
fn comment() {
    let mut importer = TestImporter::new();
    obj::read_obj(str_reader("#comment"), &mut importer);
    assert!(importer.comments.len() == 1);
    assert!(importer.comments[0].as_slice() == "comment");
}
