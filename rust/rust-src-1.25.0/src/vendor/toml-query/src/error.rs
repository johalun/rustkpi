/// Error types

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    errors {

        // Errors for tokenizer

        QueryParsingError(query: String) {
            description("parsing the query failed")
            display("Parsing the query '{}' failed", query)
        }

        EmptyQueryError {
            description("the query is empty")
            display("The query on the TOML is empty")
        }

        EmptyIdentifier {
            description("Query an empty identifier: ''")
            display("The passed query has an empty identifier")
        }

        ArrayAccessWithoutIndex {
            description("trying to access array without index")
            display("The passed query tries to access an array but does not specify the index")
        }

        ArrayAccessWithInvalidIndex {
            description("trying to pass an invalid index")
            display("The passed query tries to access an array but does not specify a valid index")
        }

        // Errors for Resolver

        IdentifierNotFoundInDocument(ident: String) {
            description("Identifier missing in document")
            display("The identfier '{}' is not present in the document", ident)
        }

        NoIndexInTable(i: usize) {
            description("Cannot deref index from table")
            display("Got an index query '[{}]' but have table", i)
        }

        NoIdentifierInArray(s: String) {
            description("Cannot query identifier in array")
            display("Got an identifier query '{}' but have array", s)
        }

        QueryingValueAsTable(s: String) {
            description("Querying a table where a value is")
            display("Got an identifier query '{}' but have value", s)
        }

        QueryingValueAsArray(i: usize) {
            description("Querying a table where a value is")
            display("Got an index query '{}' but have value", i)
        }

        CannotDeleteNonEmptyTable(tabname: Option<String>) {
            description("Cannot delete Table that is not empty")
            display("Cannot delete table '{:?}' which is not empty", tabname)
        }

        CannotDeleteNonEmptyArray(arrname: Option<String>) {
            description("Cannot delete Array that is not empty")
            display("Cannot delete array '{:?}' which is not empty", arrname)
        }

        CannotAccessBecauseTypeMismatch(expected: &'static str, actual: &'static str) {
            description("Cannot access value because of type mismatch")
            display("Cannot access {} because expected {}", actual, expected)
        }

        ArrayIndexOutOfBounds(idx: usize, arrlen: usize) {
            description("Delete index out of bounds")
            display("Cannot delete in array at {}, array has length {}", idx, arrlen)
        }

        TypeError(requested: &'static str, got: &'static str) {
            description("Type error")
            display("Type Error. Requested {}, but got {}", requested, got)
        }

        NotAvailable(query: String) {
            description("Value missing error")
            display("Value at '{}' not there", query)
        }

    }
}
