/// # Value extension
///
/// Extension trait for the toml::Value type
///

use toml::Value;

use read::TomlValueReadExt;
use set::TomlValueSetExt;
use delete::TomlValueDeleteExt;
use insert::TomlValueInsertExt;
use error::Result;

/// Conveniance trait over
///
///  * TomlValueReadExt
///  * TomlValueSetExt
///
/// for ease of use.
///
/// The very same goal can be achieved by importing each trait seperately.
pub trait TomlValueExt<'doc> :
    TomlValueReadExt<'doc> + TomlValueSetExt + TomlValueDeleteExt + TomlValueInsertExt
{

    //
    // READ functionality
    //

    /// See documentation of `TomlValueReadExt`
    #[inline]
    fn read_with_seperator(&'doc self, query: &String, sep: char) -> Result<Option<&'doc Value>> {
        TomlValueReadExt::read_with_seperator(self, query, sep)
    }

    /// See documentation of `TomlValueReadExt`
    #[inline]
    fn read_mut_with_seperator(&'doc mut self, query: &String, sep: char) -> Result<Option<&'doc mut Value>> {
        TomlValueReadExt::read_mut_with_seperator(self, query, sep)
    }

    /// See documentation of `TomlValueReadExt`
    #[inline]
    fn read(&'doc self, query: &String) -> Result<Option<&'doc Value>> {
        TomlValueReadExt::read_with_seperator(self, query, '.')
    }

    /// See documentation of `TomlValueReadExt`
    #[inline]
    fn read_mut(&'doc mut self, query: &String) -> Result<Option<&'doc mut Value>> {
        TomlValueReadExt::read_mut_with_seperator(self, query, '.')
    }

    //
    // SET functionality
    //

    /// See documentation of `TomlValueSetExt`
    #[inline]
    fn set_with_seperator(&mut self, query: &String, sep: char, value: Value) -> Result<Option<Value>> {
        TomlValueSetExt::set_with_seperator(self, query, sep, value)
    }

    /// See documentation of `TomlValueSetExt`
    #[inline]
    fn set(&mut self, query: &String, value: Value) -> Result<Option<Value>> {
        TomlValueSetExt::set_with_seperator(self, query, '.', value)
    }

    //
    // DELETE functionality
    //

    /// See documentation of `TomlValueDeleteExt`
    #[inline]
    fn delete_with_seperator(&mut self, query: &String, sep: char) -> Result<Option<Value>> {
        TomlValueDeleteExt::delete_with_seperator(self, query, sep)
    }

    /// See documentation of `TomlValueDeleteExt`
    #[inline]
    fn delete(&mut self, query: &String) -> Result<Option<Value>> {
        TomlValueDeleteExt::delete(self, query)
    }

    //
    // INSERT functionality
    //

    /// See documentation of `TomlValueInsertExt`
    #[inline]
    fn insert_with_seperator(&mut self, query: &String, sep: char, value: Value) -> Result<Option<Value>> {
        TomlValueInsertExt::insert_with_seperator(self, query, sep, value)
    }

    /// See documentation of `TomlValueInsertExt`
    #[inline]
    fn insert(&mut self, query: &String, value: Value) -> Result<Option<Value>> {
        TomlValueInsertExt::insert(self, query, value)
    }
}

impl<'doc> TomlValueExt<'doc> for Value { }

