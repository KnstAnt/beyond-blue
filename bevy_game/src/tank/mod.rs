use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use bevy::prelude::Component;
use bevy::prelude::shape::Cube;
use std::{collections::LinkedList, f32::consts::PI};
use bevy_rapier3d::prelude::*;
use iyes_loopless::prelude::*;

use crate::{AppState, player::{LocalHandles}, game::{set_player_control, set_network_control, OutMessageState, MesState}, terrain::get_pos_on_ground, menu::is_play_online};
use crate::loading::ModelAssets;
use crate::player:: PlayerData;


mod body;
mod body_physics;
mod turret;
mod cannon;
mod shot;
pub(crate) mod utils;

use body::*;
use body_physics::*;
use turret::*;
use cannon::*;
use shot::*;

pub use body::Data as TankBodyData;
pub use turret::Data as TurretRotation;
pub use cannon::Data as CannonRotation;
//pub use shot::Data as TankShotData;
/* 
pub use body::Message as TankBodyMessage;
pub use body::OutMessage as OutBodyMessage;
pub use body::State as TankControlBody;
pub use turret::Message as TankTurretMessage;
pub use turret::OutMessage as OutTurretMessage;
pub use turret::State as TankControlTurret;
pub use cannon::Message as TankCannonMessage;
pub use cannon::OutMessage as OutCannonMessage;
pub use cannon::State as TankControlCannon;
pub use cannon::ActionShot as TankActionShot;
pub use body::InMessage as InBodyMessage;
pub use turret::InMessage as InTurretMessage;
pub use cannon::InMessage as InCannonMessage;
*/



#[derive(Component, Debug)]
pub struct TankShotData {
    pub radius: f32,
    pub shot_speed_min: f32,
    pub shot_speed_delta: f32,
    pub shot_live_max_time: f32,
    pub explosion_force: f32,
}

impl TankShotData {
    fn init() -> Self {
        Self {
            radius: 0.02,
            shot_speed_min: 10.,
            shot_speed_delta: 5.,
            shot_live_max_time: 30.,
            explosion_force: 20.,
        }
    }

    pub fn shot_speed(&self, time: f32) -> f32 {   
        self.shot_speed_min + self.shot_speed_delta * time
    }
}

#[derive(Component, Debug, Clone)]
pub struct TankEntityes {
    pub body: Entity,
    pub turret: Entity,
    pub cannon: Entity,
    pub fire_point: Entity,
    pub wheels: LinkedList<Entity>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone, PartialEq)]
pub struct NewTank {
    pub handle: usize,
    pub pos: Vec2,
    pub angle: f32,
}

#[derive(Debug)]
pub struct NewTanksData {
    pub vector: Vec<NewTank>,
}

impl Default for NewTanksData {
    fn default() -> Self {
        Self { vector: vec![] }
    }
}

pub struct TankPlugin;

impl Plugin for TankPlugin {
    fn build(&self, app: &mut App) {
        let before_system_set = SystemSet::on_update(AppState::Playing)
            .with_system(update_body_position_from_net.run_if(is_play_online))
            .with_system(update_turret_rotation_from_net.run_if(is_play_online))
            .with_system(update_cannon_rotation_from_net.run_if(is_play_online))

            .with_system(update_player_body_control.before(update_body_moving))
            .with_system(update_body_moving
                .after(update_player_body_control)
                .before(update_player_turret_rotation)
            )
            
            .with_system(update_player_turret_rotation
                .after(update_body_moving)
                .before(update_player_cannon_rotation)
            )

            .with_system(update_player_cannon_rotation
                .after(update_player_turret_rotation)
                .before(update_cannon_debug_line)
                .before(create_player_cannon_shot)
            )
            .with_system(update_cannon_debug_line.after(update_player_cannon_rotation))
            .with_system(create_player_cannon_shot.after(update_player_cannon_rotation));

 /*     let before_system_set = SystemSet::on_update(AppState::Playing)
            .with_system(update_body_position_from_net.before(update_body_moving))
            .with_system(update_player_body_control.before(update_body_moving))
            .with_system(update_body_moving
                    //                  .label(InputLabel::ApplyInput)
                    //                  .after(InputLabel::PrepInput)
                    .before(update_turret_rotation_from_net)
                    .before(update_player_turret_rotation)
            )
            .with_system(
                update_turret_rotation_from_net
                    .after(update_body_moving)
                    .before(update_cannon_rotation),
            )
            .with_system(
                update_player_turret_rotation
                    .after(update_body_moving)
                    .before(update_cannon_rotation),
            )
            .with_system(
                update_cannon_rotation
                    .after(update_turret_rotation)
                    .before(update_cannon_debug_line)
                    .before(update_cannon_shot),
            )
            .with_system(update_cannon_debug_line.after(update_cannon_rotation))
            .with_system(update_cannon_shot.after(update_cannon_rotation));
*/
        //  let after_system_set = SystemSet::on_update(AppState::Playing)
        //    .with_system(print_after_system)
        //      .with_system(handle_explosion_events);

        //   let update_system_set = SystemSet::on_update(AppState::Playing)
        //    .with_system(print_update_system)
        //     .with_system(display_events)
        //      .with_system(remove_shots)
        //       .with_system(apply_explosion)
        //      .with_system(process_explosion_event)
        //        .with_system(accelerate_system)
        //     ;

        app.init_resource::<NewTanksData>()
            .add_system_set(SystemSet::on_enter(AppState::Loading).with_system(setup))
            .add_system_set_to_stage(CoreStage::PreUpdate, before_system_set)
            .add_system(obr_spawn_tanks.run_if(is_create_tanks))
            .add_system_set(ConditionSet::new().run_if(is_create_tanks).into());
    }
}

