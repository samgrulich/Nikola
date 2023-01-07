struct Particle {
    @location(0) position: vec2<f32>,
    @location(1) velocity: vec2<f32>,
    @location(2) mass: f32,
    @location(3) density: f32,
}

@group(0) @binding(0) var<storage, read_write> ins:  array<Particle>;
@group(0) @binding(1) var<storage, read_write> outs: array<Particle>;
@group(0) @binding(2) var<storage> time_step: f32;
@group(0) @binding(3) var<storage> rest_density: f32;


let H = 1.2f;
let PI = 3.1415926535f;
let gas_constant = 1f;
let surface_treshold = 1f;
let tension_coeficient = 0.2f;
let viscous_coeficient = 0.7f;


fn poly6_kernel(ri: vec2<f32>, rj: vec2<f32>) -> f32 {
    let r = distance(ri, rj);

    if (H < r || r < 0f) {
        return 0f;
    }

    return 315f / (64f * PI * pow(H, 9f)) * pow(pow(H, 2f) - pow(r, 2f), 3f);
}

fn grad_poly6_kernel(ri: vec2<f32>, rj: vec2<f32>) -> f32 {
    let r = distance(ri, rj);

    if (H < r || r < 0f) {
        return 0f;
    }

    return -945f / (64f * PI * pow(H, 9f)) * pow(pow(H, 2f) - pow(r, 2f), 2f) * r;
}

fn lap_poly6_kernel(ri: vec2<f32>, rj: vec2<f32>) -> f32 {
    let r = distance(ri, rj);

    if (H < r || r < 0f) {
        return 0f;
    }

    return 945f / (32f * PI * pow(H, 9f)) * (pow(H, 2f) - pow(r, 2f)) * (3f * pow(r, 2f) - pow(H, 2f));
}

fn spiky_kernel(ri: vec2<f32>, rj: vec2<f32>) -> f32 {
    let r = distance(ri, rj);

    if (H < r || r < 0f) {
        return 0f;
    }

    return 15f / (PI * pow(H, 6f)) * pow(H - r, 3f);
}

fn grad_spiky_kernel(ri: vec2<f32>, rj: vec2<f32>) -> f32 {
    let r = distance(ri, rj);

    if (H < r || r < 0f) {
        return 0f;
    }

    return 15f / (PI * pow(H, 6f)) * pow((H - r), 2f) * r;
}

fn viscosity_kernel(ri: vec2<f32>, rj: vec2<f32>) -> f32 {
    let r = distance(ri, rj);

    if (H < r || r < 0f) {
        return 0f;
    }

    return 15f / (2f * PI * pow(H, 3f)) * ( - pow(r, 3f) / (2f * pow(H, 3f)) + pow(r, 2f) / pow(H, 2f) + H / (2f * r) - 1f);
}

fn lap_viscosity_kernel(ri: vec2<f32>, rj: vec2<f32>) -> f32 {
    let r = distance(ri, rj);

    if (H < r || r < 0f) {
        return 0f;
    }

    return 45f / (PI * pow(H, 6f)) * (H - r);
}



fn calc_density(mj: f32, ri: vec2<f32>, rj: vec2<f32>) -> f32 {
    return mj * poly6_kernel(ri, rj);
}

fn calc_particle_pressure(k: f32, ro: f32, rest_ro: f32) -> f32 {
    return k * (ro - rest_ro);
}

fn calc_pressure(mj: f32, pi: f32, pj: f32, roj: f32, ri: vec2<f32>, rj: vec2<f32>) -> vec2<f32> {
    let r = normalize(rj - ri);
    return -mj * (pi + pj) / (2f * roj) * grad_spiky_kernel(ri, rj) * r;
}

// don't forget to multiply by mu
fn calc_viscosity(mj: f32, vi: vec2<f32>, vj: vec2<f32>, roj: f32, ri: vec2<f32>, rj: vec2<f32>) -> vec2<f32> {
    return (vj - vi) / roj * lap_viscosity_kernel(ri, rj);
}

fn calc_color_field(mj: f32, roj: f32, smoothed: f32, r: vec2<f32>) -> vec2<f32> {
    return mj * (1f / roj) * smoothed * r;
}

fn grad_color_field(mj: f32, roj: f32, ri: vec2<f32>, rj: vec2<f32>) -> vec2<f32> {
    let r = normalize(rj - ri);
    return calc_color_field(mj, roj, grad_poly6_kernel(ri, rj), r);
}

fn lap_color_field(mj: f32, roj: f32, ri: vec2<f32>, rj: vec2<f32>) -> vec2<f32> {
    let r = normalize(rj - ri);
    return calc_color_field(mj, roj, lap_poly6_kernel(ri, rj), r); 
}

fn calc_tension(grad: vec2<f32>, lap: vec2<f32>) -> vec2<f32> {
    let n = length(grad);
    let lap = length(lap);

    if (abs(n) < H) {
        return vec2(0f);
    }

    let force = - tension_coeficient * lap * normalize(grad);
}



@compute @workgroup_size(8, 8)
fn main(
    @builtin(global_invocation_id) global_id: vec3<u32>,
) {
    let id = global_id.y * 4u + global_id.x;
    var particle = ins[id];

    let pressure = calc_particle_pressure(gas_constant, particle.density, rest_density);

    // calculate density
    var density = 0f;
    for (var j: i32 = 0; j < i32(arrayLength(&ins)); j++) {
        if (id == u32(j)) {
            continue;
        }

        let neighbor = ins[j];
        density += calc_density(neighbor.mass, particle.position, neighbor.position);
    }

    particle.density = density;
    ins[id] = particle;

// todo: test if working properly, in case move the density update into separate unit
    storageBarrier();

    // calculate forces 
    var pressure_force = vec2(0f);
    var viscous_force = vec2(0f);
    var tension_grad = vec2(0f);
    var tension_lap = vec2(0f);

    for (var j: i32 = 0; j < i32(arrayLength(&ins)); j++) {
        if (id == u32(j)) {
            continue;
        }

        let neighbor = ins[j];
        
        // pressure calculation
        let neighbor_pressure = calc_particle_pressure(gas_constant, neighbor.density, rest_density);
        pressure_force += calc_pressure(neighbor.mass, pressure, neighbor_pressure, neighbor.density, particle.position, neighbor.position);
    
        // viscosity calculation
        viscous_force += calc_viscosity(neighbor.mass, particle.velocity, neighbor.velocity, neighbor.density, particle.position, neighbor.position);

        // surface tension calculation 
        tension_grad += grad_color_field(neighbor.mass, neighbor.density, particle.position, neighbor.position);
        tension_lap  +=  lap_color_field(neighbor.mass, neighbor.density, particle.position, neighbor.position);
    }

    let tension_force = calc_tension(tension_grad, tension_lap);
    let forces = pressure_force + viscous_coeficient * viscous_force + tension_force; 

    // calculate acceleration 
    let acceleration = forces / density;

    // calculate velocity
    particle.velocity += acceleration * time_step;

    // calculate new position 
    particle.position += particle.velocity;

    // update particle velocity, position, density
    outs[id] = particle;
}
