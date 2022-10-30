use bevy::prelude::Component;
use bevy::prelude::*;
//use iyes_loopless::prelude::*;

use crate::camera::{CameraState, CameraTarget};
use crate::game::SPEED_EPSILON;
//use crate::matchbox_net::*;
use crate::ballistics::calc_shot_dir;
use crate::input::*;

use crate::tank::{NewTank, NewTanksData, TankShotData};
use crate::AppState;

pub type PlayerHandle = usize;

#[derive(Component, Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct PlayerData {
    pub handle: PlayerHandle,
}

#[derive(Debug)]
pub struct LocalHandles {
    pub handles: Vec<PlayerHandle>,
}

impl Default for LocalHandles {
    fn default() -> Self {
        Self { handles: vec![0] }
    }
}

#[derive(Component, Copy, Clone, PartialEq, Debug, Default)]
pub struct ControlFire {
    pub time: f32,
    pub is_shot: bool,
}

#[derive(Component, Copy, Clone, PartialEq, Debug, Default)]
pub struct ControlMove {
    pub movement: Vec2,
}

#[derive(Component, Copy, Clone, PartialEq, Debug, Default)]
pub struct ControlTurret {
    pub speed: f32,
}

#[derive(Component, Copy, Clone, PartialEq, Debug, Default)]
pub struct ControlCannon {
    pub speed: f32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[repr(u16)]
pub enum Actions {
    Up,
    Down,
    Left,
    Right,
    TurretLeft,
    TurretRight,
    CannonUp,
    CannonDown,
    CannonShot,
    CorrectPos,
}

unsafe impl Send for Actions {}
unsafe impl Sync for Actions {}

impl Default for Actions {
    fn default() -> Self {
        Actions::Up
    }
}

impl TryFrom<u16> for Actions {
    type Error = &'static str;
    fn try_from(code: u16) -> Result<Self, Self::Error> {
        match code {
            0 => Ok(Actions::Up),
            1 => Ok(Actions::Down),
            2 => Ok(Actions::Left),
            3 => Ok(Actions::Right),
            4 => Ok(Actions::TurretLeft),
            5 => Ok(Actions::TurretRight),
            6 => Ok(Actions::CannonUp),
            7 => Ok(Actions::CannonDown),
            8 => Ok(Actions::CannonShot),
            _ => Err("Actions try_from error value!"),
        }
    }
}

impl TryInto<u16> for Actions {
    type Error = &'static str;
    fn try_into(self) -> Result<u16, Self::Error> {
        match self {
            Actions::Up => Ok(0),
            Actions::Down => Ok(1),
            Actions::Left => Ok(2),
            Actions::Right => Ok(3),
            Actions::TurretLeft => Ok(4),
            Actions::TurretRight => Ok(5),
            Actions::CannonUp => Ok(6),
            Actions::CannonDown => Ok(7),
            Actions::CannonShot => Ok(8),
            _ => Err("Actions try_into error value!"),
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        let before_system_set = SystemSet::on_update(AppState::Playing)
            .with_system(
                crate::player::prep_wheel_input
                    .label("player_input")
                    .after("keys_input"),
            )
            .with_system(
                crate::player::prep_turret_input
                    .label("player_input")
                    .after("keys_input"),
            )
            /*          .with_system(
                          crate::player::prep_cannon_input
                              .label("player_input")
                              .after("keys_input"),
                      )
            */
            .with_system(
                crate::player::prep_shot_input
                    .label("player_input")
                    .after("keys_input"),
            );

        app
        .add_plugin(InputPlugin::<Actions>::default())
        .insert_resource(LocalHandles::default())
        .add_system_set(
            SystemSet::on_enter(AppState::Playing)
                .with_system(setup), //  
        )
        .add_system_set(
            SystemSet::on_update(AppState::Playing)
                .with_system(process_correct_pos)//.run_if(is_play_offline))
            )



//        .add_system_set_to_stage(CoreStage::PreUpdate, State::<AppState>::get_driver())
 //       .add_system_set_to_stage(CoreStage::PostUpdate, State::<AppState>::get_driver())
        .add_system_set_to_stage(CoreStage::PreUpdate, before_system_set)
 //       .add_system_set_to_stage(CoreStage::Update, update_system_set)
 //       .add_system_set_to_stage(CoreStage::PostUpdate, after_system_set)
        ;
    }
}

fn setup(mut commands: Commands) {
    println!("Player setup");

    let mut game_control = GameControl::<Actions>::new();

    game_control.add_key_action(Actions::Up, KeyCode::W);
    game_control.add_key_action(Actions::Down, KeyCode::S);
    game_control.add_key_action(Actions::Left, KeyCode::A);
    game_control.add_key_action(Actions::Right, KeyCode::D);
    game_control.add_key_action(Actions::TurretLeft, KeyCode::Left);
    game_control.add_key_action(Actions::TurretRight, KeyCode::Right);
    game_control.add_key_action(Actions::CannonUp, KeyCode::Up);
    game_control.add_key_action(Actions::CannonDown, KeyCode::Down);
    game_control.add_key_action(Actions::CannonShot, KeyCode::Space);
    game_control.add_mouse_action(Actions::CannonShot, MouseButton::Left);

    game_control.add_key_action(Actions::CorrectPos, KeyCode::Delete);

    commands.insert_resource(game_control);
    println!("Player setup complete");
}
pub fn prep_wheel_input(
    //    time: Res<Time>,
    local_handles: Res<LocalHandles>,
    mut query: Query<(&mut ControlMove, &PlayerData)>,
    game_control: Res<GameControl<Actions>>,
) {
    if local_handles.handles.is_empty() {
        return;
    }

    if query.is_empty() {
        return;
    }

    let (mut control, player) = query.single_mut();
    assert!(*local_handles.handles.first().unwrap() == player.handle);

    let mut movement = Vec2::ZERO;

    if let Some(key_state) = game_control.get_key_state(Actions::Up) {
        movement -= if key_state.just_pressed || key_state.pressed {
            Vec2::Y
        } else {
            Vec2::ZERO
        }
    }

    if let Some(key_state) = game_control.get_key_state(Actions::Down) {
        movement += if key_state.just_pressed || key_state.pressed {
            Vec2::Y
        } else {
            Vec2::ZERO
        }
    }

    if let Some(key_state) = game_control.get_key_state(Actions::Left) {
        movement -= if key_state.just_pressed || key_state.pressed {
            Vec2::X
        } else {
            Vec2::ZERO
        }
    }

    if let Some(key_state) = game_control.get_key_state(Actions::Right) {
        movement += if key_state.just_pressed || key_state.pressed {
            Vec2::X
        } else {
            Vec2::ZERO
        }
    }

    if control.movement != movement {
        control.movement = movement;
    }
}

pub fn prep_turret_input(
    //    time: Res<Time>,
    local_handles: Res<LocalHandles>,
    mut turret_query: Query<(&GlobalTransform, &mut ControlTurret, &PlayerData)>,
    mut cannon_query: Query<(&GlobalTransform, &mut ControlCannon, &PlayerData)>,
    mut fire_pos_query: Query<(&GlobalTransform, &TankShotData, &ControlFire)>,
    game_control: Res<GameControl<Actions>>,
    camera_state: ResMut<CameraState>,
) {
    if local_handles.handles.is_empty() {
        return;
    }

    if turret_query.is_empty() {
        return;
    }

    if cannon_query.is_empty() {
        return;
    }

    if fire_pos_query.is_empty() {
        return;
    }

    let (turret_global_transform, mut turret_control, player) = turret_query.single_mut();
    assert!(*local_handles.handles.first().unwrap() == player.handle);
    let mut turret_rotation: f32 = 0.;

    let (cannon_global_transform, mut cannon_control, player) = cannon_query.single_mut();
    assert!(*local_handles.handles.first().unwrap() == player.handle);
    let mut cannon_rotation = 0.;

    if let Some(key_state) = game_control.get_key_state(Actions::TurretLeft) {
        turret_rotation += if key_state.just_pressed || key_state.pressed {
            1.
        } else {
            0.
        }
    }

    if let Some(key_state) = game_control.get_key_state(Actions::TurretRight) {
        turret_rotation += if key_state.just_pressed || key_state.pressed {
            -1.
        } else {
            0.
        }
    }

    if let Some(key_state) = game_control.get_key_state(Actions::CannonUp) {
        cannon_rotation += if key_state.just_pressed || key_state.pressed {
            1.
        } else {
            0.
        }
    }

    if let Some(key_state) = game_control.get_key_state(Actions::CannonDown) {
        cannon_rotation += if key_state.just_pressed || key_state.pressed {
            -1.
        } else {
            0.
        }
    }

    if cannon_rotation == 0. && turret_rotation == 0. {
        if let Some(target) = camera_state.mouse_hit_position {
            let (global_transform, shot_data, shot_control) = fire_pos_query.single_mut();

            let pos = global_transform.translation();

            let shot_dir = calc_shot_dir(
                pos,
                target,
                shot_data.shot_speed(shot_control.time),
                shot_data.radius,
                9.8,
            );

            let (_scale, rotation, _pos) = turret_global_transform.to_scale_rotation_translation();
            let local_dir = Transform::from_rotation(rotation)
                .compute_matrix()
                .inverse()
                .transform_point3(shot_dir);
            let dot_forward = local_dir.dot(Vec3::NEG_Z);
            let dot_left = local_dir.dot(Vec3::NEG_X);

            turret_rotation = if dot_forward > 0. {
                (dot_left * 1.4).min(1.)
            } else {
                if dot_left > 0. {
                    1.
                } else {
                    -1.
                }
            };

            if turret_rotation.abs() < SPEED_EPSILON {
                turret_rotation = 0.;
            }

            let (_scale, rotation, _pos) = cannon_global_transform.to_scale_rotation_translation();
            let local_dir = Transform::from_rotation(rotation)
                .compute_matrix()
                .inverse()
                .transform_point3(shot_dir);

            cannon_rotation = if dot_forward > 0. {
                (local_dir.dot(Vec3::Y) * 1.4).min(1.)
            } else {
                0.
            };

            if cannon_rotation.abs() < SPEED_EPSILON {
                cannon_rotation = 0.;
            }
        }
    }

    if turret_control.speed != turret_rotation {
        turret_control.speed = turret_rotation;
    }

    if cannon_control.speed != cannon_rotation {
        cannon_control.speed = cannon_rotation;
    }
}

pub fn prep_shot_input(
    time: Res<Time>,
    local_handles: Res<LocalHandles>,
    mut query: Query<(&mut ControlFire, &PlayerData)>,
    game_control: Res<GameControl<Actions>>,
) {
    if local_handles.handles.is_empty() {
        return;
    }

    if query.is_empty() {
        return;
    }

    let (mut control, player) = query.single_mut();
    assert!(*local_handles.handles.first().unwrap() == player.handle);

    let mut new_control: ControlFire = ControlFire::default();

    if let Some(key_state) = game_control.get_key_state(Actions::CannonShot) {
        if key_state.just_pressed {
            new_control.time = 0.;
            new_control.is_shot = false;
        }

        if key_state.pressed {
            new_control.time = control.time + time.delta_seconds();
        }

        if key_state.just_released {
            new_control.time = control.time;
            new_control.is_shot = true;
        }
    }

    if *control != new_control {
        *control = new_control;
    }
}

pub fn process_correct_pos(
    //    time: Res<Time>,
    local_handles: Res<LocalHandles>,
    mut query: Query<(&mut Transform, &PlayerData), (With<ControlMove>, Without<CameraTarget>)>,
    game_control: Res<GameControl<Actions>>,
    camera_state: ResMut<CameraState>,
    mut camera_target_query: Query<&mut Transform, (With<CameraTarget>, Without<ControlMove>)>,
    mut spawn_tank_data: ResMut<NewTanksData>,
) {
    if local_handles.handles.is_empty() {
        return;
    }

    /*    if let Some(center_screen_hit_position) = camera_state.center_screen_hit_position {
            log::info!("center: {}", center_screen_hit_position);
        }
    */
    if let Some(key_state) = game_control.get_key_state(Actions::CorrectPos) {
        if !key_state.just_pressed {
            return;
        }

        if let Ok(mut transform) = camera_target_query.get_single_mut() {
            if let Some(center_screen_hit_position) = camera_state.center_screen_hit_position {
                transform.translation = center_screen_hit_position;
            }
        }

        let start_pos = if let Some(target) = camera_state.mouse_hit_position {
            target
        } else {
            log::info!("process_correct_pos camera_state error!");
            return;
        };

        if query.is_empty() {
            let start_angle = camera_state.pitch; //rng.gen_range(-std::f32::consts::PI..std::f32::consts::PI);

            spawn_tank_data.vector.push(NewTank {
                handle: *local_handles.handles.first().unwrap(),
                pos: Vec2::new(start_pos.x, start_pos.z),
                angle: start_angle,
            });

            return;
        }

        let (mut position, player) = query.single_mut();
        assert!(*local_handles.handles.first().unwrap() == player.handle);

        log::info!("process_correct_pos pressed");

        position.translation.x = start_pos.x;
        position.translation.y = start_pos.y + 1.;
        position.translation.z = start_pos.z;
    }
}