fn is_create_tanks(data: Res<NewTanksData>) -> bool {
    //   println!("tank is_create_tanks");
    !data.vector.is_empty()
}

fn setup(
    mut commands: Commands,
    //   asset_server: Res<AssetServer>,
) {
    println!("Tank setup");

    //   let _scenes: Vec<HandleUntyped> = asset_server.load_folder("Tank_1/PARTS").unwrap();
}

pub fn obr_spawn_tanks(
    local_handles: Res<LocalHandles>,
    mut data: ResMut<NewTanksData>,
    query: Query<(&MesState<TankBodyData>, &PlayerData)>,
    mut commands: Commands,
    model_assets: Res<ModelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    rapier_context: Res<RapierContext>,
) {
    'input_cicle: for data in &data.vector {

        for (mut _state, player) in query.iter() {
            if data.handle == player.handle {
                continue 'input_cicle;
            }
        }

        if let Some(pos) = get_pos_on_ground(Vec3::new(data.pos.x, 1., data.pos.y), &rapier_context) {   

/*
        let entityes = create_debug_tank(
            &mut commands,
            data.handle,
            Vec3::new(pos.x, pos.y + 1., pos.z),
            data.angle,
            &mut meshes,
            &mut materials,
        );
*/
        
            let entityes = create_tank(
                &mut commands,
                data.handle,
                Vec3::new(pos.x, pos.y + 1., pos.z),
                data.angle,
                &model_assets,
                &mut meshes,
                &mut materials,
            );

            if *local_handles.handles.first().unwrap() == data.handle {
                set_player_control(&mut commands, &entityes);
            } else {
                set_network_control(&mut commands, &entityes, data.pos, data.angle);
            }
        }
    }

    data.vector.clear();
}

fn create_debug_tank(
    mut commands: &mut Commands,
    player_handle: usize,
    pos: Vec3,
    angle: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> TankEntityes {
    let body_size = Vec3::new(1., 0.45, 1.6);
    let config = VehicleConfig::new(body_size);

    let body = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(pos)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, angle)),
            ..Default::default()
        })
        .id();

    let (body, wheels) = create_body(
        body,
        &mut commands,
        pos,
        angle,
        config,
        CollisionGroups::new(0b0010, 0b1111),
        SolverGroups::new(0b0010, 0b1111),
    );

    /*    commands
            .entity(body)
            .insert(LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED );
    */

    let turret = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.10, 0.0)),
            ..Default::default()
        })
        .id();

    commands.entity(body).add_child(turret);

    let cannon = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0., 0.50, -0.45)),
            ..Default::default()
        })
        .id();

    commands.entity(turret).add_child(cannon);

    let fire_point = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., -0.5)),
            ..Default::default()
        })
        .insert(TankShotData::init())
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(Cube::new(0.1))),
                material: materials.add(Color::RED.into()),
                transform: Transform::from_translation(Vec3::new(0., 0., -0.5)),
                //        rotation:
                ..Default::default()
            });
        })
        .id();

    commands.entity(cannon).add_child(fire_point);

    commands.entity(body).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(turret).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(cannon).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(fire_point).insert(PlayerData {
        handle: player_handle,
    });
    wheels.iter().for_each(|wheel| {
        commands.entity(*wheel).insert(PlayerData {
            handle: player_handle,
        });
    });

    let data = TankEntityes {
        body,
        turret,
        cannon,
        fire_point,
        wheels,
    };

    commands.entity(body).insert(data.clone());

    data
}

