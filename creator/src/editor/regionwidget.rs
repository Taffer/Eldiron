use crate::prelude::*;

pub struct RegionWidget {
    pub rect                : (usize, usize, usize, usize),
    pub region_id           : Uuid,

    grid_size               : usize,
    widgets                 : Vec<AtomWidget>,

    area_widgets            : Vec<AtomWidget>,
    editing_widgets         : Vec<AtomWidget>,
    character_widgets       : Vec<AtomWidget>,

    offset                  : (isize, isize),
    screen_offset           : (usize, usize),

    pub tile_selector       : TileSelectorWidget,
    pub character_selector  : CharacterSelectorWidget,

    pub behavior_graph      : Box::<NodeGraph>,

    mouse_wheel_delta       : (isize, isize),

    mouse_hover_pos         : (usize, usize),
    pub clicked             : Option<(isize, isize)>,

    bottom_size             : usize,
    toolbar_size            : usize,
}

impl EditorContent for RegionWidget {

    fn new(_text: Vec<String>, rect: (usize, usize, usize, usize), _behavior_type: BehaviorType, asset: &Asset, context: &ScreenContext) -> Self {

        let toolbar_size = 33;
        let bottom_size = 250;

        let mut widgets = vec![];

        let mut mode_button = AtomWidget::new(vec!["Draw Tiles".to_string(), "Edit Areas".to_string(), "Characters".to_string(), "Settings".to_string()], AtomWidgetType::SliderButton,
        AtomData::new("Mode", Value::Empty()));
        mode_button.atom_data.text = "Mode".to_string();
        mode_button.set_rect((rect.0 + 10, rect.1 + rect.3 - bottom_size - toolbar_size - 5, 200, 40), asset, context);
        mode_button.custom_color = Some([217, 64, 51, 255]);
        mode_button.hover_help_title = Some("Region Mode".to_string());
        mode_button.hover_help_text = Some("Select \"Draw Tiles\" (hotkey 'D') for drawing the tiles in the region. \"Edit Area\" ('E') to create and edit named areas and their behavior. \"Characters\" ('A') to place character instances and \"Settings\" ('S') to edit the settings of the region.".to_string());

        widgets.push(mode_button);

        // Tile Selector
        let mut tile_selector = TileSelectorWidget::new(vec!(), (rect.0, rect.1 + rect.3 - bottom_size, rect.2, bottom_size), asset, &context);
        tile_selector.set_tile_type(vec![TileUsage::Environment], None, None, &asset);

        let character_selector = CharacterSelectorWidget::new(vec!(), (rect.0, rect.1 + rect.3 - bottom_size, rect.2, bottom_size), asset, &context);

        // Graph
        let mut behavior_graph = NodeGraph::new(vec!(), (rect.0, rect.1 + rect.3 - bottom_size, rect.2, bottom_size), BehaviorType::Regions, asset, &context);

        behavior_graph.set_mode(GraphMode::Detail, &context);

        // Area Widgets
        let mut area_widgets : Vec<AtomWidget> = vec![];

        let mut regions_button = AtomWidget::new(vec![], AtomWidgetType::SliderButton,
        AtomData::new("Area", Value::Empty()));
        regions_button.atom_data.text = "Area".to_string();
        regions_button.set_rect((rect.0 + 230, rect.1 + rect.3 - bottom_size - toolbar_size - 5, 180, 40), asset, context);
        regions_button.hover_help_title = Some("Cycles Areas".to_string());
        regions_button.hover_help_text = Some("Cycles through the current areas.".to_string());
        area_widgets.push(regions_button);

        let mut add_area_button = AtomWidget::new(vec!["Add Area".to_string()], AtomWidgetType::Button,
            AtomData::new("Add Area", Value::Empty()));
        add_area_button.set_rect((rect.0 + 230 + 200, rect.1 + rect.3 - bottom_size - toolbar_size - 5, 140, 40), asset, context);
        add_area_button.hover_help_title = Some("Add Area".to_string());
        add_area_button.hover_help_text = Some("Adds a new, empty area.".to_string());
        area_widgets.push(add_area_button);

        let mut del_area_button = AtomWidget::new(vec!["Delete".to_string()], AtomWidgetType::Button,
            AtomData::new("Delete", Value::Empty()));
        del_area_button.state = WidgetState::Disabled;
        del_area_button.set_rect((rect.0 + 230 + 200 + 150, rect.1 + rect.3 - bottom_size - toolbar_size - 5, 140, 40), asset, context);
        del_area_button.hover_help_title = Some("Delete Area".to_string());
        del_area_button.hover_help_text = Some("Deletes the current area.".to_string());
        area_widgets.push(del_area_button);

        let mut rename_area_button = AtomWidget::new(vec!["Rename".to_string()], AtomWidgetType::Button,
            AtomData::new("Rename", Value::Empty()));
        rename_area_button.state = WidgetState::Disabled;
        rename_area_button.set_rect((rect.0 + 230 + 200 + 150 + 150, rect.1 + rect.3 - bottom_size - toolbar_size - 5, 140, 40), asset, context);
        rename_area_button.hover_help_title = Some("Rename Area".to_string());
        rename_area_button.hover_help_text = Some("Renames the current area.".to_string());
        area_widgets.push(rename_area_button);

        let mut area_editing_mode = AtomWidget::new(vec!["Add Tile".to_string(), "Remove".to_string(), "Pick".to_string()], AtomWidgetType::SliderButton,
        AtomData::new("Area", Value::Empty()));
        area_editing_mode.atom_data.text = "Area".to_string();
        area_editing_mode.set_rect((rect.0 +  230 + 200 + 150 + 150 + 150, rect.1 + rect.3 - bottom_size - toolbar_size - 5, 160, 40), asset, context);
        area_editing_mode.hover_help_title = Some("Area Editing Mode".to_string());
        area_editing_mode.hover_help_text = Some("Adds, removes the clicked tile to / from the area or selects the clicked area.".to_string());
        area_widgets.push(area_editing_mode);

        // Character Widgets
        let mut character_widgets : Vec<AtomWidget> = vec![];

        let mut char_editing_mode = AtomWidget::new(vec!["Add Instance".to_string(), "Remove".to_string()], AtomWidgetType::SliderButton,
        AtomData::new("Area", Value::Empty()));
        char_editing_mode.atom_data.text = "Area".to_string();
        char_editing_mode.set_rect((rect.0 + 230, rect.1 + rect.3 - bottom_size - toolbar_size - 5, 190, 40), asset, context);
        character_widgets.push(char_editing_mode);

        // Editing Widgets
        let mut editing_widgets : Vec<AtomWidget> = vec![];

        let mut clear_button = AtomWidget::new(vec!["Clear".to_string()], AtomWidgetType::CheckButton,
            AtomData::new("Clear", Value::Empty()));
        clear_button.set_rect((rect.0 + 230, rect.1 + rect.3 - bottom_size - toolbar_size - 5, 140, 40), asset, context);
        clear_button.hover_help_title = Some("Drawing Mode".to_string());
        clear_button.hover_help_text = Some("Toggles between drawing and clearing modes.\nHotkey: 'C'.".to_string());
        editing_widgets.push(clear_button);

        let mut mode_button = AtomWidget::new(vec!["Pick".to_string()], AtomWidgetType::CheckButton,
        AtomData::new("Pick", Value::Empty()));
        mode_button.atom_data.text = "Pick".to_string();
        mode_button.set_rect((rect.0 + 230 + 150, rect.1 + rect.3 - bottom_size - toolbar_size - 5, 150, 40), asset, context);
        mode_button.hover_help_title = Some("Pick".to_string());
        mode_button.hover_help_text = Some("Picks the next selected tile.\nHotkey: 'P'.".to_string());
        editing_widgets.push(mode_button);

        Self {
            rect,
            region_id               : Uuid::new_v4(),
            grid_size               : 32,

            widgets                 : widgets,

            area_widgets,
            editing_widgets,
            character_widgets,

            offset                  : (0, 0),
            screen_offset           : (0, 0),

            tile_selector,
            character_selector,
            behavior_graph          : Box::new(behavior_graph),

            mouse_wheel_delta       : (0, 0),
            mouse_hover_pos         : (0, 0),
            clicked                 : None,

            bottom_size,
            toolbar_size,
        }
    }

