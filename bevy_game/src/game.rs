use bevy::prelude::*;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;
use iyes_loopless::prelude::*;
use rand::prelude::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::explosion::NetData as ExplosionData;
use crate::explosion::*;
use crate::menu::{is_play_offline, is_play_online};
use crate::network::NetPlugin;
use crate::player::*;
use crate::shot::*;
use crate::tank::*;

use crate::cleanup::cleanup_system;

use crate::camera::CameraPlugin;
use crate::loading::ModelAssets;
use crate::terrain::*;
use crate::AppState;

pub struct GamePlugin;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum PlayingState {
    CreateTerrain,
    CreateDynamic,
    Complete,
}

#[derive(Debug, Resource)]
pub(crate) struct PlayingScene {
    playing_state: PlayingState,
}

impl Default for PlayingScene {
    fn default() -> Self {
        Self {
            playing_state: PlayingState::CreateTerrain,
        }
    }
}

#[derive(Component)]
pub struct Game;

#[derive(Component)]
pub struct GameClose;

pub struct TempForCamera;

pub const COLLISION_TERRAIN: u32 = 0b000001;
pub const COLLISION_UNIT: u32 = 0b000010;
pub const COLLISION_WHEEL: u32 = 0b00100;
pub const COLLISION_ENVIRONMENT: u32 = 0b001000;
pub const COLLISION_MISSILE: u32 = 0b010000;
pub const COLLISION_TRIGGER: u32 = 0b100000;
pub const COLLISION_ALL: u32 = 0b111111;
pub const EXCLUDE_TERRAIN: u32 = 0b111110;

pub const MAX_OUT_DELTA_TIME: f32 = 1.;
pub const MIN_OUT_DELTA_TIME: f32 = 0.3;
pub const OUT_ANGLE_EPSILON: f32 = 1.0 * std::f32::consts::PI / 180.;
pub const ANGLE_EPSILON: f32 = 1.0 * std::f32::consts::PI / 180.;
pub const ANGLE_SPEED_EPSILON: f32 = 1.0 * std::f32::consts::PI / 180.;
pub const POS_EPSILON: f32 = 0.03;
pub const POS_EPSILON_QRT: f32 = POS_EPSILON * POS_EPSILON;
pub const VEL_EPSILON: f32 = 0.01;
pub const VEL_EPSILON_QRT: f32  = VEL_EPSILON * VEL_EPSILON;


#[derive(Debug, Default, Resource)]
pub struct OutMessageState<T>
//where T: 'static + Serialize + Deserialize + DeserializeOwned + Default + Component + PartialEq,
{
    pub delta_time: f32,
    pub old_data: T,
}

#[derive(Debug, Default, Resource)]
pub struct OutGameMessages<T>
//where T: 'static + Serialize + Deserialize + DeserializeOwned + Default + Component + PartialEq,
{
    pub data: Vec<T>,
}

#[derive(Debug, Default, Resource)]
pub struct InMesMap<T>
//where T: 'static + Serialize + Deserialize + DeserializeOwned + Default + Component + PartialEq,
{
    pub data: HashMap<PlayerHandle, T>,
}

#[derive(Debug, Default, Resource)]
pub struct InMesVec<T>
//where T: 'static + Serialize + Deserialize + DeserializeOwned + Default + Component + PartialEq,
{
    pub data: Vec<(PlayerHandle, T)>,
}

#[derive(Component, Debug, Default, PartialEq, Resource)]
pub struct MesState<T: Default + Component> {
    pub data: T,
    pub time: f32,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Component, Debug, Clone, PartialEq)]
pub enum GameMessage {
    None,
    DataRequest,
    InitData(NewTankData),
    BodyMove(TankBodyData),
    TurretRotate(TurretRotation),
    CannonRotate(CannonRotation),
    Shot(ShotData),
    Explosion(ExplosionData),
}

impl Default for GameMessage {
    fn default() -> Self {
        GameMessage::None
    }
}

