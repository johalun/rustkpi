/// The Toml Set extensions

use toml::Value;

use tokenizer::tokenize_with_seperator;
use tokenizer::Token;
use error::*;

pub trait TomlValueSetExt {

    /// Extension function for setting a value in the current toml::Value document
    /// using a custom seperator
    ///
    /// # Semantics
    ///
    /// The function _never_ creates intermediate data structures (Tables or Arrays) in the
    /// document.
    ///
    /// # Return value
    ///
    /// * If the set operation worked correctly, `Ok(None)` is returned.
    /// * If the set operation replaced an existing value `Ok(Some(old_value))` is returned
    /// * On failure, `Err(e)` is returned:
    ///     * If the query is `"a.b.c"` but there is no table `"b"`: error
    ///     * If the query is `"a.b.[0]"` but "`b"` is not an array: error
    ///     * If the query is `"a.b.[3]"` but the array at "`b"` has no index `3`: error
    ///     * etc.
    ///
    fn set_with_seperator(&mut self, query: &str, sep: char, value: Value) -> Result<Option<Value>>;

    /// Extension function for setting a value from the current toml::Value document
    ///
    /// See documentation of `TomlValueSetExt::set_with_seperator`
    fn set(&mut self, query: &str, value: Value) -> Result<Option<Value>> {
        self.set_with_seperator(query, '.', value)
    }

}

impl TomlValueSetExt for Value {

    fn set_with_seperator(&mut self, query: &str, sep: char, value: Value) -> Result<Option<Value>> {
        use resolver::mut_resolver::resolve;

        let mut tokens = try!(tokenize_with_seperator(query, sep));
        let last = tokens.pop_last();

        let val = try!(resolve(self, &tokens, true))
            .unwrap(); // safe because of resolve() guarantees
        let last = last.unwrap_or_else(|| Box::new(tokens));

        match *last {
            Token::Identifier { ident, .. } => {
                match val {
                    &mut Value::Table(ref mut t) => {
                        Ok(t.insert(ident, value))
                    },
                    &mut Value::Array(_) => {
                        let kind = ErrorKind::NoIdentifierInArray(ident);
                        Err(Error::from(kind))
                    }
                    _ => {
                        let kind = ErrorKind::QueryingValueAsTable(ident);
                        Err(Error::from(kind))
                    }
                }
            }

            Token::Index { idx, .. } => {
                match val {
                    &mut Value::Array(ref mut a) => {
                        if a.len() > idx {
                            let result = a.swap_remove(idx);
                            a.insert(idx, value);
                            Ok(Some(result))
                        } else {
                            a.push(value);
                            Ok(None)
                        }
                    }
                    &mut Value::Table(_) => {
                        let kind = ErrorKind::NoIndexInTable(idx);
                        Err(Error::from(kind))
                    }
                    _ => {
                        let kind = ErrorKind::QueryingValueAsArray(idx);
                        Err(Error::from(kind))
                    }
                }
            }

        }
    }

}

#[cfg(test)]
mod test {
    use super::*;
    use toml::Value;
    use toml::from_str as toml_from_str;