    fn resize(&mut self, width: usize, height: usize, context: &mut ScreenContext) {
        self.rect.2 = width;
        self.rect.3 = height;

        self.widgets[0].set_rect2((self.rect.0 + 10, self.rect.1 + self.rect.3 - self.bottom_size - self.toolbar_size - 5, 200, 40));

        self.area_widgets[0].set_rect2((self.rect.0 + 230, self.rect.1 + self.rect.3 - self.bottom_size - self.toolbar_size - 5, 180, 40));
        self.area_widgets[1].set_rect2((self.rect.0 + 230 + 200, self.rect.1 + self.rect.3 - self.bottom_size - self.toolbar_size - 5, 140, 40));
        self.area_widgets[2].set_rect2((self.rect.0 + 230 + 200 + 150, self.rect.1 + self.rect.3 - self.bottom_size - self.toolbar_size - 5, 140, 40));
        self.area_widgets[3].set_rect2((self.rect.0 + 230 + 200 + 150 + 150, self.rect.1 + self.rect.3 - self.bottom_size - self.toolbar_size - 5, 140, 40));
        self.area_widgets[4].set_rect2((self.rect.0 +  230 + 200 + 150 + 150 + 150, self.rect.1 + self.rect.3 - self.bottom_size - self.toolbar_size - 5, 160, 40));

        self.character_widgets[0].set_rect2((self.rect.0 + 230, self.rect.1 + self.rect.3 - self.bottom_size - self.toolbar_size - 5, 190, 40));

        self.editing_widgets[0].set_rect2((self.rect.0 + 230, self.rect.1 + self.rect.3 - self.bottom_size - self.toolbar_size - 5, 140, 40));
        self.editing_widgets[1].set_rect2((self.rect.0 + 230 + 150, self.rect.1 + self.rect.3 - self.bottom_size - self.toolbar_size - 5, 150, 40));

        self.behavior_graph.rect = (self.rect.0, self.rect.1 + self.rect.3 - self.bottom_size, width, self.bottom_size);
        self.behavior_graph.set_mode_and_rect(GraphMode::Detail, self.behavior_graph.rect, context);
        self.tile_selector.rect = (self.rect.0, self.rect.1 + self.rect.3 - self.bottom_size, width, self.bottom_size);
        self.tile_selector.resize(width, self.bottom_size);
        self.character_selector.rect = (self.rect.0, self.rect.1 + self.rect.3 - self.bottom_size, width, self.bottom_size);
        self.character_selector.resize(width, self.bottom_size);
    }

