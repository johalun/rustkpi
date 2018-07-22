extern crate html_diff;

use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

fn get_file_content<P: AsRef<Path>>(p: &P) -> io::Result<String> {
    let mut f = File::open(p)?;
    let mut buffer = String::with_capacity(1000);
    f.read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn print_error(arg: &str, v: io::Result<String>) {
    if let Err(err) = v {
        println!("\"{}\": error: {}", arg, err);
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() & 1 != 0 {
        println!("Need to pass an even number of HTML files");
        return
    }
    for args in args.chunks(2) {
        let arg1 = &args[0];
        let arg2 = &args[1];
        match (get_file_content(&arg1), get_file_content(&arg2)) {
            (Ok(content1), Ok(content2)) => {
                let differences = html_diff::get_differences(&content1, &content2);
                for diff in differences {
                    println!("=> {}", diff.to_string());
                }
            }
            (x, y) => {
                print_error(arg1, x);
                print_error(arg2, y);
            }
        }
    }
}
