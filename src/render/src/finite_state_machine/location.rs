
// Some states here
pub struct Stalling;

pub struct Moving {
    goal_pos: Vector3<f32>,
    // Quaternion describing the position of the center
    start: SphericalRotation<f32>,
    // Quaternion describing the goal position
    goal: SphericalRotation<f32>,
    // Alpha coefficient between 0 and 1
    alpha: f32,
    // Start time
    t0: f32,
}
use crate::math;
use cgmath::{Vector4, Vector3};

impl Moving {
    #[inline]
    fn w0() -> f32 {
        5_f32
    }

    fn new(goal_pos: Vector3<f32>, start: SphericalRotation<f32>, goal: SphericalRotation<f32>) -> Moving {
        let t0 = utils::get_current_time();

        let alpha = 0_f32;
        Moving {
            goal_pos,
            start,
            goal,
            t0,
            alpha,
        }
    }
}


use crate::finite_state_machine::{
    State,
    Transition,
    T,
    FiniteStateMachine
};
use crate::time::DeltaTime;
use crate::Projection;
use crate::renderable::{
 HiPSSphere,
 catalog::{Manager},
 ProjetedGrid
};
use crate::event_manager::EventManager;
use crate::viewport::ViewPort;
impl State for Stalling {
    fn update<P: Projection>(&mut self,
        // Time of the previous frame
        _dt: DeltaTime,
        // Renderables
        _sphere: &mut HiPSSphere,
        _catalogs: &mut Manager,
        _grid: &mut ProjetedGrid,
        // Viewport
        _viewport: &mut ViewPort,
        // User events
        _events: &EventManager
    ) {}
}
use cgmath::InnerSpace;
use cgmath::Rotation;
use cgmath::SquareMatrix;
use crate::rotation::SphericalRotation;
impl State for Moving {
    fn update<P: Projection>(&mut self,
        // Time of the previous frame
        _dt: DeltaTime,
        // Renderables
        sphere: &mut HiPSSphere,
        catalogs: &mut Manager,
        grid: &mut ProjetedGrid,
        // Viewport
        viewport: &mut ViewPort,
        // User events
        _events: &EventManager
    ) {
        // Time elapsed since the beginning of the inertia
        let t = (utils::get_current_time() - self.t0)/1000_f32;
        
        // Undamped angular frequency of the oscillator
        // From wiki: https://en.wikipedia.org/wiki/Harmonic_oscillator
        //
        // In a damped harmonic oscillator system: w0 = sqrt(k / m)
        // where: 
        // * k is the stiffness of the ressort
        // * m is its mass
        let alpha = 1_f32 + (0_f32 - 1_f32) * (Moving::w0() * t + 1_f32) * ((-Moving::w0() * t).exp());
        let p = self.start.slerp(&self.goal, alpha);

        viewport.set_rotation::<P>(&p, sphere.config());
        sphere.ask_for_tiles::<P>(viewport.new_healpix_cells());

        self.alpha = alpha;
    }
}


use crate::event_manager::MoveToLocation;
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
        viewport: &mut ViewPort,
        // User events
        events: &EventManager,
        dt: DeltaTime
    ) -> Option<Self::E> {
        if let Some(lonlat) = events.get::<MoveToLocation>() {
            let start = *viewport.get_rotation();
            let goal_pos = lonlat.vector();
            let goal = SphericalRotation::from_sky_position(&goal_pos);
            Some(Moving::new(goal_pos.truncate(), start, goal))
        } else {
            // No left button pressed, we keep stalling
            None
        }
    }
}

use crate::utils;
use crate::event_manager::{
 SetCenterLocation,
 SetFieldOfView,
 ZoomToLocation
};
use crate::renderable::{Angle, ArcSec};
// Moving -> Stalling
impl Transition for T<Moving, Stalling> {
    type S = Moving;
    type E = Stalling;

    fn condition<P: Projection>(s: &Self::S,
        // Renderables
        _sphere: &mut HiPSSphere,
        _catalogs: &mut Manager,
        _grid: &mut ProjetedGrid,
        // Viewport
        viewport: &mut ViewPort,
        // User events
        events: &EventManager,
        dt: DeltaTime
    ) -> Option<Self::E> {
        // Priority to the user events
        // such as move
        if events.check::<SetCenterLocation>() {
            Some(Stalling {})
        // or zooming/unzooming events
        } else {
            // Criteria
            let err = math::ang_between_vect(&s.goal_pos, &viewport.compute_center_model_pos::<P>().truncate());
            //let eps = (s.alpha - 1_f32).abs();
            let thresh: Angle<f32> = ArcSec(2_f32).into();
            if err < thresh {
                Some(Stalling{})
            } else {
                None
            }
        }
    }
}

pub enum MoveSphere {
    Stalling(Stalling),
    Moving(Moving),
}

impl FiniteStateMachine for MoveSphere {
    fn init() -> Self {
        MoveSphere::Stalling(Stalling {})
    }
}

impl MoveSphere {
    fn update<P: Projection>(&mut self,
        // Time of the previous frame
        dt: DeltaTime,
        // Renderables
        sphere: &mut HiPSSphere,
        catalogs: &mut Manager,
        grid: &mut ProjetedGrid,
        // Viewport
        viewport: &mut ViewPort,
        // User events
        events: &EventManager
    ) {
        match self {
            MoveSphere::Stalling(s) => s.update::<P>(dt, sphere, catalogs, grid, viewport, events),
            MoveSphere::Moving(s) => s.update::<P>(dt, sphere, catalogs, grid, viewport, events),
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
        viewport: &mut ViewPort,
        // User events
        events: &EventManager
    ) {
        // Update the current state
        self.update::<P>(dt,
            sphere, catalogs, grid,
            viewport,
            events
        );

        // Checks whether conditions are valid after the update
        match self {
            MoveSphere::Stalling(stalling) => {
                // Checks the Stalling -> Moving condition
                if let Some(e) = stalling.check::<_, P>(sphere, catalogs, grid, viewport, events, dt) {
                    *self = MoveSphere::Moving(e);
                }
            },
            MoveSphere::Moving(moving) => {
                // Checks the Moving -> Stalling condition
                if let Some(e) = moving.check::<_, P>(sphere, catalogs, grid, viewport, events, dt) {
                    *self = MoveSphere::Stalling(e);
                }
            },
        }
    }
}