#[derive(Serialize, Deserialize, Component, Resource, Debug, Clone, PartialEq)]
pub struct NewTankData {
    pub matrix: Mat4,
}

impl From<Transform> for NewTankData {
    fn from(transform: Transform) -> Self {
        Self {
            matrix: transform.compute_matrix(),
        }
    }
}

impl From<NewTankData> for GameMessage {
    fn from(data: NewTankData) -> Self {
        GameMessage::InitData(data)
    }
}

impl From<TankBodyData> for GameMessage {
    fn from(data: TankBodyData) -> Self {
        GameMessage::BodyMove(data)
    }
}

impl From<TurretRotation> for GameMessage {
    fn from(data: TurretRotation) -> Self {
        GameMessage::TurretRotate(data)
    }
}

impl From<CannonRotation> for GameMessage {
    fn from(data: CannonRotation) -> Self {
        GameMessage::CannonRotate(data)
    }
}

impl From<ShotData> for GameMessage {
    fn from(data: ShotData) -> Self {
        GameMessage::Shot(data)
    }
}

impl From<ExplosionData> for GameMessage {
    fn from(data: ExplosionData) -> Self {
        GameMessage::Explosion(data)
    }
}

/// This plugin handles Game related stuff like movement
/// Game logic is only active during the State `AppState::Playing`
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DebugLinesPlugin::with_depth_test(true))
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            //            .add_plugin(RapierPhysicsPlugin::<&CustomFilterTag>::default())
            //            .add_plugin(RapierDebugRenderPlugin::default())
            .add_plugin(CameraPlugin::<TempForCamera>::default())
            .add_plugin(TerrainPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(TankPlugin)
            .add_plugin(ShotPlugin)
            .add_plugin(ExplosionPlugin)
            .add_plugin(NetPlugin)
            .insert_resource(InMesMap::<GameMessage>::default())
            .insert_resource(InMesMap::<TankBodyData>::default())
            .insert_resource(InMesMap::<TurretRotation>::default())
            .insert_resource(InMesMap::<CannonRotation>::default())
            .insert_resource(InMesVec::<ShotData>::default())
            .insert_resource(InMesVec::<ExplosionData>::default())
            .insert_resource(OutGameMessages::<GameMessage>::default())
            .insert_resource(OutMessageState::<TankBodyData>::default())
            .insert_resource(OutMessageState::<TurretRotation>::default())
            .insert_resource(OutMessageState::<CannonRotation>::default())
            .add_system_set(SystemSet::on_enter(AppState::PreparePlaying).with_system(setup))
            .add_system_set(
                SystemSet::on_update(AppState::PreparePlaying)
                    .with_system(on_terrain_complete.run_if(is_terrain_complete))
                    .with_system(setup_dynamic.run_if(is_create_physics)),
            )
            .add_system_set(SystemSet::on_enter(AppState::Playing).with_system(start_game))
            .add_system_set(
                SystemSet::on_update(AppState::Playing)
                    .with_system(process_in_raw_message.before(process_in_mes_map::<TankBodyData>))
                    .with_system(
                        //                       process_in_mes_map::<TankBodyData>
                        process_in_mes_tank_body.after(process_in_raw_message),
                    )
                    .with_system(process_in_mes_map::<TurretRotation>.after(process_in_raw_message))
                    .with_system(
                        process_in_mes_map::<CannonRotation>.after(process_in_raw_message),
                    ),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_if(is_terrain_complete)
                    .run_if(is_create_physics)
                    .run_if(is_play_offline)
                    .run_if(is_play_online)
                    .into(),
            )
            .add_system_set_to_stage(CoreStage::PreUpdate, State::<AppState>::get_driver())
            .add_system_set_to_stage(CoreStage::PostUpdate, State::<AppState>::get_driver())
            .add_system_set(
                SystemSet::on_exit(AppState::Playing).with_system(cleanup_system::<GameClose>),
            );
    }
}

