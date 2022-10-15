use bevy::prelude::Component;
use bevy::prelude::*;

//use crate::matchbox_net::*;
use crate::input::*;

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
            .with_system(
                crate::player::prep_cannon_input
                    .label("player_input")
                    .after("keys_input"),
            )
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
        movement += match key_state {
            KeyState::JustPressed | KeyState::Pressed => Vec2::Y,
            _ => Vec2::ZERO,
        }
    }

    if let Some(key_state) = game_control.get_key_state(Actions::Down) {
        movement -= match key_state {
            KeyState::JustPressed | KeyState::Pressed => Vec2::Y,
            _ => Vec2::ZERO,
        }
    }

    if let Some(key_state) = game_control.get_key_state(Actions::Left) {
        movement -= match key_state {
            KeyState::JustPressed | KeyState::Pressed => Vec2::X,
            _ => Vec2::ZERO,
        }
    }

    if let Some(key_state) = game_control.get_key_state(Actions::Right) {
        movement += match key_state {
            KeyState::JustPressed | KeyState::Pressed => Vec2::X,
            _ => Vec2::ZERO,
        }
    }

    if control.movement != movement {
        control.movement = movement;
    }
}

pub fn prep_turret_input(
    time: Res<Time>,
    local_handles: Res<LocalHandles>,
    mut query: Query<(&mut ControlTurret, &PlayerData)>,
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

    let mut rotation: f32 = 0.;

    if let Some(key_state) = game_control.get_key_state(Actions::TurretLeft) {
        rotation += match key_state {
            KeyState::JustPressed | KeyState::Pressed => 1.,
            _ => 0.,
        }
    }

    if let Some(key_state) = game_control.get_key_state(Actions::TurretRight) {
        rotation += match key_state {
            KeyState::JustPressed | KeyState::Pressed => -1.,
            _ => 0.,
        }
    }

    //        println!("player prep_turret_input, rotation: {}", rotation);

    //            println!("player prep_turret_input, ok");

    /*          let speed = crate::tank::utils::calc_rotation_speed(
                    time.delta_seconds(),
                    rotation_from_control,
                    PI,
                    control.speed,
                    0.5,
                    0.2,
                );
    */
    if control.speed != rotation {
        control.speed = rotation;
    }
}

pub fn prep_cannon_input(
    time: Res<Time>,
    local_handles: Res<LocalHandles>,
    mut query: Query<(&mut ControlCannon, &PlayerData)>,
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

    let mut rotation = 0.;

    if let Some(key_state) = game_control.get_key_state(Actions::CannonUp) {
        rotation += match key_state {
            KeyState::JustPressed | KeyState::Pressed => -1.,
            _ => 0.,
        }
    }

    if let Some(key_state) = game_control.get_key_state(Actions::CannonDown) {
        rotation += match key_state {
            KeyState::JustPressed | KeyState::Pressed => 1.,
            _ => 0.,
        }
    }

    /*let speed = crate::tank::utils::calc_rotation_speed(
        time.delta_seconds(),
        rotation,
        PI / 3.,
        control.speed,
        0.3,
        0.1,
    );*/

    if control.speed != rotation {
        control.speed = rotation;
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
        match key_state {
            KeyState::JustPressed => {
                new_control.time = 0.;
                new_control.is_shot = false;
            }
            KeyState::Pressed => {
                new_control.time = control.time + time.delta_seconds();
                //                        shot_data.is_shot = false;
            }
            KeyState::JustReleased => {
                new_control.is_shot = true;
            }
            _ => {
                //                        shot_data.time = 0.;
                //                        shot_data.is_shot = false;
            }
        }
    }

    if *control != new_control {
        *control = new_control;
    }
}
