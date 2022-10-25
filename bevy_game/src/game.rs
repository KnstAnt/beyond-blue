use bevy::math::Affine3A;
use bevy::prelude::*;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;
use iyes_loopless::prelude::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::collections::HashMap;
use std::fmt::Debug;

use crate::explosion::*;
use crate::menu::{is_play_offline, is_play_online};
use crate::network::NetPlugin;
use crate::player::*;
use crate::shot::*;
use crate::tank::*;
use crate::explosion::NetData as ExplosionData;

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

#[derive(Debug)]
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

pub const MAX_OUT_DELTA_TIME: f32 = 3.;
pub const MIN_OUT_DELTA_TIME: f32 = 0.5;
pub const OUT_ANGLE_EPSILON: f32 = 1.0*std::f32::consts::PI/180.;
pub const ANGLE_EPSILON: f32 = 0.3*std::f32::consts::PI/180.;
pub const SPEED_EPSILON: f32 = 0.3*std::f32::consts::PI/180.;

#[derive(Debug, Default)]
pub struct OutMessageState<T>
//where T: 'static + Serialize + Deserialize + DeserializeOwned + Default + Component + PartialEq,
{
    pub delta_time: f32,
    pub old_data: T,
}

#[derive(Debug, Default)]
pub struct OutGameMessages<T>
//where T: 'static + Serialize + Deserialize + DeserializeOwned + Default + Component + PartialEq,
{
    pub data: Vec<T>,
}

#[derive(Debug, Default)]
pub struct InMesMap<T>
//where T: 'static + Serialize + Deserialize + DeserializeOwned + Default + Component + PartialEq,
{
    pub data: HashMap<PlayerHandle, T>,
}

#[derive(Debug, Default)]
pub struct InMesVec<T>
//where T: 'static + Serialize + Deserialize + DeserializeOwned + Default + Component + PartialEq,
{
    pub data: Vec<(PlayerHandle, T)>,
}

