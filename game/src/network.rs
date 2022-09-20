use bevy::prelude::shape::Cube;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_rapier3d::plugin::*;
use common::BlueResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::runtime::Runtime;
use clap::Parser;
use iyes_loopless::prelude::*;

use peer::NetworkEvent;

use crate::menu::is_play_online;
use crate::player::PlayerHandle;
use crate::tank::*;
use crate::AppState;

const NUM_PLAYERS: usize = 2;
const FPS: usize = 60;
const MAX_PREDICTION: usize = 8;
const INPUT_DELAY: usize = 2;
// Having a "load screen" time helps with initial desync issues.  No idea why,
// but this tests well.
const LOAD_SECONDS: usize = 3;


const BLUE: Color = Color::rgb(0.8, 0.6, 0.2);
const ORANGE: Color = Color::rgb(0., 0.35, 0.8);
const MAGENTA: Color = Color::rgb(0.9, 0.2, 0.2);
const GREEN: Color = Color::rgb(0.35, 0.7, 0.35);
const PLAYER_COLORS: [Color; 4] = [BLUE, ORANGE, MAGENTA, GREEN];



#[derive(Debug, Parser)]
#[clap(name = "Example Beyond Blue peer")]
pub struct Opts {
    /// The listening address
    #[clap(long)]
    relay_address: url::Url,
}

/* 
#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
pub struct NetInput {
    pub temp: u16,
    pub is_shot: u8,
    pub temp1: u8,
    pub body_movement_x: f32,
    pub body_movement_y: f32,
    pub body_pos_x: f32,
    pub body_pos_y: f32,
    pub body_dir: f32,
    pub turret_dir: f32,
    pub turret_speed: f32,
    pub cannon_dir: f32,
    pub cannon_speed: f32,
    pub shot_pos: Vec3,
    pub shot_vel: Vec3,
}
*/


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameMessage {
    BodyMove(TankBodyOutData),
    TurretRotate(TankTurretOutData),
    CannonRotate(TankCannonOutData),
    Shot(TankShotOutData),
}

pub type GameEvent = NetworkEvent<GameMessage>;

pub struct NewNetHandles {
    last_handle: usize,
    pub handles: HashMap<String, (PlayerHandle, TankBodyOutData)>,
}
pub struct NetHandles {
    pub handles: HashMap<String, PlayerHandle>,
}
pub struct InBody {
    pub data: HashMap<PlayerHandle, TankBodyOutData>,
}
pub struct InTurret {
    pub data: HashMap<PlayerHandle, TankTurretOutData>,
}
pub struct InCannon {
    pub data: HashMap<PlayerHandle, TankCannonOutData>,
}
pub struct InShot {
    pub data: HashMap<PlayerHandle, TankShotOutData>,
}


pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        // DefaultPlugins will use window descriptor
        let opts = Opts::parse();

        app
            /*   .insert_resource(window_info)
                    .insert_resource(ClearColor(Color::BLACK))
                    .insert_resource(LogSettings {
                        level: Level::DEBUG,
                        ..default()
                    })
                    .add_plugins_with(DefaultPlugins, |plugins| plugins.disable::<LogPlugin>())
                    // Add our own log plugin to help with comparing desync output
                    .add_plugin(log_plugin::LogPlugin)
            */
  //          .add_startup_system(startup)
            .insert_resource(opts)
            .insert_resource( NewNetHandles{last_handle: 0, handles: HashMap::new()} )
            .insert_resource( NetHandles{handles: HashMap::new()} )
            .insert_resource( InBody{data: HashMap::new()} )
            .insert_resource( InTurret{data: HashMap::new()} )
            .insert_resource( InCannon{data: HashMap::new()} )
            .insert_resource( InShot{data: HashMap::new()} )
            .add_system_set(
                SystemSet::on_enter(AppState::Connecting).with_system(setup_network.label("net_setup")),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Connecting)
                .with_system(check_network.run_if(is_play_online))
            )
