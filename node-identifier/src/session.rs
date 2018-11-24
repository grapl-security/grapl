use failure::Error;
use graph_descriptions::*;
use graph_descriptions::graph_description::*;
use mysql::{Pool, Transaction};
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::str;
use uuid;
use mysql::IsolationLevel;

use stopwatch::Stopwatch;
use cache::IdentityCache;

pub trait Session: Debug {
    fn get_table_name(&self) -> &'static str;
    fn get_key_name(&self) -> &'static str;
    fn get_key(&self) -> Cow<str>;
    fn get_asset_id(&self) -> &str;
    fn get_timestamp(&self) -> u64;
    fn get_action(&self) -> Action;
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Create,
    UpdateOrCreate,
    Terminate
}

impl<'a> Session for &'a ProcessDescription {
    fn get_table_name(&self) -> &'static str {
        "process_history"
    }

    fn get_key_name(&self) -> &'static str {
        "pid"
    }

    fn get_key(&self) -> Cow<str> {
        Cow::Owned(self.pid.to_string())
    }

    fn get_asset_id(&self) -> &str {
        self.asset_id()
    }

    fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    fn get_action(&self) -> Action {
        match ProcessState::from(self.state) {
            ProcessState::Created => Action::Create,
            ProcessState::Existing => Action::UpdateOrCreate,
            ProcessState::Terminated => Action::Terminate
        }
    }
}


impl<'a> Session for &'a FileDescription {
    fn get_table_name(&self) -> &'static str {
        "file_history"
    }

    fn get_key_name(&self) -> &'static str {
        "path"
    }

    fn get_key(&self) -> Cow<str> {
        Cow::Borrowed(str::from_utf8(&self.path).expect("Failed utf8 path"))
    }

    fn get_asset_id(&self) -> &str {
        self.asset_id()
    }

    fn get_timestamp(&self) -> u64 {
        self.timestamp
    }

    fn get_action(&self) -> Action {
        match FileState::from(self.state) {
            FileState::Created => Action::Create,
            FileState::Existing => Action::UpdateOrCreate,
            FileState::Deleted => Action::Terminate
        }
    }
}


impl<'a> Session for &'a OutboundConnection {
    fn get_table_name(&self) -> &'static str {
        "connection_history"
    }

    fn get_key_name(&self) -> &'static str {
        "dir_port_ip"
    }

    fn get_key(&self) -> Cow<str> {
        let mut key = String::new();
        key.push_str("outbound");
        key.push_str(&self.port.to_string());
        key.push_str(self.asset_id());
        Cow::Owned(key)
    }

    fn get_asset_id(&self) -> &str {
        self.asset_id()
    }

    fn get_timestamp(&self) -> u64 {
        self.timestamp - (self.timestamp % 10)
    }

    fn get_action(&self) -> Action {
        match ConnectionState::from(self.state) {
            ConnectionState::Created => Action::Create,
            ConnectionState::Existing => Action::UpdateOrCreate,
            ConnectionState::Terminated => Action::Terminate
        }
    }
}

impl<'a> Session for &'a InboundConnection {
    fn get_table_name(&self) -> &'static str {
        "connection_history"
    }

    fn get_key_name(&self) -> &'static str {
        "dir_port_ip"
    }

    fn get_key(&self) -> Cow<str> {
        let mut key = String::new();
        key.push_str("inbound");
        key.push_str(&self.port.to_string());
        key.push_str(self.asset_id());
        Cow::Owned(key)
    }

    fn get_asset_id(&self) -> &str {
        self.asset_id()
    }

    fn get_timestamp(&self) -> u64 {
        self.timestamp - (self.timestamp % 10)
    }

    fn get_action(&self) -> Action {
        match ConnectionState::from(self.state) {
            ConnectionState::Created => Action::Create,
            ConnectionState::Existing => Action::UpdateOrCreate,
            ConnectionState::Terminated => Action::Terminate
        }
    }
}
