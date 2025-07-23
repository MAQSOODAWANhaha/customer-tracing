use regex::Regex;

pub fn validate_username(username: &str) -> bool {
    if username.len() < 3 || username.len() > 50 {
        return false;
    }
    
    // Username should contain only alphanumeric characters and underscores
    let re = Regex::new(r"^[a-zA-Z0-9_]+$").unwrap();
    re.is_match(username)
}

pub fn validate_password(password: &str) -> bool {
    password.len() >= 6
}

pub fn validate_name(name: &str) -> bool {
    !name.trim().is_empty() && name.len() <= 100
}

pub fn validate_phone(phone: &str) -> bool {
    if phone.is_empty() {
        return true; // Phone is optional
    }
    
    // Basic phone validation - adjust regex as needed
    let re = Regex::new(r"^[\d\-\+\(\)\s]{7,20}$").unwrap();
    re.is_match(phone)
}

pub fn validate_rate(rate: i32) -> bool {
    (0..=5).contains(&rate)
}