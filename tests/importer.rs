extern crate "bishop-obj" as obj;

use std::io::BufReader;
use obj::CallbackResult;

struct TestImporter {
    comments: Vec<String>,
    errors: Vec<(String, uint, &'static str)>,

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
    fn comment(&mut self, line: &String) -> CallbackResult {
        self.comments.push(line.clone());
        obj::Continue
    }

    fn error(&mut self, error: obj::Error) -> CallbackResult {
        self.errors.push((error.line.clone(), error.line_number, error.message));
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
