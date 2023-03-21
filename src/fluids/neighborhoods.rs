use std::collections::HashMap;
use glam::{Vec3A, vec3a};
use nohash_hasher::BuildNoHashHasher;

use crate::{
    fluids::{self, FluidParticle},
    kernel, Particle 
};



const P1: i32 = 73856093;
const P2: i32 = 19349663;
const P3: i32 = 83492791;

const CELL_SIZE: f32 = fluids::SMOOTHING_LENGHT;


pub struct Neighborhood<T: Particle> {
    pub neighbors: Vec<*const T>,
    pub gradients: Vec<Vec3A>,
}

impl<T: Particle> Neighborhood<T> {
    pub fn new(position: Vec3A, neighbors: Vec<*const T>) -> Self {
        let gradients = Self::compute_gradients(position, &neighbors);

        Neighborhood { 
            neighbors,
            gradients
        }
    }

    pub fn compute_gradients(position: Vec3A, neighbors: &Vec<*const T>) -> Vec<Vec3A> {
        neighbors.iter().map(|neighbor| {
            unsafe {
                kernel::smoothing_kernel_grad(position, (**neighbor).position())
            }
        }).collect()
    }

    pub fn get_len(&self) -> f32 {
        self.neighbors.len() as f32
    }
}



#[derive(Debug, PartialEq)]
pub struct TableMap<T: Particle> {
    pub particles: Vec<T>,

    entries: HashMap<u32, HashMap<u32, *const T>, BuildNoHashHasher<u32>>,
    ids: HashMap<u32, u32, BuildNoHashHasher<u32>>
}

impl<T: Particle> TableMap<T> {
    pub fn hash(position: Vec3A) -> u32 {
        let index_vec = (position / CELL_SIZE).floor().as_ivec3();

        let index = (index_vec.x * P1) as u32
            ^ (index_vec.y * P2) as u32
            ^ (index_vec.z * P3) as u32;

        index
    } 
}

impl<T: Particle> TableMap<T> {
    pub fn get_neighbors(&self, id: u32) -> Vec<*const T> {
        let index = self.ids.get(&id).unwrap();
        let neighborhood = self.entries.get(&index).unwrap();
        let mut particles = neighborhood.values().copied().collect::<Vec<*const T>>();
        let particle_idx = neighborhood.keys().position(|key| *key == id).unwrap();

        particles.remove(particle_idx);
        particles
    }

    pub fn get_particle_by_id(&self, id: u32) -> &T {
        self.particles.get(id as usize).unwrap()
    }
    
    pub fn get_particle_by_id_mut(&mut self, id: u32) -> &mut T {
        self.particles.get_mut(id as usize).unwrap()
    }

    pub fn get_by_position(&self, position: Vec3A) -> Option<Vec<*const T>> {
        let index = Self::hash(position);
        let neighborhood = self.entries.get(&index);

        match neighborhood {
            Some(neighborhood) => Some(neighborhood.values().copied().collect()),
            None => None
        }
    }

    pub fn get_neighborhood_2d(&self, id: u32) -> Neighborhood<T> {
        let particle = self.get_particle_by_id(id);
        
        let pos_x = particle.position().x;
        let pos_y = particle.position().y;
        
        let mut neighbors: Vec<*const T> = Vec::new();
        for y in -1..=1 {
            for x in -1..=1 {
                if x | y == 0 {
                    let mut p_neighbors = self.get_neighbors(id);
                    neighbors.append(&mut p_neighbors);
        
                    continue;
                }

                let x = (pos_x + x as f32 * CELL_SIZE) * 1.05;
                let y = (pos_y + y as f32 * CELL_SIZE) * 1.05;

                if let Some(mut particles) = self.get_by_position(vec3a(x, y, 0.0)) {
                    neighbors.append(&mut particles);
                }
            }
        }

        Neighborhood::new(particle.position(), neighbors)
    }
    
    pub fn get_neighborhood_by_position_2d(&self, position: Vec3A) -> Neighborhood<T> {
        let pos_x = position.x;
        let pos_y = position.y;
        
        let mut neighbors: Vec<*const T> = Vec::new();
        for y in -1..=1 {
            for x in -1..=1 {
                let x = (pos_x + x as f32 * CELL_SIZE) * 1.05;
                let y = (pos_y + y as f32 * CELL_SIZE) * 1.05;

                if let Some(mut particles) = self.get_by_position(vec3a(x, y, 0.0)) {
                    neighbors.append(&mut particles);
                }
            }
        }

        Neighborhood::new(position, neighbors)
    }

    fn insert(&mut self, index: u32, particle_id: u32, particle: *const T) {
        if !self.entries.contains_key(&index) {
            self.entries.insert(index, HashMap::new());
        }

        self.entries.get_mut(&index).unwrap().insert(particle_id, particle);
        self.ids.insert(particle_id, index);
    }
    