fn create_tank(
    mut commands: &mut Commands,
    //    asset_server: &Res<AssetServer>,
    player_handle: usize,
    pos: Vec3,
    angle: f32,
    model_assets: &Res<ModelAssets>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
  //  material: &Handle<bevy::prelude::StandardMaterial>,
) -> TankEntityes {
    let body_size = Vec3::new(1., 0.45, 1.6);
    let config = VehicleConfig::new(body_size);

    let body = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(pos)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, angle)),
                ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(SceneBundle {
            scene: model_assets.tank_body.clone(),
            transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, PI)),
            ..Default::default()
        });
    }).id();

    let (body, wheels) = create_body(
        body,
        &mut commands,
        pos,
        angle,
        config,
        CollisionGroups::new(0b0010, 0b1111),
        SolverGroups::new(0b0010, 0b1111),
    );

    let turret = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.10, 0.0)),
            ..Default::default()
        })
        .with_children(|parent| {
        parent.spawn_bundle(SceneBundle {
            scene: model_assets.tank_turret.clone(),
            transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, PI)),
            //               visibility: visibility.clone(),
            ..Default::default()
        });
    }).id();

    commands.entity(body).add_child(turret);

    let cannon = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.50, -0.45)),
            ..Default::default()
        })
        .with_children(|parent| {
        parent.spawn_bundle(SceneBundle {
            scene: model_assets.tank_cannon.clone(),
            transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, PI)),
            //               visibility: visibility.clone(),
            ..Default::default()
        });
    }).id();

    commands.entity(turret).add_child(cannon);

    let fire_point = commands
        .spawn_bundle(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., -0.5)),
            ..Default::default()
        })
        .insert(TankShotData::init())
  //      .id();
            .with_children(|parent| {
        parent.spawn_bundle(PbrBundle {
            mesh: meshes.add( Mesh::from(Cube::new(0.1)) ),
            material: materials.add(Color::RED.into()),
            //        rotation:
            ..Default::default()
        });
    }).id();
    

    /*        // fire point
            .with_children(|from| {
                from.spawn_bundle(TransformBundle {
                    local: Transform::from_translation(Vec3::new(0., 0., 0.5)),
                    //        rotation:
                    ..Default::default()
                })
                .insert(TankControlCannonShot::default());
            }).id();
    */
    commands.entity(cannon).add_child(fire_point);

    commands.entity(body).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(turret).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(cannon).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(fire_point).insert(PlayerData {
        handle: player_handle,
    });
    wheels.iter().for_each(|wheel| {
        commands.entity(*wheel).insert(PlayerData {
            handle: player_handle,
        });
    });

    let data = TankEntityes {
        body,
        turret,
        cannon,
        fire_point,
        wheels,
    };

    commands.entity(body).insert(data.clone());

    data
}


/*

fn create_tank(
    mut commands: &mut Commands,
    //    asset_server: &Res<AssetServer>,
    player_handle: usize,
    pos: Vec3,
    angle: f32,
    model_assets: &Res<ModelAssets>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
  //  material: &Handle<bevy::prelude::StandardMaterial>,
) -> TankEntityes {
    let body_size = Vec3::new(1., 0.45, 1.6);
    let config = VehicleConfig::new(body_size);

    let body = commands
        .spawn_bundle(SceneBundle {
            scene: model_assets.tank_body.clone(),
            transform: Transform::from_translation(pos)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, angle)),
            ..Default::default()
        })
        .id();

    let (body, wheels) = create_body(
        body,
        &mut commands,
        pos,
        angle,
        config,
        CollisionGroups::new(0b0010, 0b1111),
        SolverGroups::new(0b0010, 0b1111),
    );

    let turret = commands
        .spawn_bundle(SceneBundle {
            scene: model_assets.tank_turret.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.10, 0.0)),
            //               visibility: visibility.clone(),
            ..Default::default()
        })
        .id();

    commands.entity(body).add_child(turret);

    let cannon = commands
        .spawn_bundle(SceneBundle {
            scene: model_assets.tank_cannon.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.50, 0.45)),
            ..Default::default()
        })
        .id();

    commands.entity(turret).add_child(cannon);

    let fire_point = commands
        .spawn_bundle(TransformBundle {
            local: Transform::from_translation(Vec3::new(0., 0., -0.5)),
            global: GlobalTransform::identity(),
        })
        .insert(TankShotData::init())
  //      .id();
            .with_children(|parent| {
        parent.spawn_bundle(PbrBundle {
            mesh: meshes.add( Mesh::from(Cube::new(0.1)) ),
            material: materials.add(Color::RED.into()),
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
            //        rotation:
            ..Default::default()
        });
    }).id();
    

    /*        // fire point
            .with_children(|from| {
                from.spawn_bundle(TransformBundle {
                    local: Transform::from_translation(Vec3::new(0., 0., 0.5)),
                    //        rotation:
                    ..Default::default()
                })
                .insert(TankControlCannonShot::default());
            }).id();
    */
    commands.entity(cannon).add_child(fire_point);

    commands.entity(body).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(turret).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(cannon).insert(PlayerData {
        handle: player_handle,
    });
    commands.entity(fire_point).insert(PlayerData {
        handle: player_handle,
    });
    wheels.iter().for_each(|wheel| {
        commands.entity(*wheel).insert(PlayerData {
            handle: player_handle,
        });
    });

    let data = TankEntityes {
        body,
        turret,
        cannon,
        fire_point,
        wheels,
    };

    commands.entity(body).insert(data.clone());

    data
}
*/





