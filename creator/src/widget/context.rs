use crate::prelude::*;

use zeno::{Mask, Stroke};
use directories::{ UserDirs };
use std::{fs::File, io::BufReader};

use audio_engine::{AudioEngine, WavDecoder, OggDecoder};

#[derive(PartialEq, Clone)]
pub struct ScreenDragContext {
    pub text                            : String,
    pub color                           : [u8;4],
    pub offset                          : (isize, isize),
    pub buffer                          : Option<[u8; 180 * 32 * 4]>
}

pub struct ScreenContext<'a> {
    pub draw2d                          : Draw2D,

    pub target_fps                      : usize,
    pub default_fps                     : usize,

    pub width                           : usize,
    pub height                          : usize,

    pub data                            : GameData,

    pub switch_editor_state             : Option<EditorState>,

    pub toolbar_height                  : usize,
    pub toolbar_button_height           : usize,
    pub toolbar_button_rounding         : (f64, f64, f64, f64),
    pub toolbar_button_text_size        : f32,

    pub button_height                   : usize,
    pub button_text_size                : f32,
    pub button_rounding                 : (f64, f64, f64, f64),

    pub node_button_height              : usize,
    pub node_button_text_size           : f32,
    pub node_button_header_text_size    : f32,
    pub node_button_rounding            : (f64, f64, f64, f64),
    pub node_connector_color            : [u8;4],

    pub large_button_height             : usize,
    pub large_button_text_size          : f32,
    pub large_button_rounding           : (f64, f64, f64, f64),

    pub color_black                     : [u8;4],
    pub color_gray                      : [u8;4],
    pub color_light_gray                : [u8;4],
    pub color_white                     : [u8;4],
    pub color_light_white               : [u8;4],
    pub color_orange                    : [u8;4],
    pub color_light_orange              : [u8;4],
    pub color_green                     : [u8;4],
    pub color_light_green               : [u8;4],
    pub color_blue                      : [u8;4],
    pub color_light_blue                : [u8;4],

    pub color_node_light_gray           : [u8;4],
    pub color_node_dark_gray            : [u8;4],
    pub color_node_picker               : [u8;4],

    pub color_red                       : [u8;4],
    pub color_green_conn                : [u8;4],

    pub curr_tileset_index              : usize,
    pub curr_tile                       : Option<(usize, usize)>,
    pub selection_end                   : Option<(usize, usize)>,

    pub curr_region_index               : usize,
    pub curr_region_area_index          : usize,
    pub curr_region_tile                : Option<TileData>,

    pub curr_behavior_index             : usize,
    pub curr_systems_index              : usize,
    pub curr_items_index                : usize,

    pub drag_context                    : Option<ScreenDragContext>,

    pub curr_graph_type                 : BehaviorType,

    pub dialog_state                    : DialogState,
    pub dialog_height                   : usize,
    pub dialog_entry                    : DialogEntry,
    pub dialog_node_behavior_id         : (Uuid, Uuid, String),
    pub dialog_node_behavior_value      : (f64, f64, f64, f64, String),
    pub dialog_value                    : Value,
    pub dialog_new_name_type            : String,
    pub dialog_new_name                 : String,
    pub dialog_new_node_position        : (isize, isize),
    pub dialog_tile_usage               : Vec<TileUsage>,
    pub dialog_accepted                 : bool,

    pub dialog_position_state           : DialogState,

    pub code_editor_state               : CodeEditorWidgetState,
    pub code_editor_is_active           : bool,
    pub code_editor_visible_y           : usize,
    pub code_editor_just_opened         : bool,
    pub code_editor_mode                : CodeEditorMode,
    pub code_editor_update_node         : bool,
    pub code_editor_value               : String,
    pub code_editor_node_behavior_id    : (Uuid, Uuid, String),
    pub code_editor_node_behavior_value : Value,
    pub code_editor_error               : Option<(String, Option<usize>)>,

    pub active_position_id              : Option<(Uuid, Uuid, String)>,

    pub jump_to_position                : Option<Position>,

    pub is_running                      : bool,
    pub is_debugging                    : bool,
    pub just_stopped_running            : bool,

    pub player_id                       : Uuid,

    // Server

    pub server                          : Option<core_server::server::Server<'a>>,
    pub player_uuid                     : uuid::Uuid,

    // Masks
    pub left_arrow_mask                 : [u8;12*18],
    pub right_arrow_mask                : [u8;12*18],
    pub left_arrow_mask_small           : [u8;8*12],
    pub right_arrow_mask_small          : [u8;8*12],
    pub menu_triangle_mask              : [u8;10*10],
    pub preview_arc_mask                : [u8;20*20],
    pub menu_mask                       : [u8;20*20],
    pub open_mask                       : [u8;20*20],

    pub curr_project_path               : std::path::PathBuf,

    // Hover Help

    pub hover_help_pos                  : Option<(usize, usize)>,
    pub hover_help_pos_last             : Option<(usize, usize)>,
    pub hover_help_counter              : usize,
    pub hover_help_target               : usize,
    pub hover_help_title                : Option<String>,
    pub hover_help_text                 : Option<String>,

    // Debug renderer
    pub debug_render                    : Option<GameRender<'a>>,

    // Debug log

    pub debug_log_variables             : Vec<(String, Value)>,
    pub debug_log_messages              : Vec<MessageData>,
    pub debug_log_inventory             : Inventory,

    // Icons

    pub icons                           : FxHashMap<String, (Vec<u8>, u32, u32)>,

    // Icons

    pub scripts                         : FxHashMap<String, String>,

    // Audio

    pub audio_engine                    : Option<AudioEngine<Group>>,
}

