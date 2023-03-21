use crate::fluids;
use glam::{Vec3A, vec3a};


const H: f32 = fluids::SMOOTHING_LENGHT;
const H_POW_D: f32 = H * H * H; // should this be 2?
const FRAC_H_D: f32 = 1.0 / H_POW_D;
const FRAC_3_2PI: f32 = 3.0 / 2.0 * std::f32::consts::FRAC_1_PI;


fn cubic_spline(distance: f32) -> f32 {
    let q = 
        if distance < 1.0 {
            2.0/3.0 - distance.powi(2) + 1.0/3.0 * distance.powi(3)
        }
        else if distance < 2.0 {
            1.0/6.0 * (2.0 - distance).powi(3)
        }
        else {
            0.0
        };

    FRAC_3_2PI * q
}

pub fn smoothing_kernel(x_i: Vec3A, x_j: Vec3A) -> f32 {
    let distance = x_i.distance(x_j) / H;

    FRAC_H_D * cubic_spline(distance)
}

fn smoothing_kernel_component(x_i: f32, x_j: f32) -> f32 {
    let distance = (x_i - x_j).abs() / H;

    cubic_spline(distance)
}

pub fn smoothing_kernel_grad(x_i: Vec3A, x_j: Vec3A) -> Vec3A {
    let x = smoothing_kernel_component(x_i.x, x_j.x);
    let y = smoothing_kernel_component(x_i.y, x_j.y);
    let z = smoothing_kernel_component(x_i.z, x_j.z);

    FRAC_H_D * vec3a(x, y, z)
}

#[cfg(test)]
mod tests {
    use glam::{Vec3A, vec3a};

    use super::{
        cubic_spline,
        smoothing_kernel,
        smoothing_kernel_grad,
        FRAC_3_2PI, H, FRAC_H_D
    };

    #[test]
    pub fn test_cubic_spline() {
        assert!(cubic_spline(0.0) == 2.0/3.0 * FRAC_3_2PI);
        assert!(cubic_spline(0.5) == (2.0/3.0 - 0.25 + 1.0/3.0 * 0.125) * FRAC_3_2PI);

        assert!(cubic_spline(1.0) == 1.0/6.0 * FRAC_3_2PI);
        assert!(cubic_spline(1.5) == 1.0/6.0 * 0.125 * FRAC_3_2PI);

        assert!(cubic_spline(2.0) == 0.0);
    }

    #[test] 
    pub fn test_distance() {
        assert!(Vec3A::ZERO.distance(Vec3A::ZERO) == 0.0);
        assert!(Vec3A::ONE.distance(Vec3A::ONE) == 0.0);
        assert!(Vec3A::X.distance(Vec3A::X) == 0.0);

        assert!(Vec3A::ZERO.distance(Vec3A::X) == 1.0);
        assert!(Vec3A::ZERO.distance(Vec3A::ONE) == Vec3A::ONE.length());
    }

    #[test]
    pub fn test_smoothing_kernel() {
        assert!(smoothing_kernel(Vec3A::ZERO, Vec3A::ZERO) == FRAC_H_D * cubic_spline(0.0));
        assert!(smoothing_kernel(Vec3A::ONE, Vec3A::ONE)   == FRAC_H_D * cubic_spline(0.0));
        
        assert!(smoothing_kernel(Vec3A::ZERO, Vec3A::X)    == FRAC_H_D * cubic_spline(1.0 / H));
        assert!(smoothing_kernel(Vec3A::ZERO, Vec3A::Y)    == FRAC_H_D * cubic_spline(1.0 / H));
        assert!(smoothing_kernel(Vec3A::X, Vec3A::ZERO)    == FRAC_H_D * cubic_spline(1.0 / H));

        assert!(smoothing_kernel(Vec3A::ZERO, Vec3A::ONE)       == FRAC_H_D * cubic_spline(Vec3A::ONE.length() / H));
        assert!(smoothing_kernel(Vec3A::ONE, 2.0 * Vec3A::ONE)  == FRAC_H_D * cubic_spline(Vec3A::ONE.length() / H));
    }

    #[test]
    pub fn test_smoothing_kernel_grad() {
        assert!(smoothing_kernel_grad(Vec3A::ZERO, Vec3A::ZERO) == FRAC_H_D * Vec3A::splat(cubic_spline(0.0)));
        assert!(smoothing_kernel_grad(Vec3A::ONE, Vec3A::ONE)   == FRAC_H_D * Vec3A::splat(cubic_spline(0.0)));
        
        assert!(smoothing_kernel_grad(Vec3A::ZERO, Vec3A::X)    == FRAC_H_D * vec3a(cubic_spline(1.0 / H), cubic_spline(0.0), cubic_spline(0.0)));
        assert!(smoothing_kernel_grad(Vec3A::ZERO, Vec3A::Y)    == FRAC_H_D * vec3a(cubic_spline(0.0), cubic_spline(1.0 / H), cubic_spline(0.0)));
        assert!(smoothing_kernel_grad(Vec3A::X, Vec3A::ZERO)    == FRAC_H_D * vec3a(cubic_spline(1.0 / H), cubic_spline(0.0), cubic_spline(0.0)));
    }
}
