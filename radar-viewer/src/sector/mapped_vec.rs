use std::collections::HashMap;

#[derive(Debug)]
pub struct MappedVec<T> {
    vec: Vec<T>,
    map: HashMap<String, usize>,
}
impl<T> Default for MappedVec<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> MappedVec<T> {
    pub fn new() -> Self {
        MappedVec {
            vec: vec![],
            map: HashMap::new(),
        }
    }
    pub fn entries(&mut self) -> &mut Vec<T> {
        &mut self.vec
    }
    pub fn with_capacity(capacity: usize) -> Self {
        MappedVec {
            vec: Vec::with_capacity(capacity),
            map: HashMap::with_capacity(capacity),
        }
    }
    pub fn insert(&mut self, name: String, value: T) {
        self.vec.push(value);
        self.map.insert(name, self.vec.len() - 1);
    }
    pub fn get_by_name(&self, name: &str) -> Option<&T> {
        self.map.get(name).and_then(|index| self.vec.get(*index))
    }
    pub fn get_by_name_mut(&mut self, name: &str) -> Option<&mut T> {
        self.map
            .get(name)
            .and_then(|index| self.vec.get_mut(*index))
    }
    pub fn for_each<F>(&mut self, mut f: F) where F: FnMut(&mut T) {
        for item in self.vec.iter_mut() {
            f(item);
        }
    }

    pub fn any<F>(&self, f: F) -> bool where F: Fn(&T) -> bool {
        for item in self.vec.iter() {
            if f(item) {
                return true;
            }
        }
        return false;
    }
}

impl<'a, T> IntoIterator for &'a MappedVec<T> {
    type Item = &'a T;
    type IntoIter = MappedVecIterator<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        MappedVecIterator {
            mapped_vec: self,
            index: 0,
        }
    }
}

pub struct MappedVecIterator<'a, T> {
    mapped_vec: &'a MappedVec<T>,
    index: usize,
}


impl<'a, T> Iterator for MappedVecIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.mapped_vec.vec.get(self.index);
        if value.is_some() {
            self.index += 1;
        }
        value
    }
}

