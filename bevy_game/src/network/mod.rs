use serde::{Deserialize, Serialize};
use bevy::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::runtime::Runtime;
use clap::Parser;
use iyes_loopless::prelude::*;

use peer::NetworkEvent;
use common::BlueResult;

use crate::menu::is_play_online;
use crate::player::PlayerHandle;
use crate::AppState;

mod ping;
pub use ping::*;

use crate::game::{GameMessage, OutGameMessages};
use crate::game::InMessages;


#[derive(Debug, Parser)]
#[clap(name = "Example Beyond Blue peer")]
pub struct Opts {
    /// The listening address
    #[clap(long)]
    relay_address: url::Url,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NetMessage {
    Ping(u64, u64),
    Pong(u64, u64),
    GameData(GameMessage),
}

pub type NetEvent = NetworkEvent<NetMessage>;

pub struct NetHandles {
    last_handle: usize,
    pub handles: HashMap<String, PlayerHandle>,
}

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        // DefaultPlugins will use window descriptor
        let opts = Opts::parse();

        let before_system_set = SystemSet::on_update(AppState::Playing)
            .with_system(handle_conn_events.run_if(is_play_online));

        let after_system_set = SystemSet::on_update(AppState::Playing)
                .with_system(send_out.run_if(is_play_online))
                .with_system(update_ping.run_if(is_play_online));

        app
            .insert_resource( opts )
            .insert_resource( PingList::default() )
            .insert_resource( NetHandles{handles: HashMap::new(), last_handle: 0} )
            .add_system_set(
                SystemSet::on_enter(AppState::Connecting).with_system(setup_network.label("net_setup")),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Connecting)
                .with_system(check_network.run_if(is_play_online))
                .with_system(handle_conn_events.run_if(is_play_online))
                .with_system(update_ping.run_if(is_play_online))
            )
 /*          .add_system_set(
                SystemSet::on_update(AppState::Playing)
                .with_system(update_ping.run_if(is_play_online))
            )*/
            .add_system_set_to_stage(CoreStage::PreUpdate, before_system_set)
            .add_system_set_to_stage(CoreStage::PostUpdate, after_system_set)
            ;

        log::info!("net init plugin");
    }
}

fn setup_network(
    mut commands: Commands, 
    runtime: Res<Runtime>, 
    opts: Res<Opts>,
) {
    log::info!("setup_network start");

    let (local_in, local_out) = mpsc::channel(32);
    let (remote_in, remote_out) = mpsc::channel(32);

    let relay_address = opts.relay_address.clone();
    runtime.spawn(async move {
        let id = common::Identity::from_file("nothing".into());

        tokio::spawn(async move {
            let res = peer::Swarm::new_with_default_transport(id.get_key())
                .await?
                .spawn::<NetMessage>(relay_address, remote_in, local_out)
                .await;

            log::info!("Game swarm result: {:?}", res);

            BlueResult::Ok(())
        });
    });

    commands.insert_resource(local_in);
    commands.insert_resource(Arc::new(Mutex::new(remote_out)));

    log::info!("setup_network end");
}

fn check_network(
    ping: Res<PingList>,
    mut app_state: ResMut<State<AppState>>,
) {
//    log::info!("net check_network start");
    if ping.is_connected() {
        app_state.replace(AppState::PreparePlaying).unwrap();
        log::info!("check_network ok, set AppState::PreparePlaying");
    }
 //   log::info!("net check_network end");
}

pub fn handle_conn_events(
    mut ping: ResMut<PingList>,
    mut handles: ResMut<NetHandles>,    
    mut in_mess: ResMut<InMessages<GameMessage>>,
    from_server: Res<Arc<Mutex<mpsc::Receiver<NetEvent>>>>,
    to_server: ResMut<mpsc::Sender<NetMessage>>,
    time: Res<Time>,
) {
 //   log::info!("net handle_conn_events start");

    // The operation can't be blocking inside the bevy system.
    if let Ok(msg) = from_server.lock().unwrap().try_recv() {
        match msg {
            peer::NetworkEvent::NewConnection(peer_id) => {
                log::info!("handle_conn_events msg: NewConnection");

                if handles.handles.get(&peer_id).is_none() {
                    let new_handle = handles.last_handle + 1; 
                    assert!(new_handle < usize::MAX);       
                    handles.handles.insert(peer_id.clone(), new_handle);   
                    handles.last_handle = new_handle;  

                    let _res = to_server.try_send(NetMessage::GameData(GameMessage::DataRequest));
                }

                if !ping.is_connected() {
                    ping.start();
                }
            },

            peer::NetworkEvent::Event(peer_id, mess) => {   
                log::info!("handle_conn_events msg: Event");                

                let handle = handles.handles.get(&peer_id).unwrap().clone();

                if let NetMessage::Ping(id, temp) = mess {
                    log::info!("handle_conn_events Ping id:{:?}", id);  
                    ping.check_collision_id(id);
                    let _res = to_server.try_send(NetMessage::Pong(id, temp));
                } else if let NetMessage::Pong(id, _) = mess {
                    log::info!("handle_conn_events Pong id:{:?}", id);   
                    ping.receive_pong(id, handle, time.seconds_since_startup());
                } else if let NetMessage::GameData(data) = mess {
                    log::info!("handle_conn_events GameData");
                    in_mess.data.insert(handle, data);
                }
            },
        }
    }
 //   log::info!("net handle_conn_events end");
}

fn send_out(
    mut output: ResMut<OutGameMessages<GameMessage>>,
    to_server: ResMut<mpsc::Sender<NetMessage>>,
) {
    if output.is_changed() {

        for mess in output.data.drain(0..) {
            let res = to_server.try_send(NetMessage::GameData(mess));
            log::info!("Network send_out {:?}", res);
        }

        output.data.clear();
    }
}

