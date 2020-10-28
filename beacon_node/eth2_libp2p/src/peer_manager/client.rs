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
                                    if part2_i > 3 {
                                        kind = ClientKind::Lighthouse;
                                    } else if part2_i == 3 {
                                        if let Some(part3) = version_parts_split.next() {
                                            if let Ok(part3_i) = part3.parse::<i32>() {
                                                if part3_i >= 1 {
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
            let mut kind = ClientKind::PrysmOld;
            let mut version = String::from("unknown");
            let mut os_version = version.clone();
            if let Some(beta_version) = agent_split.next() {
                let mut beta_split = beta_version.split("-beta.");
                if let Some(agent_version) = agent_split.next() {
                    version = agent_version.into();
                    if let Some(agent_os_version) = agent_split.next() {
                        os_version = agent_os_version.into();
                    }
                }
                if beta_split.next().is_some() {
                    if let Some(a_version) = beta_split.next() {
                        if let Ok(a_v) = a_version.parse::<i32>() {
                            if a_v >= 0 {
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
