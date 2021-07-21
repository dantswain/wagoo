use crate::sphere;

pub struct TailBuffer {
    capacity: usize,
    write_pointer: usize,
    high_water_mark: usize,
    len: usize,
    data: Vec<sphere::SphereVertex>,
}

impl TailBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            write_pointer: 0,
            high_water_mark: 0,
            len: 0,
            data: Vec::<sphere::SphereVertex>::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, position: cgmath::Vector3<f32>) {
        let vertex = sphere::SphereVertex {
            position: [position.x, position.y, position.z],
        };

        if self.high_water_mark == self.write_pointer {
            self.data.push(vertex);
        } else {
            self.data[self.write_pointer] = vertex;
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

    pub fn to_vec(&self) -> Vec<sphere::SphereVertex> {
        let mut out = Vec::<sphere::SphereVertex>::new();

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
        let mut b = TailBuffer::new(3);
        assert_eq!(b.len(), 0);
        let x = cgmath::Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        b.push(x);
        assert_eq!(b.len(), 1);
        let v = b.to_vec();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].position, [0.0, 0.0, 0.0]);
        b.push(cgmath::Vector3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        });
        assert_eq!(b.len(), 2);
        b.push(cgmath::Vector3 {
            x: 2.0,
            y: 2.0,
            z: 2.0,
        });
        assert_eq!(b.len(), 3);
        b.push(cgmath::Vector3 {
            x: 3.0,
            y: 3.0,
            z: 3.0,
        });
        assert_eq!(b.len(), 3);

        let vv = b.to_vec();
        assert_eq!(vv.len(), 3);
        assert_eq!(vv[2].position, [1.0, 1.0, 1.0]);
        assert_eq!(vv[1].position, [2.0, 2.0, 2.0]);
        assert_eq!(vv[0].position, [3.0, 3.0, 3.0]);
    }
}
