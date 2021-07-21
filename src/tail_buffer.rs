pub struct TailBuffer<T: Copy> {
    capacity: usize,
    write_pointer: usize,
    high_water_mark: usize,
    len: usize,
    data: Vec<T>,
}

impl<T: Copy> TailBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            write_pointer: 0,
            high_water_mark: 0,
            len: 0,
            data: Vec::<T>::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, el: T) {
        if self.high_water_mark == self.write_pointer {
            self.data.push(el);
        } else {
            self.data[self.write_pointer] = el;
        }

        self.write_pointer = self.write_pointer + 1;
        if self.write_pointer >= self.capacity {
            self.write_pointer = 0;
        }

        if self.len < self.capacity {
            self.len = self.len + 1
        }

        self.high_water_mark = std::cmp::max(self.high_water_mark, self.write_pointer);
    }

    pub fn to_vec(&self) -> Vec<T> {
        let mut out = Vec::<T>::new();

        for ix in (0..self.write_pointer).rev() {
            out.push(self.data[ix]);
        }

        if self.high_water_mark > self.write_pointer {
            for ix in (self.write_pointer..self.capacity).rev() {
                out.push(self.data[ix]);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_works() {
        let mut b = TailBuffer::<u32>::new(3);
        assert_eq!(b.len(), 0);
        b.push(0);
        assert_eq!(b.len(), 1);
        let v = b.to_vec();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0], 0);
        b.push(1);
        assert_eq!(b.len(), 2);
        b.push(2);
        assert_eq!(b.len(), 3);
        b.push(3);
        assert_eq!(b.len(), 3);

        let vv = b.to_vec();
        assert_eq!(vv.len(), 3);
        assert_eq!(vv[2], 1);
        assert_eq!(vv[1], 2);
        assert_eq!(vv[0], 3);
    }
}