    fn draw(&mut self, frame: &mut [u8], anim_counter: usize, asset: &mut Asset, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>) {
        context.draw2d.draw_rect(frame, &self.rect, context.width, &[0,0,0,255]);

        if let Some(options) = options {
            let editor_mode = options.get_editor_mode();

            let mut rect = self.rect.clone();
            rect.3 -= self.bottom_size + self.toolbar_size;

            let grid_size = self.grid_size;

            let left_offset = (rect.2 % grid_size) / 2;
            let top_offset = (rect.3 % grid_size) / 2;

            self.screen_offset = (left_offset, top_offset);

            if let Some(region) = context.data.regions.get(&self.region_id) {

                //if context.is_running == false {

                    if editor_mode != RegionEditorMode::Characters {
                        context.draw2d.draw_region(frame, region, &rect, &(-self.offset.0, -self.offset.1), context.width, grid_size, anim_counter, asset);
                    } else {

                        context.draw2d.draw_region_with_behavior(frame, region, &rect, &(-self.offset.0, -self.offset.1), context.width, grid_size, anim_counter, asset, context);

                        /*
                        let x_tiles = (rect.2 / grid_size) as isize;
                        let y_tiles = (rect.3 / grid_size) as isize;

                        for y in 0..y_tiles {
                            for x in 0..x_tiles {
                                let values = region.get_value((x - self.offset.0, y - self.offset.1));

                                if values.is_empty() == false {
                                    let pos = (rect.0 + left_offset + (x as usize) * grid_size, rect.1 + top_offset + (y as usize) * grid_size);
                                    for value in values {
                                        let map = asset.get_map_of_id(value.0);
                                        context.draw2d.draw_animated_tile(frame, &pos, map,context.width,&(value.1, value.2), anim_counter, grid_size);
                                    }
                                }
                            }
                        }*/
                    }
                //} else {
                    //context.draw2d.draw_region_with_instances(frame, region, &rect, &(-self.offset.0, -self.offset.1), context.width, grid_size, anim_counter, asset, context);
                //}
            }

            context.draw2d.draw_rect(frame, &(rect.0, rect.1 + rect.3, rect.2, self.toolbar_size), context.width, &context.color_black);

            for w in &mut self.widgets {
                w.draw(frame, context.width, anim_counter, asset, context);
            }

            if editor_mode == RegionEditorMode::Tiles {
                self.tile_selector.draw(frame, context.width, anim_counter, asset, context);
                for w in &mut self.editing_widgets {
                    w.draw(frame, context.width, anim_counter, asset, context);
                }
            } else
            if editor_mode == RegionEditorMode::Areas {

                for w in &mut self.area_widgets {
                    w.draw(frame, context.width, anim_counter, asset, context);
                }

                if let Some(region) = context.data.regions.get(&self.region_id) {

                    let x_tiles = (rect.2 / grid_size) as isize;
                    let y_tiles = (rect.3 / grid_size) as isize;

                    let curr_area_index = context.curr_region_area_index;

                    for y in 0..y_tiles {
                        for x in 0..x_tiles {

                            let rx = x - self.offset.0;
                            let ry = y - self.offset.1;

                            for area_index in 0..region.data.areas.len() {

                                if region.data.areas[area_index].area.contains(&(rx, ry)) {
                                    let pos = (rect.0 + left_offset + (x as usize) * grid_size, rect.1 + top_offset + (y as usize) * grid_size);

                                    let mut c = context.color_white.clone();
                                    if curr_area_index == area_index {
                                        c = context.color_red.clone();
                                        c[3] = 100;
                                    } else {
                                        //if editor_mode == RegionEditorMode::Areas {
                                        //    continue;
                                        //}
                                        c[3] = 100;
                                    }
                                    context.draw2d.blend_rect(frame, &(pos.0, pos.1, grid_size, grid_size), context.width, &c);
                                }
                            }
                        }
                    }
                }
                self.behavior_graph.draw(frame, anim_counter, asset, context, &mut None);
            } else
            if editor_mode == RegionEditorMode::Characters {
                for w in &mut self.character_widgets {
                    w.draw(frame, context.width, anim_counter, asset, context);
                }
                self.character_selector.draw(frame, context.width, anim_counter, asset, context);
            }

            // Draw a white border around the tile under the mouse cursor
            if self.mouse_hover_pos != (0,0) {
                if let Some(id) = self.get_tile_id(self.mouse_hover_pos) {
                    let pos = (rect.0 + left_offset + ((id.0 + self.offset.0) as usize) * grid_size, rect.1 + top_offset + ((id.1 + self.offset.1) as usize) * grid_size);
                    if  pos.0 + grid_size < rect.0 + rect.2 && pos.1 + grid_size < rect.1 + rect.3 {
                        context.draw2d.draw_rect_outline(frame, &(pos.0, pos.1, grid_size, grid_size), context.width, context.color_light_white);
                        context.draw2d.draw_rect_outline(frame, &(pos.0 + 1, pos.1 + 1, grid_size - 2, grid_size - 2), context.width, context.color_black);
                    }
                }
            }
        }
    }

