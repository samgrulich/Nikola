use std::fs::{write, read};

use fluid_renderer::Instance;
use glam::Vec3A;


/// Struct for easier storage of information about current simulation
#[derive(Debug)]
pub struct Simulation {
    pub fps: u32,
    pub frame_stop: u32,
    pub particle_num: u32,
    pub frames: Vec<Vec3A>, 
    pub frame_index: usize,
} 

impl Simulation {
    /// create new Simulation struct 
    ///
    /// # Arguments 
    /// * `fps` - frames per second at which should be the simulation replayed
    /// * `frame_stop` - value of last valid frame
    /// * `particle_num` - number of particles in simulation
    pub fn new(fps: u32, frame_stop: u32, particle_num: u32) -> Self {
        let frames = (0..(particle_num * frame_stop)).map(|_id| Vec3A::ZERO).collect();

        Simulation { 
            fps, 
            frame_stop, 
            particle_num,
            frames,
            frame_index: 0
        } 
    }
}

impl Simulation {
    /// Get frames as vector of f32
    pub fn raw_frames(&self) -> Vec<f32> {
        self.frames
            .clone()
            .into_iter()
            .map(|position| position.to_array())
            .flatten()
            .collect::<Vec<f32>>()
    }

    /// Reconstruct positions from bytes representation
    ///
    /// # Arguments
    /// * `bytes` - native edianess bytes representation of frames
    ///
    /// # Returns
    /// vector of positions
    pub fn frames_from_bytes(bytes: Vec<u8>) -> Vec<Vec3A> {
        let float_bytes = bytes
            .chunks(4)
            .map(|chunk| f32::from_ne_bytes(chunk.try_into().unwrap()))
            .collect::<Vec<f32>>();
        let frames = float_bytes
            .chunks(3)
            .map(|chunk| Vec3A::from_slice(chunk))
            .collect::<Vec<Vec3A>>();

        frames
    }

    /// Write data of struct into file at path
    ///
    /// # Arguments
    /// * `path` - path to the target file
    ///
    /// # Returns
    /// whether the write was successful
    pub fn save(&self, path: String) -> Result<(), std::io::Error> {
        let mut bytes = self.fps.to_ne_bytes().to_vec();
        bytes.append(&mut self.frame_stop.to_ne_bytes().to_vec());
        bytes.append(&mut self.particle_num.to_ne_bytes().to_vec());
        bytes.append(&mut bytemuck::cast_slice(self.raw_frames().as_slice()).to_vec());
        
        write(path, bytes)
    }

    /// Read data of file and create Simulation struct from it
    ///
    /// # Arguments 
    /// * `path` - path to the source file
    ///
    /// # Returns
    /// Simulation or error
    pub fn from_file(path: String) -> Result<Self, std::io::Error>{
        let bytes = read(path)?;
        let frames_per_second = u32::from_ne_bytes(bytes[0..4].try_into().unwrap());
        let frame_stop = u32::from_ne_bytes(bytes[4..8].try_into().unwrap());
        let particle_num = u32::from_ne_bytes(bytes[8..12].try_into().unwrap());
        let frames = Self::frames_from_bytes(bytes[12..].try_into().unwrap());

        Ok(Simulation { 
            fps: frames_per_second, 
            frame_stop, 
            particle_num,
            frames,
            frame_index: 0,
        })
    }

    /// Updates frame_index forward in time and sets instances' positions to the according frame
    ///  
    /// # Arguments 
    /// * `instances` - instances, which will be updated
    /// * `step_length` - amount of frames to step
    pub fn step_forward(&mut self, instances: &mut Vec<Instance>, step_length: usize) {
        self.update_instances(instances);
        self.frame_index += step_length;
    }
    
    /// Updates frame_index and sets instances' positions to the according frame
    /// only backwards in time
    ///  
    /// # Arguments 
    /// * `instances` - instances, which will be updated
    /// * `step_length` - amount of frames to step
    pub fn step_back(&mut self, instances: &mut Vec<Instance>, step_length: usize) {
        self.update_instances(instances);
        self.frame_index -= step_length;
    }

    /// Set position of each instance to the according particle 
    ///
    /// # Arguments 
    /// * `instances` - instances, which will be updated
    pub fn update_instances(&self, instances: &mut Vec<Instance>) {
        if self.particle_num * (self.frame_index as u32 + 1) > self.frames.len() as u32 {
            return;
        }

        for particle in 0..self.particle_num as usize {
            let start_index = self.particle_num as usize * self.frame_index;
            let index = start_index + particle;
            instances[particle].position = self.frames[index].into();
        }
    }
}

