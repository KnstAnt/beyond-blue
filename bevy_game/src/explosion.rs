use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::prelude::{ColliderBuilder, SharedShape};
use iyes_loopless::prelude::*;

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::AppState;

use crate::game::{InMesVec, COLLISION_ENVIRONMENT, COLLISION_TRIGGER, COLLISION_UNIT};
use crate::menu::is_play_online;
use crate::player::PlayerData;

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Default, Clone, Copy, PartialEq)]
pub struct NetData {
    pub pos: Vec3,
    pub force: f32,
    pub radius: f32,
}

const LIVE_TIME: f32 = 1.;
#[derive(Component)]
struct Data {
    time: f32,
    force: f32,
    flag: bool,
}

impl Data {
    pub fn new(force: f32) -> Self {
        Self {
            time: LIVE_TIME,
            force,
            flag: true,
        }
    }
}

#[derive(Component)]
struct Marker {
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
    position: Vec3,
    force: f32,
    radius: f32,
    player: usize,
    rapier_context: &RapierContext,
) {
    //    info!("add_explosion start");

    //    info!("add_explosion process");

    log::info!("add_explosion pos: {:?}", position);

    let filter = bevy_rapier3d::prelude::QueryFilter::from(
        InteractionGroups::new(
            unsafe { bevy_rapier3d::rapier::geometry::Group::from_bits_unchecked(COLLISION_TRIGGER) },
            unsafe { bevy_rapier3d::rapier::geometry::Group::from_bits_unchecked(COLLISION_UNIT + COLLISION_ENVIRONMENT) },
        )
    );

    rapier_context.intersections_with_shape(
        position,
        Quat::IDENTITY,
        &Collider::compound(vec![(Vec3::ZERO, Quat::IDENTITY, Collider::ball(radius))]),
        filter,
        |entity| {
            commands.entity(entity).insert(Marker {
                force,
                position,
            });
            true
        }
    );

    commands
        .spawn_bundle(PointLightBundle {
            //                transform: Transform::from_xyz(5.0, 8.0, 2.0),
            point_light: PointLight {
                intensity: 3000., // lumens - roughly a 100W non-halogen incandescent bulb
                color: Color::rgb(0.8, 0.6, 0.6),
                shadows_enabled: true,
                ..default()
            },

            transform: Transform::from_translation(position),

            ..default()
        })
        .insert(Data::new(force))
        .insert(PlayerData { handle: player })
//        .insert(bevy_rapier3d::prelude::Collider::ball(radius))
 /*       .insert(bevy_rapier3d::prelude::Collider::from(Collider::compound(vec![(pos, Quat::IDENTITY, Collider::ball(radius))])))
        .insert(bevy_rapier3d::geometry::Sensor)
        .insert(bevy_rapier3d::prelude::ActiveEvents::COLLISION_EVENTS)
        .insert(CollisionGroups::new(
            unsafe { Group::from_bits_unchecked(COLLISION_TRIGGER)},
            unsafe { Group::from_bits_unchecked( COLLISION_UNIT + COLLISION_ENVIRONMENT)},
        ))
        .insert(SolverGroups::new(
            unsafe { Group::from_bits_unchecked(COLLISION_TRIGGER)}, 
                unsafe { Group::from_bits_unchecked(COLLISION_UNIT + COLLISION_ENVIRONMENT)},
        ))
 */       ;
    // TODO  add a lot of ball for emulation explosion

    //    info!("add_explosion finished");
}

fn process_explosion_event(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Data)>,
) {
    // info!("process_explosion_event start");
    for (entity, mut data) in query.iter_mut() {
        data.time -= time.delta_seconds();
        // if it finished, despawn the bomb
        if data.time <= 0. {
            //           info!("remove_shots finished");
            commands.entity(entity).despawn_recursive();
        }
    }
}

/* 
fn process_explosion_event(
    mut commands: Commands,
    time: Res<Time>,
    mut events: EventReader<bevy_rapier3d::prelude::CollisionEvent>,
    mut query: Query<(&GlobalTransform, Entity, &mut Data)>,
) {
    // info!("process_explosion_event start");

    for event in events.iter() {
        if let bevy_rapier3d::prelude::CollisionEvent::Started(e1, e2, f) = event {
            //                info!("process_explosion_event process");

            if let Ok((global_transform, _entity, data)) = query.get_mut(*e1) {
                commands.entity(*e2).insert(Marker {
                    force: data.force,
                    position: global_transform.translation(),
                });
            } else if let Ok((global_transform, _entity, data)) = query.get_mut(*e2) {
                commands.entity(*e1).insert(Marker {
                    force: data.force,
                    position: global_transform.translation(),
                });
            }
        }
    }

    for (_global_transform, entity, mut data) in query.iter_mut() {
        data.time -= time.delta_seconds();
        // if it finished, despawn the bomb
        if data.time <= 0. {
            //           info!("remove_shots finished");
            commands.entity(entity).despawn_recursive();
        }
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
        &mut Marker,
    )>,
) {
    let mut rng = rand::thread_rng();

    for (exploded_entity_transform, exploded_entity, collider, collider_mass_properties, marker) in
        query.iter_mut()
    {
        let explosion_dir = exploded_entity_transform.translation() - marker.position + Vec3::Y*1.;

        let inv_mass = match collider_mass_properties {
            ColliderMassProperties::Density(density) => {
                collider.raw.mass_properties(*density).inv_mass
            }
            ColliderMassProperties::MassProperties(mass_properties) => 1.0 / mass_properties.mass,
            ColliderMassProperties::Mass(mass) => 1.0 / mass,
        };

        let force = marker.force / (0.4 + explosion_dir.length_squared().powf(4.) + inv_mass.powf(0.7));

        //        println!("apply_explosion mass: {:?}  length: {:?}   impulse: {:?}", 1.0/inv_mass, explosion_dir.length(), explosion_force);

        let force = rng.gen_range(force*0.8..force*1.2);
        
        commands.entity(exploded_entity).insert(ExternalImpulse {
            impulse: explosion_dir.normalize() * force,
//            torque_impulse: Vec3::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize() * force,
            ..default()
        });

        commands.entity(exploded_entity).remove::<Marker>();
    }
}

fn process_in_explosion(
    mut commands: Commands, 
    mut input: ResMut<InMesVec<NetData>>,
    rapier_context: Res<RapierContext>,
) {
    for (player, explosion) in &input.data {
        log::info!(
            "Explosion process_in_explosion add_explosion pos:{:?}",
            explosion.pos
        );

        add_explosion(
            &mut commands,
            explosion.pos,
            explosion.force,
            explosion.radius,
            *player,
            &rapier_context,
        );
    }

    input.data.clear();
}
