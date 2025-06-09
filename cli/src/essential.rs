use crate::general::GeneralData;

pub fn update_cli() {
    println!("Updating CortexFlow CLI");
    println!("Looking for a newer version");
}
pub fn info(general_data: GeneralData) {
    println!("Version: {}", GeneralData::VERSION);
    println!("Author: {}", GeneralData::AUTHOR);
    println!("Description:{}", GeneralData::DESCRIPTION);
    println!("Environment: {}", general_data.get_env());
}
