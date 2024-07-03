use cgmath::Vector3;

use crate::camera::CameraViewPort;
use crate::math::angle::ToAngle;
use crate::math::projection::ProjectionType;
use crate::time::Time;

/// State for inertia
pub struct Inertia {
    // Initial angular distance
    ampl: f64,
    speed: f64,
    // Vector of rotation
    axis: Vector3<f64>,
    // The time when the inertia begins
    time_start: Time,
}

impl Inertia {
    pub fn new(ampl: f64, axis: Vector3<f64>) -> Self {
        Inertia {
            time_start: Time::now(),
            ampl: ampl,
            speed: ampl,
            axis: axis,
        }
    }

    pub fn apply(&mut self, camera: &mut CameraViewPort, proj: &ProjectionType) {
        let t = ((Time::now() - self.time_start).as_millis() / 1000.0) as f64;
        // Undamped angular frequency of the oscillator
        // From wiki: https://en.wikipedia.org/wiki/Harmonic_oscillator
        //
        // In a damped harmonic oscillator system: w0 = sqrt(k / m)
        // where:
        // * k is the stiffness of the ressort
        // * m is its mass
        let w0 = 5.0;
        // The angular distance goes from d0 to 0.0
        self.speed = self.ampl * ((-w0 * t).exp());
        /*let alpha = 1_f32 + (0_f32 - 1_f32) * (10_f32 * t + 1_f32) * (-10_f32 * t).exp();
        let alpha = alpha * alpha;
        let fov = start_fov * (1_f32 - alpha) + goal_fov * alpha;*/
        camera.apply_rotation(&self.axis, self.speed.to_angle(), proj)
    }

    pub fn get_start_ampl(&self) -> f64 {
        self.ampl
    }

    pub fn get_cur_speed(&self) -> f64 {
        self.speed
    }
}
