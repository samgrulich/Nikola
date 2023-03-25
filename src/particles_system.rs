use std::ops::{AddAssign, SubAssign};

use glam::{Vec3A, IVec3, ivec3};

use crate::Config;

pub struct ParticleSystem {
    pub domain_start: Vec3A,
    pub domain_end: Vec3A,
    pub domain_size: Vec3A,

    pub particle_radius: f32,
    pub particle_diameter: f32,
    pub support_radius: f32,

    pub particle_num: usize, // number of particles

    // Grid props
    grid_size: f32,   // cell size
    grid_dims: IVec3, // dimensions of the grid
    grid_len: usize,

    grid_ids: Vec<usize>, // particle_id (index): grid index (value)
    grid_offsets: Vec<usize>,
    grid_particles_num: Vec<usize>, // count of particles at cell

    // particle props
    pub ids: Vec<usize>,
    pub x: Vec<Vec3A>,
    pub x_0: Vec<Vec3A>,
    pub v: Vec<Vec3A>,
    pub acceleration: Vec<Vec3A>,
    pub m_v: Vec<f32>,
    pub m: Vec<f32>,
    pub density: Vec<f32>,
    pub pressure: Vec<f32>,
    pub color: Vec<Vec3A>,
    
    // sort buffers
    ids_buffer: Vec<usize>,
    x_buffer: Vec<Vec3A>,
    x_0_buffer: Vec<Vec3A>,
    v_buffer: Vec<Vec3A>,
    acceleration_buffer: Vec<Vec3A>,
    m_v_buffer: Vec<f32>,
    m_buffer: Vec<f32>,
    density_buffer: Vec<f32>,
    pressure_buffer: Vec<f32>,
    color_buffer: Vec<Vec3A>,
}

impl ParticleSystem {
    pub fn new(config: Config) -> Self {
        let domain_size = config.domain_end - config.domain_start;

        let particle_diameter = 2.0 * config.particle_radius;
        let support_radius = 4.0 * config.particle_radius;
        let m_v0 = 0.8 * particle_diameter;

        let grid_dims = (domain_size / support_radius).ceil().as_ivec3();
        let grid_len = (grid_dims.x * grid_dims.y * grid_dims.z) as usize;

        ParticleSystem { 
            domain_start: config.domain_start, 
            domain_end: config.domain_end, 
            domain_size, 

            particle_radius: config.particle_radius, 
            particle_diameter, 
            support_radius, 
            
            particle_num: config.particle_num,

            grid_size: support_radius, 
            grid_dims,
            grid_len,

            grid_ids: vec![0; config.particle_num],
            grid_offsets: vec![0; grid_len],
            grid_particles_num: vec![0; grid_len],

            ids: (0..config.particle_num).collect(),
            x: config.x.clone(), 
            x_0: config.x,
            v: config.v,
            acceleration: vec![Vec3A::ZERO; config.particle_num],
            m_v: vec![m_v0; config.particle_num],
            m: vec![m_v0 * config.density_0; config.particle_num],
            density: vec![config.density_0; config.particle_num] ,
            pressure: vec![0.0; config.particle_num],
            color: config.color,
            
            ids_buffer: vec![0; config.particle_num],
            x_buffer: vec![Vec3A::ZERO; config.particle_num],
            x_0_buffer: vec![Vec3A::ZERO; config.particle_num],
            v_buffer: vec![Vec3A::ZERO; config.particle_num],
            acceleration_buffer: vec![Vec3A::ZERO; config.particle_num],
            m_v_buffer: vec![0.0; config.particle_num],
            m_buffer: vec![0.0; config.particle_num],
            density_buffer: vec![0.0; config.particle_num],
            pressure_buffer: vec![0.0; config.particle_num],
            color_buffer: vec![Vec3A::ZERO; config.particle_num]
        }
    }
}

impl ParticleSystem {
    fn pos_to_index(&self, pos: Vec3A) -> IVec3 {
        ((pos - self.domain_start) / self.grid_size).floor().as_ivec3()
    }

    fn flatten_grid_index(&self, grid_index: IVec3) -> usize {
        (grid_index.x * self.grid_dims.y * self.grid_dims.z + grid_index.y * self.grid_dims.z + grid_index.z) as usize
    }

    pub fn get_grid_index(&self, pos: &Vec3A) -> usize {
        let grid_index = self.pos_to_index(*pos);
        let index = self.flatten_grid_index(grid_index);

        if index > self.grid_len {
            dbg!(grid_index, index, pos);
        }

        self.flatten_grid_index(self.pos_to_index(*pos))
    }

