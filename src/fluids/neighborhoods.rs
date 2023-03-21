use std::collections::BTreeMap;
use glam::{Vec3A, vec3a};

use crate::{
    fluids::{self, SmoothedParticle},
    kernel
};



const P1: i32 = 73856093;
const P2: i32 = 19349663;
const P3: i32 = 83492791;

const CELL_SIZE: f32 = fluids::SMOOTHING_LENGHT;


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



#[derive(Debug, PartialEq)]
pub struct TableMap {
    pub particles: Vec<SmoothedParticle> ,

    entries: BTreeMap<u32, Vec<*const SmoothedParticle>>,
    ids: BTreeMap<u32, (u32, usize)>,

    table_size: i32,
}

impl TableMap {
    pub fn hash(position: Vec3A, _table_size: i32) -> u32 {
        let index_vec = (position / CELL_SIZE).floor().as_ivec3();

        let index = (index_vec.x * P1) as u32
            ^ (index_vec.y * P2) as u32
            ^ (index_vec.z * P3) as u32;

        // index % table_size as u32
        index
    } 
}

impl TableMap {
    pub fn get_neighbors_by_id(&self, id: u32) -> (&[*const SmoothedParticle], &[*const SmoothedParticle]) {
        let keys = self.ids.get(&id).unwrap();
        let particles: &Vec<*const SmoothedParticle> = self.entries.get(&keys.0).unwrap();

        (&particles[0..keys.1], &particles[keys.1 + 1..])
    }

    pub fn get_particle_by_id(&self, id: u32) -> &SmoothedParticle {
        self.particles.get(id as usize).unwrap()
    }
    
    pub fn get_particle_by_id_mut(&mut self, id: u32) -> &mut SmoothedParticle {
        self.particles.get_mut(id as usize).unwrap()
    }

    pub fn get_by_position(&self, position: Vec3A) -> Option<&Vec<*const SmoothedParticle>> {
        let index = Self::hash(position, self.table_size);

        self.entries.get(&index)
    }

    pub fn get_neighborhood_2d(&self, id: u32) -> Neighborhood {
        let particle = self.get_particle_by_id(id);
        
        let pos_x = particle.position.x;
        let pos_y = particle.position.y;
        
        let mut neighbors: Vec<*const SmoothedParticle> = Vec::new();
        for y in -1..=1 {
            for x in -1..=1 {
                if x | y == 0 {
                    let particle_neighbors = self.get_neighbors_by_id(id);
                   
                    neighbors.extend_from_slice(particle_neighbors.0);
                    neighbors.extend_from_slice(particle_neighbors.1);
        
                    continue;
                }

                let x = (pos_x + x as f32 * CELL_SIZE) * 1.05;
                let y = (pos_y + y as f32 * CELL_SIZE) * 1.05;

                if let Some(particles) = self.get_by_position(vec3a(x, y, 0.0)) {
                    neighbors.extend_from_slice(
                        particles
                    );
                }
            }
        }

        Neighborhood::new(particle, neighbors)
    }

    pub fn insert(&mut self, particle: SmoothedParticle) {
        let particle_new = particle.clone();
        let particle_ptr: *const SmoothedParticle = &particle_new;

        self.particles.push(particle_new);
        self.table_size = self.particles.len() as _;
        
        self.update_ids();

        let index = Self::hash(particle.position, self.table_size);

        if self.entries.contains_key(&index) {
            let vector = self.entries
                .get_mut(&index).unwrap();
            
            self.ids.insert(particle.id, (index, vector.len()));

            vector.push(particle_ptr);
        }
        else {
            self.ids.insert(particle.id, (index, 0));

            self.entries
                .insert(index, vec![particle_ptr]);
        }

        self.update_entry_pointers();
    }

    pub fn reinsert(&mut self, particle: *const SmoothedParticle, index: u32) {
        if self.entries.contains_key(&index) {
            self.entries.get_mut(&index).unwrap().push(particle);
        }
        else {
            self.entries.insert(index, vec![particle]);
        }
    }
}

impl TableMap {
    pub fn update_entry_pointers(&mut self) {
        for (id, (index, rank)) in self.ids.iter() {
            self.entries.get_mut(index).unwrap()[*rank] = &self.particles[*id as usize]
        }
    }

    fn update_particle_factors(&mut self) {
        let mut factors = vec![0.0; self.particles.len()];

        for (i, particle) in self.particles.iter().enumerate() {
            if particle.id == i as u32 {
                continue;
            }

            let neihgborhood = self.get_neighborhood_2d(particle.id);

            factors[i] = particle.compute_dsph_factor(&neihgborhood);
        };

        for (factor, particle) in factors.iter().zip(self.particles.iter_mut()) {
            particle.dsph_factor = *factor;
        }
    }

