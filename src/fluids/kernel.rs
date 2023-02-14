use bevy::prelude::*;

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

fn smoothing_kernel_component(x_i: f32, x_j: f32, h: f32) -> f32 {
    let distance = (x_i - x_j).abs() / h;

    1.0/h.powi(DIMENSIONS) * cubic_spline(distance)
}

fn smoothing_kernel(x_i: Vec3, x_j: Vec3, h: f32) -> f32 {
    let distance = x_i.distance(x_j) / h;

    1.0/h.powi(DIMENSIONS) * cubic_spline(distance)
}

fn smoothing_kernel_grad(x_i: Vec3, x_j: Vec3, h: f32) -> Vec3 {
    let x = smoothing_kernel_component(x_i.x, x_j.x, h);
    let y = smoothing_kernel_component(x_i.y, x_j.y, h);
    let z = smoothing_kernel_component(x_i.z, x_j.z, h);

    1.0/h.powi(DIMENSIONS) * Vec3::new(x, y, z)
}

