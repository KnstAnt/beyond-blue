mod audio;
mod loading;
mod menu;
mod cleanup;
mod game;
mod network;
//mod matchbox_net;
mod input;
mod tank;
mod shot;
mod explosion;
mod terrain;
mod player;
//mod log_plugin;
mod camera;
mod ballistics;

//use crate::input::InputPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::game::GamePlugin;
//use crate::network::NetPlugin;
//use crate::tank::TankPlugin;
//use crate::player::PlayerPlugin;
//use crate::shot::ShotPlugin;
//use crate::explosion::ExplosionPlugin;


use bevy::{app::App, log::Level};

#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
//use matchbox_net::NetPlugin;


// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum AppState {
    Loading,    

    Connecting,

    PreparePlaying,

    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

pub struct AplicationPlugin;

impl Plugin for AplicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_state(AppState::Loading)
            .add_plugin(LoadingPlugin)
            .add_plugin(MenuPlugin)
            .add_plugin(InternalAudioPlugin)
            .add_plugin(GamePlugin);

        #[cfg(debug_assertions)]
        {
  //          app.add_plugin(FrameTimeDiagnosticsPlugin::default())
 //               .add_plugin(LogDiagnosticsPlugin::default());
        }
    }
}
