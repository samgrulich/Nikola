#define PI 3.1415926535

struct Particle {
    @location(0) position: vec2<f32>,
    @location(1) mass: f32,
    @location(2) velocity: vec2<f32>,
}

@group(0) @binding(0) var<storage> ins: array<Particle>;
@group(0) @binding(1) var<storage, read_write> outs: array<Particle>;
@group(0) @binding(2) var<storage> time_step: f32;

let h = 10;

fn calc_density(mj: f32, smoothed: f32) -> f32 {
    return mj * smoothed;
}

fn calc_particle_pressure(k: f32, ro: f32, rest_ro: f32) -> f32 {
    return k * (ro - rest_ro);
}

fn calc_pressure(mj: f32, PI: f32, pj: f32, roj: f32, smoothed: f32, direction: vec2<f32>) -> vec2<f32> {
    return -mj * (PI + pj) / (2 * roj) * smoothed * direction;
}

// don't forget to multiply by mu
fn calc_viscosity( mj: f32, vi: f32, vj: f32, roj: f32, laplacian_smoothed: f32, direction: vec2<f32>) -> vec2<f32> {
    return (vj - vi) / roj * laplacian_smoothed * direction;
}

fn calc_color_field(mj: f32, roj: f32, smoothed: f32) {
    return mj * 1 / roj * smoothed;
}

fn calc_tension(color_field: f32) {

}


fn smooth(ri: vec2<f32>, rj: vec2<f32>, f32 (&kernel)(f32)) {
    let r = distance(ri, rj);

    if (h < r || r < 0) {
        return 0;
    }

    return kernel(r);
}

fn poly6_kernel(r: f32) -> f32 {
    return 315 / (64 * PI * pow(h, 9)) * pow(pow(h, 2) - pow(r, 2), 3);
}

fn div_poly6_kernel(r: f32) -> f32 {
	    return (-945/(64*PI*pow(h, 9)) * pow((pow(h, 2) - pow(r, 2), 2) * r;
}

fn spiky_kernel(r: f32) -> f32 {
    return 15 / (PI * pow(h, 6)) * pow(h - r, 3);
}

fn div_spiky_kernel(r: f32) -> f32 {
    return (15 / (PI * pow(h, 6)) * pow((h - r), 2) * r;
}

fn viscosity_kernel(r: f32) -> f32 {
    return 15 / (2 * PI * pow(h, 3)) * ( - pow(r, 3) / (2 * pow(h, 3)) + pow(r, 2) / pow(h, 2) + h / (2 * r) - 1);
}

fn lap_viscosity_kernel(r: f32) -> f32 {
    return 45 / (PI * pow(h, 6)) * (h - r);
}


@compute @workgroup_size(8, 8)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let id = global_id.y * 4u + global_id.x;
    var particle = ins[id];
   
    // velocity update
    particle.vel.y -= 0.0003f;
    for (var i: i32 = 0; i < i32(arrayLength(&ins)); i++) {
        if (i == i32(id)){
            continue;
        }
        
        let other = ins[i];
        let dst = distance(particle.pos, other.pos);

        if (dst <= 0.5f) {
            particle.pos.y += sign(particle.pos.y - other.pos.y) * 0.001f;
            particle.vel *= -1f;
            break;
        }
    }

    // position update
    particle.pos += particle.vel;

    if (particle.pos.y >= 0f) {
        outs[id] = particle;
    }
}
