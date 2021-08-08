use std::collections::btree_map::Keys;
use std::collections::BTreeMap;

pub struct Grader<K, V>
where
    K: Ord,
{
    data: BTreeMap<K, Vec<V>>,
}

impl<K, V> Grader<K, V>
where
    K: Ord,
{
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.values().map(|row| row.len()).sum()
    }

    pub fn grade_num(&self) -> usize {
        self.data.len()
    }

    pub fn add(&mut self, grade: K, value: V) {
        let row = self.data.entry(grade).or_insert_with(Vec::new);
        row.push(value);
    }

    pub fn grades(&self) -> Keys<K, Vec<V>> {
        self.data.keys()
    }

    pub fn split_off(&mut self, grade: K, limit: usize) -> Option<Vec<V>> {
        let mut row = self.data.remove(&grade)?;
        if row.len() > limit {
            self.data.insert(grade, row.split_off(limit));
        }
        Some(row)
    }

    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut Vec<V>) -> bool,
    {
        self.data.retain(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        let mut grader = Grader::new();

        grader.add(1, "one");
        grader.add(2, "two");
        grader.add(3, "three");

        assert_eq!(3, grader.grade_num());
        assert_eq!(3, grader.len());

        grader.add(3, "3");
        grader.add(2, "2");
        grader.add(1, "1");

        grader.add(1, "01");
        grader.add(2, "10");
        grader.add(3, "11");

        assert_eq!(3, grader.grade_num());
        assert_eq!(9, grader.len());
        assert_eq!(
            vec![1, 2, 3],
            grader.grades().cloned().collect::<Vec<usize>>()
        );

        assert_eq!(
            vec!["one", "1", "01"],
            grader.split_off(1, 3).expect("Should exist!")
        );

        assert_eq!(2, grader.grade_num());
        assert_eq!(6, grader.len());
        assert_eq!(vec![2, 3], grader.grades().cloned().collect::<Vec<usize>>());

        assert_eq!(
            vec!["two", "2"],
            grader.split_off(2, 2).expect("Should exist!")
        );

        assert_eq!(2, grader.grade_num());
        assert_eq!(4, grader.len());
        assert_eq!(vec![2, 3], grader.grades().cloned().collect::<Vec<usize>>());

        assert_eq!(
            vec!["three"],
            grader.split_off(3, 1).expect("Should exist!")
        );

        assert_eq!(2, grader.grade_num());
        assert_eq!(3, grader.len());
        assert_eq!(vec![2, 3], grader.grades().cloned().collect::<Vec<usize>>());

        // drain out
        assert_eq!(vec!["10"], grader.split_off(2, 2).expect("Should exist!"));
        assert_eq!(vec!["3"], grader.split_off(3, 1).expect("Should exist!"));
        assert_eq!(vec!["11"], grader.split_off(3, 3).expect("Should exist!"));

        // empty test
        assert_eq!(0, grader.grade_num());
        assert_eq!(0, grader.len());
        assert_eq!(None, grader.split_off(3, 3));
    }
}
