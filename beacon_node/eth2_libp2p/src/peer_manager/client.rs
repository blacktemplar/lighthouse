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
            let kind = ClientKind::Lighthouse;
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
                            if a_v > 24 || (a_v == 24 && (
                                    version == "a81214219dc57caa6a812ef5b656e910c1df37e3"
                                        || version == "3de626f0d0d24179b4e4c42a27509e9651605d3a"
                                        || version == "51f2cc18e5daca2c59b875cd8b380295888fc109"
                                        || version == "f04fffb5fe4623660c32969735529da3df456209"
                                        || version == "f4848e46d40862d3d16537f7a287db5c93b89215"
                                        || version == "8baa22f0650b1c6bbb8bfeb54b2abc8bf05b51c5"
                                        || version == "cb1f44872d806f095b40b0dac706a52478fbec85"
                                        || version == "bd46abc71dc984f3c8d5e57ce744e006ae603535"
                                        || version == "593442a0fa5c90db83edb32d13d6282d67d5792a"
                                        || version == "6d837705340e3828ec4757fd8f4594a60a5f3dca"
                                        || version == "a74cf5de909ed1d5a4638af9638d7a158fa700f8"
                                        || version == "94fa046ce1d6d11d47a318eefd76dad5ba91dc50"
                                        || version == "c8e93f87893d41ce7335de0c1ae2c786283d516a"
                                )) {
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