    fn get_layer_mask(&mut self, context: &mut ScreenContext) -> Option<Vec<bool>> {
        if let Some(id) = self.get_tile_id(self.mouse_hover_pos) {
            if let Some(region) = context.data.regions.get(&self.region_id) {
                let mask = region.get_layer_mask(id);
                return Some(mask);
            }
        }
        None
    }

    fn mouse_down(&mut self, pos: (usize, usize), asset: &mut Asset, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>, _toolbar: &mut Option<&mut ToolBar>) -> bool {
        let mut consumed = false;

        let mut rect = self.rect.clone();
        rect.3 -= self.bottom_size + self.toolbar_size;

        if let Some(options) = options {

            for atom in &mut self.widgets {
                if atom.mouse_down(pos, asset, context) {
                    return true;
                }
            }

            let editor_mode = options.get_editor_mode();

            if editor_mode == RegionEditorMode::Tiles {
                if self.tile_selector.mouse_down(pos, asset, context) {
                    consumed = true;
                    if let Some(selected) = &self.tile_selector.selected {
                        context.curr_region_tile = Some(selected.clone());
                    } else {
                        context.curr_region_tile = None;
                    }
                }
                for atom in &mut self.editing_widgets {
                    if atom.mouse_down(pos, asset, context) {
                        return true;
                    }
                }
            } else
            if editor_mode == RegionEditorMode::Areas {
                if context.contains_pos_for(pos, self.behavior_graph.rect) {
                    consumed = self.behavior_graph.mouse_down(pos, asset, context, &mut None, &mut None);
                    return consumed;
                } else {
                    for atom in &mut self.area_widgets {
                        if atom.mouse_down(pos, asset, context) {
                            return true;
                        }
                    }
                }
            } else
            if editor_mode == RegionEditorMode::Characters {
                if self.character_selector.mouse_down(pos, asset, context) {
                    consumed = true;
                } else {
                    for atom in &mut self.character_widgets {
                        if atom.mouse_down(pos, asset, context) {
                            return true;
                        }
                    }
                }
            }

            if consumed == false && context.contains_pos_for(pos, rect) {
                if let Some(id) = self.get_tile_id(pos) {
                    self.clicked = Some(id);
                    let editor_mode = options.get_editor_mode();

                    if editor_mode == RegionEditorMode::Tiles {
                        if self.editing_widgets[0].checked == false {
                            if self.editing_widgets[1].checked == false {
                                if let Some(selected) = &self.tile_selector.selected {
                                    if let Some(region) = context.data.regions.get_mut(&self.region_id) {
                                        region.set_value(options.get_layer(), id, selected.clone());
                                        region.save_data();
                                    }
                                }
                            } else {
                                if let Some(region) = context.data.regions.get_mut(&self.region_id) {
                                    let s = region.get_value(id);
                                    if s.len() > 0 {
                                        self.tile_selector.selected = Some(s[0].clone());
                                        self.editing_widgets[1].checked = false;
                                        self.editing_widgets[1].dirty = true;
                                    }
                                }
                            }
                        } else {
                            if let Some(region) = context.data.regions.get_mut(&self.region_id) {
                                if self.editing_widgets[0].checked == true {
                                    region.clear_value(options.get_layer(), id);
                                    region.save_data();
                                }
                            }
                        }
                    } else
                    if editor_mode == RegionEditorMode::Areas {
                        if let Some(region) = context.data.regions.get_mut(&self.region_id) {
                            if region.data.areas.len() > 0 {
                                let area = &mut region.data.areas[context.curr_region_area_index];

                                //

                                let mode = self.area_widgets[4].curr_index;

                                if mode == 0 {
                                    // Add
                                    if area.area.contains(&id) == false {
                                        area.area.push(id);
                                    }
                                } else
                                if mode == 1 {
                                    // Remove
                                    if area.area.contains(&id) == true {
                                        let index = area.area.iter().position(|&r| r == id).unwrap();
                                        area.area.remove(index);
                                    }
                                } else
                                if mode == 2 {
                                    // Pick
                                    for (index, area) in region.data.areas.iter().enumerate() {
                                        if area.area.contains(&id) {
                                            self.area_widgets[0].curr_index = index;
                                            self.area_widgets[0].dirty = true;
                                            context.curr_region_area_index = index;
                                            break;
                                        }
                                    }
                                }
                                region.save_data();
                            }
                        }
                    } else
                    if editor_mode == RegionEditorMode::Characters {
                        if let Some(id) = self.get_tile_id(pos) {
                            if let Some(meta) = self.character_selector.selected.clone() {

                                let alignment = context.data.get_behavior_default_alignment(meta.id);

                                if let Some(behavior) = context.data.get_mut_behavior(meta.id, BehaviorType::Behaviors) {
                                    if behavior.data.instances.is_none() {
                                        behavior.data.instances = Some(vec![]);
                                    }

                                    let mode = self.character_widgets[0].curr_index;

                                    if mode == 0 {
                                        // Add
                                        let index = behavior.data.instances.as_ref().unwrap().iter().position(|r| r.position == Position::new(self.region_id, id.0, id.1));

                                        if index.is_none() {
                                            let instance     = CharacterInstanceData {
                                                position    : Position::new(self.region_id, id.0, id.1),
                                                name        : None,
                                                tile        : None,
                                                alignment   : alignment };
                                            behavior.data.instances.as_mut().unwrap().push(instance);
                                        }
                                    } else
                                    if mode == 1 {
                                        // Remove
                                        if let Some(index) = behavior.data.instances.as_ref().unwrap().iter().position(|r| r.position == Position::new(self.region_id, id.0, id.1)) {
                                            behavior.data.instances.as_mut().unwrap().remove(index);
                                        }
                                    }
                                    behavior.save_data();
                                }
                            }
                        }
                    }
                }
                consumed = true;
            }
        }
        consumed
    }

