use crate::Asset;
use crate::atom:: { AtomWidget, AtomWidgetType, AtomData };
use crate::editor::ScreenContext;

pub struct NodePreviewWidget {
    pub rect                    : (usize, usize, usize, usize),
    pub widgets                 : Vec<AtomWidget>,

    pub clicked                 : bool,

    pub id                      : usize,

    pub dirty                   : bool,
    pub buffer                  : Vec<u8>,

    pub disabled                : bool,

    pub size                    : (usize, usize),

    pub clicked_id              : Option<(usize, usize, String)>,

    pub drag_size               : Option<(usize, usize)>,

    // For showing region
    pub region_tile_size        : usize,
    pub region_rect             : (usize, usize, usize, usize),
    pub region_offset           : (isize, isize),
    pub region_scroll_offset    : (isize, isize),

    pub curr_position           : Option<(usize, isize, isize)>,

    pub tile_size               : usize,

    pub graph_offset            : (isize, isize),

    pub clicked_region_id         : Option<(usize, isize, isize)>,

    pub curr_region_index         : usize,

    // Indicates that preview stopped running
    pub just_stopped_running    : bool
}

impl NodePreviewWidget {

    pub fn new(context: &ScreenContext) -> Self {

        let run_button = AtomWidget::new(vec!["Run".to_string()], AtomWidgetType::LargeButton,
        AtomData::new_as_int("run".to_string(), 0));

        let mut regions_button = AtomWidget::new(context.data.regions_names.clone(), AtomWidgetType::SmallMenuButton,
        AtomData::new_as_int("region".to_string(), 0));
        regions_button.atom_data.text = "Region".to_string();
        regions_button.curr_index = 0;

        Self {
            rect                : (0,0,0,0),
            widgets             : vec![run_button, regions_button],
            clicked             : false,

            id                  : 0,

            dirty               : true,
            buffer              : vec![],

            disabled            : false,

            size                : (300, 250),

            clicked_id          : None,

            drag_size           : None,

            region_tile_size      : 32,
            region_rect           : (0,0,0,0),
            region_offset         : (0,0),
            region_scroll_offset  : (0,0),

            curr_position       : None,

            tile_size           : 32,

            graph_offset        : (0,0),

            clicked_region_id     : None,

            curr_region_index     : 0,

            just_stopped_running: false,
        }
    }

    /// Draw the node
    pub fn draw(&mut self, _frame: &mut [u8], anim_counter: usize, asset: &mut Asset, context: &mut ScreenContext) {

        if self.buffer.len() != self.size.0 * self.size.1 * 4 {
            self.buffer = vec![0;self.size.0 * self.size.1 * 4];
            for w in &mut self.widgets {
                w.dirty = true;
            }
        }

        let rect = (0, 0, self.size.0, self.size.1);

        // Go to this position
        if let Some(jump_to_position) = context.jump_to_position {
            self.dirty = true;
            self.curr_position = Some(jump_to_position);
            self.region_scroll_offset = (0, 0);
            context.jump_to_position = None;
        }

        if self.dirty {
            for i in &mut self.buffer[..] { *i = 0 }
            let buffer_frame = &mut self.buffer[..];
            let stride = self.size.0;

            context.draw2d.draw_rounded_rect_with_border(buffer_frame, &rect, stride, &((rect.2 - 1) as f64, (rect.3 - 2) as f64), &context.color_black, &(0.0, 0.0, 20.0, 0.0), &context.color_gray, 1.5);
            context.draw2d.draw_rect(buffer_frame, &(2, 0, rect.2 - 4, 4), stride, &context.color_black);
            context.draw2d.draw_rect(buffer_frame, &(rect.2-2, 0, 2, rect.3 - 1), stride, &context.color_black);
            context.draw2d.draw_rect(buffer_frame, &(1, 1, 1, 1), stride, &[65, 65, 65, 255]);

            self.widgets[0].set_rect((20, 4, 120, 32), asset, context);
            self.widgets[0].draw(buffer_frame, stride, anim_counter, asset, context);

            self.widgets[1].set_rect((15, self.size.1 - 50, self.size.0 - 20, 25), asset, context);
            self.widgets[1].draw(buffer_frame, stride, anim_counter, asset, context);

            self.region_rect.0 = 10;
            self.region_rect.1 = 50;
            self.region_rect.2 = rect.2 - 20;
            self.region_rect.3 = rect.3 - 100;

            // Draw the region
            let region_id = context.data.regions_ids[self.curr_region_index];

            if let Some(region) = context.data.regions.get(&region_id) {

                if context.is_running {
                    self.region_offset = context.draw2d.draw_region_centered_with_instances(buffer_frame, region, &self.region_rect, context.curr_behavior_index, stride, 32, anim_counter, asset, context);
                } else
                if let Some(position) = &self.curr_position {
                    self.region_offset = context.draw2d.draw_region_centered_with_behavior(buffer_frame, region, &self.region_rect, &(position.1 - self.region_scroll_offset.0, position.2 - self.region_scroll_offset.1), stride, 32, 0, asset, context);
                } else
                if let Some(position) = context.data.get_behavior_default_position(region_id) {
                    self.region_offset = context.draw2d.draw_region_centered_with_behavior(buffer_frame, region, &self.region_rect, &(position.1 - self.region_scroll_offset.0, position.2 - self.region_scroll_offset.1), stride, 32, 0, asset, context);
                } else {
                    let offset = region.data.min_pos;
                    self.region_offset = offset;
                    context.draw2d.draw_region(buffer_frame, region, &self.region_rect, &self.region_offset, stride, self.tile_size, 0, asset);
                }
            }
            context.draw2d.blend_mask(buffer_frame, &(6, rect.3 - 23, rect.2, rect.3), rect.2, &context.preview_arc_mask[..], &(20, 20), &context.color_gray);
        }
        self.dirty = false;
    }