impl ScreenContext<'_> {

    pub fn new(width: usize, height: usize) -> Self {

        fn load_icon(file_name: &PathBuf) -> Option<(Vec<u8>, u32, u32)> {

            let decoder = png::Decoder::new(File::open(file_name).unwrap());
            if let Ok(mut reader) = decoder.read_info() {
                let mut buf = vec![0; reader.output_buffer_size()];
                let info = reader.next_frame(&mut buf).unwrap();
                let bytes = &buf[..info.buffer_size()];

                return Some((bytes.to_vec(), info.width, info.height));
            }
            None
        }

        // Icons

        let mut icons : FxHashMap<String, (Vec<u8>, u32, u32)> = FxHashMap::default();
        let icon_path = get_resource_dir().join("resources").join("icons");
        let paths: Vec<_> = fs::read_dir(icon_path.clone()).unwrap()
                                                .map(|r| r.unwrap())
                                                .collect();
        for path in paths {
            let path = &path.path();
            if let Some(icon) = load_icon(&path) {
                icons.insert(path::Path::new(&path).file_stem().unwrap().to_str().unwrap().to_string(), icon);
            }
        }

        // Scripts

        let mut scripts : FxHashMap<String, String> = FxHashMap::default();
        let scripts_path = get_resource_dir().join("resources").join("scripts");
        let paths: Vec<_> = fs::read_dir(scripts_path.clone()).unwrap()
                                                .map(|r| r.unwrap())
                                                .collect();
        for path in paths {
            let path = &path.path();
            if let Some(script) = fs::read_to_string(path).ok() {
                scripts.insert(path::Path::new(&path).file_stem().unwrap().to_str().unwrap().to_string(), script);
            }
        }

        // Masks

        let mut left_arrow_mask = [0u8; 12 * 18];
        Mask::new("M 12,0 0,9 12,18")
            .size(12, 18)
            .style(
                Stroke::new(2.0)
            )
            .render_into(&mut left_arrow_mask, None);

        let mut right_arrow_mask = [0u8; 12 * 18];
        Mask::new("M 0,0 12,9 0,18")
            .size(12, 18)
            .style(
                Stroke::new(2.0)
            )
            .render_into(&mut right_arrow_mask, None);

        let mut left_arrow_mask_small = [0u8; 8 * 12];
        Mask::new("M 8,0 0,6 8,12")
            .size(8, 12)
            .style(
                Stroke::new(2.0)
            )
            .render_into(&mut left_arrow_mask_small, None);

        let mut right_arrow_mask_small = [0u8; 8 * 12];
        Mask::new("M 0,0 8,6 0,12")
            .size(8, 12)
            .style(
                Stroke::new(2.0)
            )
            .render_into(&mut right_arrow_mask_small, None);

        let mut menu_triangle_mask = [0u8; 10 * 10];
        Mask::new("M 0,0 10,0 5,7 0,0 Z")
            .size(10, 10)
            .render_into(&mut menu_triangle_mask, None);

        let mut preview_arc_mask = [0u8; 20 * 20];
        Mask::new("M 18,18 C0,16 2,4 1,0")
            .size(20, 20)
            .style(
                Stroke::new(1.0)
            )
            .render_into(&mut preview_arc_mask, None);

        let mut menu_mask = [0u8; 20 * 20];
        Mask::new("M 0,4 L 19, 4 M 0, 10 L 19, 10, M 0,16 L 19, 16")
            .size(20, 20)
            .style(
                Stroke::new(1.0)
            )
            .render_into(&mut menu_mask, None);

        let mut open_mask = [0u8; 20 * 20];
        Mask::new("M 0,4 L 19, 4 M 0, 10 L 19, 10, M 0,16 L 19, 16")
            .size(20, 20)
            .style(
                Stroke::new(1.0)
            )
            .render_into(&mut open_mask, None);

        Self {
            draw2d                      : Draw2D::new(),

            target_fps                  : 4,
            default_fps                 : 4,

            width, height,

            data                        : GameData::new(),
            switch_editor_state         : None,

            // Editor statics
            toolbar_height              : 44 * 2,
            toolbar_button_height       : 35,
            toolbar_button_rounding     : (18.0, 18.0, 18.0, 18.0),
            toolbar_button_text_size    : 19.0,

            button_height               : 25,
            button_text_size            : 18.0,
            button_rounding             : (12.0, 12.0, 12.0, 12.0),

            large_button_height         : 30,
            large_button_text_size      : 20.0,
            large_button_rounding       : (14.0, 14.0, 14.0, 14.0),

            node_button_height          : 24,
            node_button_text_size       : 16.0,
            node_button_header_text_size: 12.0,
            node_button_rounding        : (12.0, 12.0, 12.0, 12.0),
            node_connector_color        : [174, 174, 174, 255],

            color_black                 : [25, 25, 25, 255],
            color_white                 : [255, 255, 255, 255],
            color_light_white           : [240, 240, 240, 255],
            color_gray                  : [105, 105, 105, 255],
            color_light_gray            : [155, 155, 155, 255],
            color_orange                : [208, 115, 50, 255],
            color_light_orange          : [208, 156, 112, 255],
            color_green                 : [10, 93, 80, 255],
            color_light_green           : [101, 140, 134, 255],
            color_red                   : [207, 55, 54, 255],
            color_green_conn            : [20, 143, 40, 255],
            color_blue                  : [27, 79, 136, 255],
            color_light_blue            : [78, 103, 145, 255],

            color_node_light_gray       : [102, 102, 102, 255],
            color_node_dark_gray        : [48, 48, 48, 255],
            color_node_picker           : [186, 186, 186, 255],

            // Editor state

            drag_context                : None,

            curr_graph_type             : BehaviorType::Behaviors,

            // Tiles
            curr_tileset_index          : 0,
            curr_tile                   : None,
            selection_end               : None,

            // Regions / Areas
            curr_region_index           : 0,
            curr_region_area_index      : 0,
            curr_region_tile            : None,

            // Behaviors
            curr_behavior_index         : 0,

            // Systems
            curr_systems_index          : 0,

            // Items
            curr_items_index            : 0,

            dialog_state                : DialogState::Closed,
            dialog_height               : 0,
            dialog_entry                : DialogEntry::None,
            dialog_node_behavior_id     : (Uuid::new_v4(), Uuid::new_v4(), "".to_string()),
            dialog_node_behavior_value  : (0.0, 0.0, 0.0, 0.0, "".to_string()),
            dialog_value                : Value::Empty(),
            dialog_new_name_type        : "".to_string(),
            dialog_new_name             : "".to_string(),
            dialog_new_node_position    : (0,0),
            dialog_tile_usage           : vec![],
            dialog_accepted             : false,

            dialog_position_state       : DialogState::Closed,

            code_editor_state                : CodeEditorWidgetState::Closed,
            code_editor_is_active            : false,
            code_editor_visible_y            : 0,
            code_editor_just_opened          : false,
            code_editor_mode                 : CodeEditorMode::Rhai,
            code_editor_update_node          : false,
            code_editor_value                : "".to_string(),
            code_editor_node_behavior_id     : (Uuid::new_v4(), Uuid::new_v4(), "".to_string()),
            code_editor_node_behavior_value  : Value::Empty(),
            code_editor_error                : None,

            active_position_id          : None,
            jump_to_position            : None,

            is_running                  : false,
            is_debugging                : false,
            just_stopped_running        : false,

            player_id                   : uuid::Uuid::new_v4(),

            server                      : None,
            player_uuid                 : uuid::Uuid::new_v4(),

            // UI Masks
            left_arrow_mask,
            right_arrow_mask,
            left_arrow_mask_small,
            right_arrow_mask_small,
            menu_triangle_mask,
            preview_arc_mask,
            menu_mask,
            open_mask,

            curr_project_path           : get_resource_dir(),

            hover_help_pos              : None,
            hover_help_pos_last         : None,
            hover_help_counter          : 0,
            hover_help_target           : 5,
            hover_help_title            : None,
            hover_help_text             : None,

            debug_render                : None,
            debug_log_variables         : vec![],
            debug_log_messages          : vec![],
            debug_log_inventory         : Inventory::new(),

            icons,
            scripts,

            audio_engine                : None,
        }
    }

    /// Returns true if the given rect contains the given position
    pub fn contains_pos_for(&self, pos: (usize, usize), rect: (usize, usize, usize, usize)) -> bool {
        if pos.0 >= rect.0 && pos.0 < rect.0 + rect.2 && pos.1 >= rect.1 && pos.1 < rect.1 + rect.3 {
            true
        } else {
            false
        }
    }

    /// Returns true if the given rect (with an isize offset) contains the given position
    pub fn contains_pos_for_isize(&self, pos: (usize, usize), rect: (isize, isize, usize, usize)) -> bool {
        if pos.0 as isize >= rect.0 && (pos.0 as isize) < rect.0 + rect.2 as isize && pos.1 as isize >= rect.1 && (pos.1 as isize) < rect.1 + rect.3 as isize {
            true
        } else {
            false
        }
    }

    /// Create a new project
    pub fn create_project(&mut self, name: String) -> Result<std::path::PathBuf, String> {

        if let Some(user_dirs) = UserDirs::new() {
            if let Some(dir) = user_dirs.document_dir() {

                let eldiron_path = dir.join("Eldiron");

                // Check or create "Eldiron" directory
                if fs::metadata(eldiron_path.clone()).is_ok() == false {
                    // have to create dir
                    let rc = fs::create_dir(eldiron_path.clone());

                    if rc.is_err() {
                        return Err("Could not create Eldiron directory.".to_string());
                    }
                }

                // Create project directory
                let project_path = eldiron_path.join(name);
                // Check or create "Eldiron" directory
                if fs::metadata(project_path.clone()).is_ok() == false {
                    // have to create dir
                    let rc = fs::create_dir(project_path.clone());

                    if rc.is_err() {
                        return Err("Could not create project directory.".to_string());
                    }
                }

                // Copy asset directory
                let asset_path = get_resource_dir().join("assets");
                let rc = fs_extra::dir::copy(asset_path, project_path.clone(), &fs_extra::dir::CopyOptions::new());
                if rc.is_err() {
                    return Err("Could not copy 'assets' directory".to_string());
                }

                // Copy game directory
                let game_path = get_resource_dir().join("game");
                let rc = fs_extra::dir::copy(game_path, project_path.clone(), &fs_extra::dir::CopyOptions::new());
                if rc.is_err() {
                    return Err("Could not copy 'game' directory".to_string());
                }

                return Ok(project_path);
            }
        }

        Err("Could not find Documents directory".to_string())
    }

    /// Returns a list of the current projects
    pub fn get_project_list(&self) -> Vec<String> {

        let mut projects: Vec<String> = vec![];

        if let Some(user_dirs) = UserDirs::new() {
            if let Some(dir) = user_dirs.document_dir() {

                let eldiron_path = dir.join("Eldiron");

                // Check or create "Eldiron" directory
                if fs::metadata(eldiron_path.clone()).is_ok() == true {
                    if let Some(paths) = fs::read_dir(eldiron_path).ok() {
                        for path in paths {
                            let path = &path.unwrap().path();
                            if path.is_dir() {
                                let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
                                projects.push(file_name);
                            }
                        }
                    }
                }
            }
        }
        projects
    }

    /// Returns the path for the given project name
    pub fn get_project_path(&self, name: String) -> Option<std::path::PathBuf> {
        if let Some(user_dirs) = UserDirs::new() {
            if let Some(dir) = user_dirs.document_dir() {
                return Some(dir.join("Eldiron").join(name));
            }
        }
        None
    }

    /// Resets the hover help metadata
    pub fn hover_help_reset(&mut self) {
        self.hover_help_pos = None;
        self.hover_help_pos_last = None;
        self.hover_help_counter = 0;
        self.hover_help_title = None;
        self.hover_help_text = None;
    }

    /// Opens the dialog
    pub fn open_dialog(&mut self, id: (Uuid, Uuid, String), value: Value) {
        self.dialog_state = DialogState::Opening;
        self.dialog_node_behavior_id = id;
        self.dialog_value = value;
        self.dialog_height = 0;
        self.target_fps = 60;
    }

    /// Opens the position dialog
    pub fn open_position_dialog(&mut self, id: (Uuid, Uuid, String), value: Value) {
        self.dialog_position_state = DialogState::Opening;
        self.dialog_node_behavior_id = id;
        self.dialog_value = value;
        self.dialog_height = 0;
        self.target_fps = 60;
    }

    /// Opens the position dialog
    pub fn open_code_editor(&mut self, id: (Uuid, Uuid, String), value: Value, anim: bool) {
        if anim {
            if self.code_editor_state != CodeEditorWidgetState::Open {
                self.code_editor_state = CodeEditorWidgetState::Opening;
                self.code_editor_visible_y = 0;
                self.target_fps = 60;
            }
        }
        let string;
        match &value {
            Value::PropertySink(sink) => {
                string = sink.to_string(generate_item_sink_descriptions());
            },
            _ => string = value.to_string_value()
        }
        self.code_editor_value = string.clone();
        self.code_editor_node_behavior_id = id;
        self.code_editor_node_behavior_value = value;
        self.code_editor_is_active = true;
        self.code_editor_just_opened = true;
    }

    /// Creates a property id
    pub fn create_property_id(&mut self, property: &str) -> (Uuid, Uuid, String) {
        (Uuid::new_v4(), Uuid::new_v4(), property.to_string())
    }

    /// Plays the given audio name
    pub fn play_audio(&mut self, name: String, buffered: BufReader<File>) {
        if self.audio_engine.is_none() {
            self.audio_engine = AudioEngine::with_groups::<Group>().ok();
        }

        if let Some(audio_engine) = &self.audio_engine {
            if name.ends_with("wav") {
                if let Some(wav) = WavDecoder::new(buffered).ok() {
                    if let Some(mut sound) = audio_engine.new_sound_with_group(Group::Effect, wav).ok() {
                        sound.play();
                        //audio_engine.set_group_volume(Group::Effect, 0.1);
                    }
                }
            } else
            if name.ends_with("ogg") {
                if let Some(ogg) = OggDecoder::new(buffered).ok() {
                    if let Some(mut sound) = audio_engine.new_sound_with_group(Group::Effect, ogg).ok() {
                        sound.play();
                        //audio_engine.set_group_volume(Group::Effect, 0.1);
                    }
                }
            }
        }
    }
}