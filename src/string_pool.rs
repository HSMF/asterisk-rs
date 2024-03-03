#[derive(Debug, Clone, Default)]
pub struct Pool {
    inner: Vec<String>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(usize);

impl Pool {
    /// constructs a new, empty Pool
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn add<S>(&mut self, string: S) -> Id
    where
        S: ToOwned<Owned = String>,
    {
        let string = string.to_owned();
        if let Some((index, _)) = self.inner.iter().enumerate().find(|&(_, s)| s == &string) {
            return Id(index);
        }

        self.inner.push(string);
        Id(self.inner.len() - 1)
    }

    pub fn get(&self, id: Id) -> &str {
        &self.inner[id.0]
    }
}