    /// Check if one of the atom widgets was clicked
    pub fn mouse_down(&mut self, pos: (usize, usize), asset: &mut Asset, context: &mut ScreenContext) -> bool {
        for atom_widget in &mut self.widgets {
            if atom_widget.mouse_down(pos, asset, context) {

                if atom_widget.atom_data.id == "run" {
                    if context.is_running == false {
                        context.data.create_behavior_instances();
                        context.data.activate_region_instances(context.data.regions_ids[self.curr_region_index]);
                        context.is_running = true;
                        atom_widget.text[0] = "Stop".to_string();
                        context.data.messages = vec![];
                    } else {
                        context.data.clear_instances();
                        context.is_running = false;
                        atom_widget.text[0] = "Run".to_string();
                        self.just_stopped_running = true;
                    }
                }

                self.dirty = true;
                self.clicked = true;
                self.clicked_id = atom_widget.behavior_id.clone();
                return true;
            }
        }

        // Test dragging region
        if context.contains_pos_for(pos, (0, self.size.1 - 20, 30, 20)) {
            self.drag_size = Some(self.size.clone());
            context.target_fps = 60;
            return true;
        }

        // Test region map
        if context.contains_pos_for(pos, self.region_rect) {

            let left_offset = (self.region_rect.2 % self.region_tile_size) / 2;
            let top_offset = (self.region_rect.3 % self.region_tile_size) / 2;

            let x = self.region_offset.0 + ((pos.0 - self.region_rect.0 - left_offset) / self.region_tile_size) as isize;
            let y = self.region_offset.1 + ((pos.1 - self.region_rect.1 - top_offset) / self.region_tile_size) as isize;
            //println!("{} {}", x, y);
            if let Some(region) = context.data.regions.get(&context.data.regions_ids[self.curr_region_index]) {
                self.clicked_region_id = Some((region.data.id.clone(), x, y));
            }
            return true;
        }
        false
    }

    pub fn mouse_up(&mut self, pos: (usize, usize), asset: &mut Asset, context: &mut ScreenContext) -> bool {
        self.clicked = false;
        self.clicked_id = None;
        for atom_widget in &mut self.widgets {
            if atom_widget.mouse_up(pos, asset, context) {
                self.dirty = true;
                self.clicked = false;

                if atom_widget.atom_data.text == "Region" {
                    self.curr_region_index = atom_widget.curr_index;
                }
                return true;
            }
        }

        if self.drag_size.is_some() {
            self.drag_size = None;
            context.target_fps = context.default_fps;
        }
        false
    }

    pub fn mouse_dragged(&mut self, _pos: (usize, usize), rel_pos: (isize, isize), _asset: &mut Asset, _context: &mut ScreenContext) -> bool {

        if let Some(drag_size) = &self.drag_size {
            let mut x: isize = drag_size.0 as isize + rel_pos.0;
            let mut y: isize =  drag_size.1 as isize + rel_pos.1;
            if x < 200 { x = 200; }
            if x > 600 { x = 600; }
            if y < 150 { y = 150; }
            if y > 600 { y = 600; }
            self.size = (x as usize, y as usize);
            self.dirty = true;
            return true;
        }

        false
    }

    pub fn mouse_wheel(&mut self, delta: (isize, isize), _asset: &mut Asset, _context: &mut ScreenContext) -> bool {
        self.region_scroll_offset.0 -= delta.0 / self.tile_size as isize;
        self.region_scroll_offset.1 += delta.1 / self.tile_size as isize;
        self.dirty = true;
        true
    }

    /// Stop running
    pub fn _stop(&mut self, context: &mut ScreenContext) {
        context.data.clear_instances();
        context.is_running = false;
        self.widgets[0].text[0] = "Run".to_string();
    }
}