fn is_terrain_complete(playing_scene: Res<PlayingScene>, terrain_scene: Res<TerrainScene>) -> bool {
    playing_scene.playing_state == PlayingState::CreateTerrain && terrain_scene.is_completed()
}

fn is_create_physics(playing_scene: Res<PlayingScene>) -> bool {
    playing_scene.playing_state == PlayingState::CreateDynamic
}

/*
fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
       commands.spawn_bundle(UiCameraBundle::default());

       let font = asset_server.load("fonts/FiraMono-Medium.ttf");

      commands
           .spawn_bundle(TextBundle {
               style: Style {
                   align_self: AlignSelf::FlexStart,
                   flex_direction: FlexDirection::Column,
                   ..Default::default()
               },
               text: Text {
                   sections: vec![
                       TextSection {
                           value: "Path between shooter and mouse cursor: ".to_string(),
                           style: TextStyle {
                               font: font.clone(),
                               font_size: 30.0,
                               color: Color::WHITE,
                           },
                       },
                       TextSection {
                           value: "Direct!".to_string(),
                           style: TextStyle {
                               font,
                               font_size: 30.0,
                               color: Color::WHITE,
                           },
                       },
                   ],
                   ..Default::default()
               },
               ..Default::default()
           })
           .insert(PathStatus);

}
*/

fn setup(mut commands: Commands, model_assets: Res<ModelAssets>) {
    println!("Game setup");

    commands.insert_resource(PlayingScene::default());

    commands.insert_resource(TerrainScene::new(model_assets.terrain.clone()));

    commands.spawn_bundle(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 18.0, 4.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });
}

fn on_terrain_complete(mut playing_scene: ResMut<PlayingScene>) {
    println!("Game on_terrain_complete");

    playing_scene.playing_state = PlayingState::CreateDynamic;
}

fn setup_dynamic(
    //   mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    mut playing_scene: ResMut<PlayingScene>,
    //    rapier_context: Res<RapierContext>,
    //   model_assets: Res<ModelAssets>,
    //     mut meshes: ResMut<Assets<Mesh>>,
    //       mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("Game setup_dynamic start");

    println!("Game setup_dynamic complete");

    playing_scene.playing_state = PlayingState::Complete;
    state.replace(AppState::Playing).unwrap();
    //   commands.insert_resource(NextState(SetupState::SetupComplete));
}

pub fn spawn_player(
    handle: usize,
    pos: Vec2,
    angle: f32,
    rapier_context: Res<RapierContext>,
    data: &mut NewTanksData,
) -> bool {
    println!("Game spawn_player start");

    /*    let material_handle = materials.add(StandardMaterial {
            base_color: PLAYER_COLORS[handle].clone(),
            ..default()
        });
    */
    if let Some(_new_pos) = get_pos_on_ground(Vec3::new(pos.x, 1., pos.y), &rapier_context) {
        data.vector.push(NewTank { handle, pos, angle });

        println!("Game spawn_player complete, handle:{}", handle);

        return true;
    }

    println!("Game spawn_player fault, handle:{}", handle);
    false
}

fn display_events(mut collision_events: EventReader<bevy_rapier3d::prelude::CollisionEvent>) {
    for collision_event in collision_events.iter() {
        println!("Received collision event: {:?}", collision_event);
    }
}

