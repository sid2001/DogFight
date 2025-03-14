use crate::sets::*;
use bevy::prelude::*;
use bevy::utils::info;

use super::explosion::ExplosibleObjectMarker;
use std::sync::{Arc, RwLock};

pub trait Collider: Send + Sync {
    fn check_collision_with_sphere(&self, op: &Arc<RwLock<dyn Collider>>) -> bool;
    fn check_collision_with_point(&self, op: &Arc<RwLock<dyn Collider>>) -> bool;
    fn get_radius(&self) -> Result<f32, ()>;
    fn get_center(&self) -> Result<Vec3, ()>;
    fn set_center(&mut self, c: Vec3) -> Result<(), ()>;
}

pub enum ColliderType {
    Sphere,
    Box,
    Point,
}

pub struct SphericalCollider {
    pub radius: f32,
    pub center: Vec3,
}

struct BoxCollider;
pub struct PointCollider {
    pub center: Vec3,
}

#[derive(Component, Clone, Copy)]
pub struct CollisionDamage {
    pub damage: f32,
    // this field is only required when checking collision between bullets and objects
    pub from: Option<Entity>,
}

#[derive(Component)]
pub struct ColliderInfo {
    pub collider_type: ColliderType,
    pub collider: Arc<RwLock<dyn Collider>>,
    pub immune_to: Option<Vec<Entity>>,
    // pub collider: T,
}

#[derive(Event)]
pub enum CollisionEvents {
    TakeDamage(Entity, CollisionDamage, Entity),
}

impl Collider for SphericalCollider {
    fn check_collision_with_point(&self, op: &Arc<RwLock<dyn Collider>>) -> bool {
        if let Ok(center) = op.as_ref().read().ok().unwrap().get_center() {
            (self.center - center).length() <= self.radius
        } else {
            panic!("center should be present");
        }
    }
    fn check_collision_with_sphere(&self, op: &Arc<RwLock<dyn Collider>>) -> bool {
        if let (Ok(center), Ok(radius)) = (
            op.as_ref().read().ok().unwrap().get_center(),
            op.as_ref().read().ok().unwrap().get_radius(),
        ) {
            (self.center - center).length() <= self.radius + radius
        } else {
            panic!("center or radius should be present");
        }
    }
    fn get_radius(&self) -> Result<f32, ()> {
        Ok(self.radius)
    }
    fn get_center(&self) -> Result<Vec3, ()> {
        Ok(self.center)
    }
    fn set_center(&mut self, c: Vec3) -> Result<(), ()> {
        self.center = c;
        Ok(())
    }
}

impl Collider for BoxCollider {
    fn check_collision_with_point(&self, op: &Arc<RwLock<dyn Collider>>) -> bool {
        todo!();
    }
    fn check_collision_with_sphere(&self, op: &Arc<RwLock<dyn Collider>>) -> bool {
        todo!();
    }
    fn get_center(&self) -> Result<Vec3, ()> {
        todo!();
    }
    fn get_radius(&self) -> Result<f32, ()> {
        todo!();
    }
    fn set_center(&mut self, c: Vec3) -> Result<(), ()> {
        // self.center = c;
        Ok(())
    }
}

impl Collider for PointCollider {
    fn check_collision_with_point(&self, op: &Arc<RwLock<dyn Collider>>) -> bool {
        if let Ok(center) = op.as_ref().read().ok().unwrap().get_center() {
            self.center == center
        } else {
            panic!("Center should be present");
        }
    }
    fn check_collision_with_sphere(&self, op: &Arc<RwLock<dyn Collider>>) -> bool {
        if let (Ok(center), Ok(radius)) = (
            op.as_ref().read().ok().unwrap().get_center(),
            op.as_ref().read().ok().unwrap().get_radius(),
        ) {
            (self.center - center).length() <= radius
        } else {
            panic!("Center or radius should be present");
        }
    }
    fn get_center(&self) -> Result<Vec3, ()> {
        Ok(self.center)
    }
    fn get_radius(&self) -> Result<f32, ()> {
        Err(())
    }
    fn set_center(&mut self, c: Vec3) -> Result<(), ()> {
        self.center = c;
        Ok(())
    }
}
#[derive(Component)]
pub struct ColliderMarker;

