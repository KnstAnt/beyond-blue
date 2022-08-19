use common::*;
use std::net::Ipv4Addr;

use futures::{select, FutureExt, StreamExt};
use libp2p::core::multiaddr::{Multiaddr, Protocol};
use libp2p::core::transport::OrTransport;
use libp2p::core::upgrade;
use libp2p::dns::DnsConfig;
use libp2p::gossipsub::{GossipsubEvent, IdentTopic};
use libp2p::identify::{IdentifyEvent, IdentifyInfo};
use libp2p::relay::v2::client::Client;
use libp2p::swarm::SwarmEvent;
use libp2p::tcp::{GenTcpConfig, TcpTransport};
use libp2p::{core::transport, swarm::SwarmBuilder, PeerId};
use libp2p::{identity, noise, Transport};
use libp2p_core::muxing::StreamMuxerBox;
use log::info;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::Event;

type BBSwarm = libp2p::swarm::Swarm<crate::Behaviour>;

pub struct SwarmSvc {
    swarm: BBSwarm,
}

impl SwarmSvc {
    pub async fn new_with_default_transport(local_key: identity::Keypair) -> BlueResult<Self> {
        let local_peer_id = PeerId::from(local_key.public());
        info!("Local peer id: {:?}", local_peer_id);

        let (relay_transport, client) = Client::new_transport_and_behaviour(local_peer_id);

        let noise_keys = noise::Keypair::<noise::X25519Spec>::new()
            .into_authentic(&local_key)
            .expect("Signing libp2p-noise static DH keypair failed.");

        let transport = OrTransport::new(
            relay_transport,
            DnsConfig::system(TcpTransport::new(GenTcpConfig::default().port_reuse(true)))
                .await
                .map_err(BlueError::local_err)?,
        )
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
        .multiplex(libp2p_yamux::YamuxConfig::default())
        .boxed();

        let behaviour = crate::Behaviour::new(client, &local_key)?;
        Self::try_new(transport, behaviour, local_peer_id)
    }

    pub fn try_new(
        transport: transport::Boxed<(PeerId, StreamMuxerBox)>,
        behaviour: crate::Behaviour,
        peer_id: PeerId,
    ) -> BlueResult<Self> {
        let swarm = SwarmBuilder::new(transport, behaviour, peer_id)
            .dial_concurrency_factor(10_u8.try_into().map_err(BlueError::local_err)?)
            .build();
        Ok(Self { swarm })
    }

    pub async fn spawn(
        &mut self,
        relay_address: Multiaddr,
        peer_id: Option<PeerId>,
        tx: Sender<String>,
        rx: Receiver<String>,
    ) -> BlueResult<()> {
        self.listen().await?;
        self.observe_addr(relay_address.clone()).await?;
        self.listen_on_relay(relay_address.clone())?;
        if let Some(peer_id) = peer_id {
            self.dial(relay_address, peer_id)?;
        }

        self.spawn_event_loop(tx, rx).await;

        Ok(())
    }

    async fn listen(&mut self) -> BlueResult<()> {
        self.swarm
            .listen_on(
                Multiaddr::empty()
                    .with(
                        "0.0.0.0"
                            .parse::<Ipv4Addr>()
                            .map_err(BlueError::local_err)?
                            .into(),
                    )
                    .with(Protocol::Tcp(0)),
            )
            .map_err(BlueError::local_err)?;

        let mut delay = futures_timer::Delay::new(std::time::Duration::from_secs(1)).fuse();
        loop {
            futures::select! {
                event = self.swarm.next() => {
                    match event.ok_or_else(|| BlueError::local_err("swarm stream was closed"))? {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("Listening on {:?}", address);
                        }
                        event => return Err(BlueError::local_err(format!("unexpected swarm event {:?}", event))),
                    }
                }
                _ = delay => {
                    // Likely listening on all interfaces now, thus continuing by breaking the loop.
                    break;
                }
            }
        }

        Ok(())
    }

    async fn observe_addr(&mut self, relay_address: Multiaddr) -> BlueResult<()> {
        self.swarm
            .dial(relay_address.clone())
            .map_err(BlueError::local_err)?;

        let mut learned_observed_addr = false;
        let mut told_relay_observed_addr = false;

        loop {
            match self
                .swarm
                .next()
                .await
                .ok_or_else(|| BlueError::local_err("swarm stream was closed"))?
            {
                SwarmEvent::NewListenAddr { .. } => {}
                SwarmEvent::Dialing { .. } => {}
                SwarmEvent::ConnectionEstablished { .. } => {}
                SwarmEvent::Behaviour(Event::Identify(IdentifyEvent::Sent { .. })) => {
                    info!("Told relay its public address.");
                    told_relay_observed_addr = true;
                }
                SwarmEvent::Behaviour(Event::Identify(IdentifyEvent::Received {
                    info: IdentifyInfo { observed_addr, .. },
                    ..
                })) => {
                    info!("Relay told us our public address: {:?}", observed_addr);
                    learned_observed_addr = true;
                }
                event => info!("{:?}", event),
            }

            if learned_observed_addr && told_relay_observed_addr {
                break;
            }
        }

        Ok(())
    }

    fn dial(&mut self, addr: Multiaddr, remote_peer_id: PeerId) -> BlueResult<()> {
        self.swarm
            .dial(
                addr.with(Protocol::P2pCircuit)
                    .with(Protocol::P2p(remote_peer_id.into())),
            )
            .map_err(BlueError::local_err)?;

        Ok(())
    }

    fn listen_on_relay(&mut self, relay_address: Multiaddr) -> BlueResult<()> {
        info!("relay_addr: {}", relay_address);
        self.swarm
            .listen_on(relay_address.with(Protocol::P2pCircuit))
            .map_err(BlueError::local_err)?;

        Ok(())
    }

    async fn spawn_event_loop(
        &mut self,
        remote_in: Sender<String>,
        mut local_out: Receiver<String>,
    ) {
        let stream = async_stream::stream! {
            while let Some(item) = local_out.recv().await {
                yield item;
            }
        };
        let stream = stream.fuse();

        tokio::pin!(stream);

        loop {
            select! {
                msg = stream.select_next_some() => {
                    if let Err(e) = self.swarm
                        .behaviour_mut()
                        .gossip
                        .publish(IdentTopic::new("player-info"), msg.as_bytes())
                    {
                        println!("Publish error: {:?}", e);
                    }
                },
                event = self.swarm.select_next_some() => match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        info!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(Event::Relay(event)) => {
                        info!("{:?}", event)
                    }
                    SwarmEvent::Behaviour(Event::Dcutr(event)) => {
                        info!("{:?}", event)
                    }
                    SwarmEvent::Behaviour(Event::Identify(event)) => {
                        info!("{:?}", event)
                    }
                    SwarmEvent::Behaviour(Event::Gossipsub(GossipsubEvent::Message {
                        propagation_source: peer_id,
                        message_id: id,
                        message,
                    })) => {
                        let msg = String::from_utf8_lossy(&message.data);
                        _ = remote_in.send(msg.to_string()).await;
                    },
                    SwarmEvent::ConnectionEstablished {
                        peer_id, endpoint, ..
                    } => {
                        info!("Established connection to {:?} via {:?}", peer_id, endpoint);
                    }
                    SwarmEvent::OutgoingConnectionError { peer_id, error } => {
                        info!("Outgoing connection error to {:?}: {:?}", peer_id, error);
                    }
                    _ => {}
                }
            }
        }
    }
}