pub fn start_game(
    mut commands: Commands,
    rapier_context: Res<RapierContext>,
    local_handles: Res<LocalHandles>,
    mut tank_data: ResMut<NewTanksData>,
    //   model_assets: Res<ModelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("Game start_game start");

    let handle: usize = *local_handles.handles.first().unwrap();
    /*
        let material_handle = materials.add(StandardMaterial {
            base_color: PLAYER_COLORS[handle].clone(),
            ..default()
        });
    */
    let mut rng = thread_rng();

    //  let start_pos = Vec3::ZERO;
    //  let start_angle = 0.;
    //.with_translation(Vec3::new(300., 0., -400.))
    let ground_translation = Vec3::new(50., 0., -50.);
    let start_pos = Vec3::new(rng.gen_range(-10.0..10.0), 0., rng.gen_range(-10.0..10.0)) + ground_translation;
    let start_angle = rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI);

    if let Some(pos) = get_pos_on_ground(start_pos, &rapier_context) {
        tank_data.vector.push(NewTank {
            handle,
            pos: Vec2::new(pos.x, pos.z),
            angle: start_angle,
        });

        if true {
            let mut rng = rand::thread_rng();
            //    let y: f64 = rng.gen(); // generates a float between 0 and 1

            // Spawn obstacles
            let delta = 25.;
            let pos_min_x = start_pos.x - delta;
            let pos_max_x = start_pos.x + delta;
            let pos_min_z = start_pos.z - delta;
            let pos_max_z = start_pos.z + delta;

            let qnt = (delta * delta * 1.) as usize;

            for i in 0..qnt {
                let pos_x = rng.gen_range(pos_min_x..pos_max_x);
                let pos_z = rng.gen_range(pos_min_z..pos_max_z);

                let size: f32 = if i < 100 {
                    rng.gen_range(0.25..0.6)
                } else {
                    rng.gen_range(0.07..0.2)
                };

                let half_size = size / 2.;

                let linear_damping = 0.2 / size;
                let angular_damping = 0.03 / size;

                if let Some(pos) =
                    get_pos_on_ground(Vec3::new(pos_x, half_size + 1., pos_z), &rapier_context)
                {
                    commands
                            .spawn_bundle(PbrBundle {
                                mesh: meshes.add(Mesh::from(shape::Cube::new(half_size*2.))),
                                material: materials.add(Color::BLACK.into()),
                                transform: Transform::from_translation(pos),
                                ..Default::default()
                            })
                            .insert(bevy_rapier3d::prelude::RigidBody::Dynamic)
                            .insert(bevy_rapier3d::prelude::Collider::cuboid(half_size, half_size, half_size))
                            .insert(CollisionGroups::new(
                                unsafe { Group::from_bits_unchecked(COLLISION_ENVIRONMENT)},
                                unsafe { Group::from_bits_unchecked(COLLISION_ALL)},
                            ))
                            .insert(SolverGroups::new(
                                unsafe { Group::from_bits_unchecked(COLLISION_ENVIRONMENT)},
                                unsafe { Group::from_bits_unchecked(COLLISION_ALL)},
                            ))
                            .insert(Restitution::coefficient(0.7))
                            .insert(ColliderMassProperties::Density(1.0))
                            .insert(Damping {
                                linear_damping,
                                angular_damping,
                            })
                        //                .insert(Transform::from_xyz(x as f32 * 4.0, 0.5, z as f32 * 4.0))
                        //                    .insert(PathObstacle);
                            ;
                }
            }
        }
    }

    println!("Game start_game complete, handle:{}", handle);
}

