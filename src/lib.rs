mod config;
mod solver;
mod wcsph;
mod particles_system;
mod simulation;

use std::fs::{self, ReadDir, DirEntry};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Instant, Duration};

pub use config::*;
pub use solver::*;
pub use wcsph::*;
pub use particles_system::*;
pub use simulation::*;

use glam::{vec3a, Vec3A};
use fluid_renderer::*;
use fluid_renderer::winit::event::*;


fn match_files(directory: ReadDir) -> Vec<DirEntry> {
    directory
        .filter(|path| {
            match path {
                Ok(path) => match path.path().extension() {
                    Some(ext) => ext == "nk",
                    None => false
                }
                Err(err) => {
                    dbg!(err);
                    false
                }
            }
        })
        .map(|path| {
            path.unwrap()
        }).collect()
}

pub fn load_files(path: PathBuf) -> Vec<DirEntry> {
    let local_files = fs::read_dir(path.clone()).expect("No simulations in this directory found");
    let simulation_paths = fs::read_dir(path.join("/simulations/"));

    let mut files = match_files(local_files);

    match simulation_paths {
        Ok(paths) => files.append(&mut match_files(paths)),
        Err(err) => {dbg!(err);},
    }

    files
}

pub fn run_simulation(simulation_path: String, fps: u32, particle_size: f32) {
    let InitOutput{event_loop, window, aspect_ratio} = init(); 
    let shader_source = fluid_renderer::wgpu::ShaderSource::Wgsl(std::fs::read_to_string("libs/fluid-renderer/src/shader.wgsl").unwrap().into());
    let vertices = Quad.scale(particle_size);
    let indices = Quad::INDICES;
    
    let mut simulation = Simulation::from_file(simulation_path).unwrap();
    
    let camera = Camera {
        aspect: aspect_ratio,
        fovy: 45.0,
        eye: vec3a(-200.0, 200.0, 1000.0) / 4.0,
        zfar: 10000.0,
        ..Default::default()
    };

    let instances = (0..simulation.particle_num).map(|_id| Instance::new()).collect();

    let mut state = pollster::block_on(
        State::new(
            window, 
            shader_source, 
            vertices.as_slice(), 
            indices, 
            instances, 
            camera
        )
    );


    let (mut imgui_ctxt, mut imgui_platform, mut imgui_renderer) = init_ui(&state, 8.0);
    let mut frame_delta = Duration::from_millis(0);

    let mut is_playing = false;
    let files = load_files(std::env::current_dir().unwrap())
        .into_iter()
        .map(|path| path.path().to_str().unwrap().to_string())
        .collect::<Vec<String>>();

    event_loop.run(move |event, _, control_flow| {
        let frame_start = Instant::now();
        imgui_platform.handle_event(imgui_ctxt.io_mut(), &state.window, &event);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                crate::handle_windowing(&mut state, &mut imgui_ctxt, event, control_flow)
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();

                if is_playing && simulation.frame_index < simulation.frame_stop as usize {
                    simulation.step_forward(&mut state.instances, 1);
                    state.update_instances();
                }

                imgui_platform.prepare_frame(imgui_ctxt.io_mut(), &state.window).expect("Failed to prepare ui frame");
                imgui_ctxt.io_mut().update_delta_time(frame_delta);
                let ui = imgui_ctxt.frame();

                {
                    ui.window("Info")
                        .build(|| {
                            ui.text(format!("Snimek: {}", simulation.frame_index.min(simulation.frame_stop as usize)));
                            let play_button_text = if is_playing {
                                "||"
                            } else {
                                " >"
                            };

                            if files.len() > 0 {
                                ui.menu("Soubor animace", || {
                                    for file in files.iter() {
                                        if ui.menu_item(file) {
                                            simulation = Simulation::from_file(file.clone()).unwrap();
                                            state.update_instances();
                                        }
                                    }
                                });
                            };

                            ui.group(|| {
                                if ui.button("<<") {
                                    simulation.step_back(&mut state.instances, 5);
                                    state.update_instances();
                                }

                                ui.same_line();
                                if ui.button(play_button_text) {
                                    is_playing = !is_playing;
                                }

                                ui.same_line();
                                if ui.button(">>") {
                                    simulation.step_forward(&mut state.instances, 5);
                                    state.update_instances();
                                }
                            });

                            if ui.button("Replay") {
                                simulation.frame_index = 0;
                                simulation.update_instances(&mut state.instances);
                                state.update_instances();
                            }
                        });
                }

                fluid_renderer::handle_rendering(&mut state, &mut imgui_renderer, imgui_ctxt.render(), control_flow);
                sleep(Duration::from_millis((1000.0 / fps as f32 - frame_delta.as_millis() as f32) as u64));
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }

        frame_delta = frame_start.elapsed();
    });
}


