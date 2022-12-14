use std::collections::HashMap;

use bevy::{prelude::{ResMut, Res, Resource}, time::Time};
use tokio::sync::mpsc;

use super::{NetMessage, Wrapper};


const PING_DEFAULT_VALUE: f32 = 0.05; // sec
//const PING_CONNECT_TIME: f64 = 1.; // sec
const PING_SEND_TIME: f32 = 3.; // sec
const PING_WAIT_TIME: f32 = 10.; // sec

#[derive(PartialEq)]
enum PingState {
    None,
    Send(f32),
    Wait(f32),    
}

pub struct Ping {
    time: Option<f32>,
}

impl Ping {
    pub fn update_time(&mut self, new_time: f32) {
        let tmp_time = (self.time.unwrap_or(PING_DEFAULT_VALUE)*4. + new_time)/5.;        
        log::info!("Network Ping receive, old:{:?}, new:{:?}, res:{:?}", self.time, new_time, tmp_time );
        self.time = Some(tmp_time as f32);
    }

    pub fn get_time(&self) -> f32 {
        self.time.unwrap_or(PING_DEFAULT_VALUE)
    }
    pub fn is_connected(&self) -> bool {
        self.time.is_some()
    }
}

impl Default for Ping {  
        fn default() -> Self {
        Self { 
            time: None, 
        }
    } 
}

#[derive(Resource)]
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
            state: PingState::None,//PingState::Connect(PING_CONNECT_TIME*((0.5 + rand::random::<f32>()/f32::MAX) as f64)), 
            data: HashMap::new(),
        }
    } 
}

impl PingList {
    pub(crate) fn insert(&mut self, new_handle: usize, ping: Ping) {
        self.data.insert(new_handle, ping);   
    }
    pub(crate) fn receive_pong(&mut self, pong_id: u64, handle: usize, receive_time: f32) {
        if pong_id != self.id {
            return;
        }

        if let PingState::Send(send_time) = self.state {
            if let Some(ping) = self.data.get_mut(&handle) {
                ping.update_time((receive_time - send_time)*0.5);           
            }
        }
    }
    fn update(&mut self, to_server: ResMut<Wrapper<mpsc::Sender<NetMessage>>>, current_time: f32) {
        if let PingState::None = self.state {
            return;
        } else if let PingState::Wait(wait_time) = self.state {
            if wait_time + PING_WAIT_TIME <= current_time {
                let _res = to_server.value.try_send(NetMessage::Ping(self.id, self.temp));
  //              log::info!("Network PingList update Send ping res:{:?}", _res);        
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

    pub fn start(&mut self) {
        if !self.is_connected() {
            self.state = PingState::Wait(PING_WAIT_TIME*(0.5 + rand::random::<f32>()/f32::MAX));
        }
    }


    pub fn get_time(&self, handle: usize) -> f32 {
        if let Some(ping) = self.data.get(&handle) { 
   //         log::info!("Network PingList get_time time:{:?}", ping.get_time()); 
            return ping.get_time();
        }
        PING_DEFAULT_VALUE
    }

    pub fn is_connected(&self) -> bool {
        if let PingState::None = self.state {
            return false;
        }

        true
    }

    pub(crate) fn check_collision_id(&mut self, in_id: u64) {
        if in_id == self.id { 
            self.id = rand::random::<u64>();
        }
    }
}

pub(crate) fn update_ping(
    mut ping: ResMut<PingList>,
//    to_server: ResMut<mpsc::Sender<NetMessage>>,
    to_server: ResMut<Wrapper<mpsc::Sender<NetMessage>>>, 
    time: Res<Time>,) {
        ping.update(to_server, time.into_inner().elapsed_seconds());
}