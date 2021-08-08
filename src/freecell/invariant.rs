use crate::deck::CARD_NUM;
use crate::freecell::basis::BASE_NUM;
use crate::freecell::basis::PILE_NUM;

pub const KEY_SIZE: usize = BASE_NUM + PILE_NUM + CARD_NUM;

/// A structure to hold a freecell game invariant.
#[derive(Hash, Eq, PartialEq, Debug, Copy, Clone)]
pub struct Key64 {
    data: [u8; KEY_SIZE],
}

impl Key64 {
    pub fn new() -> Self {
        Key64 {
            data: [0; KEY_SIZE],
        }
    }

    pub fn put(&mut self, index: usize, value: u8) {
        self.data[index] = value;
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;
    #[test]
    fn basics() {
        assert_eq!(64, std::mem::size_of::<Key64>());
        let mut a = Key64::new();
        let mut b = Key64::new();
        let mut key_map = HashMap::new();

        assert!(a == b);
        key_map.insert(a, "a");
        key_map.insert(b, "b");
        assert_eq!(1, key_map.len());
        assert_eq!(Some(&"b"), key_map.get(&a));

        for i in 0..KEY_SIZE {
            a.put(i, (i + 1) as u8);
            assert!(a != b);

            b.put(i, (i + 1) as u8);
            assert!(a == b);

            key_map.insert(a, "a");
            key_map.insert(b, "b");
        }
        assert_eq!(KEY_SIZE + 1, key_map.len());
    }
}
