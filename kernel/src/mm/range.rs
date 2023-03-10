pub trait Step {
    fn step(&mut self);
}

#[derive(Clone, Copy)]
pub struct Range<T: Eq + Copy + Step> {
    pub start: T,
    pub end: T,
}

pub struct Iter<T> {
    curr: T,
    end: T,
}

impl<T> Range<T>
where
    T: Eq + Copy + Step,
{
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            curr: self.start,
            end: self.end,
        }
    }
}

impl<T> Iterator for Iter<T>
where
    T: Eq + Copy + Step,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.end {
            None
        } else {
            let result = self.curr;
            self.curr.step();
            Some(result)
        }
    }
}
