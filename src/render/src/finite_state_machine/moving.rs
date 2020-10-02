
/*
// Some states here
pub struct Stalling;

use cgmath::Vector4;
pub struct Moving {
    // World position corresponding to the move
    pos_world_space: Vector4<f32>,
    time_move: f32,

    angular_dist: Angle<f32>,
    axis: Vector3<f32>
}
use cgmath::Vector3;
pub struct Inertia {
    // Init angular distance of rotation
    d0: Angle<f32>,
    // angular distance
    d: Angle<f32>,
    // The axis of rotation when the mouse has been released
    axis: Vector3<f32>,
    // The time when the inertia begins (in ms)
    t_start: f32,
}

use crate::finite_state_machine::{
    State,
    Transition,
    T,
    FiniteStateMachine
};
use crate::renderable::projection::Projection;
use crate::renderable::{
 HiPSSphere,
 catalog::{Manager},
 ProjetedGrid
};
use crate::event_manager::EventManager;
use crate::camera::CameraViewPort;
use crate::time::DeltaTime;
impl State for Stalling {
    fn update<P: Projection>(&mut self,
        // Time of the previous frame
        _dt: DeltaTime,
        // Renderables
        _sphere: &mut HiPSSphere,
        _catalogs: &mut Manager,
        _grid: &mut ProjetedGrid,
        // Viewport
        _camera: &mut CameraViewPort,
        // User events
        _events: &EventManager
    ) {}
}

use crate::renderable::Angle;
use std::collections::HashMap;
// Move the renderables between two world position on the sky
pub fn move_renderables<P: Projection>(
 // Previous model position
 x: &Vector4<f32>,
 // Current model position
 y: &Vector4<f32>,
 // Renderables
 app: &mut App,
 // Viewport
 camera: &mut CameraViewPort,
) -> (Vector3<f32>, Angle<f32>) {
    let r = camera.get_rotation();

    let x = r.rotate(x).truncate();
    let y = r.rotate(y).truncate();
    if x == y {
        (Vector3::new(1.0, 0.0, 0.0), Angle(0.0))
    } else {
        let axis = x.cross(y)
            .normalize();
        let d = math::ang_between_vect(&x, &y);

        camera.rotate::<P>(&(-axis), d);
        app.look_for_new_tiles();

        (axis, d)
    }
}

use crate::math;
use cgmath::InnerSpace;
impl State for Moving {
    fn update<P: Projection>(&mut self,
        // Time of the previous frame
        _dt: DeltaTime,
        // Renderables
        sphere: &mut HiPSSphere,
        catalogs: &mut Manager,
        grid: &mut ProjetedGrid,
        // Viewport
        camera: &mut CameraViewPort,
        // User events
        events: &EventManager
    ) {
        if let Some(lonlat) = events.get::<SetCenterLocation>() {
            let pos_world_space = lonlat.vector();

            let (axis, d) = move_renderables::<P>(
                &self.pos_world_space,
                &pos_world_space,
                sphere, catalogs, grid,
                camera
            );

            // Update the previous position
            self.pos_world_space = pos_world_space;
            self.time_move = utils::get_current_time();
            self.angular_dist = d;
            self.axis = axis;
        }
    }
}

impl Inertia {
    #[inline]
    pub fn w0() -> f32 {
        5_f32
    }
}
impl State for Inertia {
    fn update<P: Projection>(&mut self,
        // Time of the previous frame
        _dt: DeltaTime,
        // Renderables
        sphere: &mut HiPSSphere,
        catalogs: &mut Manager,
        grid: &mut ProjetedGrid,
        // Viewport
        camera: &mut CameraViewPort,
        // User events
        _events: &EventManager
    ) {
        // Time elapsed since the beginning of the inertia
        let t = (utils::get_current_time() - self.t_start)/1000_f32;
        // Undamped angular frequency of the oscillator
        // From wiki: https://en.wikipedia.org/wiki/Harmonic_oscillator
        //
        // In a damped harmonic oscillator system: w0 = sqrt(k / m)
        // where:
        // * k is the stiffness of the ressort
        // * m is its mass
        let theta = self.d0 * (Inertia::w0() * t + 1_f32) * ((-Inertia::w0() * t).exp());

        camera.apply_rotation::<P>(&(-self.axis), theta, sphere.config());

        // Ask for grand parent tiles
        sphere.ask_for_tiles::<P>(camera.new_healpix_cells());

        self.d = theta;
    }
}

use crate::event_manager::SetCenterLocation;
// Stalling -> Moving
impl Transition for T<Stalling, Moving> {
    type S = Stalling;
    type E = Moving;
   
    fn condition<P: Projection>(_s: &Self::S,
        // Renderables
        _sphere: &mut HiPSSphere,
        _catalogs: &mut Manager,
        _grid: &mut ProjetedGrid,
        // Viewport
        camera: &mut CameraViewPort,
        // User events
        events: &EventManager,
        dt: DeltaTime
    ) -> Option<Self::E> {
        if let Some(lonlat) = events.get::<SetCenterLocation>() {
            let time_move = utils::get_current_time();
            let angular_dist = Angle(0_f32);
            let axis = Vector3::new(0_f32, 0_f32, 0_f32);
            let pos_world_space = lonlat.vector();
            Some(Moving {
                pos_world_space,
                time_move,
                angular_dist,
                axis
            })
        } else {
            // No left button pressed, we keep stalling
            None
        }
    }
}

use crate::utils;
// Moving -> Stalling
impl Transition for T<Moving, Stalling> {
    type S = Moving;
    type E = Stalling;

    fn condition<P: Projection>(s: &Self::S,
        // Renderables
        sphere: &mut HiPSSphere,
        catalogs: &mut Manager,
        grid: &mut ProjetedGrid,
        // Viewport
        camera: &mut CameraViewPort,
        // User events
        events: &EventManager,
        dt: DeltaTime
    ) -> Option<Self::E> {
        if !events.check::<SetCenterLocation>() {
            // Update all the renderables
            // Ask for grand parent tiles
            let depth = camera.depth();
            let d0 = ((depth as i8) - 3).max(0) as u8;

            let parent_cells = camera.get_cells_in_fov::<P>(d0)
                .into_iter()
                .map(|cell| (cell, false))
                .collect::<HashMap<_, _>>();
            sphere.ask_for_tiles::<P>(&parent_cells);
            sphere.ask_for_tiles::<P>(camera.new_healpix_cells());

            Some(Stalling {})
        } else {
            None
        }
    }
}
use crate::event_manager::StartInertia;
 
// Moving -> Inertia
impl Transition for T<Moving, Inertia> {
    type S = Moving;
    type E = Inertia;

    fn condition<P: Projection>(s: &Self::S,
        // Renderables
        sphere: &mut HiPSSphere,
        catalogs: &mut Manager,
        grid: &mut ProjetedGrid,
        // Viewport
        camera: &mut CameraViewPort,
        // User events
        events: &EventManager,
        dt: DeltaTime
    ) -> Option<Self::E> {
        if events.check::<SetCenterLocation>() && events.check::<StartInertia>() {
            let duration_from_last_move = utils::get_current_time() - s.time_move;
            // Jump into the inertia mode if the mouse has been released in following 10ms after the last move
            let max_duration_trig_inertia = (dt * 2.0).0;
            if duration_from_last_move <= max_duration_trig_inertia {
                let axis = s.axis;
                let d0 = s.angular_dist;
                let d = d0;
                let t_start = utils::get_current_time();
                Some(Inertia {
                    d0,
                    d,
                    axis,
                    t_start
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

// Inertia -> Stalling
impl Transition for T<Inertia, Stalling> {
    type S = Inertia;
    type E = Stalling;

    fn condition<P: Projection>(s: &Self::S,
        // Renderables
        sphere: &mut HiPSSphere,
        _catalogs: &mut Manager,
        _grid: &mut ProjetedGrid,
        // Viewport
        camera: &mut CameraViewPort,
        // User events
        _events: &EventManager,
        dt: DeltaTime
    ) -> Option<Self::E> {
        if s.d < 1e-5 {
            // Update all the renderables
            // Ask for grand parent tiles
            let depth = camera.depth();
            let d0 = ((depth as i8) - 3).max(0) as u8;

            let parent_cells = camera.get_cells_in_fov::<P>(d0)
                .into_iter()
                .map(|cell| (cell, false))
                .collect::<HashMap<_, _>>();
            sphere.ask_for_tiles::<P>(&parent_cells);
            sphere.ask_for_tiles::<P>(camera.new_healpix_cells());

            Some(Stalling {})
        } else {
            None
        }
    }
}

// Inertia -> Moving
impl Transition for T<Inertia, Moving> {
    type S = Inertia;
    type E = Moving;

    fn condition<P: Projection>(_s: &Self::S,
        // Renderables
        _sphere: &mut HiPSSphere,
        _catalogs: &mut Manager,
        _grid: &mut ProjetedGrid,
        // Viewport
        camera: &mut CameraViewPort,
        // User events
        events: &EventManager,
        dt: DeltaTime
    ) -> Option<Self::E> {
        if let Some(lonlat) = events.get::<SetCenterLocation>() {
            let pos_world_space = lonlat.vector();
            let time_move = utils::get_current_time();
            let angular_dist = Angle(0_f32);
            let axis = Vector3::new(0_f32, 0_f32, 0_f32);
            Some(Moving {
                pos_world_space,
                time_move,
                angular_dist,
                axis
            })
        } else {
            // No left button pressed, we keep being in the inertia state
            None
        }
    }
}

pub enum UserMoveSphere {
    Stalling(Stalling),
    Moving(Moving),
    Inertia(Inertia)
}

impl FiniteStateMachine for UserMoveSphere {
    fn init() -> Self {
        UserMoveSphere::Stalling(Stalling {})
    }
}

impl UserMoveSphere {
    fn update<P: Projection>(&mut self,
        // Time of the previous frame
        dt: DeltaTime,
        // Renderables
        sphere: &mut HiPSSphere,
        catalogs: &mut Manager,
        grid: &mut ProjetedGrid,
        // Viewport
        camera: &mut CameraViewPort,
        // User events
        events: &EventManager
    ) {
        match self {
            UserMoveSphere::Stalling(s) => s.update::<P>(dt, sphere, catalogs, grid, camera, events),
            UserMoveSphere::Moving(s) => s.update::<P>(dt, sphere, catalogs, grid, camera, events),
            UserMoveSphere::Inertia(s) => s.update::<P>(dt, sphere, catalogs, grid, camera, events),
        }
    }

    pub fn run<P: Projection>(
        &mut self,
        // Time of the previous frame
        dt: DeltaTime,
        // Renderables
        sphere: &mut HiPSSphere,
        catalogs: &mut Manager,
        grid: &mut ProjetedGrid,
        // Viewport
        camera: &mut CameraViewPort,
        // User events
        events: &EventManager
    ) {
        // Update the current state
        self.update::<P>(dt,
            sphere, catalogs, grid,
            camera,
            events
        );

        // Checks whether conditions are valid after the update
        match self {
            UserMoveSphere::Stalling(stalling) => {
                // Checks the Stalling -> Moving condition
                if let Some(e) = stalling.check::<_, P>(sphere, catalogs, grid, camera, events, dt) {
                    *self = UserMoveSphere::Moving(e);
                }
            },
            UserMoveSphere::Moving(moving) => {
                // Checks the Moving -> Stalling condition
                if let Some(e) = moving.check::<_, P>(sphere, catalogs, grid, camera, events, dt) {
                    *self = UserMoveSphere::Stalling(e);
                // Checks the Moving -> Inertia condition
                } else if let Some(e) = moving.check::<_, P>(sphere, catalogs, grid, camera, events, dt) {
                    *self = UserMoveSphere::Inertia(e);
                }
            },
            UserMoveSphere::Inertia(inertia) => {
                // Checks the Inertia -> Stalling condition
                if let Some(e) = inertia.check::<_, P>(sphere, catalogs, grid, camera, events, dt) {
                    *self = UserMoveSphere::Stalling(e);
                // Checks the Inertia -> Moving condition
                } else if let Some(e) = inertia.check::<_, P>(sphere, catalogs, grid, camera, events, dt) {
                    *self = UserMoveSphere::Moving(e);
                }
            },
        }
    }
}*/