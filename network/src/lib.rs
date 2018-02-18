extern crate uuid;

use uuid::Uuid;

pub const MAINNET_NETWORK_ID: [u8; 16] = [
    0x12 ,0x30, 0xF1, 0x71, 0x61, 0x04, 0x41, 0x61, 0x17, 0x31, 0x00, 0x82,
    0x16, 0xA1, 0xA1, 0x10,
];
pub const TESTNET_NETWORK_ID: [u8; 16] = [
    0x12 ,0x30, 0xF1, 0x71, 0x61, 0x04, 0x41, 0x61, 0x17, 0x31, 0x00, 0x82,
    0x16, 0xA1, 0xA1, 0x11,
];

pub const MAINNET_GENESIS_TX: &'static [u8] = &[
    0x01, 0x3c, 0x01, 0xff, 0x00, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x03, 0x02, 0x9b, 0x2e, 0x4c, 0x02, 0x81, 0xc0, 0xb0, 0x2e, 0x7c, 0x53,
    0x29, 0x1a, 0x94, 0xd1, 0xd0, 0xcb, 0xff, 0x88, 0x83, 0xf8, 0x02, 0x4f,
    0x51, 0x42, 0xee, 0x49, 0x4f, 0xfb, 0xbd, 0x08, 0x80, 0x71, 0x21, 0x01,
    0x77, 0x67, 0xaa, 0xfc, 0xde, 0x9b, 0xe0, 0x0d, 0xcf, 0xd0, 0x98, 0x71,
    0x5e, 0xbc, 0xf7, 0xf4, 0x10, 0xda, 0xeb, 0xc5, 0x82, 0xfd, 0xa6, 0x9d,
    0x24, 0xa2, 0x8e, 0x9d, 0x0b, 0xc8, 0x90, 0xd1
];
pub const MAINNET_GENESIS_NONCE: u32 = 10000;

pub const TESTNET_GENESIS_TX: &'static [u8] = &[
    0x01, 0x3c, 0x01, 0xff, 0x00, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x0f, 0x02, 0x9b, 0x2e, 0x4c, 0x02, 0x81, 0xc0, 0xb0, 0x2e, 0x7c, 0x53,
    0x29, 0x1a, 0x94, 0xd1, 0xd0, 0xcb, 0xff, 0x88, 0x83, 0xf8, 0x02, 0x4f,
    0x51, 0x42, 0xee, 0x49, 0x4f, 0xfb, 0xbd, 0x08, 0x80, 0x71, 0x21, 0x01,
    0x16, 0x8d, 0x0c, 0x4c, 0xa8, 0x6f, 0xb5, 0x5a, 0x4c, 0xf6, 0xa3, 0x6d,
    0x31, 0x43, 0x1b, 0xe1, 0xc5, 0x3a, 0x3b, 0xd7, 0x41, 0x1b, 0xb2, 0x4e,
    0x88, 0x32, 0x41, 0x02, 0x89, 0xfa, 0x6f, 0x3b
];
pub const TESTNET_GENESIS_NONCE: u32 = 10001;

/// The Monero network we are in
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Network {
    /// The real world, the place where Monero coins have real world economic
    /// value.
    Mainnet,
    /// The main Monero testnet, the place where you can dream you are rich.
    Testnet,
}

impl Network {
    /// Returns the network's ID.
    pub fn id(&self) -> Uuid {
        let id = match *self {
            Network::Mainnet => MAINNET_NETWORK_ID,
            Network::Testnet => TESTNET_NETWORK_ID,
        };

        Uuid::from_bytes(&id).expect("invalid network id")
    }

    /// Returns this peer's listening port.
    pub fn listen_port(&self) -> u32 {
        match *self {
            Network::Mainnet => 18080,
            Network::Testnet => 28080,
        }
    }

    pub fn hard_forks(&self) -> HardForks {
        let parameters: &'static [HardForkParameters] = match *self {
            Network::Mainnet => {
                &[
                    HardForkParameters { version: 1, height: 1, threshold: 0, time: 1341378000 },
                    HardForkParameters { version: 2, height: 1009827, threshold: 0, time: 1442763710 },
                    HardForkParameters { version: 3, height: 1141317, threshold: 0, time: 1458558528 },
                    HardForkParameters { version: 4, height: 1220516, threshold: 0, time: 1483574400 },
                    HardForkParameters { version: 5, height: 1288616, threshold: 0, time: 1489520158 },
                    HardForkParameters { version: 6, height: 1400000, threshold: 0, time: 1503046577 },
                ]
            },
            Network::Testnet => {
                &[
                    HardForkParameters { version: 1, height: 1, threshold: 0, time: 1341378000 },
                    HardForkParameters { version: 2, height: 624634, threshold: 0, time: 1445355000 },
                    HardForkParameters { version: 3, height: 800500, threshold: 0, time: 1472415034 },
                    HardForkParameters { version: 4, height: 801219, threshold: 0, time: 1472415035 },
                    HardForkParameters { version: 5, height: 802660, threshold: 0, time: 1472415036 + 86400*180 },
                    HardForkParameters { version: 6, height: 971400, threshold: 0, time: 1501709789 },
                    HardForkParameters { version: 7, height: 1057028, threshold: 0, time: 1512211236 },
                ]
            }
        };

        HardForks::from(parameters)
    }

    pub fn genesis_tx(&self) -> (&'static [u8], u32) {
        match *self {
            Network::Mainnet => (MAINNET_GENESIS_TX, MAINNET_GENESIS_NONCE),
            Network::Testnet => (TESTNET_GENESIS_TX, TESTNET_GENESIS_NONCE),
        }
    }
}

#[derive(Debug)]
pub struct HardForks {
    pub parameters: &'static [HardForkParameters],
}

impl HardForks {
    pub fn ideal_version(&self) -> u8 {
        let back = self.parameters.len() - 1;
        self.parameters[back].version
    }
}

impl From<&'static [HardForkParameters]> for HardForks {
    fn from(parameters: &'static [HardForkParameters]) -> HardForks {
        HardForks {
            parameters,
        }
    }
}

/// The information about a hard fork.
#[derive(Debug)]
pub struct HardForkParameters {
    /// The version.
    pub version: u8,
    /// The block height.
    pub height: u64,
    // XXX: what does this mean on monero.
    /// Threshold
    pub threshold: u8,
    /// Time since epoch.
    pub time: u64,
}