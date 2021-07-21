use rand::prelude::*;

pub struct Chaos {
    rng: rand::rngs::ThreadRng,
    uniform_dist: rand::distributions::Uniform<f32>,
}

impl Chaos {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
            uniform_dist: rand::distributions::Uniform::new(0.0, 1.0),
        }
    }

    pub fn unit_noise(&mut self) -> f32 {
        self.uniform_sample() - 0.5
    }

    pub fn unit_radian_noise(&mut self) -> f32 {
        2.0 * std::f32::consts::PI * self.uniform_sample()
    }

    pub fn random_solid_color(&mut self) -> [f32; 4] {
        [
            self.uniform_sample(),
            self.uniform_sample(),
            self.uniform_sample(),
            1.0,
        ]
    }

    pub fn random_position_in_cube(&mut self, max: f32) -> cgmath::Vector3<f32> {
        cgmath::Vector3::<f32> {
            x: 2.0 * max * self.unit_noise(),
            y: 2.0 * max * self.unit_noise(),
            z: 2.0 * max * self.unit_noise(),
        }
    }

    pub fn bernoulli(&mut self, p_true: f32) -> bool {
        self.uniform_sample() < p_true
    }

    fn uniform_sample(&mut self) -> f32 {
        self.uniform_dist.sample(&mut self.rng)
    }
}
