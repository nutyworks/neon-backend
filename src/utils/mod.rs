pub(crate) mod strings {
    use rand_core::RngCore;

    pub fn generate_random_string(len: usize) -> String {
        vec![0; len]
            .iter()
            .map(|_| {
                let v = rand_core::OsRng.next_u32() % 62;
                if v <= 9 {
                    ('0' as u32) + v
                } else if 10 <= v && v <= 35 {
                    ('A' as u32) + v - 10
                } else {
                    ('a' as u32) + v - 36
                }
            })
            .map(char::from_u32)
            .scan(Some(""), |acc, x| match acc {
                Some(acc) => match x {
                    Some(x) => Some(acc.to_string() + &x.to_string()),
                    None => None,
                },
                None => None,
            })
            .collect()
    }
}
