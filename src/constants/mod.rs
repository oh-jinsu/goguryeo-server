use std::error::Error;

pub struct Constants {
    pub auth_secret: String,
}

impl Constants {
    pub fn init() -> Result<Self, Box<dyn Error>> {
        let auth_secret = std::env::var("AUTH_SECRET")?;

        Ok(Constants { auth_secret })
    }
}