    fn mouse_up(&mut self, pos: (usize, usize), asset: &mut Asset, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>, _toolbar: &mut Option<&mut ToolBar>) -> bool {
        self.clicked = None;

        let mut consumed = false;
        if let Some(options) = options {

            for atom in &mut self.widgets {
                if atom.mouse_up(pos, asset, context) {
                    if atom.atom_data.id == "Mode" {
                        context.code_editor_is_active = false;
                        if atom.curr_index == 0 {
                            options.set_editor_mode(RegionEditorMode::Tiles);
                        } else
                        if atom.curr_index == 1 {
                            options.set_editor_mode(RegionEditorMode::Areas);
                        } else
                        if atom.curr_index == 2 {
                            options.set_editor_mode(RegionEditorMode::Characters);
                            self.character_selector.collect(context);
                        } else
                        if atom.curr_index == 3 {
                            options.set_editor_mode(RegionEditorMode::Settings);
                            let value;
                            if let Some(region) = context.data.regions.get(&self.get_region_id()) {
                                value = Value::String(region.data.settings.to_string(generate_region_sink_descriptions()));
                            } else {
                                return false;
                            }
                            let id = context.create_property_id("region_settings");
                            context.code_editor_mode = CodeEditorMode::Settings;
                            context.open_code_editor(id,  value, false);
                        }
                    }
                    return true;
                }
            }

            let editor_mode = options.get_editor_mode();

            if editor_mode == RegionEditorMode::Areas {

                if context.contains_pos_for(pos, self.behavior_graph.rect) {
                    consumed = self.behavior_graph.mouse_up(pos, asset, context, &mut None, &mut None);
                } else {

                    for atom in &mut self.area_widgets {
                        if atom.mouse_up(pos, asset, context) {
                            if atom.atom_data.id == "Area" {
                                self.update_area_ui(context);
                                if let Some(region) = context.data.regions.get_mut(&self.get_region_id()) {
                                    if let Some(graph) = self.get_behavior_graph() {
                                        graph.set_behavior_id(region.behaviors[context.curr_region_area_index].data.id, context);
                                    }
                                }
                            } else
                            if atom.atom_data.id == "Add Area" {
                                if let Some(region) = context.data.regions.get_mut(&self.get_region_id()) {
                                    let id = region.create_area();
                                    self.area_widgets[0].curr_index = region.behaviors.len() - 1;
                                    if let Some(graph) = self.get_behavior_graph() {
                                        graph.set_behavior_id(id, context);
                                    }
                                }

                                self.update_area_ui(context);
                            } else
                            if atom.atom_data.id == "Delete" {
                                if let Some(region) = context.data.regions.get_mut(&self.get_region_id()) {
                                    region.delete_area(context.curr_region_area_index);
                                }

                                self.update_area_ui(context);
                            } else
                            if atom.atom_data.id == "Rename" {
                                context.dialog_state = DialogState::Opening;
                                context.dialog_height = 0;
                                context.target_fps = 60;
                                context.dialog_entry = DialogEntry::NewName;
                                if let Some(region) = context.data.regions.get_mut(&self.get_region_id()) {
                                    context.dialog_new_name = region.get_area_names()[context.curr_region_area_index].clone();
                                }
                                self.update_area_ui(context);
                            }

                            return true;
                        }
                    }
                }
            }
            if editor_mode == RegionEditorMode::Tiles {
                for atom in &mut self.editing_widgets {
                    if atom.mouse_up(pos, asset, context) {
                    }
                }
            }
            if editor_mode == RegionEditorMode::Characters {
                for atom in &mut self.character_widgets {
                    if atom.mouse_up(pos, asset, context) {
                        if atom.atom_data.id == "Area" {
                        }
                    }
                }
            }
        }
        consumed
    }

