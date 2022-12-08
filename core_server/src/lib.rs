pub mod gamedata;
pub mod server;

pub mod prelude {
    pub use crate::gamedata::GameData;
    pub use crate::gamedata::behavior::*;
    pub use crate::gamedata::region::*;
    pub use crate::gamedata::item::*;

    pub use crate::server::Server;
    pub use crate::server::region_instance::RegionInstance;
    pub use crate::server::region_pool::RegionPool;
    pub use crate::server::message::Message;
    pub use crate::server::nodes::*;
    pub use crate::server::nodes::behavior::*;
    pub use crate::server::nodes::game::*;
    pub use crate::server::nodes::area::*;
    pub use crate::server::nodes::player::*;
    pub use crate::server::nodes::item::*;
    pub use crate::server::nodes::system::*;
    pub use crate::server::nodes::utilities::*;
    pub use crate::server::script_utilities::*;

    pub use core_shared::prelude::*;

    pub use std::collections::HashMap;

    pub use rand::prelude::*;

    pub use core_shared::prelude::*;

    pub use uuid::Uuid;

    pub use std::fs;
    pub use std::path;
    pub use std::path::PathBuf;

    pub use serde::{Deserialize, Serialize};
    pub use rustc_hash::FxHashMap;
}
