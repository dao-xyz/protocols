

pub fn entity_name_is_valid(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    // check if leading or trailing spaces, if so name is invalid
    let mut chars = name.chars();
    if chars.next().unwrap().is_whitespace() || chars.last().unwrap_or('_').is_whitespace() {
        return false;
    }
    true
}
#[cfg(test)]
pub mod tests {
    use super::*;
    #[test]
    pub fn fail_empty() {
        assert!(!entity_name_is_valid(""));
    }

    #[test]
    pub fn fail_left_padding() {
        assert!(!entity_name_is_valid(" x"));
    }

    #[test]
    pub fn fail_right_padding() {
        assert!(!entity_name_is_valid("x "));
    }

    #[test]
    pub fn valid() {
        assert!(entity_name_is_valid("x"));
    }
}
