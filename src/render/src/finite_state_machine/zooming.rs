
// Some states here
pub struct Stalling;
use crate::time::DeltaTime;
use crate::rotation::SphericalRotation;
pub struct Zooming {
    // Quaternion describing the position of the center
    start_rot: SphericalRotation<f32>,
    // Quaternion describing the position under the mouse when zooming
    goal_rot: SphericalRotation<f32>,
    alpha: f32,
    // Initial field of view (rad)
    z0: Angle<f32>,
    // Goal field of view (rad)
    zf: Angle<f32>,
    // Current quantity of zoom (rad)
    z: Angle<f32>,
    // The time when the zooming begins (in ms)
    t0: f32,
}

impl Zooming {
    fn new<P: Projection>(z: Angle<f32>, z0: Angle<f32>, zf: Angle<f32>, goal_rot: SphericalRotation<f32>, viewport: &ViewPort) -> Zooming {
        let t0 = utils::get_current_time();
        let start_rot = *viewport.get_rotation();

        let alpha = 0.0;
        Zooming {
            start_rot,
            goal_rot,
            alpha,
            z0,
            zf,
            z,
            t0
        }
    }
}

use crate::renderable::Angle;
pub struct Unzooming {
    // Initial field of view (rad)
    z0: Angle<f32>,
    // Goal field of view (rad)
    zf: Angle<f32>,
    // Current quantity of zoom (rad)
    z: Angle<f32>,
    // The time when the unzooming begins (in ms)
    t0: f32,
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
 catalog::Manager,
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

impl Zooming {
    #[inline]
    pub fn w0() -> f32 {
        10_f32
    }

    #[inline]
    pub fn a0(viewport: &ViewPort) -> f32 {
        let a0_max = 8_f32;
        let a0 = a0_max / viewport.get_aperture().0;
        a0.min(a0_max)
    }
}

use crate::utils;
impl State for Zooming {
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
        let z = self.zf + (self.z0 - self.zf) * (Zooming::w0() * t + 1_f32) * ((-Zooming::w0() * t).exp());

        viewport.set_aperture::<P>(z, sphere.config());

        self.z = z;

        let a = 1_f32 + (0_f32 - 1_f32) * (Zooming::a0(viewport) * t + 1_f32) * ((-Zooming::a0(viewport) * t).exp());
        //let p = self.start_rot.slerp(&self.goal_rot, a);

        //viewport.set_rotation::<P>(&p, sphere.config());
        self.alpha = a;
    }
}

impl Unzooming {
    #[inline]
    pub fn w0() -> f32 {
        8_f32
    }
}

impl State for Unzooming {
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
        let z = self.zf + (self.z0 - self.zf) * (Unzooming::w0() * t + 1_f32) * ((-Unzooming::w0() * t).exp());

        viewport.set_aperture::<P>(z, sphere.config());

        self.z = z;
    }
}

use crate::event_manager::{ZoomToLocation, SetCenterLocation, SetFieldOfView};
// Stalling -> Zooming
impl Transition for T<Stalling, Zooming> {
    type S = Stalling;
    type E = Zooming;
   
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
        if let Some((lonlat, fov)) = events.get::<ZoomToLocation>() {
            let z0 = viewport.get_aperture();
            //let zf = z0 / (1_f32 / (NUM_WHEEL_PER_DEPTH as f32)).exp2();
            let zf = *fov;

            let zooming = *fov < z0;

            if zooming {
                let goal_rot = SphericalRotation::from_sky_position(&lonlat.vector());
                Some(Zooming::new::<P>(z0, z0, zf, goal_rot, viewport))
            } else {
                // unzooming
                None
            }
        } else {
            // No left button pressed, we keep stalling
            None
        }
    }
}
// Zooming -> Zooming
impl Transition for T<Zooming, Zooming> {
    type S = Zooming;
    type E = Zooming;
   
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
        if let Some((lonlat, fov)) = events.get::<ZoomToLocation>() {
            let z0 = viewport.get_aperture();
            //let zf = z0 / (1_f32 / (NUM_WHEEL_PER_DEPTH as f32)).exp2();
            let zf = *fov;

            let zooming = *fov < z0;

            if zooming {
                let goal_rot = SphericalRotation::from_sky_position(&lonlat.vector());
                Some(Zooming::new::<P>(s.z, s.z, zf, goal_rot, viewport))
            } else {
                // unzooming
                None
            }
        } else {
            // No left button pressed, we keep stalling
            None
        }
    }
}
use std::collections::HashMap;
// Zooming -> Stalling
impl Transition for T<Zooming, Stalling> {
    type S = Zooming;
    type E = Stalling;

    fn condition<P: Projection>(s: &Self::S,
        // Renderables
        sphere: &mut HiPSSphere,
        _catalogs: &mut Manager,
        _grid: &mut ProjetedGrid,
        // Viewport
        viewport: &mut ViewPort,
        // User events
        events: &EventManager,
        dt: DeltaTime
    ) -> Option<Self::E> {
        if events.check::<SetCenterLocation>() || events.check::<SetFieldOfView>() {
            sphere.ask_for_tiles::<P>(viewport.new_healpix_cells());

            Some(Stalling {})
        } else if (s.z - s.zf).0.abs() < 1e-4 && (s.alpha - 1_f32).abs() < 1e-3 {
            sphere.ask_for_tiles::<P>(viewport.new_healpix_cells());

            Some(Stalling {})
        } else {
            None
        }
    }
}

