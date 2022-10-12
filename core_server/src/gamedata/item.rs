//use crate::prelude::*;
use core_shared::prelude::*;

// Generate item sink

pub fn update_item_sink(sink: &mut PropertySink) {

    if sink.contains("item_type") == false {
        sink.properties.insert(0,Property::new_color("item_type".to_string(), "\"Tool\"".to_string()));
    }

    if sink.contains("state") == false {
        sink.properties.insert(1,Property::new_color("state".to_string(), "false".to_string()));
    }

    if sink.contains("stackable") == false {
        sink.properties.insert(2,Property::new_color("stackable".to_string(), "1".to_string()));
    }

    if sink.contains("static") == false {
        sink.properties.insert(3,Property::new_color("static".to_string(), "false".to_string()));
    }

    if sink.contains("price") == false {
        sink.properties.insert(4,Property::new_color("price".to_string(), "0.0".to_string()));
    }
}

pub fn generate_item_sink_descriptions() -> FxHashMap<String, Vec<String>> {
    let mut map : FxHashMap<String, Vec<String>> = FxHashMap::default();

    map.insert("item_type".to_string(), vec!["Type of the item, either \"Weapon\", \"Gear\" or \"Tool\"".to_string()]);
    map.insert("state".to_string(), vec!["true if the item should have it's own state (variables).".to_string()]);
    map.insert("stackable".to_string(), vec!["Value greater than 1 if item should be stackable. Only for items without state.".to_string()]);
    map.insert("static".to_string(), vec!["True if the item is static, i.e. cannot be picked up (campfire etc.).".to_string()]);
    map.insert("price".to_string(), vec!["The sales price of the item. 0.0 if the item cannot be sold.".to_string()]);

    map
}