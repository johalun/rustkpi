use toml::Value;

pub fn name_of_val(val: &Value) -> &'static str {
    match *val {
        Value::Array(_)    => "Array",
        Value::Boolean(_)  => "Boolean",
        Value::Datetime(_) => "Datetime",
        Value::Float(_)    => "Float",
        Value::Integer(_)  => "Integer",
        Value::String(_)   => "String",
        Value::Table(_)    => "Table",
    }
}

