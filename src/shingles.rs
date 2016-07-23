use std::cmp;
use std::hash::{Hash, Hasher, SipHasher};

pub struct Shingles<'a, T: ?Sized + 'a> {
    data: &'a T,
    size: usize,
    step: usize,
}

impl<'a, T: ?Sized> Shingles<'a, T> {
    pub fn new(data: &'a T, size: usize) -> Self {
        Shingles {
            data: data,
            size: size,
            step: 1,
        }
    }

    pub fn new_with_step(data: &'a T, size: usize, step: usize) -> Self {
        Shingles {
            data: data,
            size: size,
            step: step,
        }
    }

    pub fn hashes(self) -> ShingleHashes<'a, T> {
        ShingleHashes { shingles: self }
    }
}

impl<'a, T> Iterator for Shingles<'a, [T]> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.len() >= self.size {
            let ret = &self.data[0..self.size];
            let step = cmp::min(self.step, self.data.len());
            self.data = &self.data[step..];
            Some(ret)
        } else {
            None
        }
    }
}

impl<'a> Iterator for Shingles<'a, str> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut pos_next: usize = 0;
        let mut pos_end: usize = 0;
        let mut chars = 0;

        let iter = self.data.as_bytes().iter()
            .enumerate()
            // if char boundary
            .filter(|&(_, &b)| b < 128 || b >= 192);

        for (i, _) in iter {
            if chars == self.step {
                pos_next = i;
            }
            if chars == self.size {
                pos_end = i;
            }
            if pos_next != 0 && pos_end != 0 {
                break;
            }
            chars += 1;
        }

        let ret = if pos_end != 0 {
            Some(&self.data[0..pos_end])
        } else if chars == self.size {
            Some(self.data)
        } else {
            None
        };

        if pos_next != 0 {
            self.data = &self.data[pos_next..];
        } else {
            self.data = &self.data[self.data.len()..];
        }

        ret
    }
}

pub trait AsShingles<'a, T: ?Sized + 'a> {
    fn as_shingles(&'a self, size: usize) -> Shingles<'a, T>;

    fn as_shingles_with_step(&'a self, size: usize, step: usize) -> Shingles<'a, T>;
}

impl<'a, T: 'a> AsShingles<'a, [T]> for [T] {
    fn as_shingles(&'a self, size: usize) -> Shingles<'a, [T]> {
        Shingles::new(self, size)
    }

    fn as_shingles_with_step(&'a self, size: usize, step: usize) -> Shingles<'a, [T]> {
        Shingles::new_with_step(self, size, step)
    }
}

impl<'a> AsShingles<'a, str> for str {
    fn as_shingles(&'a self, size: usize) -> Shingles<'a, str> {
        Shingles::new(self, size)
    }

    fn as_shingles_with_step(&'a self, size: usize, step: usize) -> Shingles<'a, str> {
        Shingles::new_with_step(self, size, step)
    }
}

pub struct ShingleHashes<'a, T: ?Sized + 'a> {
    shingles: Shingles<'a, T>,
}

impl<'a, T: ?Sized, K> Iterator for ShingleHashes<'a, T>
    where Shingles<'a, T>: Iterator<Item = K>,
          K: Hash
{
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(s) = self.shingles.next() {
            let mut h = SipHasher::new();
            s.hash(&mut h);
            Some(h.finish())
        } else {
            None
        }
    }
}