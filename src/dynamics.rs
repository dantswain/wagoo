use crate::rand_util::Chaos;

pub trait DynamicSystem {
    fn step(&mut self, chaos: &mut Chaos);
    fn get_position(&self) -> cgmath::Vector3<f32>;
}

pub struct Circler {
    pub heading: f32,
    pub omega: f32,
    pub speed: f32,
    pub position: cgmath::Vector3<f32>,
}

impl Circler {
    #[allow(dead_code)]
    pub fn new(mean_speed: f32, mean_omega: f32, lims: f32, chaos: &mut Chaos) -> Self {
        Self {
            heading: chaos.unit_radian_noise(),
            omega: mean_omega + 0.1 * mean_omega * chaos.unit_noise(),
            speed: mean_speed + 0.1 * mean_speed * chaos.unit_noise(),
            position: chaos.random_position_in_cube(lims),
        }
    }
}

impl DynamicSystem for Circler {
    fn step(&mut self, chaos: &mut Chaos) {
        let vx = self.speed * self.heading.cos();
        let vy = self.speed * self.heading.sin();

        self.position.x += vx + 0.005 * chaos.unit_noise();
        self.position.y += vy + 0.005 * chaos.unit_noise();
        self.position.z += -0.001 * self.position.z + 0.01 * chaos.unit_noise();
        self.heading += self.omega + 0.05 * chaos.unit_noise();
    }

    fn get_position(&self) -> cgmath::Vector3<f32> {
        self.position
    }
}

pub struct Lorenz {
    pub sigma: f32,
    pub rho: f32,
    pub beta: f32,
    pub speed: f32,
    pub position: cgmath::Vector3<f32>,
}

impl Lorenz {
    pub fn new(sigma: f32, rho: f32, beta: f32, speed: f32, lims: f32, chaos: &mut Chaos) -> Self {
        Self {
            sigma,
            rho,
            beta,
            speed,
            position: chaos.random_position_in_cube(lims),
        }
    }
}

impl DynamicSystem for Lorenz {
    fn step(&mut self, _chaos: &mut Chaos) {
        let dt = 0.016666;
        let px = self.position.x;
        let py = self.position.y;
        let pz = self.position.z;
        self.position.x += dt * self.speed * (self.sigma * (py - px));
        self.position.y += dt * self.speed * (px * (self.rho - pz) - py);
        self.position.z += dt * self.speed * (px * py - self.beta * pz);
    }

    fn get_position(&self) -> cgmath::Vector3<f32> {
        self.position
    }
}