pub struct ColliderPlugin;
impl Plugin for ColliderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CollisionEvents>()
            .add_systems(Update, detect_collisions.in_set(UpdateSet::InGame));
    }
}

fn detect_collisions(
    query: Query<
        (
            Entity,
            &GlobalTransform,
            &ColliderInfo,
            Option<&CollisionDamage>,
        ),
        With<ColliderMarker>,
    >,
    mut ev_writer: EventWriter<CollisionEvents>,
) {
    for [(e1, g1, c1, cd1), (e2, g2, c2, cd2)] in query.iter_combinations() {
        _ = c1
            .collider
            .as_ref()
            .write()
            .ok()
            .unwrap()
            .set_center(g1.translation());
        _ = c2
            .collider
            .as_ref()
            .write()
            .ok()
            .unwrap()
            .set_center(g2.translation());
        match &c2.collider_type {
            ColliderType::Sphere => {
                if c1
                    .collider
                    .as_ref()
                    .read()
                    .ok()
                    .unwrap()
                    .check_collision_with_sphere(&c2.collider)
                {
                    if let Some(cd) = cd1 {
                        ev_writer.send(CollisionEvents::TakeDamage(e2, cd.clone(), e1));
                    }

                    if let Some(cd) = cd2 {
                        ev_writer.send(CollisionEvents::TakeDamage(e1, cd.clone(), e2));
                    }
                }
            }
            ColliderType::Point => {
                // since bullets have range after which they despawn this should be scheduled properly
                if c1
                    .collider
                    .as_ref()
                    .read()
                    .ok()
                    .unwrap()
                    .check_collision_with_point(&c2.collider)
                {
                    if let Some(cd) = cd1 {
                        ev_writer.send(CollisionEvents::TakeDamage(e2, cd.clone(), e1));
                    }

                    if let Some(cd) = cd2 {
                        ev_writer.send(CollisionEvents::TakeDamage(e1, cd.clone(), e2));
                    }
                }
            }
            _ => (),
        }
    }
}

use super::explosion::{Explosion, ExplosionEvent};
use super::spaceship::Health;
pub fn collision_response<T: Component>(
    mut query: Query<
        (
            Entity,
            &Transform,
            &ColliderInfo,
            &mut Health,
            Option<&ExplosibleObjectMarker>,
        ),
        With<T>,
    >,
    mut ev_reader: EventReader<CollisionEvents>,
    mut ev_explode: EventWriter<ExplosionEvent>,
) {
    // let health = query.single();
    for msg in ev_reader.read() {
        match msg {
            CollisionEvents::TakeDamage(e, d, e_with) => {
                // info!("collision event received");
                if let Ok((ent, trans, collider, mut health, explosible)) = query.get_mut(e.clone())
                {
                    // info!("heatlth {}", health.0);
                    if d.from.is_some_and(|e| {
                        e != ent
                            && || -> bool {
                                if collider.immune_to.is_some() {
                                    for x in collider.immune_to.clone().unwrap() {
                                        if x == e {
                                            return false;
                                        }
                                    }
                                }
                                return true;
                            }()
                    }) || d.from.is_none()
                    {
                        if health.0 <= 0. {
                            continue;
                        }
                        info!("{:?} {:?}", d.from.unwrap(), ent);
                        health.0 -= d.damage;
                        if health.0 <= 0. && explosible.is_some() {
                            // todo: condition if it is explosible
                            ev_explode.send(ExplosionEvent {
                                transform: trans.clone(),
                                explosion: Explosion {
                                    half_extent: 0.15,
                                    ..default()
                                },
                            });
                        }
                        info!("Health {}", health.0);
                    }
                }
            }
        }
    }
}

// do a study on conecpts used here
// thread safe trait
// Arc<dyn trait + Send + Sync>

// this was depression
