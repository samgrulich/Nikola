use bevy::prelude::*;

use crate::fluids;

const DIMENSIONS: i32 = 2;

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

    3.0 / (2.0 * std::f32::consts::PI) * q
}

fn smoothing_kernel_component(x_i: f32, x_j: f32, h: Option<f32>) -> f32 {
    let h = h.unwrap_or(fluids::SMOOTHING_LENGHT);
    let distance = (x_i - x_j).abs() / h;

    1.0/h.powi(DIMENSIONS) * cubic_spline(distance)
}

pub fn smoothing_kernel(x_i: Vec3, x_j: Vec3, h: Option<f32>) -> f32 {
    let h = h.unwrap_or(fluids::SMOOTHING_LENGHT);
    let distance = x_i.distance(x_j) / h;

    1.0/h.powi(DIMENSIONS) * cubic_spline(distance)
}

pub fn smoothing_kernel_grad(x_i: Vec3, x_j: Vec3, h: Option<f32>) -> Vec3 {
    let h = h.unwrap_or(fluids::SMOOTHING_LENGHT);

    let x = smoothing_kernel_component(x_i.x, x_j.x, Some(h));
    let y = smoothing_kernel_component(x_i.y, x_j.y, Some(h));
    let z = smoothing_kernel_component(x_i.z, x_j.z, Some(h));

    1.0/h.powi(DIMENSIONS) * Vec3::new(x, y, z)
}

