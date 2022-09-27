use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::{RigidBodyType, ImpulseJointSet, JointAxesMask, SharedShape, ColliderBuilder};
use bevy_rapier3d::rapier::na::{Translation3, UnitQuaternion, Vector3, Isometry3};

use iyes_loopless::prelude::*;
use serde::{Deserialize, Serialize};

use crate::AppState;

use crate::menu::is_play_online;
use crate::player::{PlayerData, PlayerHandle};

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ExplosionNetData {
    pub pos: Vec3,
    pub force: f32,
    pub radius: f32,
}
#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct OutExplosion {
    pub data: Vec<ExplosionNetData>,
}
pub struct InExplosion {
    pub data: HashMap<PlayerHandle, ExplosionNetData>,
}

const LIVE_TIME: f32 = 1.;
#[derive(Component)]
struct Data {
    time: f32,
    force: f32,
    radius: f32,
    flag: bool,
}

impl Data {    pub fn new(force: f32, radius: f32) -> Self {
        Self {
            time: 0.,
            force,
            radius,
            flag: true,
        }
    }
}


#[derive(Component)]
struct ForceMarker {
    force: f32,
    position: Vec3,
}

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        //       let before_system_set = SystemSet::on_update(AppState::Playing)
        //      .with_system(print_before_system);

        //   let after_system_set = SystemSet::on_update(AppState::Playing)
        //    .with_system(print_after_system)
        //       .with_system(handle_explosion_events);

        let update_system_set = SystemSet::on_update(AppState::Playing)
        //    .with_system(print_update_system)
        //     .with_system(display_events)
  //      .with_system(remove_shots)
        .with_system(apply_explosion)
        .with_system(process_explosion_event)
        .with_system(process_in_explosion.run_if(is_play_online))
//        .with_system(accelerate_system)
        ;

        app
        .insert_resource(OutExplosion::default())
        .insert_resource(InExplosion{data: HashMap::new()})
//        .add_system_set_to_stage(CoreStage::PreUpdate, State::<AppState>::get_driver())
//        .add_system_set_to_stage(CoreStage::PostUpdate, State::<AppState>::get_driver())
//        .add_system_set_to_stage(CoreStage::PreUpdate, before_system_set)
        .add_system_set_to_stage(CoreStage::Update, update_system_set)
  //      .add_system_set_to_stage(CoreStage::PostUpdate, after_system_set)
        ;
    }
}

pub fn add_explosion(
    commands: &mut Commands,
    //    mut meshes: ResMut<Assets<Mesh>>,
    //    mut materials: ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
    forse: f32,
    radius: f32,
    player: usize,
) {
//    log::info!("explosion add_explosion pos: {:?}", pos);

    commands
        .spawn_bundle(PointLightBundle {

            point_light: PointLight {
                intensity: 3000., // lumens - roughly a 100W non-halogen incandescent bulb
                color: Color::rgb(0.8, 0.6, 0.6),
                shadows_enabled: true,
                ..default()
            },

            transform: Transform::from_translation(pos),

            ..default()
        })
        .insert(Data::new(forse, radius))
        .insert(PlayerData { handle: player });

    //    info!("add_explosion finished");
}

