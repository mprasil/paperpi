use std::fmt;
use reqwest::{Client, StatusCode};
use serde;
use ui::{UiMsg, UiSender};

const API_HOST: &str = "https://api.paperspace.io";
const KEY_HEADER: &str = "x-api-key";

// Aliases for convenience
pub type PsSender = crossbeam_channel::Sender<PsMsg>;
pub type PsReceiver = crossbeam_channel::Receiver<PsMsg>;

#[derive(Debug)]
pub enum Error {
    Parse(String)
}

impl ::std::fmt::Display for Error {
   fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
       write!(f, "{:?}", self)
   }
}

serializable_enum! {
    /// Supported machine states
    #[derive(Debug, PartialEq)]
    pub enum MachineState {
        /// off
        Off,
        /// provisioning
        Provisioning,
        /// ready
        Ready,
        /// restarting
        Restarting,
        /// serviceready
        ServiceReady,
        /// starting
        Starting,
        /// stopping
        Stopping,
        /// upgrading
        Upgrading,
    }
    MachineStateVisitor
}

impl_as_ref_from_str! {
    MachineState {
        Off => "off",
        Provisioning => "provisioning",
        Ready => "ready",
        Restarting => "restarting",
        ServiceReady => "serviceready",
        Starting => "starting",
        Stopping => "stopping",
        Upgrading => "upgrading",
    }
    Error::Parse
}

impl fmt::Display for MachineState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub enum Response<T> {
    Ok(T),
    Err(String)
}

#[derive(Serialize,Deserialize,Debug)]
pub struct Machine {
    pub id: String,
    pub name: String,
    pub state: MachineState,
}

pub type Machines = Vec<Machine>;

#[derive(Clone)]
pub struct Paperspace {
    token: String
}

pub enum PsMsg {
    ReloadServers,
    StartServer(String),
    StopServer(String),
    RestartServer(String),
}

impl Paperspace {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string()
        }
    }

    pub fn run(&self, receiver: PsReceiver, sender: UiSender) {
        for message in receiver {
            match message {
                PsMsg::ReloadServers => {
                    sender.send(UiMsg::ShowLoadingPopup).unwrap();
                    match self.get::<Machines>("/machines/getMachines") {
                        Response::Ok(machines) => {
                            sender.send(UiMsg::ReloadServerList(machines)).unwrap();
                        },
                        Response::Err(error) => {
                            sender.send(UiMsg::ShowRefreshError(error)).unwrap();
                        }
                    }
                    sender.send(UiMsg::HideLoadingPopup).unwrap();
                },
                PsMsg::StartServer(server_id) => {
                    sender.send(UiMsg::ShowLoadingPopup).unwrap();
                    self.start_server(&server_id);
                    sender.send(UiMsg::HideLoadingPopup).unwrap();
                },
                PsMsg::StopServer(server_id) => {
                    sender.send(UiMsg::ShowLoadingPopup).unwrap();
                    self.stop_server(&server_id);
                    sender.send(UiMsg::HideLoadingPopup).unwrap();
                },
                PsMsg::RestartServer(server_id) => {
                    sender.send(UiMsg::ShowLoadingPopup).unwrap();
                    self.restart_server(&server_id);
                    sender.send(UiMsg::HideLoadingPopup).unwrap();
                },
            }
        }
    }

    pub fn get<T:serde::de::DeserializeOwned>(&self, action_url: &str) -> Response<T> {
        match Client::new()
        .get(&format!("{}{}", API_HOST, action_url))
        .header(KEY_HEADER, self.token.clone())
        .send() {
            Err(error) => Response::Err(format!("Request failed: {}", error)),
            Ok(mut response) => {
                match response.json() {
                    Err(error) => Response::Err(format!("Failed to parse json: {}", error)),
                    Ok(response) => Response::Ok(response)
                }
            }
        }
    }

    pub fn post(&self, action_url: &str) -> Response<StatusCode> {
        match Client::new()
        .post(&format!("{}{}", API_HOST, action_url))
        .header(KEY_HEADER, self.token.clone())
        .send() {
            Err(error) => Response::Err(format!("Request failed: {}", error)),
            Ok(mut response) => {
                if response.status().is_success() {
                    Response::Ok(response.status())
                } else {
                    Response::Err(format!(
                        "Request failed: [{}] {}",
                        response.status().as_str(),
                        response.text().unwrap_or_default()))
                }
            }
        }
    }

    pub fn start_server(&self, server_id: &str) -> Response<StatusCode> {
        self.post(&format!("/machines/{}/start",server_id))
    }

    pub fn stop_server(&self, server_id: &str) -> Response<StatusCode> {
        self.post(&format!("/machines/{}/stop",server_id))
    }

    pub fn restart_server(&self, server_id: &str) -> Response<StatusCode> {
        self.post(&format!("/machines/{}/restart",server_id))
    }
}