pub fn compute_simulation(
    path: String, 
    fps: u32, 
    simulation_time: u32,
    fluid_step_time: f32, 
    instances: Vec<Instance>, 
    particle_size: f32,
    particle_offset: f32,
) {
    let config = Config::from_instances( 
        vec3a(-60.0, -40.0, -60.0),
        vec3a(60.0, 40.0, 60.0),
        particle_size,
        1000.0,
        &instances
    );
    let mut fluid = WCSPHSolver::new(
        0.01,
        50000.0,
        0.01,
        fluid_step_time,
        config
    );

    let mut simulation_time = simulation_time;
    let mut frame_stop = (simulation_time * fps) as u32;
    let steps_per_frame = (1.0 / fluid_step_time / fps as f32).ceil() as u32;
    
    let mut simulation = Simulation::new(fps, frame_stop, instances.len() as u32);

    let InitOutput{event_loop, window, aspect_ratio} = init(); 
    let shader_source = fluid_renderer::wgpu::ShaderSource::Wgsl(std::fs::read_to_string("libs/fluid-renderer/src/shader.wgsl").unwrap().into());
    let vertices = Quad.scale(particle_size);
    let indices = Quad::INDICES;
    
    let camera = Camera {
        aspect: aspect_ratio,
        fovy: 45.0,
        eye: vec3a(-200.0, 200.0, 1000.0) / 4.0,
        zfar: 10000.0,
        ..Default::default()
    };

    let default_instances = instances.clone();

    let mut state = pollster::block_on(
        State::new(
            window, 
            shader_source, 
            vertices.as_slice(), 
            indices, 
            instances, 
            camera
        )
    );


    let (mut imgui_ctxt, mut imgui_platform, mut imgui_renderer) = init_ui(&state, 10.0);
    let mut frame_delta = Duration::from_millis(0);

    let mut is_playing = false;
    let total_time = Instant::now();

    let mut frame = 0;

    let mut viscosity = 0.01;
    let mut stiffness = 50000.0;
    let mut surface_tension = 0.01;

    let mut particle_size = particle_size;
    let mut particle_offset = particle_offset;
    let particle_count = 14;

    let domain_start = vec3a(-60.0, -40.0, -60.0);
    let domain_end = vec3a(60.0, 40.0, 60.0);

    let mut rest_density = 1000.0;


    
    event_loop.run(move |event, _, control_flow| {
        let frame_start = Instant::now();
        imgui_platform.handle_event(imgui_ctxt.io_mut(), &state.window, &event);

        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                crate::handle_windowing(&mut state, &mut imgui_ctxt, event, control_flow)
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                
                imgui_platform.prepare_frame(imgui_ctxt.io_mut(), &state.window).expect("Failed to prepare ui frame");
                imgui_ctxt.io_mut().update_delta_time(frame_delta);
                let ui = imgui_ctxt.frame();

                if is_playing {
                    if frame == frame_stop {
                        is_playing = false;
                        frame = 0;

                        println!("Hotovo, {}s", total_time.elapsed().as_millis() as f32 / 1000.0);
                        simulation.save((&path).clone()).unwrap();
                    }

                    for _step in 0..steps_per_frame {
                        fluid.step();
                    }

                    for (particle_id, instance_id) in fluid.ps().ids.iter().enumerate() {
                        let index = frame as usize * state.instances.len() + *instance_id;
                        simulation.frames[index] = fluid.ps().x[particle_id];
                    }

                    fluid.advect_instances(&mut state.instances);
                    state.update_instances();
                    println!("progress: {}/{} {}%, {}s", frame, frame_stop, frame*100/frame_stop, frame_start.elapsed().as_millis() as f32 / 1000.0);

                    frame += 1;
                }


                {
                    ui.window("Nastaveni")
                        .position([5.0, 5.0], imgui::Condition::FirstUseEver)
                        .size([180.0, 240.0], imgui::Condition::FirstUseEver)
                        .build(|| {
                            ui.slider("Viskozita", 0.01, 2.5, &mut viscosity);
                            ui.slider("Tuhost", 1000.0, 300_000.0, &mut stiffness);
                            ui.slider("Povrch. napeti", 0.01, 4.0, &mut surface_tension);
                            ui.slider("Hustota", 500.0, 5000.0, &mut rest_density);
                            if ui.slider("Delka sim. (s)", 1, 60, &mut simulation_time) {
                                frame_stop = (simulation_time * fps) as u32;
                                simulation.frame_stop = frame_stop;
                                simulation.frames = (0..(simulation.particle_num * frame_stop)).map(|_id| Vec3A::ZERO).collect();
                            }
                            ui.separator();

                            ui.text("Castice");
                            ui.group(|| {
                                ui.slider("Velikost", 0.1, 3.0, &mut particle_size);
                                // if ui.slider("pocet", 1, 40, &mut particle_count) {
                                //     let particle_offset = (
                                //         particle_size * particle_offset,
                                //         particle_size * particle_offset,
                                //         particle_size * particle_offset,
                                //     );
                                //
                                //     let instances = create_cube(0.04, (particle_count, particle_count, particle_count), Some(particle_offset), (-1.0, -1.0, -1.0));
                                //     state.resize_instances(instances);
                                // }
                                if ui.slider("Mezera", 0.1, 2.0, &mut particle_offset) {
                                    let particle_offset = (
                                        particle_size * particle_offset,
                                        particle_size * particle_offset,
                                        particle_size * particle_offset,
                                    );

                                    state.instances = create_cube(0.04, (particle_count, particle_count, particle_count), Some(particle_offset), (-1.0, -1.0, -1.0));
                                    state.update_instances();
                                }
                            });
                            ui.separator();

                            ui.text(format!("Snimek: {}", frame.min(simulation.frame_stop)));
                            // ui.slider("", min, max, value);

                            if is_playing {
                                ui.text(format!("Postup: {}/{} {}%, {}s", frame, frame_stop, frame*100/frame_stop, frame_start.elapsed().as_millis() as f32 / 1000.0));
                                return;
                            }

                            ui.spacing();
                            if frame == 1 {
                                if ui.button("Restart") {
                                    state.instances = default_instances.clone();
                                    state.update_instances();

                                    let config = Config::from_instances( 
                                        domain_start,
                                        domain_end,
                                        particle_size,
                                        rest_density,
                                        &state.instances
                                    );

                                    fluid = WCSPHSolver::new(
                                        viscosity,
                                        stiffness,
                                        surface_tension,
                                        fluid_step_time,
                                        config
                                    );
                                    
                                    frame = 0;
                                }

                                return;
                            }

                            if ui.button("Start") {
                                let config = Config::from_instances( 
                                    domain_start,
                                    domain_end,
                                    particle_size,
                                    rest_density,
                                    &state.instances
                                );

                                fluid = WCSPHSolver::new(
                                    viscosity,
                                    stiffness,
                                    surface_tension,
                                    fluid_step_time,
                                    config
                                );

                                is_playing = true;
                                println!("Starting simulation");
                            }
                        });
                }

                fluid_renderer::handle_rendering(&mut state, &mut imgui_renderer, imgui_ctxt.render(), control_flow);
            }
            Event::MainEventsCleared => {
                state.window().request_redraw();
            }
            _ => {}
        }

        frame_delta = frame_start.elapsed();
    });
}