/*
fn process_new_handles(
    mut handles: ResMut<NetHandles>,
    mut new_handles: ResMut<NewNetHandles>,
    rapier_context: Res<RapierContext>,
    mut tank_data: ResMut<NewTanksData>,
) {
    let mut spawned = vec![];

    for (peer_id, (handle, data)) in &new_handles.handles {
        if let GameMessage::InitData(data) = data {
            println!(
                "process_new_handles spawn new player: peer_id {:?}, handle {:?}",
                peer_id, handle
            );

            if spawn_player(
                *handle,
                data.pos,
                data.angle,
                &rapier_context,
                &mut tank_data,
            ) {
                handles.handles.insert(peer_id.clone(), handle.clone());
                spawned.push(peer_id.clone());
            }
        }
    }

    spawned.iter().for_each(|v| {
        new_handles.handles.remove(v);
    });
}
*/
pub fn process_in_raw_message(
    mut commands: Commands,
    mut raw: ResMut<InMesMap<GameMessage>>,
    mut in_body: ResMut<InMesMap<TankBodyData>>,
    mut in_turret: ResMut<InMesMap<TurretRotation>>,
    mut in_cannon: ResMut<InMesMap<CannonRotation>>,
    mut in_shot: ResMut<InMesVec<ShotData>>,
    mut in_explosion: ResMut<InMesVec<ExplosionData>>,
    player_tank_body_query: Query<&Transform, With<ControlMove>>,
 //   mut tank_parts_transforms_query: Query<&mut Transform, (With<PlayerData>, Without<TankEntityes>, Without<ControlMove>)>,
    mut tank_body_data_query: Query<(&mut Transform, &PlayerData, &mut MesState<TankBodyData>, &TankEntityes), Without<ControlMove>>,
    mut spawn_tank_data: ResMut<NewTanksData>,
    mut output: ResMut<OutGameMessages<GameMessage>>,
    //  from_server: Res<Arc<Mutex<mpsc::Receiver<NetEvent>>>>,
    //  to_server: ResMut<mpsc::Sender<NetMessage>>,
    //   time: Res<Time>,
) {
    //    log::info!("net handle_conn_events start");
    'raw_data: for (player, raw_mes) in raw.data.iter() {
        if GameMessage::DataRequest == *raw_mes {
            if player_tank_body_query.is_empty() {
                log::info!("process_in_raw_message DataRequest: no player tank data!");
                return;
            }

            log::info!("process_in_raw_message DataRequest send tank data");

            let transform = player_tank_body_query.single();
            output
                .data
                .push(GameMessage::InitData(NewTankData::from(*transform)));
        } else if let GameMessage::InitData(data) = raw_mes {
            //           println!( "process_in_raw_message InitData player:{:?}  pos:{:?}  angle:{:?}", player, data.pos, data.angle);

            for (transform, exist_player, mut mess_state, entityes) in tank_body_data_query.iter_mut() {
                if exist_player.handle == *player { // tank for player is already spawned
                    
                    let new_transform = Transform::from_matrix(data.matrix);

                    let data = crate::tank::TankPlace{
                        angle: transform.rotation.to_euler(EulerRot::YXZ).0,
                        pos: new_transform.translation,
                    };
        
                    commands.entity(entityes.body).insert(data.clone());

                        
    /*                 let old_pos = transform.translation;
                    *transform = Transform::from_matrix(data.matrix);
                    let delta_pos = transform.translation - old_pos;
        
                    for axle in &entityes.axles {
                        if let Ok(mut transform) = tank_parts_transforms_query.get_mut(*axle) {
                            transform.translation = transform.translation + delta_pos;
                        }
                    }
                
                    for wheel in &entityes.wheels {
                        if let Ok(mut transform) = tank_parts_transforms_query.get_mut(*wheel) {
                            transform.translation = transform.translation + delta_pos;
                        }
                    }
*/
                    mess_state.data.movement = Vec2::ZERO;
                    mess_state.data.pos.x = new_transform.translation.x;
                    mess_state.data.pos.y = new_transform.translation.z;
                    mess_state.data.angle = new_transform.rotation.to_euler(EulerRot::YXZ).0;

                    // remove_tank(&mut commands, entityes);   
                    continue 'raw_data;
                }
            }

            // spawn new tank
            let transform = Transform::from_matrix(data.matrix);

            spawn_tank_data.vector.push(NewTank {
                handle: *player,
                pos: Vec2 {
                    x: transform.translation.x,
                    y: transform.translation.z,
                },
                angle: transform.rotation.to_euler(EulerRot::YXZ).0,
            });
        } else if let GameMessage::BodyMove(data) = raw_mes {
            //                   log::info!("Network handle_conn_events TankBodyOutData");
            in_body.data.insert(*player, *data);
        } else if let GameMessage::TurretRotate(data) = raw_mes {
            //                   log::info!("Network handle_conn_events TankTurretOutData");
            in_turret.data.insert(*player, *data);
        } else if let GameMessage::CannonRotate(data) = raw_mes {
            //                   log::info!("Network handle_conn_events TankCannonOutData");
            in_cannon.data.insert(*player, *data);
        } else if let GameMessage::Shot(data) = raw_mes {
            //                   log::info!("Network handle_conn_events TankShotOutData");
            in_shot.data.push((*player, *data));
        } else if let GameMessage::Explosion(data) = raw_mes {
            //                   log::info!("Network handle_conn_events ExplosionData");
            in_explosion.data.push((*player, *data));
        }
    }

    raw.data.clear();
    //   log::info!("net handle_conn_events end");
}

