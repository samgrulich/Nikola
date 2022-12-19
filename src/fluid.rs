use crate::computer;
use crate::general::State;

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone)]
struct Particle {
    density: f32,
    position: [f32; 3],
    velocity: [f32; 3],
}

unsafe impl bytemuck::Zeroable for Particle {}
unsafe impl bytemuck::Pod for Particle {}

struct Fluid {
    particles: Vec<Particle>,

    computer: computer::Computer,
    particles_buffer: wgpu::Buffer, // probbably?
}

// the particles should be loaded onto gpu, and then there is no 
// subsequent need to read it on the CPU again, thus it should be accessed only 
// by GPU (shaders)
impl Fluid {
    pub async fn new(state: &State, particle_count: usize) -> Self {
        // initialize particles with positions
        let mut particles: Vec<Particle> = vec![
            Particle {density: 0., position: [0., 0., 0.], velocity: [0., 0., 0.]}; particle_count.pow(3)
        ];

        for z in 0..particle_count {
            for y in 0..particle_count {
                for x in 0..particle_count {
                    let particle_idx = z * particle_count.pow(2) + y * particle_count + x;
                    particles[particle_idx].position = [
                        x as f32, 
                        y as f32, 
                        z as f32,
                    ];
                }
            }
        }
      
        // initialize computer
        let particles_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor { 
            label: Some("Color write buffer"),
            contents: bytemuck::cast_slice(particles.as_slice()),
            usage: 
                wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::STORAGE,
        });

        let computer = computer::Computer::new(
            state,
            computer::Dimensions::new(particle_count.pow(3) as u32, 1), 
            computer::Shader { path: "./res/shaders/compute.wgsl".to_string(), entry_point: "main".to_string() }, 
            vec![particles_buffer.as_entire_binding()]
        ).await;

        // initialization of the object
        Fluid { particles, computer, particles_buffer }
    } 

    pub fn step() {
        // todo:
        //  create kd-tree buffer 
    }
}