#[derive(Component, Debug, Default, PartialEq)]
pub struct MesState<T: Default + Component> {
    pub data: T,
    pub time: f64,
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

#[derive(Serialize, Deserialize, Component, Debug, Clone, PartialEq)]
pub struct NewTankData {
    pub matrix: Mat4,
}

impl From<Transform> for NewTankData {
    fn from(transform: Transform) -> Self {
        Self{matrix: transform.compute_matrix()}
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
            //          .add_plugin(RapierDebugRenderPlugin::default())
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
                    .with_system(
                        obr_in_raw_message
                        .before(obr_in_mes_map::<TankBodyData>),
                    )
                    .with_system(
 //                       obr_in_mes_map::<TankBodyData>
                        obr_in_mes_tank_body
                        .after(obr_in_raw_message),
                    )
                    .with_system(
                        obr_in_mes_map::<TurretRotation>
                        .after(obr_in_raw_message),
                    )
                    .with_system(
                        obr_in_mes_map::<CannonRotation>
                        .after(obr_in_raw_message),
                    )
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
    /*
       let start_pos_x = 0.0;
       let start_pos_z = 0.0;


           let body_pos = get_pos_on_ground(
               Vec3::new(
                   start_pos_x - 11.,
                   2.,
                   start_pos_z
               ),
               &rapier_context,
           ).unwrap();

           println!("Game setup_dynamic body_pos {}", body_pos);

           let player_tank_control_data = create_tank(
               &mut commands,
               &model_assets,
               body_pos );

           commands.insert_resource(player_tank_control_data);


           let mut rng = rand::thread_rng();
       //    let y: f64 = rng.gen(); // generates a float between 0 and 1

           // Spawn obstacles
           for x in -4..=4 {
               for z in -4..=4 {
                   let size: f32 = rng.gen();
                   let half_size = size/2. + 0.1;
                   commands
                       .spawn_bundle(PbrBundle {
                           mesh: meshes.add(Mesh::from(shape::Cube::new(half_size*2.))),
                           material: materials.add(Color::BLACK.into()),
                           transform: Transform::from_translation(
                               get_pos_on_ground(
                                   Vec3::new(
                                       start_pos_x + x as f32 * 2.0,
                                       half_size,
                                       start_pos_z + z as f32 * 2.0,
                                   ),
                                   &rapier_context,
                               )
                               .unwrap(),
                           ),
                           ..Default::default()
                       })
                       .insert(bevy_rapier3d::prelude::RigidBody::Dynamic)
                       .insert(bevy_rapier3d::prelude::Collider::cuboid(half_size, half_size, half_size))
                       .insert(CollisionGroups::new(0b0010, 0b1111))
                       .insert(SolverGroups::new(0b0010, 0b1111))
                       .insert(Restitution::coefficient(0.7))
                       .insert(ColliderMassProperties::Density(1.0));
                   //                .insert(Transform::from_xyz(x as f32 * 4.0, 0.5, z as f32 * 4.0))
                   //                    .insert(PathObstacle);
               }
           }
    */
    println!("Game setup_dynamic complete");
    playing_scene.playing_state = PlayingState::Complete;
    state.replace(AppState::Playing).unwrap();
    //   commands.insert_resource(NextState(SetupState::SetupComplete));
}

pub fn spawn_player(
    handle: usize,
    pos: Vec2,
    angle: f32,
    rapier_context: &RapierContext,
    data: &mut NewTanksData,
) -> bool {
    println!("Game spawn_player start");

    /*    let material_handle = materials.add(StandardMaterial {
            base_color: PLAYER_COLORS[handle].clone(),
            ..default()
        });
    */
    if let Some(new_pos) = get_pos_on_ground(Vec3::new(pos.x, 1., pos.y), rapier_context) {
        data.vector.push(NewTank {
            handle,
            pos,
            angle,
        });

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

    let start_pos = Vec3::new(rng.gen_range(-10.0..10.0), 0., rng.gen_range(-10.0..10.0));
    let start_angle = rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI);

    if let Some(pos) = get_pos_on_ground(start_pos, &rapier_context) {
        tank_data.vector.push(NewTank {
            handle,
            pos: Vec2::new(pos.x, pos.y),
            angle: start_angle,
        });

        /*
                let start_pos_x = 0.0;
                let start_pos_z = 0.0;

                let mut rng = rand::thread_rng();
            //    let y: f64 = rng.gen(); // generates a float between 0 and 1

                // Spawn obstacles
                for x in -20..=20 {
                    for z in -20..=20 {
                        let size: f32 = rng.gen_range(0.05..0.1);
                        let half_size = size/2.;
                        commands
                            .spawn_bundle(PbrBundle {
                                mesh: meshes.add(Mesh::from(shape::Cube::new(half_size*2.))),
                                material: materials.add(Color::BLACK.into()),
                                transform: Transform::from_translation(
                                    get_pos_on_ground(
                                        Vec3::new(
                                            start_pos_x + x as f32 * 0.4,
                                            half_size,
                                            start_pos_z + z as f32 * 0.4,
                                        ),
                                        &rapier_context,
                                    )
                                    .unwrap(),
                                ),
                                ..Default::default()
                            })
                            .insert(bevy_rapier3d::prelude::RigidBody::Dynamic)
                            .insert(bevy_rapier3d::prelude::Collider::cuboid(half_size, half_size, half_size))
                            .insert(CollisionGroups::new(0b0010, 0b1111))
                            .insert(SolverGroups::new(0b0010, 0b1111))
                            .insert(Restitution::coefficient(0.7))
                            .insert(ColliderMassProperties::Density(1.0));
                        //                .insert(Transform::from_xyz(x as f32 * 4.0, 0.5, z as f32 * 4.0))
                        //                    .insert(PathObstacle);
                    }
                }
        */
    }

    println!("Game start_game complete, handle:{}", handle);
}

/* 
fn obr_new_handles(
    mut handles: ResMut<NetHandles>,
    mut new_handles: ResMut<NewNetHandles>,
    rapier_context: Res<RapierContext>,
    mut tank_data: ResMut<NewTanksData>,
) {
    let mut spawned = vec![];

    for (peer_id, (handle, data)) in &new_handles.handles {
        if let GameMessage::InitData(data) = data {
            println!(
                "obr_new_handles spawn new player: peer_id {:?}, handle {:?}",
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

pub fn obr_in_raw_message(
    mut raw: ResMut<InMesMap<GameMessage>>,
    mut in_body: ResMut<InMesMap<TankBodyData>>,
    mut in_turret: ResMut<InMesMap<TurretRotation>>,
    mut in_cannon: ResMut<InMesMap<CannonRotation>>,
    mut in_shot: ResMut<InMesVec<ShotData>>,
    mut in_explosion: ResMut<InMesVec<ExplosionData>>,
    player_tank_data: Query<&Transform, With<ControlMove>>,
    query_tank_data: Query<&PlayerData, With<MesState<TankBodyData>>>,
    mut spawn_tank_data: ResMut<NewTanksData>,
    mut output: ResMut<OutGameMessages<GameMessage>>,

    //  from_server: Res<Arc<Mutex<mpsc::Receiver<NetEvent>>>>,
    //  to_server: ResMut<mpsc::Sender<NetMessage>>,
    //   time: Res<Time>,
) {
    //    log::info!("net handle_conn_events start");    
    'raw_data: for (player, raw_mes) in raw.data.iter() {
        if GameMessage::DataRequest == *raw_mes {            
            if player_tank_data.is_empty() {
                log::info!("obr_in_raw_message DataRequest: no player tank data!");
                return;
            }            

            log::info!("obr_in_raw_message DataRequest send tank data");
            
            let transform = player_tank_data.single();
            output.data.push(GameMessage::InitData(NewTankData::from(*transform)));
        } else if let GameMessage::InitData(data) = raw_mes {
 //           println!( "obr_in_raw_message InitData player:{:?}  pos:{:?}  angle:{:?}", player, data.pos, data.angle);

            for exist_player in &query_tank_data {
                if exist_player.handle == *player { // tank for player is already spawned
                    continue 'raw_data;
                }
            }

            let transform = Transform::from_matrix(data.matrix);

            spawn_tank_data.vector.push(NewTank {
                handle: *player,
                pos: Vec2{x: transform.translation.x, y: transform.translation.z},
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

pub fn obr_in_mes_map<T>(
    time: Res<Time>,
    mut input: ResMut<InMesMap<T>>,
    mut query: Query<(&mut MesState<T>, &PlayerData)>,
//    mut output: ResMut<OutGameMessages<GameMessage>>,
) where T: 'static + Serialize + DeserializeOwned + Default + Debug + Component + PartialEq + Copy {
    for (mut state, player) in query.iter_mut() {
        if let Some(data) = input.data.get_mut(&player.handle) {
            state.data = *data;
            state.time = time.seconds_since_startup();
 //           log::info!("obr_in_mes_map data:{:?}", data);
        }
    }

    input.data.clear();
}


//TODO send to player request for the init data
pub fn obr_in_mes_tank_body(
    time: Res<Time>,
    mut input: ResMut<InMesMap<TankBodyData>>,
    mut query: Query<(&mut MesState<TankBodyData>, &PlayerData)>,
//    mut output: ResMut<OutGameMessages<GameMessage>>,
    mut spawn_tank_data: ResMut<NewTanksData>,
) {
    for (mut state, player) in query.iter_mut() {
        if let Some(data) = input.data.get_mut(&player.handle) {
            state.data = *data;
            state.time = time.seconds_since_startup();
//            log::info!("obr_in_mes_tank_body data:{:?}", data);
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
pub fn set_player_control(
    commands: &mut Commands, 
    entityes: &TankEntityes
) {
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
            data: TankBodyData{
                movement: Vec2::ZERO,
                pos,
                angle,
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