//            .add_system_set(SystemSet::on_enter(AppState::Playing).with_system(spawn_players))
            .add_system_set(
                SystemSet::on_update(AppState::Playing)
                .with_system(handle_conn_events.run_if(is_play_online))
          //      .with_system(send_out.run_if(is_play_online))
                .with_system(send_out_body.run_if(is_play_online))
                .with_system(send_out_turret.run_if(is_play_online))
                .with_system(send_out_cannon.run_if(is_play_online))
                .with_system(send_out_shot.run_if(is_play_online))
            )
            .add_system_set(
                ConditionSet::new()
                    .run_if(is_play_online)
                    .into(),
            )
            /*       .add_system_set(
                       SystemSet::on_update(AppState::Playing)
                       .with_system(keyboard_input)
                   )
            */
            ;

        // Be sure to setup all four stages.
        // We don't despawn in this example, but you may want to :)
        /*   app.add_stage_before(
                CoreStage::Last,
                PhysicsStages::DetectDespawn,
                SystemStage::parallel().with_system_set(RapierPhysicsPlugin::<NoUserData>::get_systems(
                    PhysicsStages::DetectDespawn,
                )),
            );
        */
        // Configure plugin without system setup, otherwise your simulation will run twice
        /*   app.add_plugin(
                RapierPhysicsPlugin::<NoUserData>::default()
                    // Scale of 8 since that's the factor size of our ball & players
                    .with_physics_scale(8.)
                    // This allows us to hook in the systems ourselves above in the GGRS schedule
                    .with_default_system_setup(false),
            );
        */

        log::info!("net init plugin");
    }
}

fn setup_network(
    mut commands: Commands, 
    runtime: Res<Runtime>, 
    opts: Res<Opts>,
) {
    log::info!("net setup_network start");

    let (local_in, local_out) = mpsc::channel(32);
    let (remote_in, remote_out) = mpsc::channel(32);

    let relay_address = opts.relay_address.clone();
    runtime.spawn(async move {
        let id = common::Identity::from_file("nothing".into());

        tokio::spawn(async move {
            let res = peer::Swarm::new_with_default_transport(id.get_key())
                .await?
                .spawn::<GameMessage>(relay_address, remote_in, local_out)
                .await;

            log::info!("Game swarm result: {:?}", res);

            BlueResult::Ok(())
        });
    });

    commands.insert_resource(local_in);
    commands.insert_resource(Arc::new(Mutex::new(remote_out)));

    log::info!("net setup_network end");
}

fn check_network(
    mut app_state: ResMut<State<AppState>>,
) {
    // TODO check is connection switch on
    app_state.replace(AppState::PreparePlaying).unwrap();
}