/* 
pub fn add_explosion(
    commands: &mut Commands,
    //    mut meshes: ResMut<Assets<Mesh>>,
    //    mut materials: ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
    forse: f32,
    radius: f32,
    player: usize,
) {
//    log::info!("explosion add_explosion pos: {:?}", pos);

    let tra = pos;
    let rot = Quat::default();
    let collider = Collider::compound(vec![(tra, rot, Collider::ball(radius))]);

    commands
        .spawn_bundle(PointLightBundle {

            point_light: PointLight {
                intensity: 3000., // lumens - roughly a 100W non-halogen incandescent bulb
                color: Color::rgb(0.8, 0.6, 0.6),
                shadows_enabled: true,
                ..default()
            },

            transform: Transform::from_translation(pos),

            ..default()
        })
        .insert(Data::new(forse, radius))
        .insert(PlayerData { handle: player })
        .insert(collider)
        .insert(bevy_rapier3d::geometry::Sensor)
        .insert(bevy_rapier3d::prelude::ActiveEvents::COLLISION_EVENTS)
        .insert(CollisionGroups::new(0b1000, 0b0011))
        .insert(SolverGroups::new(0b1000, 0b0011)) 
        ;

        // TODO  add a lot of ball for emulation explosion

    //    info!("add_explosion finished");
}
*/
fn process_explosion_event(
    mut commands: Commands,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut query: Query<(&GlobalTransform, Entity, &mut Data)>,
) {
    // info!("process_explosion_event start");

    for (global_transform, entity, mut data) in query.iter_mut() {

        data.time += time.delta_seconds();

        if data.time >= LIVE_TIME {
            commands.entity(entity).remove::<Data>();
            commands.entity(entity).despawn_recursive();
            continue;
        }

        if data.flag {
            data.flag = false;

            let filter = QueryFilter {
                flags: bevy_rapier3d::rapier::prelude::QueryFilterFlags::EXCLUDE_SENSORS,
                groups: Some(InteractionGroups::new(0b1000, 0b0011)),
                exclude_collider: None,
                exclude_rigid_body: None,
                predicate: None,
            };

            rapier_context.intersections_with_shape(
                global_transform.translation(),
                Quat::IDENTITY,
                &Collider::ball(data.radius),
                filter,
                |entity| {
            // TODO  add a lot of ball for emulation explosion
                    commands.entity(entity).insert(ForceMarker {
                        force: data.force,
                        position: global_transform.translation(),
                    });                    
                    true
                },
            );
        }
    }
}
/* 
fn process_explosion_event(
    mut commands: Commands,
 //   time: Res<Time>,
    mut events: EventReader<bevy_rapier3d::prelude::CollisionEvent>,
    mut query: Query<(&GlobalTransform, Entity,/*  &ColiderMarker, */&Data)>,
) {
    // info!("process_explosion_event start");

    for (global_transform, entity, data) in query.iter_mut() {
        // info!("process_explosion_event start");

        //     info!("remove_shots tick");
        for event in events.iter() {
            if let bevy_rapier3d::prelude::CollisionEvent::Started(e1, e2, f) = event {
                //                info!("process_explosion_event process");

                if e1 == &entity {
                    commands.entity(*e2).insert(ForceMarker {
                        force: data.force,
                        position: global_transform.translation(),
                    });

                } else if e2 == &entity {
                    commands.entity(*e1).insert(ForceMarker {
                        force: data.force,
                        position: global_transform.translation(),                        
                    });
                }
            }
        }

        commands.entity(entity).despawn_recursive();
    }
}

*/
fn apply_explosion(
    mut commands: Commands,
    mut query: Query<(
        &GlobalTransform,
        Entity,
        &bevy_rapier3d::prelude::Collider,
        &ColliderMassProperties,
        &mut ForceMarker,
    )>,
) {
    for (exploded_entity_transform, exploded_entity, collider, collider_mass_properties, marker) in
        query.iter_mut()
    {
        let explosion_dir = exploded_entity_transform.translation() - marker.position;

        let inv_mass = match collider_mass_properties {
            ColliderMassProperties::Density(density) => {
                collider.raw.mass_properties(*density).inv_mass
            }
            ColliderMassProperties::MassProperties(mass_properties) => 1.0 / mass_properties.mass,
            ColliderMassProperties::Mass(mass) => 1.0 / mass,
        };

        let force = marker.force / (1. + explosion_dir.length_squared() + inv_mass);

        //        println!("apply_explosion mass: {:?}  length: {:?}   impulse: {:?}", 1.0/inv_mass, explosion_dir.length(), explosion_force);

        commands.entity(exploded_entity).insert(ExternalImpulse {
            impulse: explosion_dir.normalize() * force,
            //            torque_impulse: Vec3::X,
            ..default()
        });

        commands.entity(exploded_entity).remove::<ForceMarker>();
    }
}

fn process_in_explosion(mut commands: Commands, mut input: ResMut<InExplosion>) {
    for (player, explosion) in &input.data {
        log::info!("Explosion obr_in_explosion add_explosion pos:{:?}", explosion.pos);
        add_explosion(
            &mut commands,
            explosion.pos,
            explosion.force,
            explosion.radius,
            *player,
        );
    }

    input.data.clear();
}
