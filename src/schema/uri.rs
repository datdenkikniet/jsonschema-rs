#[derive(Clone, Debug)]
pub enum UriParseError {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Uri {
    normalized: bool,
    value: String,
}

impl Uri {
    pub fn from_string(input: String) -> Result<Self, UriParseError> {
        Ok(Self {
            normalized: false,
            value: input,
        })
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn normalized(&self) -> bool {
        self.normalized
    }
}
