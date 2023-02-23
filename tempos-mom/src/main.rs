mod config;
mod task;

use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tokio::task::futures::TaskLocalFuture;

struct UdpAdapter {
    // source_sock: UdpSocket,
    // source_addr: SocketAddr,
    sink_sock: UdpSocket,
    sink_addr: SocketAddr,
}

impl UdpAdapter {
    pub fn new(config: &str) -> anyhow::Result<Self> {
        let cfg: Vec<&str> = config.split(&"?").collect();
        let addr = cfg.get(0).unwrap_or(&"");

        let props_str: Vec<&str> = cfg.get(1).unwrap_or(&"").split(&['&', '=']).collect();
        let mut props = HashMap::new();
        for i in (0..props_str.len()).step_by(2) {
            props.insert(props_str[i], props_str[i + 1]);
        }

        let sink_sock = UdpSocket::bind(addr)?;
        sink_sock.set_read_timeout(Some(Duration::from_millis(100)))?;

        let sink_addr: SocketAddr = props.get("sink").unwrap().parse()?;

        Ok(Self {
            sink_sock,
            sink_addr,
        })
    }

    pub fn read(&self, buf: &mut [u8]) -> anyhow::Result<usize> {
        let (bytes_read, _) = self.sink_sock.recv_from(buf)?;

        Ok(bytes_read)
    }

    pub fn write(&self, buf: &[u8]) -> anyhow::Result<()> {
        self.sink_sock.send_to(&buf, &self.sink_addr)?;

        Ok(())
    }
}

const UDP_ADAPTER_CONFIG: &str = "udp/127.0.0.1:8001?sink=127.0.0.1:8002&prova=hello";

#[derive(Debug, PartialEq, Eq, Hash)]
struct Node {
    id: u32,
    load: u32,
    channel: SocketAddr,
}

struct Core {
    nodes: HashMap<u32, Node>,
    topics: HashMap<String, Vec<u32>>,
}

impl Core {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            topics: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, id: u32, channel: SocketAddr) {
        let node = Node {
            id: id,
            load: 0,
            channel: channel,
        };
        self.nodes.insert(id, node);
    }

    pub fn remove_node(&mut self, id: u32) {
        self.nodes.remove(&id);
    }

    pub fn update_node_load(&mut self, id: u32, load: u32) {
        if let Some(mut node) = self.nodes.get_mut(&id) {
            node.load = load;
        }
    }

    pub fn get_topic(&self, topic: &str) -> Option<&Vec<u32>> {
        self.topics.get(topic)
    }

    pub fn get_topic_mut(&mut self, topic: &str) -> Option<&mut Vec<u32>> {
        self.topics.get_mut(topic)
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let sock = UdpSocket::bind("0.0.0.0:8080")?;
    let mut buf = [0; 1024];
    let mut msg_type;

    // setup the core
    let mut core = Core::new();

    // make the socket non-blocking
    sock.set_read_timeout(Some(Duration::from_millis(100)))?;

    loop {
        // receive a message handling the timeout
        let (bytes_read, addr) = match sock.recv_from(&mut buf) {
            Ok((bytes_read, addr)) => (bytes_read, addr),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    println!("Nodes: {:?}", core.nodes);
                    continue;
                } else {
                    return Err(e.into());
                }
            }
        };

        msg_type = u8::from_be_bytes([buf[0]]);

        match msg_type {
            tempos::msg_type::REGISTRATION => {
                let node_id = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                let topic_len = u32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);
                let topic = std::str::from_utf8(&buf[9..9 + topic_len as usize])?;
                core.add_node(node_id, addr);
                if let Some(topic) = core.get_topic_mut(topic) {
                    topic.push(node_id);
                } else {
                    core.topics.insert(topic.to_string(), vec![node_id]);
                }
                log::debug!("REGISTRATION message from {} for topic {}", node_id, topic);
            }
            tempos::msg_type::MONITORING => {
                let node_id = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                let load = f32::from_be_bytes([buf[5], buf[6], buf[7], buf[8]]);
                // normalize the load from 0 to 100
                let load = (load * 100.0) as u32;
                // core.get_nodes().get(&node_id);
                core.update_node_load(node_id, load);
            }
            tempos::msg_type::UNREGISTRATION => {
                let node_id = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                log::debug!("UNREGISTRATION message from {}", node_id);
                core.remove_node(node_id);
            }
            tempos::msg_type::INVOK => {
                let topic_len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]);
                let topic = std::str::from_utf8(&buf[5..5 + topic_len as usize])?;
                log::debug!("INVOK message for topic '{}'", topic);
                if let Some(topic) = core.get_topic(topic) {
                    if !topic.is_empty() {
                        let node_id = topic[0];
                        let node = core.nodes.get(&node_id).unwrap();
                        log::debug!("Sending INVOK to node {}, buffer: {:?}", node_id, &buf);
                        sock.send_to(&buf, &node.channel)?;
                    }
                } else {
                    log::warn!("No node registered for topic '{}'", topic);
                }
            }
            _ => println!("Unhandled message type"),
        }
    }

    Ok(())
}

fn one_thread() {
    let running = Arc::new(AtomicBool::new(true));
    let r = Arc::clone(&running);
    ctrlc::set_handler(move || {
        println!("Received Ctrl+C, exiting...");
        r.store(false, Ordering::Relaxed);
    })
    .unwrap();

    let mut buf = [0u8; 4096];
    let t_handle = std::thread::spawn(move || {
        let strs: Vec<&str> = UDP_ADAPTER_CONFIG.split('/').collect();

        let proto = strs.get(0).unwrap_or(&"");
        let config = strs.get(1).unwrap_or(&"");

        let adapter = match proto {
            &"udp" => UdpAdapter::new(config).unwrap(),
            _ => UdpAdapter::new(config).unwrap(),
        };

        while running.load(Ordering::Relaxed) {
            if let Ok(_) = adapter.read(&mut buf) {
                adapter.write(&buf).unwrap();
            }
        }
    });

    t_handle.join().unwrap();
}
