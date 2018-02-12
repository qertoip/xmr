use std::net::{SocketAddr, ToSocketAddrs};

use clap::ArgMatches;
use failure::Error;
use common_failures::io::{IoContextExt, Target, Operation};
use network::Network;
use peers::default_peers;

#[derive(Debug)]
pub struct Config {
    pub network: Network,
    pub peers: Vec<SocketAddr>,
    pub threads: usize,
    pub listen_port: u32,
}

pub fn parse(matches: &ArgMatches) -> Result<Config, Error> {
    let network = match matches.is_present("testnet") {
        true => Network::Testnet,
        false => Network::Mainnet,
    };

    let peers = match value_t!(matches.value_of("connect"), SocketAddr) {
        Ok(addr) => {
            let mut peers = Vec::with_capacity(1);
            peers.push(addr);
            peers
        }
        Err(_e) => {
            default_peers(network)
        },
    };

    let threads = value_t!(matches.value_of("threads"), usize).unwrap_or(1);

    let listen_port = value_t!(matches.value_of("listenport"), u32).unwrap_or(network.listen_port());

    Ok(Config {
        network,
        peers,
        threads,
        listen_port,
    })
}
