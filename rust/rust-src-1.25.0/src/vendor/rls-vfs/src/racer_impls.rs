extern crate racer;

use std::io;
use std::path::Path;

use {Vfs, FileContents, Error};

impl racer::FileLoader for Vfs {
    fn load_file(&self, path: &Path) -> io::Result<String> {
        match self.0.load_file(path) {
            Ok(FileContents::Text(t)) => Ok(t),
            Ok(FileContents::Binary(_)) => Err(
                io::Error::new(io::ErrorKind::Other, Error::BadFileKind),
            ),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        }
    }
}
