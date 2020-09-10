pub trait Event {
    const KEY_NAME: &'static str;
    type Data;

    fn to(data: Self::Data) -> EventType;
    fn get(event: &EventType) -> &Self::Data;
}

use cgmath::Vector2;

pub struct MoveToLocation;
impl Event for MoveToLocation {
    type Data = LonLatT<f32>;
    const KEY_NAME: &'static str = "MoveToLocation";
    
    fn to(data: Self::Data) -> EventType {
        EventType::MoveToLocation(data)
    }
    fn get(event: &EventType) -> &Self::Data {
        match event {
            EventType::MoveToLocation(data) => data,
            _ => unreachable!()
        }
    }
}

pub struct ZoomToLocation;
impl Event for ZoomToLocation {
    type Data = (LonLatT<f32>, Angle<f32>);
    const KEY_NAME: &'static str = "ZoomToLocation";
    
    fn to(data: Self::Data) -> EventType {
        EventType::ZoomToLocation(data)
    }
    fn get(event: &EventType) -> &Self::Data {
        match event {
            EventType::ZoomToLocation(data) => data,
            _ => unreachable!()
        }
    }
}

pub struct SetCenterLocation;
impl Event for SetCenterLocation {
    type Data = LonLatT<f32>;
    const KEY_NAME: &'static str = "SetCenterLocation";
    
    fn to(data: Self::Data) -> EventType {
        EventType::SetCenterLocation(data)
    }
    fn get(event: &EventType) -> &Self::Data {
        match event {
            EventType::SetCenterLocation(data) => data,
            _ => unreachable!()
        }
    }
}
pub struct StartInertia;
impl Event for StartInertia {
    type Data = ();
    const KEY_NAME: &'static str = "StartInertia";
    
    fn to(data: Self::Data) -> EventType {
        EventType::StartInertia(data)
    }
    fn get(event: &EventType) -> &Self::Data {
        match event {
            EventType::StartInertia(data) => data,
            _ => unreachable!()
        }
    }
}
pub struct SetFieldOfView;
impl Event for SetFieldOfView {
    type Data = ();
    const KEY_NAME: &'static str = "SetFieldOfView";
    
    fn to(data: Self::Data) -> EventType {
        EventType::SetFieldOfView(data)
    }
    fn get(event: &EventType) -> &Self::Data {
        match event {
            EventType::SetFieldOfView(data) => data,
            _ => unreachable!()
        }
    }
}
use crate::{
    Angle,
    math::LonLatT
};
// An enum of the different possible user interactions
pub enum EventType {
    SetCenterLocation(LonLatT<f32>),
    SetFieldOfView(()),
    StartInertia(()),
    ZoomToLocation((LonLatT<f32>, Angle<f32>)),
    MoveToLocation(LonLatT<f32>)
}

use std::collections::HashMap;
#[derive(Default)]
pub struct EventManager(HashMap<&'static str, Option<EventType>>);

impl EventManager {
    pub fn new() -> EventManager {
        let mut manager = EventManager(HashMap::new());

        manager.insert_new_event::<SetCenterLocation>();
        manager.insert_new_event::<StartInertia>();
        manager.insert_new_event::<ZoomToLocation>();
        manager.insert_new_event::<MoveToLocation>();
        manager.insert_new_event::<SetFieldOfView>();

        manager
    }
    
    // Private method for inserting new events in the manager
    fn insert_new_event<E: Event>(&mut self) {
        self.0.insert(E::KEY_NAME, None);
    }

    pub fn enable<E: Event>(&mut self, data: E::Data) {
        let val = E::to(data);

        let v = self.0
            .get_mut(E::KEY_NAME)
            .unwrap();
        *v = Some(val);
    }

    /*pub fn _disable<E: Event>(&mut self) {
        let v = self.0
            .get_mut(E::KEY_NAME)
            .unwrap();
        *v = None;
    }*/

    pub fn get<E: Event>(&self) -> Option<&E::Data> {
        let v = self.0.get(E::KEY_NAME).unwrap();

        if let Some(event) = v {
            Some(E::get(event))
        } else {
            None
        }
    }

    pub fn check<E: Event>(&self) -> bool {
        self.0.get(E::KEY_NAME)
            .unwrap()
            .is_some()
    }

    // Reset the user events at the end
    // of each frame
    pub fn reset(&mut self) {
        for (_, val) in self.0.iter_mut() {
            *val = None;
        }
    }
}