    #[test]
    fn test_set_with_seperator_into_table() {
        let mut toml : Value = toml_from_str(r#"
        [table]
        a = 0
        "#).unwrap();

        let res = toml.set_with_seperator(&String::from("table.a"), '.', Value::Integer(1));

        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(is_match!(res, Value::Integer(0)));

        assert!(is_match!(toml, Value::Table(_)));
        match toml {
            Value::Table(ref t) => {
                assert!(!t.is_empty());

                let inner = t.get("table");
                assert!(inner.is_some());

                let inner = inner.unwrap();
                assert!(is_match!(inner, &Value::Table(_)));
                match inner {
                    &Value::Table(ref t) => {
                        assert!(!t.is_empty());

                        let a = t.get("a");
                        assert!(a.is_some());

                        let a = a.unwrap();
                        assert!(is_match!(a, &Value::Integer(1)));
                    },
                    _ => panic!("What just happenend?"),
                }
            },
            _ => panic!("What just happenend?"),
        }
    }

    #[test]
    fn test_set_with_seperator_into_table_key_nonexistent() {
        let mut toml : Value = toml_from_str(r#"
        [table]
        "#).unwrap();

        let res = toml.set_with_seperator(&String::from("table.a"), '.', Value::Integer(1));

        assert!(res.is_ok());
        let res = res.unwrap();

        assert!(res.is_none());

        assert!(is_match!(toml, Value::Table(_)));
        match toml {
            Value::Table(ref t) => {
                assert!(!t.is_empty());

                let inner = t.get("table");
                assert!(inner.is_some());

                let inner = inner.unwrap();
                assert!(is_match!(inner, &Value::Table(_)));
                match inner {
                    &Value::Table(ref t) => {
                        assert!(!t.is_empty());

                        let a = t.get("a");
                        assert!(a.is_some());

                        let a = a.unwrap();
                        assert!(is_match!(a, &Value::Integer(1)));
                    },
                    _ => panic!("What just happenend?"),
                }
            },
            _ => panic!("What just happenend?"),
        }
    }

    #[test]
    fn test_set_with_seperator_into_array() {
        use std::ops::Index;

        let mut toml : Value = toml_from_str(r#"
        array = [ 0 ]
        "#).unwrap();

        let res = toml.set_with_seperator(&String::from("array.[0]"), '.', Value::Integer(1));

        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(is_match!(res, Value::Integer(0)));

        assert!(is_match!(toml, Value::Table(_)));
        match toml {
            Value::Table(ref t) => {
                assert!(!t.is_empty());

                let inner = t.get("array");
                assert!(inner.is_some());

                let inner = inner.unwrap();
                assert!(is_match!(inner, &Value::Array(_)));
                match inner {
                    &Value::Array(ref a) => {
                        assert!(!a.is_empty());
                        assert!(is_match!(a.index(0), &Value::Integer(1)));
                    },
                    _ => panic!("What just happenend?"),
                }
            },
            _ => panic!("What just happenend?"),
        }
    }

    #[test]
    fn test_set_with_seperator_into_table_index_nonexistent() {
        use std::ops::Index;

        let mut toml : Value = toml_from_str(r#"
        array = []
        "#).unwrap();

        let res = toml.set_with_seperator(&String::from("array.[0]"), '.', Value::Integer(1));

        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.is_none());

        assert!(is_match!(toml, Value::Table(_)));
        match toml {
            Value::Table(ref t) => {
                assert!(!t.is_empty());

                let inner = t.get("array");
                assert!(inner.is_some());

                let inner = inner.unwrap();
                assert!(is_match!(inner, &Value::Array(_)));
                match inner {
                    &Value::Array(ref a) => {
                        assert!(!a.is_empty());
                        assert!(is_match!(a.index(0), &Value::Integer(1)));
                    },
                    _ => panic!("What just happenend?"),
                }
            },
            _ => panic!("What just happenend?"),
        }
    }

    #[test]
    fn test_set_with_seperator_into_nested_table() {
        let mut toml : Value = toml_from_str(r#"
        [a.b.c]
        d = 0
        "#).unwrap();

        let res = toml.set_with_seperator(&String::from("a.b.c.d"), '.', Value::Integer(1));

        assert!(res.is_ok());

        let res = res.unwrap();
        assert!(res.is_some());
        let res = res.unwrap();
        assert!(is_match!(res, Value::Integer(0)));

        assert!(is_match!(toml, Value::Table(_)));
        match toml {
            Value::Table(ref t) => {
                assert!(!t.is_empty());

                let a = t.get("a");
                assert!(a.is_some());

                let a = a.unwrap();
                assert!(is_match!(a, &Value::Table(_)));
                match a {
                    &Value::Table(ref a) => {
                        assert!(!a.is_empty());

                        let b_tab = a.get("b");
                        assert!(b_tab.is_some());

                        let b_tab = b_tab.unwrap();
                        assert!(is_match!(b_tab, &Value::Table(_)));
                        match b_tab {
                            &Value::Table(ref b) => {
                                assert!(!b.is_empty());

                                let c_tab = b.get("c");
                                assert!(c_tab.is_some());

                                let c_tab = c_tab.unwrap();
                                assert!(is_match!(c_tab, &Value::Table(_)));
                                match c_tab {
                                    &Value::Table(ref c) => {
                                        assert!(!c.is_empty());

                                        let d = c.get("d");
                                        assert!(d.is_some());

                                        let d = d.unwrap();
                                        assert!(is_match!(d, &Value::Integer(1)));
                                    },
                                    _ => panic!("What just happenend?"),
                                }
                            },
                            _ => panic!("What just happenend?"),
                        }
                    },
                    _ => panic!("What just happenend?"),
                }
            },
            _ => panic!("What just happenend?"),
        }
    }

    #[test]
    fn test_set_with_seperator_into_nonexistent_table() {
        let mut toml : Value = toml_from_str("").unwrap();

        let res = toml.set_with_seperator(&String::from("table.a"), '.', Value::Integer(1));

        assert!(res.is_err());

        let res = res.unwrap_err();
        assert!(is_match!(res.kind(), &ErrorKind::IdentifierNotFoundInDocument(_)));
    }

    #[test]
    fn test_set_with_seperator_into_nonexistent_array() {
        let mut toml : Value = toml_from_str("").unwrap();

        let res = toml.set_with_seperator(&String::from("[0]"), '.', Value::Integer(1));

        assert!(res.is_err());

        let res = res.unwrap_err();
        assert!(is_match!(res.kind(), &ErrorKind::NoIndexInTable(0)));
    }

    #[test]
    fn test_set_with_seperator_ident_into_ary() {
        let mut toml : Value = toml_from_str(r#"
        array = [ 0 ]
        "#).unwrap();

        let res = toml.set_with_seperator(&String::from("array.foo"), '.', Value::Integer(2));

        assert!(res.is_err());
        let res = res.unwrap_err();

        assert!(is_match!(res.kind(), &ErrorKind::NoIdentifierInArray(_)));
    }

    #[test]
    fn test_set_with_seperator_index_into_table() {
        let mut toml : Value = toml_from_str(r#"
        foo = { bar = 1 }
        "#).unwrap();

        let res = toml.set_with_seperator(&String::from("foo.[0]"), '.', Value::Integer(2));

        assert!(res.is_err());
        let res = res.unwrap_err();

        assert!(is_match!(res.kind(), &ErrorKind::NoIndexInTable(_)));
    }

    #[test]
    fn test_set_with_seperator_ident_into_non_structure() {
        let mut toml : Value = toml_from_str(r#"
        val = 0
        "#).unwrap();

        let res = toml.set_with_seperator(&String::from("val.foo"), '.', Value::Integer(2));

        assert!(res.is_err());
        let res = res.unwrap_err();

        assert!(is_match!(res.kind(), &ErrorKind::QueryingValueAsTable(_)));
    }

    #[test]
    fn test_set_with_seperator_index_into_non_structure() {
        let mut toml : Value = toml_from_str(r#"
        foo = 1
        "#).unwrap();

        let res = toml.set_with_seperator(&String::from("foo.[0]"), '.', Value::Integer(2));

        assert!(res.is_err());
        let res = res.unwrap_err();

        assert!(is_match!(res.kind(), &ErrorKind::QueryingValueAsArray(_)));
    }

}

