use rand::prelude::*;
use bevy::prelude::*;
use bevy_prototype_debug_lines::*;
use bevy_rapier3d::prelude::*;
use iyes_loopless::prelude::*;

use crate::explosion::*;
use crate::menu::{is_play_offline, is_play_online};
use crate::network::{NetPlugin, NewNetHandles};
use crate::player::*;
use crate::shot::*;
use crate::tank::*;

use crate::cleanup::cleanup_system;

use crate::camera::{CameraPlugin, CameraState, CameraTarget};
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

/// This plugin handles Game related stuff like movement
/// Game logic is only active during the State `AppState::Playing`
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DebugLinesPlugin::with_depth_test(true))
            .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
            //            .add_plugin(RapierPhysicsPlugin::<&CustomFilterTag>::default())
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_plugin(CameraPlugin::<TempForCamera>::default())
            .add_plugin(TerrainPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(TankPlugin)
            .add_plugin(ShotPlugin)
            .add_plugin(ExplosionPlugin)
            .add_plugin(NetPlugin)
            .add_system_set(SystemSet::on_enter(AppState::PreparePlaying).with_system(setup))
            .add_system_set(
                SystemSet::on_update(AppState::PreparePlaying)
                    .with_system(on_terrain_complete.run_if(is_terrain_complete))
                    .with_system(setup_dynamic.run_if(is_create_physics)),
            )

            .add_system_set(SystemSet::on_enter(AppState::Playing).with_system(start_game))
            .add_system_set(
                SystemSet::on_update(AppState::Playing)
                    .with_system(obr_new_handles.run_if(is_play_online) )
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
    //   rapier_context: Res<RapierContext>,
    //   model_assets: Res<ModelAssets>,
    //  mut meshes: ResMut<Assets<Mesh>>,
    //    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /*
         if get_pos_on_ground(Vec3::new(0., 0., 0.), &rapier_context).is_none() {
            return;
        }

        println!("Game setup_dynamic start");

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
                        mesh: meshes.add(Mesh::from(Cube::new(half_size*2.))),
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
    pos: Vec3,
    angle: f32,
    rapier_context: &RapierContext,
    data: &mut NewTanksData,
) {
    println!("Game spawn_player start");

    /*    let material_handle = materials.add(StandardMaterial {
            base_color: PLAYER_COLORS[handle].clone(),
            ..default()
        });
    */
    if let Some(pos) = get_pos_on_ground(pos, rapier_context) {
        data.vector.push(NewTank {
            handle,
            pos: Vec3::new(pos.x, pos.y + 1., pos.z),
            angle,
        });
    }

    println!("Game spawn_player complete, handle:{}", handle);
}

fn display_events(mut collision_events: EventReader<bevy_rapier3d::prelude::CollisionEvent>) {
    for collision_event in collision_events.iter() {
        println!("Received collision event: {:?}", collision_event);
    }
}

pub fn start_game(
    rapier_context: Res<RapierContext>,
    local_handles: Res<LocalHandles>,
    mut tank_data: ResMut<NewTanksData>,
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

    let start_pos = Vec3::new(rng.gen_range(-6.0..6.0), 0., rng.gen_range(-6.0..6.0));

    if let Some(pos) = get_pos_on_ground(start_pos, &rapier_context) {
        tank_data.vector.push(NewTank {
            handle,
            pos: Vec3::new(pos.x, pos.y + 1., pos.z),
            angle: rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI),
        });
    }

    println!("Game start_game complete, handle:{}", handle);
}

fn obr_new_handles(
    mut new_handles: ResMut<NewNetHandles>,
    rapier_context: Res<RapierContext>,
    mut tank_data: ResMut<NewTanksData>,
) {
    for (peer_id, (handle, data)) in &new_handles.handles {
        println!(
            "obr_new_handles spawn new player: peer_id {:?}, handle {:?}",
            peer_id, handle
        );

        spawn_player(
            *handle,
            Vec3::new(data.pos.x, 0., data.pos.y),
            data.dir,
            &rapier_context,
            &mut tank_data,
        );
    }

    new_handles.handles.clear();
}

