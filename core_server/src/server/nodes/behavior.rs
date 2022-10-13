use crate::prelude::*;

/// expression
pub fn expression(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {

    let rc = eval_bool_expression_instance(instance_index, (behavior_type, id.0, id.1, "expression".to_string()), data);
    if let Some(rc) = rc {
        if rc == true {
            return BehaviorNodeConnector::Success;
        }
    }
    BehaviorNodeConnector::Fail
}

pub fn script(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {
    _ = eval_dynamic_script_instance(instance_index, (behavior_type, id.0, id.1, "script".to_string()), data);
    BehaviorNodeConnector::Bottom
}

/// message
pub fn message(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {

    let mut message_type : MessageType = MessageType::Status;
    let mut text;

    // Message Type
    if let Some(value) = get_node_value((id.0, id.1, "type"), data, behavior_type) {
        if let Some(m_type) = value.to_integer() {
            message_type = match m_type {
                1 => MessageType::Say,
                2 => MessageType::Yell,
                3 => MessageType::Private,
                4 => MessageType::Debug,
                _ => MessageType::Status
            }
        }
    }

    if let Some(value) = get_node_value((id.0, id.1, "text"), data, behavior_type) {
        text = value.to_string_value();
    } else {
        text = "Message".to_string();
    }

    if text.contains("${DIRECTION}") {
        text = text.replace("${DIRECTION}", &data.action_direction_text);
    } else
    if text.contains("${SUBJECT}") {
        text = text.replace("${SUBJECT}", &data.action_subject_text);
    }

    // Do I need to evaluate the script for variables ?
    if text.contains("${") {
        data.scopes[instance_index].push("Self", data.instances[instance_index].name.clone());
        if let Some(target_index) = data.instances[instance_index].target_instance_index {
            data.scopes[instance_index].push("Target", data.instances[target_index].name.clone());
        }
        let r = data.engine.eval_with_scope::<String>(&mut data.scopes[instance_index], format!("`{}`", text).as_str());
        if let Some(rc) = r.ok() {
            text = rc;
        }
    }

    // Formating if needed
    text = match message_type {
        MessageType::Say => format!("{} says \"{}\".", data.instances[instance_index].name, text),
        _ => text
    };

    let message_data = MessageData {
        message_type,
        message         : text.clone(),
        from            : data.instances[instance_index].name.clone(),
        buffer          : None
    };

     data.instances[instance_index].messages.push(message_data.clone());
    if let Some(target_index) = data.instances[instance_index].target_instance_index {
        data.instances[target_index].messages.push(message_data);
    }

    // Output it
    data.messages.push((text,  message_type));
    BehaviorNodeConnector::Bottom
}

pub fn random_walk(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {
    let mut p : Option<Position> = None;
    let mut dp : Option<Position> = None;

    let mut distance = f32::MAX;

    if let Some(v) = &mut data.instances[instance_index].position {
        p = Some(v.clone());
    }

    let mut max_distance : f32 = 0.0;
    if let Some(rc) = eval_number_expression_instance(instance_index, (behavior_type, id.0, id.1, "max_distance".to_string()), data) {
        max_distance = rc;
    }

    if let Some(behavior) = data.behaviors.get_mut(&id.0) {
        if let Some(node) = behavior.nodes.get_mut(&id.1) {

            if let Some(value) = node.values.get("position") {
                dp = match value {
                    Value::Position(v) => {
                        Some(v.clone())
                    },
                    _ => None
                };

                if let Some(dp) = &mut dp {
                    if let Some(p) = &p {
                        distance = compute_distance(p, dp).round();
                    }
                }

                // If we are within the max distance, do a random walk, otherwise just go back towards the position
                if distance <= max_distance {
                    dp = p.clone();
                    if let Some(dp) = &mut dp {

                        let mut rng = thread_rng();
                        let random = rng.gen_range(0..4);

                        if random == 0 {
                            dp.y -= 1;
                        } else
                        if random == 1 {
                            dp.x += 1;
                        } else
                        if random == 2 {
                            dp.y += 1;
                        } else
                        if random == 3 {
                            dp.x -= 1;
                        }
                    }
                }
            }
        }
    }

    let mut speed : f32 = 8.0;
    if let Some(rc) = eval_number_expression_instance(instance_index, (behavior_type, id.0, id.1, "speed".to_string()), data) {
        speed = rc;
    }

    let mut delay_between_movement : f32 = 10.0;
    if let Some(rc) = eval_number_expression_instance(instance_index, (behavior_type, id.0, id.1, "delay".to_string()), data) {
        delay_between_movement = rc;
    }

    // Apply the speed delay
    let delay = 10.0 - speed.clamp(0.0, 10.0);
    data.instances[instance_index].sleep_cycles = (delay + delay_between_movement) as usize;

    let rc  = walk_towards(instance_index, p, dp,false, data);
    if  rc == BehaviorNodeConnector::Right {
        data.instances[instance_index].max_transition_time = delay as usize + 1;
        data.instances[instance_index].curr_transition_time = 1;
    }

    rc
}

/// Pathfinder
pub fn pathfinder(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {
    let mut p : Option<Position> = None;
    let mut dp : Option<Position> = None;

    let mut distance = f32::MAX;

    if let Some(v) = &mut data.instances[instance_index].position {
        p = Some(v.clone());
    }

    if let Some(behavior) = data.behaviors.get_mut(&id.0) {
        if let Some(node) = behavior.nodes.get_mut(&id.1) {
            if let Some(value) = node.values.get("destination") {
                dp = match value {
                    Value::Position(v) => {
                        Some(v.clone())
                    },
                    _ => None
                };

                if let Some(dp) = &dp {
                    if let Some(p) = &p {
                        distance = compute_distance(p, dp).round();
                    }
                }
            }
        }
    }

    let mut speed : f32 = 8.0;
    if let Some(rc) = eval_number_expression_instance(instance_index, (behavior_type, id.0, id.1, "speed".to_string()), data) {
        speed = rc;
    }

    // Apply the speed delay
    let delay = 10.0 - speed.clamp(0.0, 10.0);
    data.instances[instance_index].sleep_cycles = delay as usize;

    // Success if we reached the to_distance already
    if distance == 0.0 {
        return BehaviorNodeConnector::Success;
    }

    let rc  = walk_towards(instance_index, p, dp,false, data);
    if  rc == BehaviorNodeConnector::Right {
        data.instances[instance_index].max_transition_time = delay as usize + 1;
        data.instances[instance_index].curr_transition_time = 1;
    }

    rc
}

/// Lookout
pub fn lookout(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {

    let mut max_distance : f32 = 7.0;
    if let Some(rc) = eval_number_expression_instance(instance_index, (behavior_type, id.0, id.1, "max_distance".to_string()), data) {
        max_distance = rc;
    }

    // Find the chars within the given distance

    let mut chars : Vec<usize> = vec![];

    if let Some(position) = &data.instances[instance_index].position {
        for inst_index in 0..data.instances.len() {
            if inst_index != instance_index {
                if data.instances[inst_index].state == BehaviorInstanceState::Normal {
                    if let Some(pos) = &data.instances[inst_index].position {
                        if pos.region == position.region {
                            let dx = position.x - pos.x;
                            let dy = position.y - pos.y;
                            let d = ((dx * dx + dy * dy) as f32).sqrt();
                            if d <= max_distance {
                                chars.push(inst_index);
                            }
                        }
                    }
                }
            }
        }
    }

    // Evaluate the expression on the characters who are in range
    for inst_ind in &chars {
        if let Some(rc) = eval_bool_expression_instance(*inst_ind, (behavior_type, id.0, id.1, "expression".to_string()), data) {
            if rc {
                data.instances[instance_index].target_instance_index = Some(*inst_ind);
                return BehaviorNodeConnector::Success;
            }
        }
    }

    data.instances[instance_index].target_instance_index = None;
    BehaviorNodeConnector::Fail
}

/// CloseIn
pub fn close_in(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {

    let mut p : Option<Position> = None;
    let mut dp : Option<Position> = None;

    let mut distance = f32::MAX;
    let mut to_distance = 1_f32;

    if let Some(rc) = eval_number_expression_instance(instance_index, (behavior_type, id.0, id.1, "to_distance".to_string()), data) {
        to_distance = rc;
    }

    if let Some(v) = &mut data.instances[instance_index].position {
        p = Some(v.clone());
    }

    let target_index = data.instances[instance_index].target_instance_index;

    if let Some(target_index) = target_index {
        if let Some(v) = &mut data.instances[target_index].position {
            dp = Some(v.clone());
            if let Some(p) = &p {
                distance = compute_distance(p, v);
            }
        }
    }

    let mut speed : f32 = 8.0;
    if let Some(rc) = eval_number_expression_instance(instance_index, (behavior_type, id.0, id.1, "speed".to_string()), data) {
        speed = rc;
    }

    // Apply the speed delay
    let delay = 10.0 - speed.clamp(0.0, 10.0);
    data.instances[instance_index].sleep_cycles = delay as usize;

    // Success if we reached the to_distance already
    if distance <= to_distance {
        return BehaviorNodeConnector::Success;
    }

    let rc = walk_towards(instance_index, p, dp, true, data);
    if  rc == BehaviorNodeConnector::Right {
        data.instances[instance_index].max_transition_time = delay as usize + 1;
        data.instances[instance_index].curr_transition_time = 1;
    }
    rc
}

// Multi choice
pub fn multi_choice(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {

    if data.instances[instance_index].multi_choice_answer.is_some() {
        if Some(id.1) == data.instances[instance_index].multi_choice_answer {

            let npc_index = get_local_instance_index(instance_index, data);
            drop_communication(instance_index, npc_index, data);

            BehaviorNodeConnector::Bottom
        }
        else {
            BehaviorNodeConnector::Right
        }
    } else {
        let mut header = "".to_string();
        let mut text = "".to_string();
        let mut answer = "".to_string();

        if let Some(value) = get_node_value((id.0, id.1, "header"), data, behavior_type) {
            if let Some(h) = value.to_string() {
                header = h;
            }
        }

        if let Some(value) = get_node_value((id.0, id.1, "text"), data, behavior_type) {
            if let Some(t) = value.to_string() {
                text = t;
            }
        }

        if let Some(value) = get_node_value((id.0, id.1, "answer"), data, behavior_type) {
            if let Some(a) = value.to_string() {
                answer = a;
            }
        }

        let mcd = MultiChoiceData {
            id              : id.1,
            header,
            text,
            answer,
            pos             : None,
            buffer          : None,

            item_amount     : None,
            item_behavior_id: None,
            item_price      : None,
        };

        let player_index = instance_index;
        let npc_index = get_local_instance_index(instance_index, data);

        data.instances[player_index].multi_choice_data.push(mcd);

        let t = data.get_time();

        let com = PlayerCommunication {
            player_index,
            npc_index,
            npc_behavior_tree       : data.curr_executing_tree,
            player_answer           : None,
            start_time              : t,
            end_time                : t + 1000 * 20, // 20 Secs
        };

        //println!("{}, {}", data.instances[player_index].name, data.instances[npc_index].name);

        // TODO: Add one Communication structure per player
        if data.instances[npc_index].communication.is_empty() {
            data.instances[npc_index].communication.push(com.clone());
        }

        if data.instances[player_index].communication.is_empty() {
            data.instances[player_index].communication.push(com);
        }

        BehaviorNodeConnector::Right
    }
}

// Multi choice
pub fn sell(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {

    if data.instances[instance_index].multi_choice_answer.is_some() {
        if let Some(id) = data.instances[instance_index].multi_choice_answer {

            println!("{:?}", data.instances[instance_index].multi_choice_answer);

            let npc_index = get_local_instance_index(instance_index, data);

            let mut traded_item : Option<InventoryItem> = None;

            // Remove the item
            if let Some(mess) = data.scopes[npc_index].get_mut("inventory") {
                if let Some(mut inv) = mess.write_lock::<Inventory>() {

                    /*
                    for item in &inv.items {
                        if item.id == id {
                            println!("{}", item.name);
                        }
                    }*/
                    if let Some(item) = inv.remove_item(id, 1) {
                        traded_item = Some(item);
                    }
                }
            }

            // Add the item
            if let Some(item) = traded_item {
                if let Some(mess) = data.scopes[instance_index].get_mut("inventory") {
                    if let Some(mut inv) = mess.write_lock::<Inventory>() {
                        inv.add_item(item);
                    }
                }
            }

            drop_communication(instance_index, npc_index, data);
            BehaviorNodeConnector::Bottom
        }
        else {
           BehaviorNodeConnector::Right
        }
    } else {

        let mut header = "".to_string();

        if let Some(value) = get_node_value((id.0, id.1, "header"), data, behavior_type) {
            if let Some(h) = value.to_string() {
                header = h;
            }
        }

        let player_index = instance_index;
        let npc_index = get_local_instance_index(instance_index, data);

        data.instances[player_index].multi_choice_data = vec![];

        if let Some(mess) = data.scopes[npc_index].get_mut("inventory") {
            if let Some(inv) = mess.read_lock::<Inventory>() {

                let mut index = 1;
                let mut added_items = vec![];

                for item in &inv.items {

                    if item.price != 0.0 && added_items.contains(&item.id) == false {

                        let amount = 1;

                        let mcd = MultiChoiceData {
                            id                  : item.id,
                            header              : if index == 1 { header.clone() } else { "".to_string() },
                            text                : item.name.clone(),
                            answer              : index.to_string(),
                            pos                 : None,
                            buffer              : None,

                            item_behavior_id    : Some(item.id),
                            item_price          : Some(item.price),
                            item_amount         : Some(amount),
                        };

                        added_items.push(item.id);
                        data.instances[player_index].multi_choice_data.push(mcd);
                        index += 1;
                    }
                }
            }
        }

        if data.instances[player_index].multi_choice_data.is_empty() == false {
            let t = data.get_time();

            let com = PlayerCommunication {
                player_index,
                npc_index,
                npc_behavior_tree       : data.curr_executing_tree,
                player_answer           : None,
                start_time              : t,
                end_time                : t + 1000 * 20, // 20 Secs
            };

            if data.instances[npc_index].communication.is_empty() {
                data.instances[npc_index].communication.push(com.clone());
            }

            if data.instances[player_index].communication.is_empty() {
                data.instances[player_index].communication.push(com);
            }
        }

        BehaviorNodeConnector::Right
    }
}

/// Systems Call
pub fn call_system(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {

    let mut systems_id : Option<Uuid> = None;
    let mut systems_tree_id : Option<Uuid> = None;

    // // Try to get from node_values
    // if let Some(node_value) = data.instances[instance_index].node_values.get(&(behavior_type, id.1)) {
    //     systems_id = Some(node_value.0 as usize);
    //     systems_tree_id = Some(node_value.1 as usize);
    // } else {

    // The id's were not yet computed search the system trees, get the ids and store them.
    if let Some(value) = get_node_value((id.0, id.1, "system"), data, behavior_type) {
        for (index, name) in data.system_names.iter().enumerate() {
            if let Some(str) = value.to_string() {
                if *name == str {
                    systems_id = Some(data.system_ids[index]);
                    break
                }
            }
        }
    }

    if let Some(value) = get_node_value((id.0, id.1, "tree"), data, behavior_type) {
        if let Some(systems_id) = systems_id {
            if let Some(system) = data.systems.get(&systems_id) {
                for (node_id, node) in &system.nodes {
                    if let Some(str) = value.to_string() {
                        if node.behavior_type == BehaviorNodeType::BehaviorTree && node.name == str {
                            systems_tree_id = Some(*node_id);
                                //data.instances[instance_index].node_values.insert((behavior_type, id.1), (systems_id as f64, *node_id as f64, 0.0, 0.0, "".to_string()));
                            break;
                        }
                    }
                }
            }
        }
    }


    //println!("systems id {:?}", systems_id);
    //println!("systems_tree_id id {:?}", systems_tree_id);

    if let Some(systems_id) = systems_id {
        if let Some(systems_tree_id) = systems_tree_id {
            data.instances[instance_index].systems_id = systems_id;
            data.execute_systems_node(instance_index, systems_tree_id);
            return BehaviorNodeConnector::Success;
        }
    }

    BehaviorNodeConnector::Fail
}

/// Behavior Call
pub fn call_behavior(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {

    let behavior_instance : Option<usize> = Some(get_local_instance_index(instance_index, data));
    let mut behavior_tree_id : Option<Uuid> = None;

    // TODO: Precompute this

    /*
    // The id's were not yet computed search the system trees, get the ids and store them.
    if let Some(value) = get_node_value((id.0, id.1, "execute_for"), data, behavior_type) {
        if value.0 == 0.0 {
            // Run the behavior on myself
            behavior_instance = Some(instance_index);
        } else {
            // Run the behavior on the target
            if let Some(target_index) = data.instances[instance_index].target_instance_index {
                behavior_instance = Some(target_index);
            }
        }
    }
    */

    if let Some(value) = get_node_value((id.0, id.1, "tree"), data, behavior_type) {
        if let Some(tree_name) = value.to_string() {
            if let Some(behavior_instance) = behavior_instance {
                if let Some(behavior) = data.behaviors.get(&data.instances[behavior_instance].behavior_id) {
                    for (node_id, node) in &behavior.nodes {
                        if node.behavior_type == BehaviorNodeType::BehaviorTree && node.name == tree_name {
                            behavior_tree_id = Some(*node_id);
                            break;
                        }
                    }
                }
            }
        }
    }

    // println!("behavior instance {:?}", behavior_instance);
    // println!("behavior_tree_id id {:?}", behavior_tree_id);

    if let Some(behavior_instance) = behavior_instance {
        if let Some(behavior_tree_id) = behavior_tree_id {
            data.execute_node(behavior_instance, behavior_tree_id, Some(instance_index));
            return BehaviorNodeConnector::Success;
        }
    }
    BehaviorNodeConnector::Fail
}

/// Lock Tree
pub fn lock_tree(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {

    let mut behavior_instance : Option<usize> = None;
    let mut behavior_tree_id : Option<Uuid> = None;
    let mut is_target = false;

    // We cannot precompute this as the values for the target may change

    // The id's were not yet computed search the system trees, get the ids and store them.
    if let Some(v) = get_node_value((id.0, id.1, "for"), data, behavior_type) {
        if let Some(value) = v.to_integer() {
            if value == 0 {
                // Run the behavior on myself
                behavior_instance = Some(get_local_instance_index(instance_index, data));
            } else {
                // Run the behavior on the target
                if let Some(target_index) = data.instances[instance_index].target_instance_index {
                    behavior_instance = Some(target_index);
                    is_target = true;
                }
            }
        }
    }

    if let Some(v) = get_node_value((id.0, id.1, "tree"), data, behavior_type) {
        if let Some(value) = v.to_string() {
            if let Some(behavior_instance) = behavior_instance {
                if let Some(behavior) = data.behaviors.get(&data.instances[behavior_instance].behavior_id) {
                    for (node_id, node) in &behavior.nodes {
                        if node.behavior_type == BehaviorNodeType::BehaviorTree && node.name == value {
                            behavior_tree_id = Some(*node_id);
                            break;
                        }
                    }
                }
            }
        }
    }

    // println!("behavior instance {:?}", behavior_instance);
    // println!("behavior_tree_id id {:?}", behavior_tree_id);

    if let Some(behavior_instance) = behavior_instance {
        if let Some(behavior_tree_id) = behavior_tree_id {
            // Lock the tree
            data.instances[behavior_instance].locked_tree = Some(behavior_tree_id);
            if is_target {
                // If we call lock on a target, we target ourself for the target
                data.instances[behavior_instance].target_instance_index = Some(instance_index);
            }
            return BehaviorNodeConnector::Success;
        }
    }
    BehaviorNodeConnector::Fail
}

/// Unlock Tree
pub fn unlock_tree(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {

    let mut behavior_instance : Option<usize> = None;

    if let Some(v) = get_node_value((id.0, id.1, "for"), data, behavior_type) {
        if let Some(value) = v.to_integer() {
            if value == 0 {
                // Run the behavior on myself
                behavior_instance = Some(get_local_instance_index(instance_index, data));
            } else {
                // Run the behavior on the target
                if let Some(target_index) = data.instances[instance_index].target_instance_index {
                    behavior_instance = Some(target_index);
                }
            }
        }
    }

    if let Some(behavior_instance) = behavior_instance {
        // Unlock the tree
        data.instances[behavior_instance].locked_tree = None;
        data.instances[behavior_instance].target_instance_index = None;
    }
    BehaviorNodeConnector::Bottom
}

/// Set State
pub fn set_state(instance_index: usize, id: (Uuid, Uuid), data: &mut RegionInstance, behavior_type: BehaviorType) -> BehaviorNodeConnector {

    let mut behavior_instance : Option<usize> = None;

    // The id's were not yet computed search the system trees, get the ids and store them.
    if let Some(v) = get_node_value((id.0, id.1, "for"), data, behavior_type) {
        if let Some(value) = v.to_integer() {
            if value == 0 {
                // Run the behavior on myself
                behavior_instance = Some(get_local_instance_index(instance_index, data));
            } else {
                // Run the behavior on the target
                if let Some(target_index) = data.instances[instance_index].target_instance_index {
                    behavior_instance = Some(target_index);
                }
            }
        }
    }

    if let Some(value) = get_node_value((id.0, id.1, "state"), data, behavior_type) {
        if let Some(behavior_instance) = behavior_instance {
            //println!("behavior instance {:?}", behavior_instance);
            if let Some(v) = value.to_integer() {
                data.instances[behavior_instance].state = match v {
                    1 => BehaviorInstanceState::Hidden,
                    2 => BehaviorInstanceState::Killed,
                    3 => BehaviorInstanceState::Purged,

                    _ => BehaviorInstanceState::Normal,
                };
            }

            // If != normal, clean this instance from all targets
            if data.instances[behavior_instance].state != BehaviorInstanceState::Normal {
                for i in 0..data.instances.len() {
                    if data.instances[i].target_instance_index == Some(behavior_instance) {
                        data.instances[i].target_instance_index = None;
                        data.instances[i].locked_tree = None;
                    }
                }
            }
        }
    }

    BehaviorNodeConnector::Bottom
}
