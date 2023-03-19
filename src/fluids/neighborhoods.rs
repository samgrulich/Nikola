use std::collections::BTreeMap;
use glam::{Vec3A, vec3a};

use crate::{
    fluids::{self, SmoothedParticle},
    kernel
};



const P1: i32 = 73856093;
const P2: i32 = 19349663;
const P3: i32 = 83492791;


pub struct Neighborhood {
    pub neighbors: Vec<*const SmoothedParticle>,
    pub gradients: Vec<Vec3A>,
}

impl<'a> Neighborhood {
    pub fn new(particle: &'a SmoothedParticle, neighbors: Vec<*const SmoothedParticle>) -> Self {
        let gradients = Self::compute_gradients(particle, &neighbors);

        Neighborhood { 
            neighbors,
            gradients
        }
    }

    pub fn compute_gradients(particle: &'a SmoothedParticle, neighbors: &Vec<*const SmoothedParticle>) -> Vec<Vec3A> {
        neighbors.iter().map(|neighbor| {
            unsafe {
                kernel::smoothing_kernel_grad(particle.position, (**neighbor).position)
            }
        }).collect()
    }
}

impl Neighborhood {
    pub fn get_len(&self) -> f32 {
        self.neighbors.len() as f32
    }
}


pub struct TableMap {
    pub particles: Vec<SmoothedParticle> ,

    entries: BTreeMap<i32, Vec<*const SmoothedParticle>>,
    ids: BTreeMap<u32, (i32, usize)>,

    table_size: i32,
}

impl TableMap {
    pub fn hash(position: Vec3A, table_size: i32) -> i32 {
        let index_vec = (position / fluids::SMOOTHING_LENGHT).as_ivec3();

        ((index_vec.x * P1)
            ^ (index_vec.y * P2)
            ^ (index_vec.z * P3)
        ) % table_size
    }
}

impl TableMap {
    pub fn get_neighbors_by_id(&self, id: u32) -> (&[*const SmoothedParticle], &[*const SmoothedParticle]) {
        let keys = self.ids.get(&id).unwrap();
        let particles: &Vec<*const SmoothedParticle> = self.entries.get(&keys.0).unwrap();

        particles.split_at(keys.1)
    }

    pub fn get_particle_by_id(&self, id: u32) -> &SmoothedParticle {
        self.particles.get(id as usize).unwrap()
    }
    
    pub fn get_particle_by_id_mut(&mut self, id: u32) -> &mut SmoothedParticle {
        self.particles.get_mut(id as usize).unwrap()
    }

    pub fn get_by_position(&self, position: Vec3A) -> &[*const SmoothedParticle] {
        let index = Self::hash(position, self.table_size);

        self.entries.get(&index).unwrap().as_slice()
    }

    pub fn get_neighborhood_2d(&self, id: u32) -> Neighborhood {
        let particle = self.get_particle_by_id(id);
        
        let pos_x = particle.position.x as i32;
        let pos_y = particle.position.y as i32;
        
        let mut neighbors: Vec<*const SmoothedParticle> = Vec::new();
        for y in pos_y-1..=pos_y+1 {
            if pos_y == y {
                continue;
            }

            for x in pos_x-1..=pos_x+1 {
                if pos_x == x {
                    continue;
                }

                neighbors.extend_from_slice(
                    self.get_by_position(vec3a(x as f32, y as f32, 0.0))
                );
            }
        }

        let particle_neighbors = self.get_neighbors_by_id(id);
        neighbors.extend_from_slice(particle_neighbors.0);
        neighbors.extend_from_slice(particle_neighbors.1);

        Neighborhood::new(particle, neighbors)
    }

    pub fn insert(&mut self, particle: SmoothedParticle) {
        let index = Self::hash(particle.position, self.table_size);

        if self.entries.contains_key(&index) {
            let vector = self.entries
                .get_mut(&index).unwrap();
            
            self.ids.insert(particle.id, (index, vector.len()));

            vector.push(&particle);
        }
        else {
            self.ids.insert(particle.id, (index, 0));

            self.entries
                .insert(index, vec![&particle]);
        }

        self.particles.push(particle);
    }

    pub fn reinsert(&mut self, particle: *const SmoothedParticle) {
        let particle = unsafe { &*particle };
        let index = Self::hash(particle.position, self.table_size);

        if self.entries.contains_key(&index) {
            let vector = self.entries
                .get_mut(&index).unwrap();
            
            self.ids.insert(particle.id, (index, vector.len()));

            vector.push(&*particle);
        }
        else {
            self.ids.insert(particle.id, (index, 0));

            self.entries
                .insert(index, vec![&*particle]);
        }
    }
}

impl TableMap {
    pub fn update_particle_factors(&mut self) {
        let mut factors = vec![0.0; self.particles.len()];

        for (i, particle) in self.particles.iter().enumerate() {
            let neihgborhood = self.get_neighborhood_2d(particle.id);

            factors[i] = particle.compute_dsph_factor(&neihgborhood);
        };

        for (factor, particle) in factors.iter().zip(self.particles.iter_mut()) {
            particle.dsph_factor = *factor;
        }
    }
}

impl TableMap {
    pub fn new() -> Self {
        TableMap { 
            particles: Vec::new(), 
            entries: BTreeMap::new(), 
            ids: BTreeMap::new(), 
            table_size: 0, 
        }
    }

    pub fn from_particles(particles: Vec<SmoothedParticle>) -> Self {
        let mut table = Self::new();
        table.table_size = particles.len() as _;
        
        for particle in particles {
            table.insert(particle);
        }

        table.update_particle_factors();
       
        table
    }

    pub fn update(&mut self) {
        let mut particles_to_insert: Vec<*const SmoothedParticle> = Vec::new();

        for particle in self.particles.iter() {
            let new_index = Self::hash(particle.position, self.table_size);

            let (old_index, rank) = self.ids.get(&particle.id).unwrap();
            
            if new_index != *old_index {
                self.entries
                    .get_mut(old_index).unwrap()
                    .remove(*rank);

                particles_to_insert.push(&*particle);
            }
        }

        for particle in particles_to_insert {
            self.reinsert(particle);
        }
    }
}