/* 
pub fn handle_conn_events(
    mut net_data: ResMut<NetData>,
    from_server: Res<Arc<Mutex<mpsc::Receiver<GameEvent>>>>,
) {
    // The operation can't be blocking inside the bevy system.
    if let Ok(msg) = from_server.lock().unwrap().try_recv() {
        match msg {
            peer::NetworkEvent::NewConnection(peer_id) => {
                if net_data.handles.get(&peer_id).is_none() && net_data.new_handles.get(&peer_id).is_none() {
                    let new_handle = net_data.last_handle + 1;
                    net_data.new_handles.insert(peer_id, new_handle);
                    assert!(new_handle < usize::MAX);
                    net_data.last_handle = new_handle; 
                }
            }

            peer::NetworkEvent::Event(peer_id, input) => {               
                if net_data.handles.contains_key(&peer_id) {
                    let handle = net_data.handles.get(&peer_id).unwrap().clone();
                    net_data.move_data.insert(handle, input);
                }
            }
        }
    }
}
pub struct NewNetHandles {
    last_handle: usize,
    pub new_handles: HashMap<String, PlayerHandle>,
}
pub struct NetHandles {
    pub handles: HashMap<String, PlayerHandle>,
}
pub struct InBody {
    pub data: HashMap<PlayerHandle, TankBodyOutData>,
}
pub struct InTurret {
    pub data: HashMap<PlayerHandle, TankTurretOutData>,
}
pub struct InCannon {
    pub data: HashMap<PlayerHandle, TankCannonOutData>,
}
pub struct InShot {
    pub data: HashMap<PlayerHandle, TankShotOutData>,
}

*/
pub fn handle_conn_events(
    mut new_handles: ResMut<NewNetHandles>,
    mut handles: ResMut<NetHandles>,
    mut body: ResMut<InBody>,
    mut turret: ResMut<InTurret>,
    mut cannon: ResMut<InCannon>,
    mut shot: ResMut<InShot>,
    from_server: Res<Arc<Mutex<mpsc::Receiver<GameEvent>>>>,
) {
//    log::info!("net handle_conn_events start");

    // The operation can't be blocking inside the bevy system.
    if let Ok(msg) = from_server.lock().unwrap().try_recv() {
        match msg {
            peer::NetworkEvent::NewConnection(_peer_id) => {
            }

            peer::NetworkEvent::Event(peer_id, mess) => {               
                if handles.handles.get(&peer_id).is_none() 
                    && new_handles.handles.get(&peer_id).is_none() {
                        if let GameMessage::BodyMove(data) = mess.clone() {
                            let new_handle = new_handles.last_handle + 1;
                            assert!(new_handle < usize::MAX);
                            new_handles.handles.insert(peer_id.clone(), (new_handle, data));  
                            handles.handles.insert(peer_id.clone(), new_handle); 
                            new_handles.last_handle = new_handle;                  
                    } else {
                        return;
                    }
                };

                let handle = handles.handles.get(&peer_id).unwrap().clone();

                if let GameMessage::BodyMove(data) = mess {
                    log::info!("Network handle_conn_events TankBodyOutData");
                    body.data.insert(handle, data);
                } else if let GameMessage::TurretRotate(data) = mess {
                    log::info!("Network handle_conn_events TankTurretOutData");
                    turret.data.insert(handle, data);
                } else if let GameMessage::CannonRotate(data) = mess {
                    log::info!("Network handle_conn_events TankCannonOutData");
                    cannon.data.insert(handle, data);
                } else if let GameMessage::Shot(data) = mess {
                    log::info!("Network handle_conn_events TankShotOutData");
                    shot.data.insert(handle, data);
                }
            }
        }
    }

 //   log::info!("net handle_conn_events end");
}
/* 
pub fn send_out(
    body_data: Res<TankBodyOutData>,
    turret_control: Res<TankTurretOutData>,
    cannon_control: Res<TankCannonOutData>,
    shot_control: Res<TankShotOutData>,
    to_server: ResMut<mpsc::Sender<GameMessage>>,
) {
    let out_data = GameMessage {
        body_movement_x: body_data.movement.x,
        body_movement_y: body_data.movement.y,
        body_pos_x: body_data.pos.x,
        body_pos_y: body_data.pos.y,
        body_dir: body_data.dir,

        turret_speed: turret_control.speed,
        turret_dir: turret_control.dir,

        cannon_speed: cannon_control.speed,
        cannon_dir: cannon_control.dir,

        is_shot: if shot_control.is_shot { 1 } else { 0 },
        shot_pos: shot_control.pos,
        shot_vel: shot_control.vel,

        ..default()
    };

    // Do not allow inputs for the first while.
 //   if game_state.frame > (FPS * LOAD_SECONDS) as u32 {
 //       res.inp = game_control.get_key_states();
 //   }

    //   log::info!("net input x:{} y:{} dir:{}", body_control.pos.x, body_control.pos.y, body_control.dir);

    let res = to_server.try_send(out_data);

    log::info!("Network send_out {:?}", res);

}
*/
pub fn send_out_body(
    data: Res<TankBodyOutData>,
    to_server: ResMut<mpsc::Sender<GameMessage>>,
) {
    if data.is_changed() {
        let res = to_server.try_send(GameMessage::BodyMove(data.to_owned()));
        log::info!("Network send_out_body {:?}", res);
    }
}

pub fn send_out_turret(
    data: Res<TankTurretOutData>,
    to_server: ResMut<mpsc::Sender<GameMessage>>,
) {
    if data.is_changed() {
        let res = to_server.try_send(GameMessage::TurretRotate(data.to_owned()));
        log::info!("Network send_out_turret {:?}", res);
    }
}

pub fn send_out_cannon(
    data: Res<TankCannonOutData>,
    to_server: ResMut<mpsc::Sender<GameMessage>>,
) {
    if data.is_changed() {
        let res = to_server.try_send(GameMessage::CannonRotate(data.to_owned()));
        log::info!("Network send_out_cannon {:?}", res);
    }
}

pub fn send_out_shot(
    data: Res<TankShotOutData>,
    to_server: ResMut<mpsc::Sender<GameMessage>>,
) {
    if data.is_changed() {
        let res = to_server.try_send(GameMessage::Shot(data.to_owned()));
        log::info!("Network send_out_shot {:?}", res);
    }
}
