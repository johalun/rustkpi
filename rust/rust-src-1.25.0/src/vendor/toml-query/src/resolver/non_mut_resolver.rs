/// The query resolver that operates on the AST and the TOML object

use std::ops::Index;

use toml::Value;
use tokenizer::Token;
use error::*;

/// Resolves the path in the passed document recursively
///
/// # Guarantees
///
/// If error_if_not_found is set to true, this function does not return Ok(None) in any case.
///
pub fn resolve<'doc>(toml: &'doc Value, tokens: &Token, error_if_not_found: bool) -> Result<Option<&'doc Value>> {
    match toml {
        &Value::Table(ref t) => {
            match tokens {
                &Token::Identifier { ref ident, .. } => {
                    match t.get(ident) {
                        None => if error_if_not_found {
                            let err = ErrorKind::IdentifierNotFoundInDocument(ident.to_owned());
                            return Err(Error::from(err))
                        } else {
                            Ok(None)
                        },
                        Some(sub_document) => match tokens.next() {
                            Some(next) => resolve(sub_document, next, error_if_not_found),
                            None       => Ok(Some(sub_document)),
                        },
                    }
                },

                &Token::Index { idx, .. } => {
                    let kind = ErrorKind::NoIndexInTable(idx);
                    Err(Error::from(kind))
                },
            }
        },

        &Value::Array(ref ary) => {
            match tokens {
                &Token::Index { idx, .. } => {
                    match tokens.next() {
                        Some(next) => resolve(ary.get(idx).unwrap(), next, error_if_not_found),
                        None       => Ok(Some(ary.index(idx))),
                    }
                },
                &Token::Identifier { ref ident, .. } => {
                    let kind = ErrorKind::NoIdentifierInArray(ident.clone());
                    Err(Error::from(kind))
                },
            }
        },

        _ => match tokens {
            &Token::Identifier { ref ident, .. } => {
                Err(Error::from(ErrorKind::QueryingValueAsTable(ident.clone())))
            },

            &Token::Index { idx, .. } => {
                Err(Error::from(ErrorKind::QueryingValueAsArray(idx)))
            },
        }
    }
}

#[cfg(test)]
mod test {
    use toml::from_str as toml_from_str;
    use toml::Value;
    use tokenizer::*;
    use error::*;
    use super::resolve;

    macro_rules! do_resolve {
        ( $toml:ident => $query:expr ) => {
            resolve(&$toml, &tokenize_with_seperator(&String::from($query), '.').unwrap(), true)
        }
    }

    #[test]
    fn test_resolve_empty_toml_simple_query() {
        let toml = toml_from_str("").unwrap();
        let result = do_resolve!(toml => "example");

        assert!(result.is_err());
        let result = result.unwrap_err();

        let errkind = result.kind();
        assert!(is_match!(errkind, &ErrorKind::IdentifierNotFoundInDocument { .. }));
    }

