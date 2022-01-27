

use crate::prelude::*;

use crate::widget::*;

use crate::tab::TabWidget;
use crate::asset::Asset;

use core::cmp::max;

pub struct TileMapEditor {
    rect            : (u32, u32, u32, u32),
    tab_widget      : TabWidget,//Box<dyn Widget>
}

impl Widget for TileMapEditor {
    
    fn new(rect: (u32, u32, u32, u32)) -> Self where Self: Sized {

        Self {
            rect,
            tab_widget      : TabWidget::new((0,0, WIDTH, HEIGHT / 2))
        }
    }

    /// Update the editor
    fn update(&mut self) {
    }

    fn draw(&self, frame: &mut [u8], asset: &Asset) {

        asset.draw_rect(frame, &self.tab_widget.get_content_rect(), [0,0,0,255]);

        let scale = 2_f32;
        let map = asset.get_map_of_id(0);

        let scaled_grid_size = (map.settings.grid_size as f32 * scale) as u32;

        let x_tiles = map.width / map.settings.grid_size;
        let y_tiles = map.height / map.settings.grid_size;

        let total_tiles = x_tiles * y_tiles;
        let total_tiles_scaled = ((x_tiles * y_tiles) as f32 * scale) as u32;

        let screen_x = WIDTH / scaled_grid_size;
        let screen_y = (HEIGHT / 2 - self.tab_widget.get_default_element_height()) / scaled_grid_size;

        let tiles_per_page = screen_x * screen_y;

        let pages = max( total_tiles_scaled / tiles_per_page, 1);

        //println!("{}", pages);

        self.tab_widget.set_pagination(pages);

        let page = self.tab_widget.curr_page.get();

        let mut x_off = 0_u32;
        let mut y_off = 0_u32;

        let offset = page * tiles_per_page;

        for tile in 0..tiles_per_page {

            if tile + offset >= total_tiles {
                break;
            }

            let x_step = (x_off as f32 * map.settings.grid_size as f32 * scale) as u32;
            let y_step = (y_off as f32 * map.settings.grid_size as f32 * scale) as u32;

            let x = (tile+offset) % x_tiles;
            let y = (tile+offset) / x_tiles;

            asset.draw_tile(frame, &(x_step, y_step), 0_u32, &(x, y), scale);
            x_off += 1;

            if x_off >= screen_x {
                x_off = 0;
                y_off += 1;
                if y_off >= screen_y {
                    break;
                }
            }
        }

        /*
        for y in 0..y_tiles {
            for x in 0..x_tiles {

                let x_step = (x_off as f32 * map.settings.grid_size as f32 * scale) as u32;
                let y_step = (y_off as f32 * map.settings.grid_size as f32 * scale) as u32;

                asset.draw_tile(frame, &(x_step, y_step), 0_u32, &(x, y), scale);
                x_off += 1;

                if x_off >= screen_x {
                    x_off = 0;
                    y_off += 1;
                    if y_off >= screen_y {
                        break;
                    }
                }
            }
            if y_off >= screen_y {
                break;
            }            
        }*/

        self.tab_widget.draw(frame, asset);
    }

    fn mouse_down(&self, pos: (u32, u32)) -> bool {
        self.tab_widget.mouse_down(pos)
    }

    fn mouse_up(&self, _pos: (u32, u32)) {
        //println!("text {:?}", pos);
    }

    fn get_rect(&self) -> &(u32, u32, u32, u32) {
        return &self.rect;
    }
}