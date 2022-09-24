use bevy::prelude::shape::Cube;
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_rapier3d::plugin::*;
use common::BlueResult;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::runtime::Runtime;
use clap::Parser;
use iyes_loopless::prelude::*;
use rand::prelude::*;

use peer::NetworkEvent;

use crate::explosion::{OutExplosion, InExplosion};
use crate::menu::is_play_online;
use crate::player::PlayerHandle;
use crate::shot::{InShot, TankShotOutData};
use crate::tank::*;
use crate::AppState;

#[derive(Debug, Parser)]
#[clap(name = "Example Beyond Blue peer")]
pub struct Opts {
    /// The listening address
    #[clap(long)]
    relay_address: url::Url,
}


const PING_DEFAULT_VALUE: f32 = 0.05; // sec
const PING_SEND_TIME: f64 = 3.; // sec
const PING_WAIT_TIME: f64 = 10.; // sec

enum PingState {
    Send(f64),
    Wait(f64),    
}
pub struct Ping {
    time: f32,
}

impl Ping {
    pub fn update_time(&mut self, new_time: f64) {
        let tmp_time = (self.time*4. + new_time as f32)/5.;        
 //        log::info!("Network Ping receive, old:{:?}, new:{:?}, res:{:?}", self.time, new_time, tmp_time );
        self.time = tmp_time;
    }

    pub fn get_time(&self) -> f32 {
        self.time
    }
}

impl Default for Ping {  
        fn default() -> Self {
        Self { 
            time: PING_DEFAULT_VALUE, 
        }
    } 
}

pub struct PingList {
    id: u64,
    temp: u64, //for changing message
    state: PingState,
    data: HashMap<usize, Ping>,
}

impl Default for PingList {  
    fn default() -> Self {
        Self { 
            id: rand::random::<u64>(), 
            temp: 0,
            state: PingState::Wait(PING_WAIT_TIME*((0.5 + rand::random::<f32>()/f32::MAX) as f64)), 
            data: HashMap::new(),
        }
    } 
}

impl PingList {
    fn receive_pong(&mut self, pong_id: u64, handle: usize, receive_time: f64) {
        if pong_id != self.id {
            return;
        }

        if let PingState::Send(send_time) = self.state {
            if let Some(ping) = self.data.get_mut(&handle) {
                ping.update_time((receive_time - send_time)*0.5);           
            }
        }
    }
    fn update(&mut self, to_server: ResMut<mpsc::Sender<GameMessage>>, current_time: f64) {
        if let PingState::Wait(wait_time) = self.state {
            if wait_time + PING_WAIT_TIME <= current_time {
                let _res = to_server.try_send(GameMessage::Ping(self.id, self.temp));
 //               log::info!("Network PingList update Send ping res:{:?}", res);        
                self.state = PingState::Send(current_time);
                self.temp += 1;
            }
        } else if let PingState::Send(send_time) = self.state {
            if send_time + PING_SEND_TIME <= current_time {
  //              log::info!("Network PingList update set Wait");        
                self.state = PingState::Wait(current_time);
            }
        }
    }
    pub fn get_time(&self, handle: usize) -> f32 {
        if let Some(ping) = self.data.get(&handle) { 
   //         log::info!("Network PingList get_time time:{:?}", ping.get_time()); 
            return ping.get_time();
        }
        PING_DEFAULT_VALUE
    }
    fn check_collision_id(&mut self, in_id: u64) {
        if in_id == self.id { 
            self.id = rand::random::<u64>();
        }
    }
}


#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameMessage {
    Ping(u64, u64),
    Pong(u64, u64),
    BodyMove(TankBodyOutData),
    TurretRotate(TankTurretOutData),
    CannonRotate(TankCannonOutData),
    Shot(TankShotOutData),
    Explosion(OutExplosion),
}

pub type GameEvent = NetworkEvent<GameMessage>;

