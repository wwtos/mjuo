pub enum PropertyType {
    String,
    Integer,
    Float,
    Bool,
}

#[derive(Debug, Clone)]
pub enum Property {
    String(String),
    Integer(i32),
    Float(f32),
    Bool(bool)
}
