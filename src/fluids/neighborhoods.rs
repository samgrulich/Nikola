use bevy::prelude::{Vec3, IVec3};

use std::{
    ops::{Deref, DerefMut},
    collections::HashMap,
};

use crate::{
    fluids::{self, SmoothedParticle},
    memory::Rcc,
};


const P1: i32 = 73856093;
const P2: i32 = 19349663;
const P3: i32 = 83492791;


#[derive(Clone, Copy)]
pub struct GVec3 {
    x: f32, 
    y: f32,
    z: f32,
}

impl GVec3 {
    pub fn new_i32(x: i32, y: i32, z: i32) -> Self {
        GVec3 { x: x as f32, y: y as f32, z: z as f32 }
    }

    pub fn from_f32(vector: Vec3) -> Self {
        GVec3 { x: vector.x, y: vector.y, z: vector.z }
    }

    pub fn from_i32(vector: IVec3) -> Self {
        GVec3 { x: vector.x as f32, y: vector.y as f32, z: vector.z as f32 }
    }
}

impl GVec3 {
    pub fn as_f32(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    pub fn as_i32(&self) -> IVec3 {
        IVec3::new(self.x as i32, self.y as i32, self.z as i32)
    }
}

pub fn hash_index(position_index: GVec3, table_size: i32) -> i32 {
    let index_vec = position_index.as_i32();

    ((index_vec.x * P1)
        ^ (index_vec.y * P2)
        ^ (index_vec.z * P3)
    ) % table_size
}

pub fn hash_function(position: Vec3, table_size: i32) -> i32 {
    let index_vec = GVec3::from_f32((position / fluids::SMOOTHING_LENGHT).floor());

    hash_index(index_vec, table_size)
}

pub struct Neighborhoods {
    entries: HashMap<i32, Vec<Rcc<SmoothedParticle>>>,
    max_size: i32,
}

impl Neighborhoods {
    pub fn new(max_size: i32) -> Self {
        Neighborhoods {
            entries: HashMap::new(),
            max_size
        }
    }

    pub fn from(particles: &mut Vec<Rcc<SmoothedParticle>>) -> Self {
        let mut neighborhoods = Neighborhoods {
            entries: HashMap::new(),
            max_size: particles.len() as i32,
        };

        particles.iter().for_each(|particle| {
            let particle = particle.clone(); 
            let result = neighborhoods.insert(particle);

            if let Some(err) = result.err() {
                dbg!(err);
            }
        });

        neighborhoods
    }
}

impl Neighborhoods {
    fn get_index(&self, position: Vec3) -> i32 {
        hash_function(position, self.max_size)
    }

    fn get_entry(&self, position: Vec3) -> (Option<&Vec<Rcc<SmoothedParticle>>>, i32) {
        let index = self.get_index(position);
        let result = self.entries.get(&index);

        (result, index)
    }

    fn get_entry_mut(&mut self, position: Vec3) -> (Option<&mut Vec<Rcc<SmoothedParticle>>>, i32) {
        let index = self.get_index(position);
        let result = self.entries.get_mut(&index);

        (result, index)
    }
    
    fn get_entry_by_index(&self, position_index: GVec3) -> (Option<&Vec<Rcc<SmoothedParticle>>>, i32) {
        let index = hash_index(position_index, self.max_size);
        let result = self.entries.get(&index);

        (result, index)
    }

    pub fn insert(&mut self, particle: Rcc<SmoothedParticle>) -> Result<(), &str> {
        let particle = particle.clone();
        let entries = self.get_entry_mut(particle.position);

        if let (None, index) = entries {
            let list = vec![particle];
            self.entries.insert(index, list);

            return Ok(());
        }

        if let (Some(particle_list), _) = entries {
            let listed_particle = particle_list.iter().find(|&listed_particle| {listed_particle.id == particle.id});

            if !listed_particle.is_none() {
                return Err("Particle already inserted");
            } 

            particle_list.push(particle);
        }

        Ok(())
    }

    pub fn get(&self, position: Vec3) -> Option<&Vec<Rcc<SmoothedParticle>>> {
        self.get_entry(position).0
    }

    pub fn get_neighbors(&self, position: Vec3) -> Option<Vec<Rcc<SmoothedParticle>>> {
        // make a dependency to position inside the cell (instead checking 3x3x3, check only 2x2x2)
        // possible perforamnce issues due to the copy() calls
        
        let mut result = match self.get_entry(position).0 {
            Some(list) => list.clone(),
            None => return None
        };

        // remove original particle
        let mut current_index = None;
        for (index, particle) in result.iter().enumerate() {
            if particle.position == position {
                current_index = Some(index);
            }
        }
        
        if current_index.is_some() {
            result.remove(current_index.unwrap());
        }

        let pos_index = (position / fluids::SMOOTHING_LENGHT).floor();
        let pos_index = IVec3::new(pos_index.x as i32, pos_index.y as i32, pos_index.z as i32);

        for z in pos_index.z-1..=pos_index.z+1 {
            for y in pos_index.y-1..=pos_index.y+1 {
                for x in pos_index.x-1..=pos_index.x+1 {
                    match self.get_entry_by_index(GVec3::new_i32(x, y, z)).0 {
                        Some(list) => result.append(&mut list.clone()), 
                        _ => ()
                    }
                }
            }
        }

        Some(result)
    }
}

impl Deref for Neighborhoods {
    type Target = HashMap<i32, Vec<Rcc<SmoothedParticle>>>;

    fn deref(&self) -> &Self::Target {
        &self.entries 
    }
}

impl DerefMut for Neighborhoods {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries 
    }
}
