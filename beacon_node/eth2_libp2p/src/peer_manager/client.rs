//! Known Ethereum 2.0 clients and their fingerprints.
//!
//! Currently using identify to fingerprint.

use libp2p::identify::IdentifyInfo;
use serde::Serialize;

/// Various client and protocol information related to a node.
#[derive(Clone, Debug, Serialize)]
pub struct Client {
    /// The client's name (Ex: lighthouse, prism, nimbus, etc)
    pub kind: ClientKind,
    /// The client's version.
    pub version: String,
    /// The OS version of the client.
    pub os_version: String,
    /// The libp2p protocol version.
    pub protocol_version: String,
    /// Identify agent string
    pub agent_string: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum ClientKind {
    /// A lighthouse node (the best kind).
    Lighthouse,
    LighthouseOld,
    /// A Nimbus node.
    Nimbus,
    /// A Teku node.
    Teku,
    /// A Prysm node.
    Prysm,
    PrysmOld,
    /// A lodestar node.
    Lodestar,
    /// An unknown client.
    Unknown,
}

impl Default for Client {
    fn default() -> Self {
        Client {
            kind: ClientKind::Unknown,
            version: "unknown".into(),
            os_version: "unknown".into(),
            protocol_version: "unknown".into(),
            agent_string: None,
        }
    }
}

impl Client {
    /// Builds a `Client` from `IdentifyInfo`.
    pub fn from_identify_info(info: &IdentifyInfo) -> Self {
        let (kind, version, os_version) = client_from_agent_version(&info.agent_version);

        Client {
            kind,
            version,
            os_version,
            protocol_version: info.protocol_version.clone(),
            agent_string: Some(info.agent_version.clone()),
        }
    }
}

impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ClientKind::Lighthouse => write!(
                f,
                "Lighthouse: version: {}, os_version: {}",
                self.version, self.os_version
            ),
            ClientKind::LighthouseOld => write!(
                f,
                "Lighthouse Old: version: {}, os_version: {}",
                self.version, self.os_version
            ),
            ClientKind::Teku => write!(
                f,
                "Teku: version: {}, os_version: {}",
                self.version, self.os_version
            ),
            ClientKind::Nimbus => write!(
                f,
                "Nimbus: version: {}, os_version: {}",
                self.version, self.os_version
            ),
            ClientKind::Prysm => write!(
                f,
                "Prysm: version: {}, os_version: {}",
                self.version, self.os_version
            ),
            ClientKind::PrysmOld => write!(
                f,
                "Prysm Old: version: {}, os_version: {}",
                self.version, self.os_version
            ),

            ClientKind::Lodestar => write!(f, "Lodestar: version: {}", self.version),
            ClientKind::Unknown => {
                if let Some(agent_string) = &self.agent_string {
                    write!(f, "Unknown: {}", agent_string)
                } else {
                    write!(f, "Unknown")
                }
            }
        }
    }
}

