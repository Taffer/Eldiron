use theframework::*;

pub mod browser;
pub mod editor;
pub mod misc;
pub mod project;
pub mod region;
pub mod sidebar;
pub mod tilemap;

pub mod prelude {
    pub use ::serde::{Deserialize, Serialize};
    pub use theframework::prelude::*;

    pub use crate::browser::*;
    pub use crate::misc::*;
    pub use crate::project::*;
    pub use crate::region::*;
    pub use crate::sidebar::*;
    pub use crate::tilemap::*;
}

use crate::editor::Editor;

fn main() {
    // std::env::set_var("RUST_BACKTRACE", "1");

    let editor = Editor::new();
    let mut app = TheApp::new();

    _ = app.run(Box::new(editor));
}
