use std::ops;


pub struct Timestep {
    delta: f64,
}

impl Timestep {

    pub fn nanos(&self) -> i64 {
        (self.delta * 1000.0) as i64
    }

    pub fn millis(&self) -> f64 {
        self.delta
    }

    pub fn seconds(&self) -> f64 {
        self.delta / 1000.0       
    }
}

impl From<f64> for Timestep {

    fn from(delta: f64) -> Timestep {
        Timestep { delta: delta }
    }
}

impl Into<f64> for Timestep {

    fn into(self) -> f64 {
        self.delta
    }
}

impl ops::AddAssign<f64> for Timestep {
    
    fn add_assign(&mut self, rhs: f64) {
        self.delta += rhs;
    }
}

impl ops::SubAssign<f64> for Timestep {

    fn sub_assign(&mut self, rhs: f64) {
        self.delta -= rhs;
    }
}

impl ops::MulAssign<f64> for Timestep {

    fn mul_assign(&mut self, rhs: f64) {
        self.delta *= rhs;
    }
}

impl ops::DivAssign<f64> for Timestep {

    fn div_assign(&mut self, rhs: f64) {
        self.delta /= rhs;
    }
}

