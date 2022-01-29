use std::time::{SystemTime, UNIX_EPOCH};

use crate::asset::Asset;

pub trait ScreenWidget {

    fn new() -> Self where Self: Sized;

    fn update(&mut self);
    fn draw(&self, frame: &mut [u8]);

    fn mouse_down(&self, pos: (u32, u32)) -> bool {
        let mut changed = false;
        for w in self.get_widgets() {
            if w.contains_pos(pos) {
                if w.mouse_down(pos) {
                    changed = true;
                }
            }
        }
        changed 
    }

    fn mouse_up(&self, pos: (u32, u32)) -> bool {
        let mut changed = false;
        for w in self.get_widgets() {
            if w.contains_pos(pos) {
                if w.mouse_up(pos) {
                    changed = true;
                }
            }
        }
        changed 
    }

    fn mouse_dragged(&self, _pos: (u32, u32)) {
    }

    fn get_asset(&self) -> &Asset;
    fn get_widgets(&self) -> &Vec<Box<dyn Widget>>;

    /// Gets the current time in milliseconds
    fn get_time(&self) -> u128 {
        let stop = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");    
            stop.as_millis()
    }
}

// General purpose widgets
pub mod button;
pub mod tab;

/// The widget state

//#[derive(PartialEq)]
// pub enum WidgetState {
//     Disabled,
//     Normal,
//     //Right
// }

pub trait Widget {

    fn new(text: Vec<String>, rect: (u32, u32, u32, u32)) -> Self where Self: Sized;

    fn update(&mut self);
    fn draw(&self, frame: &mut [u8], asset: &Asset);

    fn mouse_down(&self, _pos: (u32, u32)) -> bool {
        false
    }

    fn mouse_up(&self, _pos: (u32, u32)) -> bool {
        false
    }

    fn mouse_dragged(&self, _pos: (u32, u32)) {
    }

    //fn set_cb(&mut self) {
    //}

    //fn set_cb<T: FnMut()>(&mut self, f: T) {
        // { self.f = Some(Box::new(f)//); }
    //}

    fn contains_pos(&self, pos: (u32, u32)) -> bool {
        let rect = self.get_rect();

        if pos.0 >= rect.0 && pos.0 < rect.0 + rect.2 && pos.1 >= rect.1 && pos.1 < rect.1 + rect.3 {
            true
        } else {
            false
        }
    }

    fn contains_pos_for(&self, pos: (u32, u32), rect: (u32, u32, u32, u32)) -> bool {
        if pos.0 >= rect.0 && pos.0 < rect.0 + rect.2 && pos.1 >= rect.1 && pos.1 < rect.1 + rect.3 {
            true
        } else {
            false
        }
    }

    fn get_rect(&self) -> &(u32, u32, u32, u32);

    fn get_content_rect(&self) -> (u32, u32, u32, u32) {
        let r = self.get_rect();
        (r.0, r.1, r.2, r.3)
    }

    /// Set the current state of the widget
    fn set_state(&self, _state: u32) {
    }

    // The following are widget specific and optional

    // TabWidget
    fn set_pagination(&self, _pages: u32) {
    }

    fn get_page_rect(&self, _page: u32) -> (u32, u32, u32, u32) {
        (0,0,0,0)
    }

    // Default colors

    fn get_color_background(&self) -> [u8; 4] {
        [43, 43, 43, 255]
    }

    fn get_color_disabled(&self) -> [u8; 4] {
        [43 / 2, 43 / 2, 43 / 2, 255]
    }

    fn get_color_selection(&self) -> [u8; 4] {
        [73, 73, 73, 255]
    }

    fn get_color_selection_blue(&self) -> [u8; 4] {
        [59, 70, 90, 255]
    }

    fn get_color_text(&self) -> [u8; 4] {
        [255, 255, 255, 255]
    }
}