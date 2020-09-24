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
                            if a_v > 24 || (a_v == 24
                                && version != "245c18784eda370ea3218e8704651edad763978d"
                                && version != "9219dc77ff6d99c18a618d5d86dc12279576238e"
                                && version != "1de230110d6bfc544fb753928d3e8d32284d54d8"
                                && version != "b6607fac25414d99d1e3c90dc002feba14dfb783"
                                && version != "e6277ec536c3891024e230d3439728987978392e"
                                && version != "0961fef727413165b418028f45eea9816cfc929d"
                                && version != "366b98ac83b38b38527dac6275e4b5ac270298cc"
                                && version != "c2425e81d79a17f19e831b383ae68cfcfdaf253e"
                                && version != "8f2950d374bb4e0e0fcfa954c7931116552f52e6"
                                && version != "f4a68643439f3db856ca6e027f6afdf6b63f0cf7"
                                && version != "c1a7c65e0516aaa33ffe1cd9169b24d1805b1a65"
                                && version != "7fd2536d54f924a675b5b3ba4f9fc6bd51fe88ac"
                                && version != "0e6797d80d7d4dfaa381c93a0b52fbb902e231af"
                                && version != "b2b4c2660df92600f535abb19c896c89e11fd740"
                                && version != "787857c38b26bd6fdb25b06bec33b5726f04b2f1"
                                && version != "1cc21ed3c42cf4ccf2c5b2740378fa4b32a61a8e"
                                && version != "7588e491ab1c77afb9d589681d72adf841c46eaa"
                                && version != "afce363e00f875a3ed86ea18f1d005234f5625cd"
                                && version != "7de3ce0b31a0c5bec47010a60d45d1ea0328a572"
                                && version != "6e6b871cc157550351878ac7c7750505bc76c725"
                                && version != "2349012bd007fdd6f7cd6e1f1ed2eb05adf6cc6e"
                                && version != "b4c0a89d49c7bc7d20aedb69a09494ae0b61572e"
                                && version != "d368156a08a282085255c4e1a916903a400f492d"
                                && version != "63149a3dc3787e73380d8f807767d2b5124960c0"
                                && version != "fbe088625aae04b6616250d46452fc0471b4324e"
                                && version != "ecbab20bad63829ef4bd1ecc15f2c5085b145fea"
                                && version != "381b5be0fc917d326b482670f672c398cc10c665"
                                && version != "6803f3308a8994b1f9f00b53872c3a49c4a80977"
                                && version != "60558b7970a9aefafc79407410cc0ecc97b61b40"
                                && version != "7854b91ae035b47e754104d2206420ebb9588ee0"
                                && version != "c9c7cc7b9367fc71bc144237cd597eba3bfc9a09"
                                && version != "b538f5073d765093b6affc8dd983ef4130b82375"
                                && version != "3ed7b23ed7e937d31bd2975c34d8df6970e8d6a4"
                                && version != "c2b94d04ed27f908c4f3bb2850b132c50da63219"
                                && version != "3316516d22cfbed13bf0f0b3b31295065acf68f6"
                                && version != "12c1daaf2b32d4b2fd58737681bf877196971d61"
                                && version != "f09620c9f609b36eb8b48bfa0224c6bfa72922a6"
                                && version != "e47e7067c4ff973e6bf64c98cbc049e36c5da4cc"
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
