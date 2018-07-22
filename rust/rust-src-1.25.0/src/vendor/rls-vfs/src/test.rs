use super::{VfsInternal, Change, FileLoader, File, Error, make_line_indices};
use Span;
use span::{Row, Column};
use std::path::{Path, PathBuf};

struct MockFileLoader;

impl FileLoader for MockFileLoader {
    fn read<U>(file_name: &Path) -> Result<File<U>, Error> {
        let text = format!("{}\nHello\nWorld\nHello, World!\n", file_name.display());
        Ok(File {
            line_indices: make_line_indices(&text),
            text: text,
            changed: false,
            user_data: None,
        })
    }

    fn write(file_name: &Path, file: &FileKind) -> Result<(), Error> {
        if file_name.display().to_string() == "foo" {
            assert_eq!(file.changed, true);
            assert_eq!(file.text, "foo\nHfooo\nWorld\nHello, World!\n");
        }

        Ok(())
    }
}

fn make_change(with_len: bool) -> Change {
    let (row_end, col_end, len) = if with_len {
        // If len is present, we shouldn't depend on row_end/col_end
        // at all, because they may be invalid.
        (0, 0, Some(3))
    } else {
        (1, 4, None)
    };
    Change::ReplaceText {
        span: Span::new(
            Row::new_zero_indexed(1),
            Row::new_zero_indexed(row_end),
            Column::new_zero_indexed(1),
            Column::new_zero_indexed(col_end),
            "foo",
        ),
        len: len,
        text: "foo".to_owned(),
    }
}

fn make_change_2(with_len: bool) -> Change {
    let (row_end, col_end, len) = if with_len {
        // If len is present, we shouldn't depend on row_end/col_end
        // at all, because they may be invalid.
        (0, 0, Some(4))
    } else {
        (3, 2, None)
    };
    Change::ReplaceText {
        span: Span::new(
            Row::new_zero_indexed(2),
            Row::new_zero_indexed(row_end),
            Column::new_zero_indexed(4),
            Column::new_zero_indexed(col_end),
            "foo",
        ),
        len: len,
        text: "aye carumba".to_owned(),
    }
}

fn test_has_changes(with_len: bool) {
    let vfs = VfsInternal::<MockFileLoader, ()>::new();

    assert!(!vfs.has_changes());
    vfs.load_file(&Path::new("foo")).unwrap();
    assert!(!vfs.has_changes());
    vfs.on_changes(&[make_change(with_len)]).unwrap();
    assert!(vfs.has_changes());
    vfs.file_saved(&Path::new("bar")).unwrap();
    assert!(vfs.has_changes());
    vfs.file_saved(&Path::new("foo")).unwrap();
    assert!(!vfs.has_changes());
}

#[test]
fn test_has_changes_without_len() {
    test_has_changes(false)
}

#[test]
fn test_has_changes_with_len() {
    test_has_changes(true)
}

#[test]
fn test_cached_files() {
    let vfs = VfsInternal::<MockFileLoader, ()>::new();
    assert!(vfs.get_cached_files().is_empty());
    vfs.load_file(&Path::new("foo")).unwrap();
    vfs.load_file(&Path::new("bar")).unwrap();
    let files = vfs.get_cached_files();
    assert!(files.len() == 2);
    assert!(files[Path::new("foo")] == "foo\nHello\nWorld\nHello, World!\n");
    assert!(files[Path::new("bar")] == "bar\nHello\nWorld\nHello, World!\n");
}

#[test]
fn test_flush_file() {
    let vfs = VfsInternal::<MockFileLoader, ()>::new();
    // Flushing an uncached-file should succeed.
    vfs.flush_file(&Path::new("foo")).unwrap();
    vfs.load_file(&Path::new("foo")).unwrap();
    vfs.flush_file(&Path::new("foo")).unwrap();
    assert!(vfs.get_cached_files().is_empty());
}

fn test_changes(with_len: bool) {
    let vfs = VfsInternal::<MockFileLoader, ()>::new();

    vfs.on_changes(&[make_change(with_len)]).unwrap();
    let files = vfs.get_cached_files();
    assert!(files.len() == 1);
    assert!(files[&PathBuf::from("foo")] == "foo\nHfooo\nWorld\nHello, World!\n");
    assert!(
        vfs.load_file(&Path::new("foo")) == Ok("foo\nHfooo\nWorld\nHello, World!\n".to_owned())
    );
    assert!(
        vfs.load_file(&Path::new("bar")) == Ok("bar\nHello\nWorld\nHello, World!\n".to_owned())
    );

    vfs.on_changes(&[make_change_2(with_len)]).unwrap();
    let files = vfs.get_cached_files();
    assert!(files.len() == 2);
    assert!(files[&PathBuf::from("foo")] == "foo\nHfooo\nWorlaye carumballo, World!\n");
    assert!(
        vfs.load_file(&Path::new("foo")) ==
            Ok("foo\nHfooo\nWorlaye carumballo, World!\n".to_owned())
    );
}

#[test]
fn test_changes_without_len() {
    test_changes(false)
}

#[test]
fn test_changes_with_len() {
    test_changes(true)
}

