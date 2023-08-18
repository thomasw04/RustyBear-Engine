use std::ops;
use instant::Instant;


pub struct Timestep {
    delta: f64,
    last: Instant,
}

impl Timestep {

    pub fn new() -> Timestep
    {
        Timestep { delta: 0.0, last: Instant::now() }
    }

    pub fn step_fwd(&mut self) -> &mut Self
    {
        self.delta = self.last.elapsed().as_nanos() as f64 / 1000000.0;
        self.last = Instant::now();
        self
    } 

    pub fn norm(&self) -> f32
    {
        (self.delta / 10.0) as f32
    }

    pub fn micros(&self) -> i64
    {
        (self.delta * 1000.0) as i64
    }

    pub fn millis(&self) -> f64
    {
        self.delta
    }

    pub fn seconds(&self) -> f64 
    {
        self.delta / 1000.0       
    }
}

impl From<f64> for Timestep {

    fn from(delta: f64) -> Timestep 
    {
        Timestep { delta, last: Instant::now() }
    }
}

impl From<Timestep> for f64 {

    fn from(value: Timestep) -> f64 
    {
        value.delta
    }
}

impl ops::AddAssign<f64> for Timestep {
    
    fn add_assign(&mut self, rhs: f64) 
    {
        self.delta += rhs;
    }
}

impl ops::SubAssign<f64> for Timestep {

    fn sub_assign(&mut self, rhs: f64) 
    {
        self.delta -= rhs;
    }
}

impl ops::MulAssign<f64> for Timestep {

    fn mul_assign(&mut self, rhs: f64) 
    {
        self.delta *= rhs;
    }
}

impl ops::DivAssign<f64> for Timestep {

    fn div_assign(&mut self, rhs: f64) 
    {
        self.delta /= rhs;
    }
}

