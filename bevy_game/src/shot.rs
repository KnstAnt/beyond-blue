use crate::explosion::add_explosion;
use crate::game::GameMessage;
use crate::game::InMesVec;
use crate::game::OutGameMessages;
use crate::menu::is_play_offline;
use crate::menu::is_play_online;
use crate::network::PingList;
use crate::player::*;
use crate::terrain::get_pos_on_ground;
use crate::AppState;
use bevy::prelude::shape::UVSphere;
use bevy::prelude::*;
use bevy_rapier3d::plugin::*;
use bevy_rapier3d::prelude::*;
use iyes_loopless::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::explosion::NetData as ExplosionData;

#[repr(C)]
#[derive(Serialize, Deserialize, Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct ShotData {
    pub is_shot: bool,
    pub pos: Vec3,
    pub vel: Vec3,
    pub radius: f32,
}

#[derive(Component)]
pub struct ShotExplosionData {
    timer: Timer,
    pub explosion_radius: f32,
    pub explosion_force: f32,
}

impl ShotExplosionData {
    pub fn new(live_max_time: f32, explosion_force: f32) -> Self {
        Self {
            timer: Timer::new(Duration::from_secs_f32(live_max_time), false),
            explosion_radius: explosion_force.powf(0.333333),
            explosion_force,
        }
    }
}

pub struct ShotPlugin;

impl Plugin for ShotPlugin {
    fn build(&self, app: &mut App) {
        /* *   let before_system_set = SystemSet::on_update(AppState::Playing)
                .with_system(remove_shots)
                .with_system(process_in_shot.run_if(is_play_online))
                .with_system(handle_explosion_events)
                ;
        */
        let update_system_set = SystemSet::on_update(AppState::Playing)
            .with_system(create_shot_from_net.run_if(is_play_online))
            .with_system(process_shots_game_net.run_if(is_play_online))
            .with_system(handle_explosion_events_net.run_if(is_play_online))            
            .with_system(process_shots_game_local.run_if(is_play_offline))
            .with_system(handle_explosion_events_local.run_if(is_play_offline));
        //   let after_system_set = SystemSet::on_update(AppState::Playing)
        //    .with_system(print_after_system)
        //      .with_system(handle_explosion_events)    ;

        app
 //       .add_system_set_to_stage(CoreStage::PreUpdate, State::<AppState>::get_driver())
 //       .add_system_set_to_stage(CoreStage::PostUpdate, State::<AppState>::get_driver())
        .add_system_set_to_stage(CoreStage::Update, update_system_set)
   //     .add_system_set_to_stage(CoreStage::PostUpdate, after_system_set)
        ;
    }
}

pub fn create_shot_from_net(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ping: Res<PingList>,
    mut input: ResMut<InMesVec<ShotData>>,
) {
    for (player, data) in input.data.iter_mut() {
        if !data.is_shot {
            continue;
        }

        data.is_shot = false;

        //TODO add compensation of ping: delta pos:shot_action.vel*ping.get_time(player.handle)
        //apply gravity to velosity
        let shot_pos = data.pos + data.vel * ping.get_time(*player);
        let shot_vel = data.vel - Vec3::Y * 9.8 * ping.get_time(*player);

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(UVSphere {
                    radius: data.radius,
                    sectors: 8,
                    stacks: 8,
                })),

                material: materials.add(StandardMaterial {
                    base_color: Color::RED,
                    emissive: Color::rgba_linear(100.0, 0.0, 0.0, 0.0),
                    ..default()
                }),

                transform: Transform::from_translation(shot_pos),

                ..default()
            })
//            .insert(ShotExplosionData::new(data.shot_live_max_time, data.explosion_force))
            .insert(PlayerData{handle: player.clone()})
            .insert(bevy_rapier3d::prelude::RigidBody::Dynamic)
            .insert(bevy_rapier3d::prelude::Collider::ball(data.radius))
            //                .insert_bundle(collider)
            .insert(bevy_rapier3d::prelude::ActiveEvents::COLLISION_EVENTS)
            .insert(Restitution::coefficient(0.01))
            .insert(Friction::coefficient(1.0))
            .insert(ColliderMassProperties::Density(5.))
            .insert(Ccd::enabled())
            .insert(Velocity {
                linvel: shot_vel,
                angvel: Vec3::ZERO,
            })
            .insert(CollisionGroups::new(0b0100, 0b0011))
            .insert(SolverGroups::new(0b0100, 0b0011))
            .insert(bevy_rapier3d::prelude::ActiveHooks::FILTER_CONTACT_PAIRS)
//          .insert(CustomFilterTag::GroupShot)
            ;
    }
}

