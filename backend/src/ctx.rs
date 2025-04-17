#[derive(Clone, Debug)]
pub struct Ctx {
    user_id: String,
    username: String,
}

// Constructor.
impl Ctx {
    pub fn new(user_id: String, username: String) -> Self {
        Self { user_id, username }
    }
}

// Property Accessors.
impl Ctx {
    pub fn user_id(&self) -> &str {
        &self.user_id
    }
    
    pub fn username(&self) -> &str {
        &self.username
    }
}