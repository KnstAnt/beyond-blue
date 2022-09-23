use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
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

#[derive(Component)]
struct Data {
    timer: Timer,
    pub force: f32,
}

impl Data {
    pub fn new(force: f32) -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(0.1), false),
            force,
        }
    }
}

#[derive(Component)]
struct Marker {
    pub force: f32,
    pub position: Vec3,
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
        .with_system(obr_in_explosion.run_if(is_play_online))
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
/*
fn handle_explosion_events(
    mut commands: Commands,
    //    mut meshes: ResMut<Assets<Mesh>>,
    //    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventReader<bevy_rapier3d::prelude::CollisionEvent>,
    query: Query<(&GlobalTransform, Entity, &ShotData)>,
) {
    for event in events.iter() {
        if let bevy_rapier3d::prelude::CollisionEvent::Started(e1, e2, f) = event {
            for (global_transform, entity, shot_data) in query.iter() {
                /*           match event {
                                bevy_rapier3d::prelude::CollisionEvent::Started(e1, e2, f)
                                | bevy_rapier3d::prelude::CollisionEvent::Stopped(e1, e2, f)
                                => {
                        //  ..          if *f ^ CollisionEventFlags::SENSOR {
                        //                continue;
                        //            }
                */
                if e1 == &entity || e2 == &entity {
                    println!("handle_explosion_events  translation: {:?}", global_transform.translation());

                    add_explosion(&mut commands, entity, global_transform.translation(), &shot_data);
                }
            }
        }
    }
}
*/
pub fn add_explosion(
    commands: &mut Commands,
    //    mut meshes: ResMut<Assets<Mesh>>,
    //    mut materials: ResMut<Assets<StandardMaterial>>,
    pos: Vec3,
    forse: f32,
    radius: f32,
    player: usize,
) {
    //    info!("add_explosion start");

    //    info!("add_explosion process");

    log::info!("explosion add_explosion pos: {:?}", pos);

    commands
        .spawn_bundle(PointLightBundle {
            //                transform: Transform::from_xyz(5.0, 8.0, 2.0),
            point_light: PointLight {
                intensity: 3000., // lumens - roughly a 100W non-halogen incandescent bulb
                color: Color::rgb(0.8, 0.6, 0.6),
                shadows_enabled: true,
                ..default()
            },

            transform: Transform::from_translation(pos),

            ..default()
        })
        .insert(Data::new(forse))
        .insert(PlayerData { handle: player })
        .insert(bevy_rapier3d::prelude::Collider::ball(radius))
        .insert(bevy_rapier3d::geometry::Sensor)
        .insert(bevy_rapier3d::prelude::ActiveEvents::COLLISION_EVENTS)
        .insert(CollisionGroups::new(0b1000, 0b0011))
        .insert(SolverGroups::new(0b1000, 0b0011));
    //    info!("add_explosion finished");
}

fn process_explosion_event(
    mut commands: Commands,
    time: Res<Time>,
    mut events: EventReader<bevy_rapier3d::prelude::CollisionEvent>,
    mut query: Query<(&GlobalTransform, Entity, &mut Data)>,
) {
    // info!("process_explosion_event start");

    for (global_transform, entity, mut data) in query.iter_mut() {
        //     info!("remove_shots tick");
        for event in events.iter() {
            if let bevy_rapier3d::prelude::CollisionEvent::Started(e1, e2, f) = event {
                //                info!("process_explosion_event process");

                if e1 == &entity {
                    commands.entity(*e2).insert(Marker {
                        force: data.force,
                        position: global_transform.translation(),
                    });
                } else if e2 == &entity {
                    commands.entity(*e1).insert(Marker {
                        force: data.force,
                        position: global_transform.translation(),
                    });
                }
            }
        }

        data.timer.tick(time.delta());
        // if it finished, despawn the bomb
        if data.timer.finished() {
            //           info!("remove_shots finished");
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn apply_explosion(
    mut commands: Commands,
    mut query: Query<(
        &GlobalTransform,
        Entity,
        &bevy_rapier3d::prelude::Collider,
        &ColliderMassProperties,
        &mut Marker,
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

        commands.entity(exploded_entity).remove::<Marker>();
    }
}

fn obr_in_explosion(mut commands: Commands, mut input: ResMut<InExplosion>) {
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
