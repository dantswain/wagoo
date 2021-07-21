pub struct Sampler {
    period: u8,
    count: u8,
}

impl Sampler {
    pub fn new(period: u8) -> Self {
        Self {
            period,
            count: period - 1, // so that the first check returns true
        }
    }

    pub fn check(&mut self) -> bool {
        self.count = (self.count + 1) % self.period;
        self.count == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_works() {
        let mut s = Sampler::new(3);
        assert!(s.check());
        assert!(s.check() == false);
        assert!(s.check() == false);
        assert!(s.check());
        assert!(s.check() == false);
        assert!(s.check() == false);
        assert!(s.check());
    }
}