#[test]
fn test_change_add_file() {
    let vfs = VfsInternal::<MockFileLoader, ()>::new();
    let new_file = Change::AddFile {
        file: PathBuf::from("foo"),
        text: "Hello, World!".to_owned(),
    };
    vfs.on_changes(&[new_file]).unwrap();

    let files = vfs.get_cached_files();
    assert_eq!(files.len(), 1);
    assert_eq!(files[&PathBuf::from("foo")], "Hello, World!");
}

fn test_user_data(with_len: bool) {
    let vfs = VfsInternal::<MockFileLoader, i32>::new();

    // New files have no user data.
    vfs.load_file(&Path::new("foo")).unwrap();
    vfs.with_user_data(&Path::new("foo"), |u| {
        assert_eq!(u, Err(Error::NoUserDataForFile));
        Ok(())
    }).unwrap();

    // Set and read data.
    vfs.set_user_data(&Path::new("foo"), Some(42)).unwrap();
    vfs.with_user_data(&Path::new("foo"), |u| {
        assert_eq!(*u.unwrap().1, 42);
        Ok(())
    }).unwrap();
    assert_eq!(
        vfs.set_user_data(&Path::new("bar"), Some(42)),
        Err(Error::FileNotCached)
    );

    // ensure_user_data should not be called if the userdata already exists.
    vfs.ensure_user_data(&Path::new("foo"), |_| panic!())
        .unwrap();

    // Test ensure_user_data is called.
    vfs.load_file(&Path::new("bar")).unwrap();
    vfs.ensure_user_data(&Path::new("bar"), |_| Ok(1)).unwrap();
    vfs.with_user_data(&Path::new("bar"), |u| {
        assert_eq!(*u.unwrap().1, 1);
        Ok(())
    }).unwrap();

    // compute and read data.
    vfs.with_user_data(&Path::new("foo"), |u| {
        assert_eq!(u.as_ref().unwrap().0, "foo\nHello\nWorld\nHello, World!\n");
        *u.unwrap().1 = 43;
        Ok(())
    }).unwrap();
    vfs.with_user_data(&Path::new("foo"), |u| {
        assert_eq!(*u.unwrap().1, 43);
        Ok(())
    }).unwrap();
    assert_eq!(
        vfs.with_user_data(&Path::new("foo"), |u| {
            assert_eq!(*u.unwrap().1, 43);
            Err(Error::BadLocation): Result<(), Error>
        }),
        Err(Error::BadLocation)
    );
    vfs.with_user_data(&Path::new("foo"), |u| {
        assert_eq!(*u.unwrap().1, 43);
        Ok(())
    }).unwrap();

    // Clear and read data.
    vfs.set_user_data(&Path::new("foo"), None).unwrap();
    vfs.with_user_data(&Path::new("foo"), |u| {
        assert_eq!(u, Err(Error::NoUserDataForFile));
        Ok(())
    }).unwrap();

    // Compute (clear) and read data.
    vfs.set_user_data(&Path::new("foo"), Some(42)).unwrap();
    assert_eq!(
        vfs.with_user_data(&Path::new("foo"), |_| {
            Err(Error::NoUserDataForFile): Result<(), Error>
        }),
        Err(Error::NoUserDataForFile)
    );
    vfs.with_user_data(&Path::new("foo"), |u| {
        assert_eq!(u, Err(Error::NoUserDataForFile));
        Ok(())
    }).unwrap();

    // Flushing a file should clear user data.
    vfs.set_user_data(&Path::new("foo"), Some(42)).unwrap();
    vfs.flush_file(&Path::new("foo")).unwrap();
    vfs.load_file(&Path::new("foo")).unwrap();
    vfs.with_user_data(&Path::new("foo"), |u| {
        assert_eq!(u, Err(Error::NoUserDataForFile));
        Ok(())
    }).unwrap();

    // Recording a change should clear user data.
    vfs.set_user_data(&Path::new("foo"), Some(42)).unwrap();
    vfs.on_changes(&[make_change(with_len)]).unwrap();
    vfs.with_user_data(&Path::new("foo"), |u| {
        assert_eq!(u, Err(Error::NoUserDataForFile));
        Ok(())
    }).unwrap();
}

#[test]
fn test_user_data_without_len() {
    test_user_data(false)
}

#[test]
fn test_user_data_with_len() {
    test_user_data(true)
}

fn test_write(with_len: bool) {
    let vfs = VfsInternal::<MockFileLoader, ()>::new();

    vfs.on_changes(&[make_change(with_len)]).unwrap();
    vfs.write_file(&Path::new("foo")).unwrap();
    let files = vfs.get_cached_files();
    assert!(files.len() == 1);
    let files = vfs.get_changes();
    assert!(files.is_empty());
}

#[test]
fn test_write_without_len() {
    test_write(false)
}

#[test]
fn test_write_with_len() {
    test_write(true)
}

#[test]
fn test_clear() {
    let vfs = VfsInternal::<MockFileLoader, ()>::new();
    vfs.load_file(&Path::new("foo")).unwrap();
    vfs.load_file(&Path::new("bar")).unwrap();
    assert!(vfs.get_cached_files().len() == 2);
    vfs.clear();
    assert!(vfs.get_cached_files().is_empty());
}

// TODO test with wide chars