pub struct NewNetHandles {
    last_handle: usize,
    pub handles: HashMap<String, (PlayerHandle, TankBodyOutData)>,
}
pub struct NetHandles {
    pub handles: HashMap<String, PlayerHandle>,
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
            .insert_resource( opts )
            .insert_resource( PingList::default() )
            .insert_resource( NewNetHandles{last_handle: 0, handles: HashMap::new()} )
            .insert_resource( NetHandles{handles: HashMap::new()} )
            .insert_resource( InBody{data: HashMap::new()} )
            .insert_resource( InTurret{data: HashMap::new()} )
            .insert_resource( InCannon{data: HashMap::new()} )
            .insert_resource( InShot{data: HashMap::new()} )
            .insert_resource( InExplosion{data: HashMap::new()} )
              .add_system_set(
                SystemSet::on_enter(AppState::Connecting).with_system(setup_network.label("net_setup")),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Connecting)
                .with_system(check_network.run_if(is_play_online))
            )
            .add_system_set(
                SystemSet::on_update(AppState::Playing)
                .with_system(handle_conn_events.run_if(is_play_online))
                .with_system(update_ping.run_if(is_play_online))
                .with_system(send_out_body.run_if(is_play_online))
                .with_system(send_out_turret.run_if(is_play_online))
                .with_system(send_out_cannon.run_if(is_play_online))
                .with_system(send_out_shot.run_if(is_play_online))
                .with_system(send_out_explosion.run_if(is_play_online))
            );

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

pub fn handle_conn_events(
    mut ping: ResMut<PingList>,
    mut new_handles: ResMut<NewNetHandles>,
    mut handles: ResMut<NetHandles>,
    mut in_body: ResMut<InBody>,
    mut in_turret: ResMut<InTurret>,
    mut in_cannon: ResMut<InCannon>,
    mut in_shot: ResMut<InShot>,
    mut explosion: ResMut<InExplosion>,
    mut out_body: ResMut<TankBodyOutData>,
    mut out_turret: ResMut<TankTurretOutData>,
    mut out_cannon: ResMut<TankCannonOutData>,
    from_server: Res<Arc<Mutex<mpsc::Receiver<GameEvent>>>>,
    to_server: ResMut<mpsc::Sender<GameMessage>>,
    time: Res<Time>,
) {
//    log::info!("net handle_conn_events start");

    // The operation can't be blocking inside the bevy system.
    if let Ok(msg) = from_server.lock().unwrap().try_recv() {
        match msg {
            peer::NetworkEvent::NewConnection(_peer_id) => {
                out_body.set_changed();
                out_turret.set_changed(); //TODO
                out_cannon.set_changed(); //TODO
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
                            ping.data.insert(new_handle, Ping::default());                                       
                    } else {
                        return;
                    }
                };

                let handle = handles.handles.get(&peer_id).unwrap().clone();

                if let GameMessage::Ping(id, temp) = mess {
//                    log::info!("Network handle_conn_events Ping id:{:?}", id);  
                    ping.check_collision_id(id);
                    let _res = to_server.try_send(GameMessage::Pong(id, temp));
                } else if let GameMessage::Pong(id, _) = mess {
 //                   log::info!("Network handle_conn_events Pong id:{:?}", id);   
                    ping.receive_pong(id, handle, time.seconds_since_startup());
                } else if let GameMessage::BodyMove(data) = mess {
 //                   log::info!("Network handle_conn_events TankBodyOutData");
                    in_body.data.insert(handle, data);
                } else if let GameMessage::TurretRotate(data) = mess {
 //                   log::info!("Network handle_conn_events TankTurretOutData");
                    in_turret.data.insert(handle, data);
                } else if let GameMessage::CannonRotate(data) = mess {
 //                   log::info!("Network handle_conn_events TankCannonOutData");
                    in_cannon.data.insert(handle, data);
                } else if let GameMessage::Shot(data) = mess {
 //                   log::info!("Network handle_conn_events TankShotOutData");
                    in_shot.data.insert(handle, data);
                }   else if let GameMessage::Explosion(data_array) = mess {
 //                   log::info!("Network handle_conn_events ExplosionOutDataArray");
                    for data in data_array.data {
                        explosion.data.insert(handle, data);
                    }
                }
            }
        }
    }

 //   log::info!("net handle_conn_events end");
}

pub fn update_ping(
    mut ping: ResMut<PingList>,
    to_server: ResMut<mpsc::Sender<GameMessage>>,
    time: Res<Time>,) {
        ping.update(to_server, time.seconds_since_startup());
}
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
        log::info!("Network send_out_turret dir:{:?} speed:{:?}", data.dir, data.speed);
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

pub fn send_out_explosion(
    data: Res<OutExplosion>,
    to_server: ResMut<mpsc::Sender<GameMessage>>,
) {
    if data.is_changed() {
        let res = to_server.try_send(GameMessage::Explosion(data.to_owned()));
        log::info!("Network send_out_explosion {:?}", res);
    }
}

