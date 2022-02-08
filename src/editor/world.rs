//use crate::prelude::*;

use crate::widget::*;
use crate::Asset;

use crate::tileselector::{ TileSelectorWidget, TileSelectorHelper };

pub struct WorldEditor {
    rect                    : (u32, u32, u32, u32),
    content_rect            : (u32, u32, u32, u32),
    tileselector_widget     : TileSelectorWidget,
}

impl Widget for WorldEditor {
    
    fn new(_text: Vec<String>, rect: (u32, u32, u32, u32), asset: &Asset) -> Self where Self: Sized {

        let mut tileselector_widget = TileSelectorWidget::new(vec![], (rect.0, rect.1 + rect.3 - rect.3 / 3, rect.2, rect.3 / 3), asset);
        let tileselector_helper = TileSelectorHelper {};

        tileselector_helper.set_usage(&mut tileselector_widget, tileselector::TileSelectorUsage::Environment);

        Self {
            rect,
            content_rect                    : (rect.0, rect.1, rect.2, (rect.3 / 3) * 2),
            tileselector_widget,
        }
    }

    /// Update the editor
    fn update(&mut self) {
    }

    fn draw(&mut self, frame: &mut [u8], anim_counter: u32, asset: &mut Asset) {
        asset.draw_area(frame, &self.content_rect, anim_counter);
        self.tileselector_widget.draw(frame, anim_counter, asset);
    }

    fn mouse_down(&mut self, pos: (u32, u32), asset: &mut Asset) -> bool {
        let mut consumed = false;
        if self.tileselector_widget.mouse_down(pos, asset) {
            consumed = true;
        }
        if consumed == false && self.contains_pos_for(pos, self.content_rect) {

            if let Some(tile_id) = self.tileselector_widget.selected {

                let grid_size = asset.grid_size as u32;
                let id = (((pos.0 - self.content_rect.0) / grid_size) as isize, ((pos.1 - self.content_rect.1) / grid_size) as isize);

                let tile = asset.get_tile(&tile_id);
                asset.set_area_value(id, (tile_id.0, tile_id.1, tile_id.2, tile.usage));

                return true;
            }
        }
        consumed
    }

    fn mouse_up(&mut self, pos: (u32, u32), asset: &mut Asset) -> bool {
        let mut consumed = false;
        if self.tileselector_widget.mouse_up(pos, asset) {
            consumed = true;
        }
        consumed
    }

    /// Set the screen_end_selected point
    fn mouse_dragged(&mut self, pos: (u32, u32), asset: &mut Asset) -> bool {
        let mut consumed = false;
        if self.tileselector_widget.mouse_dragged(pos, asset) {
            consumed = true;
        }
        if consumed == false && self.contains_pos_for(pos, self.content_rect) {

            if let Some(tile_id) = self.tileselector_widget.selected {

                let grid_size = asset.grid_size as u32;
                let id = (((pos.0 - self.content_rect.0) / grid_size) as isize, ((pos.1 - self.content_rect.1) / grid_size) as isize);

                let tile = asset.get_tile(&tile_id);
                asset.set_area_value(id, (tile_id.0, tile_id.1, tile_id.2, tile.usage));

                return true;
            }
        }        
        consumed        
    }

    fn get_rect(&self) -> &(u32, u32, u32, u32) {
        return &self.rect;
    }
}