    fn update_ids(&mut self) {
        for (id, entry) in self.ids.iter_mut() {
            let position = self.particles[*id as usize].position;

            entry.0 = TableMap::hash(position, self.table_size);
        }
    }

    fn update_ranks(&mut self) {
        for (id, val) in self.ids.iter_mut() {
            let particle = &self.particles[*id as usize];
            let index = Self::hash(particle.position, self.table_size);
            let particle_ptr: *const SmoothedParticle = &*particle;
            let rank = self.entries
                .get(&index).unwrap()
                .iter()
                .position(|particle| *particle == particle_ptr).unwrap();

            *val = (index, rank);
        }
    }

    pub fn update_refs(&mut self) {
        self.update_entry_pointers();
        self.update_ids();
    }
}

impl TableMap {
    pub fn new() -> Self {
        TableMap { 
            particles: Vec::new(), 
            entries: BTreeMap::new(), 
            ids: BTreeMap::new(), 
            table_size: 1, 
        }
    }

    pub fn from_particles(particles: Vec<SmoothedParticle>) -> Self {
        let mut table = Self::new();
        table.table_size = particles.len() as _;
        
        for particle in particles {
            table.insert(particle);
        }

        table.update_particle_factors();
        // table.update_refs();
       
        table
    }

    pub fn update(&mut self) {
        let mut particles_to_insert: Vec<(*const SmoothedParticle, u32)> = Vec::new();

        for particle in self.particles.iter().rev() {
            let new_index = Self::hash(particle.position, self.table_size);
            let (old_index, rank) = self.ids.get(&particle.id).unwrap();
            
            if new_index != *old_index {
                self.entries
                    .get_mut(old_index).unwrap()
                    .remove(*rank);

                particles_to_insert.push((&*particle, new_index));
            }
        }

        for (particle, index) in particles_to_insert.iter().rev() {
            self.reinsert(*particle, *index);
        }

        self.update_ranks();
        self.update_particle_factors();
    }
}


#[cfg(test)]
mod tests {
    use glam::{Vec3A, vec3a};
    use crate::{TableMap, SMOOTHING_LENGHT};

    use super::SmoothedParticle;

    fn table_1_setup() -> (TableMap, SmoothedParticle) {
        let particle = SmoothedParticle::new(0, vec3a(1.0, 1.0, 0.0));
        (TableMap::from_particles(vec![particle.clone()]), particle)
    }

    fn table_9_setup() -> (TableMap, Vec<SmoothedParticle>) {
        let particles = (0..3).flat_map(|y| {
            (0..3).map(move |x| {
                SmoothedParticle::new(y * 3 + x, vec3a(x as f32, y as f32, 0.0))
            })
        }).collect::<Vec<SmoothedParticle>>();

        (TableMap::from_particles(particles.clone()), particles)
    }

    // todo: maybe add test of entries pointer validity

    #[test]
    fn test_neighbors() {
        let particles = vec![
            SmoothedParticle::new(0, Vec3A::ZERO), 
            SmoothedParticle::new(1, Vec3A::X * SMOOTHING_LENGHT), 
            SmoothedParticle::new(2, Vec3A::X * 2.0 * SMOOTHING_LENGHT), 
        ];

        let table = TableMap::from_particles(particles.clone());
        let neighborhood = table.get_neighborhood_2d(1);

        let neighbors = vec![particles[0].clone(), particles[2].clone()];
        
        for (calculated_neighbor, neighbor) in neighborhood.neighbors.iter().zip(neighbors.iter()) {
            let calculated_neighbor = unsafe {&**calculated_neighbor};

            dbg!(calculated_neighbor, neighbor);
            assert!(calculated_neighbor.position == neighbor.position);
        }

        assert!(table.table_size == 3);
        assert!(table.particles.len() == 3);
        assert!(neighborhood.neighbors.len() == 2);
    }

    #[test]
    fn test_gradients() {

    }

    #[test]
    fn test_table_insert() {
        let particle = SmoothedParticle::new(0, Vec3A::ONE);
        let mut table = TableMap::new();

        table.insert(particle.clone());

        assert!(table.particles == vec![particle]);
    }

    #[test]
    fn test_table_from_particles() {
        let (calculated_table, particle) = table_1_setup();

        let mut table = TableMap::new();
        table.insert(particle);
        table.update();
       
        assert!(table.particles == calculated_table.particles);
        assert!(table.ids == calculated_table.ids);
    }

    #[test]
    fn test_table_get() {
        let (table, particles) = table_9_setup(); 

        // let particle = table.get_by_position(vec3a(1.0, 0.0, 0.0));
        // dbg!(unsafe {(*(*particle)[0]).clone()});
        // assert!(unsafe {(*(*particle)[0]).clone()} == *&particles[0]);

        let particle = table.get_particle_by_id(1);
        assert!(*particle == *&particles[1]);
    }
}
