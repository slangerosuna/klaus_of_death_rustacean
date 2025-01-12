use crate::*;
use std::any::Any;
use std::cell::SyncUnsafeCell;
use std::sync::Arc;

// increase this every time you add a new component type
const COMPONENT_TYPES: usize = 2;

pub struct GameState {
    pub entities: Vec<Arc<SyncUnsafeCell<Entity>>>,
    pub components: Vec<Vec<Arc<SyncUnsafeCell<ComponentStruct>>>>,
    pub resources: Vec<Box<dyn Resource>>,

    pub scheduler: *mut Scheduler,

    pub next_entity_id: u32,
    pub conf: &'static Config,

    pub should_close: bool,
}

impl GameState {
    pub fn close(&mut self) {
        self.should_close = true;
    }

    pub fn get_scheduler(&self) -> &Scheduler {
        unsafe { &*self.scheduler }
    }

    pub fn get_scheduler_mut(&mut self) -> &mut Scheduler {
        unsafe { &mut *self.scheduler }
    }

    pub const fn get_component_type() -> ComponentType {
        COMPONENT_TYPES
    }

    pub fn new(scheduler: *mut Scheduler, conf: &'static Config) -> GameState {
        GameState {
            entities: Vec::new(),
            components: vec![Vec::new(); COMPONENT_TYPES],
            resources: Vec::new(),
            next_entity_id: 0,
            scheduler,

            conf,
            should_close: false,
        }
    }

    pub fn create_entity<'a>(&mut self, name: String) -> &'a mut Entity {
        let entity = Entity::new(self.next_entity_id, name);
        let rc = Arc::new(SyncUnsafeCell::new(entity));

        self.entities.push(rc.clone());
        self.next_entity_id += 1;

        unsafe { &mut *rc.get() }
    }

    pub fn get_entity(&self, id: usize) -> Option<&'static mut Entity> {
        if id >= self.entities.len() {
            return None;
        }
        Some(unsafe { &mut *self.entities[id].get() })
    }

    pub fn get_entity_mut(&self, id: usize) -> Option<&'static mut Entity> {
        if id >= self.entities.len() {
            return None;
        }
        Some(unsafe { &mut *self.entities[id].get() })
    }

    pub fn get_entities_with<T: Component>(
        &self,
        component_type: ComponentType,
    ) -> Vec<&'static Entity> {
        self.components[component_type]
            .iter()
            .map(|component| {
                let component = component.get();
                let entity = unsafe { &*component }.owner as usize;
                unsafe { &*self.entities[entity].get() }
            })
            .collect()
    }

    pub fn get_entities_with_mut<T: Component>(
        &mut self,
        component_type: ComponentType,
    ) -> Vec<&'static mut Entity> {
        self.components[component_type]
            .iter_mut()
            .map(|component| {
                let component = component.get();
                let entity = unsafe { &*component }.owner as usize;
                unsafe { &mut *self.entities[entity].get() }
            })
            .collect()
    }

    pub fn get_components<T: Component>(&self, component_type: ComponentType) -> Vec<&'static T> {
        self.components[component_type]
            .iter()
            .map(|component| {
                let component = component.get();
                let component = unsafe { &*component };
                let component = &component.component;
                let component = &**component;
                unsafe { &*((component as &dyn Any).downcast_ref_unchecked::<T>() as *const T) }
            })
            .collect()
    }

    pub fn get_components_mut<T: Component>(
        &mut self,
        component_type: ComponentType,
    ) -> Vec<&'static mut T> {
        self.components[component_type]
            .iter_mut()
            .map(|component| {
                let component = component.get();
                let component = unsafe { &mut *component };
                let component = &mut component.component;
                let component = &mut **component;
                let component =
                    unsafe { (component as &mut dyn std::any::Any).downcast_mut_unchecked::<T>() };
                let component = component as *mut T;
                unsafe { &mut *component }
            })
            .collect()
    }

    pub fn add_resource<T: Resource>(&mut self, resource: T) {
        self.resources.push(Box::new(resource));
    }

    pub fn get_resource<T: Resource>(&self) -> Option<&'static T> {
        for resource in &self.resources {
            if let Some(r) = resource.as_ref().as_any().downcast_ref::<T>() {
                return Some(unsafe { &*(r as *const _) });
            }
        }
        None
    }

    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&'static mut T> {
        for resource in &mut self.resources {
            if let Some(r) = (resource.as_mut() as &mut dyn std::any::Any).downcast_mut::<T>() {
                return Some(unsafe { &mut*(r as *mut _) });
            }
        }
        None
    }
}