use cgmath::Vector2;
// Stalling -> Unzooming
impl Transition for T<Stalling, Unzooming> {
    type S = Stalling;
    type E = Unzooming;
   
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
        if let Some((_, fov)) = events.get::<ZoomToLocation>() {
            let z0 = viewport.get_aperture();

            let unzooming = *fov > z0;

            if unzooming {
                let zf = *fov;

                let t0 = utils::get_current_time();
                let z = z0;

                Some(Unzooming {
                    t0,
                    z0,
                    zf,
                    z
                })
            } else {
                // zooming
                None
            }
        } else {
            // No left button pressed, we keep stalling
            None
        }
    }
}
// Unzooming -> Unzooming
impl Transition for T<Unzooming, Unzooming> {
    type S = Unzooming;
    type E = Unzooming;
   
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
        if let Some((_, fov)) = events.get::<ZoomToLocation>() {
            let z0 = viewport.get_aperture();

            let unzooming = *fov > z0;

            if unzooming {
                let zf = *fov;

                let t0 = utils::get_current_time();
                let z = z0;

                Some(Unzooming {
                    t0,
                    z0,
                    zf,
                    z
                })
            } else {
                // zooming
                None
            }
        } else {
            // No left button pressed, we keep stalling
            None
        }
    }
}
// Unzooming -> Stalling
impl Transition for T<Unzooming, Stalling> {
    type S = Unzooming;
    type E = Stalling;

    fn condition<P: Projection>(s: &Self::S,
        // Renderables
        sphere: &mut HiPSSphere,
        _catalogs: &mut Manager,
        _grid: &mut ProjetedGrid,
        // Viewport
        viewport: &mut ViewPort,
        // User events
        events: &EventManager,
        dt: DeltaTime
    ) -> Option<Self::E> {
        if events.check::<SetCenterLocation>() || events.check::<SetFieldOfView>() {
            sphere.ask_for_tiles::<P>(viewport.new_healpix_cells());

            Some(Stalling {})
        } else if (s.z - s.zf).0.abs() < 1e-4 {
            //viewport.unzoom::<P>(s.zf, sphere, catalog, grid);
            sphere.ask_for_tiles::<P>(viewport.new_healpix_cells());

            Some(Stalling {})
        } else {
            None
        }
    }
}

// Zooming -> Unzooming
impl Transition for T<Zooming, Unzooming> {
    type S = Zooming;
    type E = Unzooming;

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
        if let Some((_, fov)) = events.get::<ZoomToLocation>() {
            let z0 = viewport.get_aperture();

            let unzooming = *fov > z0;

            if unzooming {
                let zf = *fov;

                let t0 = utils::get_current_time();
                let z = z0;

                Some(Unzooming {
                    t0,
                    z0,
                    zf,
                    z
                })
            } else {
                // zooming
                None
            }
        } else {
            // No left button pressed, we keep stalling
            None
        }
    }
}

// Unzooming -> Zooming
impl Transition for T<Unzooming, Zooming> {
    type S = Unzooming;
    type E = Zooming;

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
        if let Some((lonlat, fov)) = events.get::<ZoomToLocation>() {
            let z0 = viewport.get_aperture();

            let zooming = *fov < z0;

            if zooming {
                let zf = *fov;

                let goal_rot = SphericalRotation::from_sky_position(&lonlat.vector());
                Some(Zooming::new::<P>(s.z, s.z, zf, goal_rot, viewport))
            } else {
                // unzooming
                None
            }
        } else {
            // No left button pressed, we keep stalling
            None
        }
    }
}

pub enum UserZoom{
    Stalling(Stalling),
    Zooming(Zooming),
    Unzooming(Unzooming)
}

impl FiniteStateMachine for UserZoom {
    fn init() -> Self {
        UserZoom::Stalling(Stalling {})
    }
}

impl UserZoom {
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
            UserZoom::Stalling(s) => s.update::<P>(dt, sphere, catalogs, grid, viewport, events),
            UserZoom::Zooming(s) => s.update::<P>(dt, sphere, catalogs, grid, viewport, events),
            UserZoom::Unzooming(s) => s.update::<P>(dt, sphere, catalogs, grid, viewport, events),
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
            UserZoom::Stalling(stalling) => {
                // Checks the Stalling -> Moving condition
                if let Some(e) = stalling.check::<_, P>(sphere, catalogs, grid, viewport, events, dt) {
                    *self = UserZoom::Zooming(e);
                } else if let Some(e) = stalling.check::<_, P>(sphere, catalogs, grid, viewport, events, dt) {
                    *self = UserZoom::Unzooming(e);
                }
            },
            UserZoom::Zooming(zooming) => {
                // Checks the Zooming -> Stalling condition
                if let Some(e) = zooming.check::<_, P>(sphere, catalogs, grid, viewport, events, dt) {
                    *self = UserZoom::Stalling(e);
                // Checks the Zooming -> Zooming condition
                } else if let Some(e) = zooming.check::<_, P>(sphere, catalogs, grid, viewport, events, dt) {
                    *self = UserZoom::Zooming(e);
                // Checks the Zooming -> Unzooming condition
                } else if let Some(e) = zooming.check::<_, P>(sphere, catalogs, grid, viewport, events, dt) {
                    *self = UserZoom::Unzooming(e);
                }
            },
            UserZoom::Unzooming(unzooming) => {
                // Checks the Unzooming -> Stalling condition
                if let Some(e) = unzooming.check::<_, P>(sphere, catalogs, grid, viewport, events, dt) {
                    *self = UserZoom::Stalling(e);
                // Checks the Unzooming -> Unzooming condition
                } else if let Some(e) = unzooming.check::<_, P>(sphere, catalogs, grid, viewport, events, dt) {
                    *self = UserZoom::Unzooming(e);
                // Checks the Unzooming -> Zooming condition
                } else if let Some(e) = unzooming.check::<_, P>(sphere, catalogs, grid, viewport, events, dt) {
                    *self = UserZoom::Zooming(e);
                }
            },
        }
    }
}