impl std::fmt::Display for ClientKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// helper function to identify clients from their agent_version. Returns the client
// kind and it's associated version and the OS kind.
fn client_from_agent_version(agent_version: &str) -> (ClientKind, String, String) {
    let mut agent_split = agent_version.split('/');
    match agent_split.next() {
        Some("Lighthouse") => {
            let mut kind = ClientKind::LighthouseOld;
            let mut version = String::from("unknown");
            let mut os_version = version.clone();
            if let Some(agent_version) = agent_split.next() {
                version = agent_version.into();
                if let Some(agent_os_version) = agent_split.next() {
                    os_version = agent_os_version.into();
                }

                let mut version_split = version.split("-");
                if let Some(version_parts) = version_split.next() {
                    let mut version_parts_split = version_parts.split(".");
                    if let Some(part1) = version_parts_split.next() {
                        if part1.eq("v0") {
                            if let Some(part2) = version_parts_split.next() {
                                if let Ok(part2_i) = part2.parse::<i32>() {
                                    if part2_i > 2 {
                                        kind = ClientKind::Lighthouse;
                                    } else if part2_i == 2 {
                                        if let Some(part3) = version_parts_split.next() {
                                            if let Ok(part3_i) = part3.parse::<i32>() {
                                                if part3_i >= 10 {
                                                    kind = ClientKind::Lighthouse;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            (kind, version, os_version)
        }
        Some("teku") => {
            let kind = ClientKind::Teku;
            let mut version = String::from("unknown");
            let mut os_version = version.clone();
            if agent_split.next().is_some() {
                if let Some(agent_version) = agent_split.next() {
                    version = agent_version.into();
                    if let Some(agent_os_version) = agent_split.next() {
                        os_version = agent_os_version.into();
                    }
                }
            }
            (kind, version, os_version)
        }
        Some("github.com") => {
            let kind = ClientKind::PrysmOld;
            let unknown = String::from("unknown");
            (kind, unknown.clone(), unknown)
        }
        Some("Prysm") => {
            let mut kind =  ClientKind::PrysmOld;
            let mut version = String::from("unknown");
            let mut os_version = version.clone();
            if let Some(alpha_version) = agent_split.next() {
                let mut alpha_split = alpha_version.split("-alpha.");
                if let Some(agent_version) = agent_split.next() {
                    version = agent_version.into();
                    if let Some(agent_os_version) = agent_split.next() {
                        os_version = agent_os_version.into();
                    }
                }
                if alpha_split.next().is_some() {
                    if let Some(a_version) = alpha_split.next() {
                        if let Ok(a_v) = a_version.parse::<i32>() {
                            if a_v > 26 || (a_v == 26
                                && version != "22bcfd2c340a8cb682468e5a7112404aee1e3849"
                                && version != "ba440abe2de1b1d77ffb2c0ec3507bdd19475b18"
                                && version != "09640ae22df05d9de4c4c69551a8f39c936a92b7"
                                && version != "719e99ffd9d4dcb6dda56a21d00656b81b297ce5"
                                && version != "1a4129f5a6c6d30fd6710e763a85073f87d884d0"
                                && version != "de3f112a0500234a27d9cd8b29a60dc26fba6394"
                                && version != "3734bfacce3b927c56c83c7acbbf24b4b8a76835"
                                && version != "d5e2b51d6631e82275a46b141c60899e4256f9a3"
                                && version != "cdd28abc4b2c9df035e136f7b6e31788180adb71"
                                && version != "8c8f59e242eaf35f27dfb195367df4aab5f82ac8"
                                && version != "b1f9f97062fccaf7d0c02ed0b69225577e254c40"
                                && version != "7545d3f2b39a6de5817e8895a0a1e83844d086d7"
                                && version != "bdf8bf7be27358b7325854ed9d3bb8c03d853413"
                                && version != "b928e9531c6b5b7ad4b8548404fd3e0867966d56"
                                && version != "1f6afa85475a43cc3e1e9e6fd954c36a7a4129a8"
                                && version != "303edbde58c2a6883ee59ec8d57eb7d53e5ad867"
                            ) {
                                kind = ClientKind::Prysm;
                            }
                        }
                    }
                }
            }
            (kind, version, os_version)
        }
        Some("nim-libp2p") => {
            let kind = ClientKind::Nimbus;
            let mut version = String::from("unknown");
            let mut os_version = version.clone();
            if let Some(agent_version) = agent_split.next() {
                version = agent_version.into();
                if let Some(agent_os_version) = agent_split.next() {
                    os_version = agent_os_version.into();
                }
            }
            (kind, version, os_version)
        }
        Some("js-libp2p") => {
            let kind = ClientKind::Lodestar;
            let mut version = String::from("unknown");
            let mut os_version = version.clone();
            if let Some(agent_version) = agent_split.next() {
                version = agent_version.into();
                if let Some(agent_os_version) = agent_split.next() {
                    os_version = agent_os_version.into();
                }
            }
            (kind, version, os_version)
        }
        _ => {
            let unknown = String::from("unknown");
            (ClientKind::Unknown, unknown.clone(), unknown)
        }
    }
}