    pub fn insert_particles(&mut self, particles: Vec<T>) {
        self.particles = particles;
        let entries = self.particles
            .iter()
            .map(|particle| {
                let index = Self::hash(particle.position());
                (index, particle.id(), &(*particle) as *const T)
            }).collect::<Vec<(u32, u32, *const T)>>();

        for (index, id, particle) in entries {
            self.insert(index, id, particle);
        }
    }
}

impl<T: Particle> TableMap<T> {
    pub fn new() -> Self {
        TableMap { 
            particles: Vec::new(), 
            entries: HashMap::with_hasher(BuildNoHashHasher::default()), 
            ids: HashMap::with_hasher(BuildNoHashHasher::default()),
        }
    }

    pub fn from_particles(particles: Vec<T>) -> Self {
        let mut table = Self::new();
        
        table.insert_particles(particles);
       
        table
    }

    pub fn update(&mut self) { 
        let mut particles_to_insert: Vec<(u32, u32, *const T)> = Vec::new();

        for particle in self.particles.iter() {
            let new_index = Self::hash(particle.position());
            let old_index = self.ids.get(&particle.id()).unwrap();
            
            if new_index != *old_index {
                self.entries
                    .get_mut(old_index).unwrap()
                    .remove(&particle.id());

                particles_to_insert.push((new_index, particle.id(), &*particle));
            }
        }

        for (index, id, particle) in particles_to_insert {
            self.insert(index, id, particle);
        }
    }
}

impl TableMap<FluidParticle> {
    pub fn update_particle_factors(&mut self) {
        let mut factors = vec![0.0; self.particles.len()];

        for (i, particle) in self.particles.iter().enumerate() {
            if particle.id() == i as u32 {
                continue;
            }

            let neihgborhood = self.get_neighborhood_2d(particle.id);

            factors[i] = particle.compute_dsph_factor(&neihgborhood);
        };

        for (factor, particle) in factors.iter().zip(self.particles.iter_mut()) {
            particle.dsph_factor = *factor;
        }
    }
}



#[cfg(test)]
mod tests {
    use glam::{Vec3A, vec3a};
    use crate::{TableMap, SMOOTHING_LENGHT, Particle};

    use super::FluidParticle;

    fn table_1_setup() -> (TableMap<FluidParticle>, FluidParticle) {
        let particle = FluidParticle::new(0, vec3a(1.0, 1.0, 0.0));
        (TableMap::from_particles(vec![particle.clone()]), particle)
    }

    fn table_9_setup() -> (TableMap<FluidParticle>, Vec<FluidParticle>) {
        let particles = (0..3).flat_map(|y| {
            (0..3).map(move |x| {
                FluidParticle::new(y * 3 + x, vec3a(x as f32, y as f32, 0.0))
            })
        }).collect::<Vec<FluidParticle>>();

        (TableMap::from_particles(particles.clone()), particles)
    }

    // todo: maybe add test of entries pointer validity

    #[test]
    fn test_neighbors() {
        let particles = vec![
            FluidParticle::new(0, Vec3A::ZERO), 
            FluidParticle::new(1, Vec3A::X * SMOOTHING_LENGHT), 
            FluidParticle::new(2, Vec3A::X * 2.0 * SMOOTHING_LENGHT), 
        ];

        let table = TableMap::from_particles(particles.clone());
        let neighborhood = table.get_neighborhood_2d(1);

        let neighbors = vec![particles[0].clone(), particles[2].clone()];
        
        for (calculated_neighbor, neighbor) in neighborhood.neighbors.iter().zip(neighbors.iter()) {
            let calculated_neighbor = unsafe {&**calculated_neighbor};

            dbg!(calculated_neighbor, neighbor);
            assert!(calculated_neighbor.position == neighbor.position);
        }

        assert!(table.particles.len() == 3);
        assert!(neighborhood.neighbors.len() == 2);
    }

    #[test]
    fn test_gradients() {

    }

    // #[test]
    // fn test_table_insert() {
    //     let particle = SmoothedParticle::new(0, Vec3A::ONE);
    //     let mut table = TableMap::new();
    //
    //     table.insert(particle.clone());
    //
    //     assert!(table.particles == vec![particle]);
    // }

    #[test]
    fn test_table_from_particles() {
        let (calculated_table, particle) = table_1_setup();

        let mut table = TableMap::new();
        table.insert_particles(vec![particle]);
        table.update();
       
        assert!(table.particles == calculated_table.particles);
        assert!(table.ids == calculated_table.ids);
    }

    #[test]
    fn test_table_get() {
        let (table, particles) = table_9_setup(); 
        let particle = table.get_particle_by_id(1);

        assert!(*particle == *&particles[1]);
    }
}