fn handle_explosion_events_net(
    mut commands: Commands,
    //    mut meshes: ResMut<Assets<Mesh>>,
    //    mut materials: ResMut<Assets<StandardMaterial>>,
    local_handles: Res<LocalHandles>,
    mut events: EventReader<bevy_rapier3d::prelude::CollisionEvent>,
    query: Query<(&GlobalTransform, Entity, &ShotExplosionData, &PlayerData)>,
    mut output: ResMut<OutGameMessages<GameMessage>>,
) {
    for event in events.iter() {
        if let bevy_rapier3d::prelude::CollisionEvent::Started(e1, e2, _f) = event {
            for (global_transform, entity, shot_data, player) in query.iter() {
                /*           match event {
                                bevy_rapier3d::prelude::CollisionEvent::Started(e1, e2, f)
                                | bevy_rapier3d::prelude::CollisionEvent::Stopped(e1, e2, f)
                                => {
                        //  ..          if *f ^ CollisionEventFlags::SENSOR {
                        //                continue;
                        //            }
                */
                if e1 == &entity || e2 == &entity {
                    //                println!("handle_explosion_events  translation: {:?}", global_transform.translation());
                    if *local_handles.handles.first().unwrap() == player.handle {
                        let pos = Vec3::new(
                            global_transform.translation().x,
                            global_transform.translation().y + 0.1,
                            global_transform.translation().z,
                        );

                        log::info!("Shot handle_explosion_events pos:{:?}", pos);
                        add_explosion(
                            &mut commands,
                            pos,
                            shot_data.explosion_force,
                            shot_data.explosion_radius,
                            player.handle,
                        );

                        output.data.push(GameMessage::from(ExplosionData {
                            pos,
                            force: shot_data.explosion_force,
                            radius: shot_data.explosion_radius,
                        }));
                    }

                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

fn handle_explosion_events_local(
    mut commands: Commands,
    //    mut meshes: ResMut<Assets<Mesh>>,
    //    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventReader<bevy_rapier3d::prelude::CollisionEvent>,
    query: Query<(&GlobalTransform, Entity, &ShotExplosionData, &PlayerData)>,
) {
    for event in events.iter() {
        if let bevy_rapier3d::prelude::CollisionEvent::Started(e1, e2, _f) = event {
            for (global_transform, entity, shot_data, player) in query.iter() {
                /*           match event {
                                bevy_rapier3d::prelude::CollisionEvent::Started(e1, e2, f)
                                | bevy_rapier3d::prelude::CollisionEvent::Stopped(e1, e2, f)
                                => {
                        //  ..          if *f ^ CollisionEventFlags::SENSOR {
                        //                continue;
                        //            }
                */
                if e1 == &entity || e2 == &entity {
                    //                println!("handle_explosion_events  translation: {:?}", global_transform.translation());
                    let pos = Vec3::new(
                        global_transform.translation().x,
                        global_transform.translation().y + 0.1,
                        global_transform.translation().z,
                    );

                    log::info!("Shot handle_explosion_events pos:{:?}", pos);
                    add_explosion(
                        &mut commands,
                        pos,
                        shot_data.explosion_force,
                        shot_data.explosion_radius,
                        player.handle,
                    );

                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

fn process_shots_game_local(
    mut commands: Commands,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    mut query: Query<(
        Entity,
        &GlobalTransform,
        &mut ShotExplosionData,
        &bevy_rapier3d::prelude::Collider,
        &PlayerData,
    )>,
) {
    //info!("remove_shots");

    for (entity, global_transform, mut shot_data, collider, player) in query.iter_mut() {
        // timers gotta be ticked, to work
        shot_data.timer.tick(time.delta());

        //     info!("remove_shots tick");

        // if it finished, despawn the bomb
        if shot_data.timer.finished() {
            //                       info!("remove_shots finished");
            commands.entity(entity).despawn_recursive();
            return;
        }


        let half_height = if let Some(ball) = collider.as_ball() {
            ball.radius() as f32
        } else {
            0.1f32
        };


        // bug: test terrain
        if let Some(pos) = get_pos_on_ground(
            Vec3::new(
                global_transform.translation().x,
                half_height,
                global_transform.translation().z,
            ),
            &rapier_context,
        ) {
            if global_transform.translation().y == 0. || pos.y < global_transform.translation().y {
                continue;
            }

 //           pos.y += half_height;
            //            println!("remove_shots get_pos_on_ground pos: {:?}  translation: {:?}", pos, global_transform.translation());
            //    log::info!("Shot remove_shots pos:{:?}", pos);

            add_explosion(
                &mut commands,
                pos,
                shot_data.explosion_force,
                shot_data.explosion_radius,
                player.handle,
            );

            commands.entity(entity).despawn_recursive();

            continue;
        } else if global_transform.translation().y < -10. {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        //    add_explosion(&mut commands, entity, global_transform.translation, &shot_data);
        // commands.entity(entity).despawn_recursive();
    }
}

fn process_shots_game_net(
    mut commands: Commands,
    time: Res<Time>,
    rapier_context: Res<RapierContext>,
    local_handles: Res<LocalHandles>,
    mut query: Query<(
        Entity,
        &GlobalTransform,
        &mut ShotExplosionData,
        &PlayerData,
    )>,
    mut output: ResMut<OutGameMessages<GameMessage>>,
) {
    //info!("remove_shots");

    for (entity, global_transform, mut shot_data, player) in query.iter_mut() {
        // timers gotta be ticked, to work
        shot_data.timer.tick(time.delta());

        //     info!("remove_shots tick");

        // if it finished, despawn the bomb
        if shot_data.timer.finished() {
            //                       info!("remove_shots finished");
            commands.entity(entity).despawn_recursive();
            return;
        }

        // bug: test terrain
        if let Some(mut pos) = get_pos_on_ground(
            Vec3::new(
                global_transform.translation().x,
                0.1,
                global_transform.translation().z,
            ),
            &rapier_context,
        ) {
            if global_transform.translation().y == 0. || pos.y < global_transform.translation().y {
                continue;
            }

            pos.y += 0.1;
            //            println!("remove_shots get_pos_on_ground pos: {:?}  translation: {:?}", pos, global_transform.translation());

            if *local_handles.handles.first().unwrap() == player.handle {
                log::info!("Shot remove_shots pos:{:?}", pos);
                add_explosion(
                    &mut commands,
                    pos,
                    shot_data.explosion_force,
                    shot_data.explosion_radius,
                    player.handle,
                );

                output.data.push(GameMessage::from(ExplosionData {
                    pos,
                    force: shot_data.explosion_force,
                    radius: shot_data.explosion_radius,
                }));
            }

            commands.entity(entity).despawn_recursive();

            continue;
        } else if global_transform.translation().y < -10. {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        //    add_explosion(&mut commands, entity, global_transform.translation, &shot_data);
        // commands.entity(entity).despawn_recursive();
    }
}
