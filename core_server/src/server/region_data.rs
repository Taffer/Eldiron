use crate::prelude::*;

pub struct RegionData {
    /// The character sheets in the region
    pub sheets                      : Vec<Sheet>,

    /// Behavior Instance Data
    pub character_instances         : Vec<BehaviorInstance>,

    /// Holds the tile data and areas for the region
    pub region_data                 : GameRegionData,

    /// The behavior graphs for the regions area
    pub region_area_behavior        : Vec<GameBehaviorData>,

    /// The displacements for this region
    pub displacements               : FxHashMap<(isize, isize), TileData>,

    /// Do characters move per tile or per pixel ?
    pub pixel_based_movement        : bool,

    /// The loot in the region
    pub loot                        : FxHashMap<(isize, isize), Vec<Item>>,

    /// The node functions
    pub nodes                       : FxHashMap<BehaviorNodeType, NodeDataCall>,

    /// The text of the current movement
    pub action_direction_text       : String,


    /// During action execution for regions this indicates the calling behavior index
    pub curr_action_inst_index      : Option<usize>,

    /// The current instance index of the current "Player" when executing the Game behavior per player
    pub curr_player_inst_index      : usize,

    /// The index of the game instance
    game_instance_index             : Option<usize>,

    /// Player uuid => player instance index
    pub player_uuid_indices         : FxHashMap<Uuid, usize>,

    /// Current characters per region
    pub characters                  : FxHashMap<Uuid, Vec<CharacterData>>,

    // Characters instance indices in a given area
    pub area_characters             : FxHashMap<usize, Vec<usize>>,

    // The character instances from the previous tick, used to figure out onEnter, onLeave etc events
    pub prev_area_characters        : FxHashMap<usize, Vec<usize>>,

    // Lights for this region
    pub lights                      : Vec<LightData>,

    /// Tick count used for timing
    pub tick_count                  : usize,

    /// How many ticks for one minute (gets read from the game settings)
    pub ticks_per_minute            : usize,

    /// The current character sheet index
    pub curr_index                  : usize,

    /// The current area behavior index sheet index
    pub curr_area_index             : usize,
}

impl RegionData {
    pub fn new() -> Self {

        let mut nodes : FxHashMap<BehaviorNodeType, NodeDataCall> = FxHashMap::default();

        // BEHAVIOR
        nodes.insert(BehaviorNodeType::Script, node_script);
        nodes.insert(BehaviorNodeType::Message, node_message);
        nodes.insert(BehaviorNodeType::Audio, node_audio);
        nodes.insert(BehaviorNodeType::HasTarget, node_has_target);
        nodes.insert(BehaviorNodeType::RandomWalk, node_random_walk);

        // PLAYER
        nodes.insert(BehaviorNodeType::Action, node_player_action);
        nodes.insert(BehaviorNodeType::Move, node_player_move);

        // REGION
        nodes.insert(BehaviorNodeType::Always, node_always_area);
        nodes.insert(BehaviorNodeType::ActionArea, node_action_area);
        nodes.insert(BehaviorNodeType::EnterArea, node_enter_area);
        nodes.insert(BehaviorNodeType::InsideArea, node_inside_area);
        nodes.insert(BehaviorNodeType::LeaveArea, node_leave_area);
        nodes.insert(BehaviorNodeType::TeleportArea, node_teleport_area);
        nodes.insert(BehaviorNodeType::MessageArea, node_message_area);
        nodes.insert(BehaviorNodeType::AudioArea, node_audio_area);
        nodes.insert(BehaviorNodeType::LightArea, node_light_area);

        Self {
            sheets                  : vec![],
            character_instances     : vec![],
            region_data             : GameRegionData::new(),
            region_area_behavior    : vec![],
            displacements           : FxHashMap::default(),
            pixel_based_movement    : true,
            loot                    : FxHashMap::default(),

            nodes,

            action_direction_text   : "".to_string(),

            curr_action_inst_index          : None,
            curr_player_inst_index          : 0,
            game_instance_index             : None,
            player_uuid_indices             : FxHashMap::default(),

            characters                      : FxHashMap::default(),
            area_characters                 : FxHashMap::default(),
            prev_area_characters            : FxHashMap::default(),
            lights                          : vec![],

            tick_count                      : 5 * 60 * 4, // 5am
            ticks_per_minute                : 4,

            curr_index              : 0,
            curr_area_index         : 0,
        }
    }

    /// Returns the layered tiles at the given position and checks for displacements
    pub fn get_tile_at(&self, pos: (isize, isize)) -> Vec<TileData> {
        let mut rc = vec![];
        if let Some(t) = self.displacements.get(&pos) {
            rc.push(t.clone());
        } else {
            if let Some(t) = self.region_data.layer1.get(&pos) {
                rc.push(t.clone());
            }
            if let Some(t) = self.region_data.layer2.get(&pos) {
                rc.push(t.clone());
            }
            if let Some(t) = self.region_data.layer3.get(&pos) {
                rc.push(t.clone());
            }
            if let Some(t) = self.region_data.layer4.get(&pos) {
                rc.push(t.clone());
            }
        }
        rc
    }

    /// Returns the current position of the character index, takes into account an ongoing animation
    pub fn get_instance_position(&self, inst_index: usize) -> Option<Position> {
        if let Some(old_position) = &self.character_instances[inst_index].old_position {
            return Some(old_position.clone());
        }
        self.character_instances[inst_index].position.clone()
    }
}