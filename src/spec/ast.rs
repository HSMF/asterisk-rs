#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub(super) rules: Vec<Rule>,
    pub(super) configs: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub(super) name: String,
    pub(super) typ: String,
    pub(super) expansions: Vec<Expansion>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expansion {
    pub(super) tokens: Vec<String>,
    pub(super) code: String,
}
