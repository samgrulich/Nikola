use crate::fluids;
use glam::{Vec3A, vec3a};


const H: f32 = fluids::SMOOTHING_LENGHT;
const H_POW_D: f32 = H * H;
const FRAC_H_D: f32 = 1.0 / H_POW_D;
const FRAC_3_2PI: f32 = 3.0 / 2.0 * std::f32::consts::FRAC_1_PI;


fn cubic_spline(distance: f32) -> f32 {
    let q = if distance < 1.0 {
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

    FRAC_H_D * cubic_spline(distance)
}

pub fn smoothing_kernel_grad(x_i: Vec3A, x_j: Vec3A) -> Vec3A {
    let x = smoothing_kernel_component(x_i.x, x_j.x);
    let y = smoothing_kernel_component(x_i.y, x_j.y);
    let z = smoothing_kernel_component(x_i.z, x_j.z);

    FRAC_H_D * vec3a(x, y, z)
}