    #[test]
    fn test_resolve_present_bool() {
        let toml = toml_from_str("example = true").unwrap();
        let result = do_resolve!(toml => "example");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Boolean(true)));
    }

    #[test]
    fn test_resolve_present_integer() {
        let toml = toml_from_str("example = 1").unwrap();
        let result = do_resolve!(toml => "example");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Integer(1)));
    }

    #[test]
    fn test_resolve_present_float() {
        let toml = toml_from_str("example = 1.0").unwrap();
        let result = do_resolve!(toml => "example");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Float(1.0)));
    }

    #[test]
    fn test_resolve_present_string() {
        let toml = toml_from_str("example = 'string'").unwrap();
        let result = do_resolve!(toml => "example");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::String(_)));
        match result {
            &Value::String(ref s) => assert_eq!("string", s),
            _ => panic!("What just happened?"),
        }
    }

    #[test]
    fn test_resolve_present_array_bools() {
        let toml = toml_from_str("example = [ true, false ]").unwrap();
        let result = do_resolve!(toml => "example");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Array(_)));
        match result {
            &Value::Array(ref ary) => {
                assert_eq!(ary[0], Value::Boolean(true));
                assert_eq!(ary[1], Value::Boolean(false));
            },
            _ => panic!("What just happened?"),
        }
    }

    #[test]
    fn test_resolve_present_array_integers() {
        let toml = toml_from_str("example = [ 1, 1337 ]").unwrap();
        let result = do_resolve!(toml => "example");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Array(_)));
        match result {
            &Value::Array(ref ary) => {
                assert_eq!(ary[0], Value::Integer(1));
                assert_eq!(ary[1], Value::Integer(1337));
            },
            _ => panic!("What just happened?"),
        }
    }

    #[test]
    fn test_resolve_present_array_floats() {
        let toml = toml_from_str("example = [ 1.0, 133.25 ]").unwrap();
        let result = do_resolve!(toml => "example");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Array(_)));
        match result {
            &Value::Array(ref ary) => {
                assert_eq!(ary[0], Value::Float(1.0));
                assert_eq!(ary[1], Value::Float(133.25));
            },
            _ => panic!("What just happened?"),
        }
    }

    #[test]
    fn test_resolve_array_index_query_1() {
        let toml = toml_from_str("example = [ 1 ]").unwrap();
        let result = do_resolve!(toml => "example.[0]");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Integer(1)));
    }

    #[test]
    fn test_resolve_array_index_query_2() {
        let toml = toml_from_str("example = [ 1, 2, 3, 4, 5 ]").unwrap();
        let result = do_resolve!(toml => "example.[4]");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Integer(5)));
    }

    #[test]
    fn test_resolve_table_element_query() {
        let toml = toml_from_str(r#"
        [table]
        value = 42
        "#).unwrap();
        let result = do_resolve!(toml => "table.value");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Integer(42)));
    }

    #[test]
    fn test_resolve_table_with_many_elements_element_query() {
        let toml = toml_from_str(r#"
        [table]
        value1 = 42
        value2 = 43
        value3 = 44
        value4 = 45
        value5 = 46
        "#).unwrap();
        let result = do_resolve!(toml => "table.value1");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Integer(42)));
    }

    #[test]
    fn test_resolve_table_array_query() {
        let toml = toml_from_str(r#"
        [table]
        value1 = [ 42.0, 50.0 ]
        "#).unwrap();
        let result = do_resolve!(toml => "table.value1");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Array(_)));
        match result {
            &Value::Array(ref ary) => {
                assert_eq!(ary[0], Value::Float(42.0));
                assert_eq!(ary[1], Value::Float(50.0));
            },
            _ => panic!("What just happened?"),
        }
    }

    #[test]
    fn test_resolve_table_array_element_query() {
        let toml = toml_from_str(r#"
        [table]
        value1 = [ 42 ]
        "#).unwrap();
        let result = do_resolve!(toml => "table.value1.[0]");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Integer(42)));
    }

    #[test]
    fn test_resolve_multi_table_query() {
        let toml = toml_from_str(r#"
        [table0]
        value = [ 1 ]
        [table1]
        value = [ "Foo" ]
        [table2]
        value = [ 42.0 ]
        [table3]
        value = [ true ]
        "#).unwrap();
        let result = do_resolve!(toml => "table1.value.[0]");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::String(_)));
        match result {
            &Value::String(ref s) => assert_eq!("Foo", s),
            _ => panic!("What just happened?"),
        }
    }

    static FRUIT_TABLE : &'static str = r#"
    [[fruit.blah]]
      name = "apple"

      [fruit.blah.physical]
        color = "red"
        shape = "round"

    [[fruit.blah]]
      name = "banana"

      [fruit.blah.physical]
        color = "yellow"
        shape = "bent"
    "#;

    #[test]
    fn test_resolve_array_table_query_1() {
        let toml = toml_from_str(FRUIT_TABLE).unwrap();
        let result = do_resolve!(toml => "fruit.blah.[0].name");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::String(_)));
        match result {
            &Value::String(ref s) => assert_eq!("apple", s),
            _ => panic!("What just happened?"),
        }
    }

    #[test]
    fn test_resolve_array_table_query_2() {
        let toml = toml_from_str(FRUIT_TABLE).unwrap();
        let result = do_resolve!(toml => "fruit.blah.[0].physical");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Table(_)));
        match result {
            &Value::Table(ref tab) => {
                match tab.get("color") {
                    Some(&Value::String(ref s)) => assert_eq!("red", s),
                    _ => assert!(false),
                }
                match tab.get("shape") {
                    Some(&Value::String(ref s)) => assert_eq!("round", s),
                    _ => assert!(false),
                }
            },
            _ => panic!("What just happened?"),
        }
    }

    #[test]
    fn test_resolve_query_on_result() {
        let toml = toml_from_str(FRUIT_TABLE).unwrap();
        let result = do_resolve!(toml => "fruit.blah.[1].physical");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        let tokens = tokenize_with_seperator(&String::from("color"), '.').unwrap();
        let result = resolve(result, &tokens, true);

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::String(_)));
        match result {
            &Value::String(ref s) => assert_eq!("yellow", s),
            _ => panic!("What just happened?"),
        }
    }

    #[test]
    fn test_resolve_query_empty_table() {
        let toml = toml_from_str(r#"
        [example]
        "#).unwrap();
        let result = do_resolve!(toml => "example");

        assert!(result.is_ok());
        let result = result.unwrap();

        assert!(result.is_some());
        let result = result.unwrap();

        assert!(is_match!(result, &Value::Table(_)));
        match result {
            &Value::Table(ref t) => assert!(t.is_empty()),
            _ => panic!("What just happened?"),
        }
    }

    #[test]
    fn test_resolve_query_member_of_empty_table() {
        let toml = toml_from_str(r#"
        [example]
        "#).unwrap();
        let result = do_resolve!(toml => "example.foo");

        assert!(result.is_err());
        let result = result.unwrap_err();

        let errkind = result.kind();
        assert!(is_match!(errkind, &ErrorKind::IdentifierNotFoundInDocument { .. }));
    }

    #[test]
    fn test_resolve_query_index_in_table() {
        let toml = toml_from_str(r#"
        [example]
        "#).unwrap();
        let result = do_resolve!(toml => "example.[0]");

        assert!(result.is_err());
        let result = result.unwrap_err();

        let errkind = result.kind();
        assert!(is_match!(errkind, &ErrorKind::NoIndexInTable { .. }));
    }

    #[test]
    fn test_resolve_query_identifier_in_array() {
        let toml = toml_from_str(r#"
        [example]
        foo = [ 1, 2, 3 ]
        "#).unwrap();
        let result = do_resolve!(toml => "example.foo.bar");

        assert!(result.is_err());
        let result = result.unwrap_err();

        let errkind = result.kind();
        assert!(is_match!(errkind, &ErrorKind::NoIdentifierInArray { .. }));
    }

    #[test]
    fn test_resolve_query_value_as_table() {
        let toml = toml_from_str(r#"
        [example]
        foo = 1
        "#).unwrap();
        let result = do_resolve!(toml => "example.foo.bar");

        assert!(result.is_err());
        let result = result.unwrap_err();

        let errkind = result.kind();
        assert!(is_match!(errkind, &ErrorKind::QueryingValueAsTable { .. }));
    }

    #[test]
    fn test_resolve_query_value_as_array() {
        let toml = toml_from_str(r#"
        [example]
        foo = 1
        "#).unwrap();
        let result = do_resolve!(toml => "example.foo.[0]");

        assert!(result.is_err());
        let result = result.unwrap_err();

        let errkind = result.kind();
        assert!(is_match!(errkind, &ErrorKind::QueryingValueAsArray { .. }));
    }

}

