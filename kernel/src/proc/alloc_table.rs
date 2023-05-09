use alloc::vec::Vec;

#[derive(Clone)]
pub struct AllocTable<T: Clone> {
    entries: Vec<Option<T>>,
}

impl<T: Clone> Default for AllocTable<T> {
    fn default() -> Self {
        Self {
            entries: Default::default(),
        }
    }
}

impl<T: Clone> AllocTable<T> {
    pub fn new(entries: Vec<Option<T>>) -> Self {
        Self { entries }
    }

    pub fn alloc(&mut self, item: T) -> usize {
        let pos = self.entries.iter().position(|x| x.is_none());
        if let Some(pos) = pos {
            self.entries[pos] = Some(item);
            pos
        } else {
            self.entries.push(Some(item));
            self.entries.len() - 1
        }
    }

    pub fn get(&self, id: usize) -> Option<T> {
        self.entries[id].clone()
    }

    pub fn get_mut(&mut self, id: usize) -> &mut Option<T> {
        &mut self.entries[id]
    }

    pub fn dealloc(&mut self, id: usize) -> bool {
        if id >= self.entries.len() {
            false
        } else {
            let flag = self.entries[id].is_some();
            self.entries[id] = None;
            flag
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len() - self.entries.iter().filter(|x| x.is_none()).count()
    }
}