    pub fn is_index_valid(&self, grid_index: IVec3) -> bool {
        grid_index.cmpge(IVec3::ZERO).all() && grid_index.cmplt(self.grid_dims).all()
    }

    pub fn update_grid_id(&mut self) {
        for i in self.grid_particles_num.iter_mut() {
            *i = 0;
        }
        for (i, val) in self.x.iter().enumerate() {
            let grid_index = self.get_grid_index(val);
            self.grid_ids[i] = grid_index;
            self.grid_particles_num[grid_index].add_assign(1);
        }
        // for (i, val) in self.grid_particles_num.iter().enumerate() {
        //     self.grid_particles_num_temp[i] = *val;
        // }
    }

    pub fn sort(&mut self) {
        let mut grid_particles_num_temp = self.grid_particles_num.clone();
        let mut new_grid_ids: Vec<usize> = vec![0; self.particle_num];
        let mut new_ids: Vec<usize> = vec![0; self.grid_ids.len()];
        let mut new_offsets: Vec<usize> = vec![0; self.grid_len];
        let mut total_offset: usize = 0;

        for (grid_index, offset) in self.grid_particles_num.iter().enumerate() {
            new_offsets[grid_index] = total_offset;
            self.grid_offsets[grid_index] = total_offset;
            total_offset += offset;
        }

        for particle_id in 0..self.particle_num { 
            let grid_index = self.grid_ids[particle_id];
            let grid_offset = new_offsets[grid_index];
            let offset = grid_particles_num_temp[grid_index];

            new_ids[particle_id] = grid_offset + offset - 1;
            grid_particles_num_temp[grid_index].sub_assign(1);
        }

        for (particle_id, &new_particle_id) in new_ids.iter().enumerate() {
            new_grid_ids[new_particle_id] = self.grid_ids[particle_id];
            self.ids_buffer[new_particle_id] = self.ids[particle_id];
            self.x_buffer[new_particle_id] = self.x[particle_id]; 
            self.x_0_buffer[new_particle_id] = self.x_0[particle_id]; 
            self.v_buffer[new_particle_id] = self.v[particle_id]; 
            self.acceleration_buffer[new_particle_id] = self.acceleration[particle_id]; 
            self.m_v_buffer[new_particle_id] = self.m_v[particle_id]; 
            self.m_buffer[new_particle_id] = self.m[particle_id]; 
            self.density_buffer[new_particle_id] = self.density[particle_id]; 
            self.pressure_buffer[new_particle_id] = self.pressure[particle_id]; 
            self.color_buffer[new_particle_id] = self.color[particle_id]; 
        }
        
        for i in 0..self.particle_num {
            self.ids[i] = self.ids_buffer[i];
            self.x[i] = self.x_buffer[i]; 
            self.x_0[i] = self.x_0_buffer[i]; 
            self.v[i] = self.v_buffer[i]; 
            self.acceleration[i] = self.acceleration_buffer[i]; 
            self.m_v[i] = self.m_v_buffer[i]; 
            self.m[i] = self.m_buffer[i]; 
            self.density[i] = self.density_buffer[i]; 
            self.pressure[i] = self.pressure_buffer[i]; 
            self.color[i] = self.color_buffer[i]; 
        }
    }

    pub fn initialize_particle_system(&mut self) {
        self.update_grid_id();
        // todo: do prefix sum self.grid_particles_num
        self.sort();
    }

    pub fn for_all_neighbords<F, T>(&self, p_i: usize, task: F, ret: &mut T) 
    where 
        F: Fn(usize, usize, &mut T)
    {
        let center_cell = self.pos_to_index(self.x[p_i]);

        for z in -1..=1 {
            for y in -1..=1 {
                for x in -1..=1 {
                    let offset = ivec3(x, y, z);
                    let final_index = center_cell + offset;
                    
                    // dbg!(p_i, center_cell, offset);
                    if !self.is_index_valid(final_index) {
                        continue;
                    }

                    let grid_index = self.flatten_grid_index(final_index);

                    let base_offset = self.grid_offsets[grid_index];
                    for p_j in 0..self.grid_particles_num[grid_index] {
                        let p_j = base_offset + p_j;
                        if p_i != p_j && (self.x[p_i] - self.x[p_j]).length() < self.support_radius {
                            task(p_i, p_j, ret);
                        }
                    }
                }
            }
        }

    }
}