pub fn process_in_mes_map<T>(
    time: Res<Time>,
    mut input: ResMut<InMesMap<T>>,
    mut query: Query<(&mut MesState<T>, &PlayerData)>,
    //    mut output: ResMut<OutGameMessages<GameMessage>>,
) where
    T: 'static + Serialize + DeserializeOwned + Default + Debug + Component + PartialEq + Copy,
{
    for (mut state, player) in query.iter_mut() {
        if let Some(data) = input.data.get_mut(&player.handle) {
            state.data = *data;
            state.time = time.elapsed_seconds();
            //           log::info!("process_in_mes_map data:{:?}", data);
        }
    }

    input.data.clear();
}

//TODO send to player request for the init data
pub fn process_in_mes_tank_body(
    time: Res<Time>,
    mut input: ResMut<InMesMap<TankBodyData>>,
    mut query: Query<(&mut MesState<TankBodyData>, &PlayerData)>,
    //    mut output: ResMut<OutGameMessages<GameMessage>>,
    mut spawn_tank_data: ResMut<NewTanksData>,
) {
    for (mut state, player) in query.iter_mut() {
        if let Some(data) = input.data.get_mut(&player.handle) {
            state.data = *data;
            state.time = time.elapsed_seconds();
            //            log::info!("process_in_mes_tank_body data:{:?}", data);
        }
    }

    'input_cicle: for (input_player, data) in input.data.iter() {
        for (mut _state, query_player) in query.iter() {
            if *input_player == query_player.handle {
                continue 'input_cicle;
            }
        }

        //        output.data.push(GameMessage::DataRequest);
        spawn_tank_data.vector.push(NewTank {
            handle: *input_player,
            pos: data.pos,
            angle: data.angle,
        });

        break;
    }

    input.data.clear();
}

/*
fn send_out<T>(
    mut message: ResMut<OutMessageTime<T>>,
    mut output: ResMut<OutGameMessages<GameMessage>>,
) where T: 'static + Serialize + DeserializeOwned + Default + Component + PartialEq {
    if message.is_changed() {
        output.data.push(GameMessage::from(message.data));
    //    log::info!("Network send_out_explosion {:?}", res);
    }
}
*/
pub fn set_player_control(commands: &mut Commands, entityes: &TankEntityes) {
    commands
        .entity(entityes.body)
        .insert(ControlMove::default());
    commands
        .entity(entityes.turret)
        .insert(ControlTurret::default());
    commands
        .entity(entityes.cannon)
        .insert(ControlCannon::default());
    commands
        .entity(entityes.fire_point)
        .insert(ControlFire::default());
    //  wheels
}

pub fn set_network_control(
    commands: &mut Commands,
    entityes: &TankEntityes,
    pos: Vec2,
    angle: f32,
) {
    commands
        .entity(entityes.body)
        .insert(MesState::<TankBodyData> {
            data: TankBodyData {
                movement: Vec2::ZERO,
                delta_time_linear: 0,
                delta_time_angular: 0,
                pos,
                angle,
                linvel: Vec2::ZERO,
                angvel: 0.,
            },
            time: 0.,
        });
    commands
        .entity(entityes.turret)
        .insert(MesState::<TurretRotation>::default());
    commands
        .entity(entityes.cannon)
        .insert(MesState::<CannonRotation>::default());
    commands
        .entity(entityes.fire_point)
        .insert(MesState::<ShotData>::default());
}