    fn mouse_hover(&mut self, pos: (usize, usize), asset: &mut Asset, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>, _toolbar: &mut Option<&mut ToolBar>) -> bool {

        for atom in &mut self.widgets {
            if atom.mouse_hover(pos, asset, context) {
                return true;
            }
        }

        if let Some(options) = options {
            let editor_mode = options.get_editor_mode();
            if editor_mode == RegionEditorMode::Areas {
                for atom in &mut self.area_widgets {
                    if atom.mouse_hover(pos, asset, context) {
                        return true;
                    }
                }
            } else
            if editor_mode == RegionEditorMode::Tiles {
                for atom in &mut self.editing_widgets {
                    if atom.mouse_hover(pos, asset, context) {
                        return true;
                    }
                }
            }
            if editor_mode == RegionEditorMode::Characters {
                for atom in &mut self.character_widgets {
                    if atom.mouse_hover(pos, asset, context) {
                        return true;
                    }
                }
            }
        }

        self.mouse_hover_pos = pos.clone();
        true
    }

    fn mouse_dragged(&mut self, pos: (usize, usize), asset: &mut Asset, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>, _toolbar: &mut Option<&mut ToolBar>) -> bool {

        let mut consumed = false;
        if let Some(options) = options {
            let editor_mode = options.get_editor_mode();

            if editor_mode == RegionEditorMode::Areas {
                if context.contains_pos_for(pos, self.behavior_graph.rect) {
                    consumed = self.behavior_graph.mouse_dragged(pos, asset, context, &mut None, &mut None);
                    return consumed;
                }
            }

            if consumed == false && context.contains_pos_for(pos, self.rect) {
                if let Some(id) = self.get_tile_id(pos) {
                    if self.clicked != Some(id) {

                        self.clicked = Some(id);
                        let editor_mode = options.get_editor_mode();

                        if editor_mode == RegionEditorMode::Tiles {
                            if let Some(selected) = &self.tile_selector.selected {
                                if let Some(region) = context.data.regions.get_mut(&self.region_id) {
                                    region.set_value(options.get_layer(), id, selected.clone());
                                    region.save_data();
                                }
                            }
                        }
                    }
                }

                consumed = true;
            }
        }
        consumed
    }

