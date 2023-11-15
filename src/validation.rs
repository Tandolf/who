pub fn check_length(value: &String) -> bool {
    value.chars().count() <= 255
}

pub fn check_token_length(value: &String) -> (&str, bool) {
    let tokens = value.split('.');
    for t in tokens {
        if t.chars().count() > 63 {
            return (t, false);
        }
    }
    (value, true)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rand::{distributions::Alphanumeric, thread_rng, Rng};

    use super::*;

    #[test]
    pub fn too_long_total_length() {
        let v: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(256)
            .map(char::from)
            .collect();

        assert!(!check_length(&v))
    }

    #[test]
    pub fn total_length_okey() {
        let v: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(255)
            .map(char::from)
            .collect();

        assert!(check_length(&v))
    }

    #[test]
    pub fn token_length_okey() {
        let mut t1: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        let t2: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(15)
            .map(char::from)
            .collect();

        t1.push('.');
        t1.push_str(&t2);

        assert_eq!((t1.as_str(), true), check_token_length(&t1))
    }

    #[test]
    pub fn token_length_not_okey() {
        let mut t1: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(65)
            .map(char::from)
            .collect();

        let t2: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(65)
            .map(char::from)
            .collect();

        let expected = t1.clone();

        t1.push('.');
        t1.push_str(&t2);

        assert_eq!((expected.as_str(), false), check_token_length(&t1))
    }
}
