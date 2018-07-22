extern crate html_diff;

use std::io::Read;
use std::fs::{self, File};
use std::path::Path;

fn read_file<P: AsRef<Path>>(p: P) -> String {
    let mut f = File::open(p).expect("read_file::open failed");
    let mut content = String::new();
    f.read_to_string(&mut content).expect("read_file::read_to_end failed");
    content
}

fn run_test(path: &str) -> bool {
    let path_len = path.len();
    let original = read_file(path);
    let compare_path = &format!("{}_compare.html", &path[..path_len - 5]);
    let compare = read_file(&compare_path);
    let expected_out = read_file(&format!("{}.stdout", &path[..path_len - 5]))
                           .split('\n')
                           .filter(|s| !s.trim().is_empty())
                           .map(|s| s.to_owned())
                           .collect::<Vec<String>>()
                           .join("\n");
    let differences = html_diff::get_differences(&original, &compare);
    let mut out = Vec::new();
    for diff in differences {
        out.push(format!("=> {}", diff.to_string()));
    }
    let out = out.join("\n");
    if out != expected_out {
        println!("comparison between {:?} and {:?} failed.\nGot: {:?}\nExpected: {:?}",
                 path, compare_path, out, expected_out);
        false
    } else {
        true
    }
}

fn visit_test_dir<P: AsRef<Path>>(dir: &P) -> usize {
    let mut failures = 0;
    for entry in fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("cannot get entry value");
        let path = entry.path();
        if path.is_file() {
            let path_s = path.to_str().expect("to_str failed");
            if path_s.ends_with("_compare.html") || !path_s.ends_with(".html") {
                continue
            }
            if !run_test(path_s) {
                failures += 1;
            }
        }
    }
    failures
}

#[test]
fn test_files() {
    assert_eq!(visit_test_dir(&"test_files"), 0);
}
