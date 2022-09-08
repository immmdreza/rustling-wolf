#[derive(Debug, Clone)]
pub enum AskWorld {
    RawString(String),
}

#[derive(Debug, Clone)]
pub enum WorldAnswered {
    RawString(String),
}
