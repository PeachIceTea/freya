pub fn random_string(length: usize) -> String {
    use rand::distributions::{Alphanumeric, DistString};

    Alphanumeric.sample_string(&mut rand::thread_rng(), length)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_string_length() {
        let length = 10;
        let random_str = random_string(length);
        assert_eq!(random_str.len(), length);
    }

    #[test]
    fn test_random_string_uniqueness() {
        let length = 15;
        let random_str1 = random_string(length);
        let random_str2 = random_string(length);
        assert_ne!(random_str1, random_str2);
    }
}