    fn mouse_wheel(&mut self, delta: (isize, isize), asset: &mut Asset, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>, _toolbar: &mut Option<&mut ToolBar>) -> bool {

        let mut consumed = false;
        if let Some(options) = options {
            let editor_mode = options.get_editor_mode();

            if editor_mode == RegionEditorMode::Tiles {
                if context.contains_pos_for(self.mouse_hover_pos, self.tile_selector.rect) && self.tile_selector.mouse_wheel(delta, asset, context) {
                    consumed = true;
                }
            } else
            if editor_mode == RegionEditorMode::Areas {
                if context.contains_pos_for(self.mouse_hover_pos, self.behavior_graph.rect) && self.behavior_graph.mouse_wheel(delta, asset, context, &mut None, &mut None) {
                    consumed = true;
                }
            } else
            if editor_mode == RegionEditorMode::Characters {
                if context.contains_pos_for(self.mouse_hover_pos, self.character_selector.rect) && self.character_selector.mouse_wheel(delta, asset, context) {
                    consumed = true;
                }
            }

            if consumed == false {
                self.mouse_wheel_delta.0 += delta.0;
                self.mouse_wheel_delta.1 += delta.1;

                self.offset.0 -= self.mouse_wheel_delta.0 / self.grid_size as isize;
                self.offset.1 += self.mouse_wheel_delta.1 / self.grid_size as isize;

                self.mouse_wheel_delta.0 -= (self.mouse_wheel_delta.0 / self.grid_size as isize) * self.grid_size as isize;
                self.mouse_wheel_delta.1 -= (self.mouse_wheel_delta.1 / self.grid_size as isize) * self.grid_size as isize;
            }
        }
        true
    }

    /// Key down
    fn key_down(&mut self, char: Option<char>, _key: Option<WidgetKey>, _asset: &mut Asset, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>, _toolbar: &mut Option<&mut ToolBar>) -> bool {

        if let Some(options) = options {
            if let Some(char) = char {
                if char == 'd' {
                    self.widgets[0].curr_index = 0;
                    self.widgets[0].dirty = true;
                    options.set_editor_mode(RegionEditorMode::Tiles);
                    return true;
                } else
                if char == 'e' {
                    self.widgets[0].curr_index = 1;
                    self.widgets[0].dirty = true;
                    options.set_editor_mode(RegionEditorMode::Areas);
                    return true;
                } else
                if char == 'a' {
                    self.widgets[0].curr_index = 2;
                    self.widgets[0].dirty = true;
                    options.set_editor_mode(RegionEditorMode::Characters);
                    self.character_selector.collect(context);
                    return true;
                } else
                if char == 's' {
                    self.widgets[0].curr_index = 3;
                    self.widgets[0].dirty = true;
                    options.set_editor_mode(RegionEditorMode::Settings);
                    let value;
                    if let Some(region) = context.data.regions.get_mut(&self.get_region_id()) {
                        value = Value::String(region.data.settings.to_string(generate_region_sink_descriptions()));
                    } else {
                        return false;
                    }
                    let id = context.create_property_id("region_settings");
                    context.code_editor_mode = CodeEditorMode::Settings;
                    context.open_code_editor(id, value, false);
                    return true;
                } else
                if char == 'c' {
                    self.editing_widgets[0].checked = !self.editing_widgets[0].checked;
                    self.editing_widgets[0].dirty = true;
                    return true;
                } else
                if char == 'p' {
                    self.editing_widgets[1].checked = !self.editing_widgets[0].checked;
                    self.editing_widgets[1].dirty = true;
                    return true;
                }
            }
        }
        false
    }

    /// Sets a region id
    fn set_region_id(&mut self, id: Uuid, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>) {
        self.region_id = id;
        if let Some(region) = context.data.regions.get_mut(&self.region_id) {

            self.area_widgets[0].text = region.get_area_names();
            self.area_widgets[0].dirty = true;

            if context.curr_region_area_index >= region.data.areas.len() {
                context.curr_region_area_index = 0;
            }
            if region.behaviors.len() > 0 {
                self.behavior_graph.set_behavior_id(region.behaviors[context.curr_region_area_index].data.id, context);
            }
        }

        if let Some(options) = options {
            let mode = options.get_editor_mode();
            if mode == RegionEditorMode::Settings {
                let value;
                if let Some(region) = context.data.regions.get_mut(&id) {
                    value = Value::String(region.data.settings.to_string(generate_region_sink_descriptions()));
                } else {
                    return
                }

                let id = context.create_property_id("region_settings");
                context.code_editor_mode = CodeEditorMode::Settings;
                context.open_code_editor(id, value, false);
            }
        }

    }

