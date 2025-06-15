/* 
    General details 
*/

pub struct GeneralData {
    env: String,
}

impl GeneralData {
    pub const VERSION: &str = "0.1";
    pub const AUTHOR: &str = "CortexFlow";
    pub const DESCRIPTION: &str = "";

    pub fn new(env: String) -> Self {
        GeneralData { env: env.to_string() }
    }
    pub fn set_env(mut self, env: String) {
        self.env = env;
    }
    pub fn get_env(self) -> String {
        self.env
    }
    pub fn get_env_output(self) {
        println!("{:?}", self.env)
    }
}
