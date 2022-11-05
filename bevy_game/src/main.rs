// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//use clap::Parser;

use bevy::prelude::{App, ClearColor, Color, Msaa, WindowDescriptor};
use bevy::DefaultPlugins;
//use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_game::AplicationPlugin;


#[tokio::main]
async fn main() {

    std::env::set_var("RUST_BACKTRACE", "full");
    
    let mut app = App::new();

    app
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::rgb(0.4, 0.4, 0.4)))
        .insert_resource(WindowDescriptor {
            width: 800.,
            height: 600.,
            title: "Bevy game".to_string(), // ToDo
            ..Default::default()
        })
        .insert_resource(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
        )
        .add_plugins(DefaultPlugins)
//        .add_plugin(WorldInspectorPlugin::new())
        .add_system(bevy::window::close_on_esc)
        .add_plugin(AplicationPlugin)
        .run();
}