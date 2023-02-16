use bevy::prelude::Vec3;

use std::rc::Rc;
use std::ops::{Deref, DerefMut};
use std::collections::{HashMap, LinkedList};

use crate::fluids::{self, SmoothedParticle};


const P1: f32 = 73856093.0;
const P2: f32 = 19349663.0;
const P3: f32 = 83492791.0;


pub fn hash_function(position: Vec3, table_size: i32) -> i32 {
    let position = position.clone();
    let index_vec = (position / fluids::SMOOTHING_LENGHT).floor();

    ((index_vec.x * P1).floor() as i32 
        ^ (index_vec.y * P2).floor() as i32
        ^ (index_vec.z * P3).floor() as i32 
    ) % table_size
}

pub struct Neighborhoods {
    entries: HashMap<i32, LinkedList<Rc<SmoothedParticle>>>,
    max_size: i32,
}

impl Neighborhoods {
    pub fn new(max_size: i32) -> Self {
        Neighborhoods {
            entries: HashMap::new(),
            max_size
        }
    }

    pub fn from(particles: Vec<Rc<SmoothedParticle>>) -> Self {
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

    fn get_entry(&self, position: Vec3) -> (Option<&LinkedList<Rc<SmoothedParticle>>>, i32) {
        let index = self.get_index(position);
        let result = self.entries.get(&index);

        (result, index)
    }

    fn get_entry_mut(&mut self, position: Vec3) -> (Option<&mut LinkedList<Rc<SmoothedParticle>>>, i32) {
        let index = self.get_index(position);
        let result = self.entries.get_mut(&index);

        (result, index)
    }

    pub fn insert(&mut self, particle: Rc<SmoothedParticle>) -> Result<(), &str> {
        let particle = particle.clone();
        let entries = self.get_entry_mut(particle.position);

        if let (None, index) = entries {
            let list = LinkedList::from([particle]);
            self.entries.insert(index, list);

            return Ok(());
        }

        if let (Some(particle_list), _) = entries {
            let listed_particle = particle_list.iter().find(|&listed_particle| {listed_particle.id == particle.id});

            if !listed_particle.is_none() {
                return Err("Particle already inserted");
            } 

            particle_list.push_back(particle);
        }

        Ok(())
    }

    pub fn get(&self, position: Vec3) -> Option<&LinkedList<Rc<SmoothedParticle>>> {
        self.get_entry(position).0
    }
}

impl Deref for Neighborhoods {
    type Target = HashMap<i32, LinkedList<Rc<SmoothedParticle>>>;

    fn deref(&self) -> &Self::Target {
        &self.entries 
    }
}

impl DerefMut for Neighborhoods {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries 
    }
}