    /// Get the tile id
    fn get_tile_id(&self, pos: (usize, usize)) -> Option<(isize, isize)> {
        let grid_size = self.grid_size;
        if pos.0 > self.rect.0 + self.screen_offset.0 && pos.1 > self.rect.1 + self.screen_offset.1
        && pos.0 < self.rect.0 + self.rect.2 - self.screen_offset.0  && pos.1 < self.rect.1 + self.rect.3 - self.screen_offset.1 - self.bottom_size
        {
            let x = ((pos.0 - self.rect.0 - self.screen_offset.0) / grid_size) as isize - self.offset.0;
            let y = ((pos.1 - self.rect.1 - self.screen_offset.0) / grid_size) as isize - self.offset.1;
            return Some((x, y));
        }
        None
    }

    /// Returns the selected tile
    fn get_selected_tile(&self) -> Option<TileData> {
        self.tile_selector.selected.clone()
    }

    /// Return the tile_selector
    fn get_tile_selector(&mut self) -> Option<&mut TileSelectorWidget> {
        Some(&mut self.tile_selector)
    }

    /// Return the behavior graph
    fn get_behavior_graph(&mut self) -> Option<&mut NodeGraph> {
        Some(&mut self.behavior_graph)
    }

    /// Returns the region_id
    fn get_region_id(&self) -> Uuid {
        self.region_id
    }

    /// Returns the rect for DnD
    fn get_rect(&self) -> (usize, usize, usize, usize) {
        self.behavior_graph.rect.clone()
    }

    /// Adds the given node to the behavior graph (after DnD)
    fn add_node_of_name(&mut self, name: String, position: (isize, isize), context: &mut ScreenContext) {
        self.behavior_graph.add_node_of_name(name, position, context);
    }

    /// Update based on changes
    fn update_from_dialog(&mut self, id: (Uuid, Uuid, String), value: Value, asset: &mut Asset, context: &mut ScreenContext, options: &mut Option<Box<dyn EditorOptions>>) {
        if id.2 == "region_settings" {
            let mut sink = PropertySink::new();
            if sink.load_from_string(context.code_editor_value.clone()) {
                context.code_editor_error = None;
                let id = self.get_region_id();
                if let Some(region) = context.data.regions.get_mut(&id) {
                    region.data.settings = sink;
                    region.save_data();
                }
            } else {
                context.code_editor_error = Some((sink.error.clone().unwrap().1, Some(sink.error.unwrap().0)));
            }
        } else {
            self.behavior_graph.update_from_dialog(id, value, asset, context, options);
        }
    }

    /// Update the area ui
    fn update_area_ui(&mut self, context: &mut ScreenContext) {
        if let Some(region) = context.data.regions.get(&self.get_region_id()) {

            let area_count = region.data.areas.len();

            if area_count == 0 {
                self.area_widgets[0].text = vec![];
                self.area_widgets[0].curr_index = 0;
                self.area_widgets[0].state = WidgetState::Disabled;
                self.area_widgets[2].state = WidgetState::Disabled;
                self.area_widgets[3].state = WidgetState::Disabled;
            } else {
                self.area_widgets[0].text = region.get_area_names();
                self.area_widgets[0].state = WidgetState::Normal;
                self.area_widgets[1].state = WidgetState::Normal;
                self.area_widgets[2].state = WidgetState::Normal;
                self.area_widgets[3].state = WidgetState::Normal;
            }

            for a in &mut self.area_widgets {
                a.dirty = true;
            }

            context.curr_region_area_index = self.area_widgets[0].curr_index;

            region.save_data();
        }
    }

    /// Sets a new name for the current area
    fn set_area_name(&mut self, name: String, context: &mut ScreenContext) {
        if let Some(region) = context.data.regions.get_mut(&self.get_region_id()) {
            region.data.areas[context.curr_region_area_index].name = name;
            self.update_area_ui(context);
        }
    }

    // Undo / Redo

    fn is_undo_available(&self, context: &ScreenContext) -> bool {
        if self.widgets[0].curr_index == 0 {
            // Tiles
            if let Some(region) = context.data.regions.get(&self.get_region_id()) {
                return region.is_undo_available();
            }
        }
        false
    }
    fn is_redo_available(&self, context: &ScreenContext) -> bool {
        if self.widgets[0].curr_index == 0 {
            // Tiles
            if let Some(region) = context.data.regions.get(&self.get_region_id()) {
                return region.is_redo_available();
            }
        }
        false
    }

    fn undo(&mut self, context: &mut ScreenContext) {
        if self.widgets[0].curr_index == 0 {
            // Tiles
            if let Some(region) = context.data.regions.get_mut(&self.get_region_id()) {
                region.undo();
            }
        }
    }

    fn redo(&mut self, context: &mut ScreenContext) {
        if self.widgets[0].curr_index == 0 {
            // Tiles
            if let Some(region) = context.data.regions.get_mut(&self.get_region_id()) {
                region.redo();
            }
        }
    }

}