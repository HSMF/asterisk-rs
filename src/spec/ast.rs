#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    rules: Vec<Rule>,
    configs: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    name: String,
    typ: String,
    expansions: Vec<Expansion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expansion {
    tokens: Vec<String>,
    code: String,
}
