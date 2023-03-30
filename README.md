# Nikola
This is school "maturita" project. It is practical part of introduction of fluid simulation paper. 

The repository contains rust program for simulation of fluids. Using state equation for computation of pressure 
and SPH aproximations for computation of partial derivates. The simulator uses imgui libraries to handle user 
interface and wgpu for particle rendering. The particles are rendered by separate repository Fluid-renderer, which
is heavily modified version of [learn-wgpu](https://github.com/sotrh/learn-wgpu).

--- 
## Setup
Programs must be launched from the root directory, because of certain dependencies. 
You may not find compiled binary for your desktop. Then you would have to compile the 
code yourself. Running
```cargo build --release```
should do the trick.

## Launch
You can run the program using either
```cargo run --release```
or 
```./target/release/nikola```
Both ways are acceptable.

The program has two modes 1st (default) is simulation generation mode. This mode presents user 
bunch of parameters to run the simulation with. After the simulation is completed. File named `simulation.nk`
is created. This file would be rewritten on next simulation run. To keep it simply rename it.

The 2nd mode is simulation player. User can pick any simulation file in the current directory.
You can enter the simulation player by running the command with `run` flag. Ex. 
```cargo run --release run```

---
## Major Sources 
1. SPH tutorial - KOSCHIER, Dan; BENDER, Jan; SOLENTHALER, Barbara; TESCHNER, Matthias.
Smoothed Particle Hydrodynamics Techniques for the Physics Based Simulation of Fluids
and Solids. Eurographics 2019 - Tutorials. 2019. Aviable from DOI: `10.2312/EGT.20191035`.
2. SPH course notes - BRIDSON, Robert; FEDKIW, Ronald; MÜLLER-FISCHER, Matthias. Fluid simulation.
ACM SIGGRAPH 2006 Course notes. 2006. Available from doi: `10.1145/1185657`.
1185730
3. Similar SPH implementation - https://github.com/erizmr/SPH_Taichi

All other sources are properly mentioned in the last section of the [whitepaper](./whitepaper.pdf).
