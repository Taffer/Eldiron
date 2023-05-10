extern crate ref_thread_local;
use ref_thread_local::{RefThreadLocal};
use crate::prelude::*;
use rhai::Dynamic;

/// Evaluates the script of a node value. Stores the compiled AST inside the node for future reference.
pub fn eval_script(id: (Uuid, Uuid), value_name: &str, nodes: &mut FxHashMap<Uuid, GameBehaviorData>) {
    if let Some(item) = nodes.get_mut(&id.0) {
        if let Some(node) = item.nodes.get_mut(&id.1) {
            for (name, value) in &node.values {
                if *name == value_name {
                    if let Some(script) = value.to_string() {
                        let engine = &ENGINE.borrow();
                        if let Some(ast) = node.asts.get(value_name) {
                            let _rc = engine.eval_ast::<Dynamic>(ast);
                        } else {
                            let rc  = engine.compile(script);
                            if rc.is_ok() {
                                if let Some(ast) = rc.ok() {
                                    let _rc = engine.eval_ast::<Dynamic>(&ast);
                                    node.asts.insert(value_name.to_string(), ast);
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
    }
}

/// Evaluates a boolean expression in the given instance.
pub fn eval_bool_expression_instance(instance_index: usize, id: (BehaviorType, Uuid, Uuid, String), data: &mut RegionInstance) -> Option<bool> {
    //add_target_to_scope(instance_index, data);

    if let Some(ast) = data.ast.get(&id) {
        let r = data.engine.eval_ast_with_scope(&mut data.scopes[instance_index], ast);
        if r.is_ok() {
            return Some(r.unwrap());
        } else {
            println!("{:?}", r);
        }
    } else {
        if let Some(value) = get_node_value((id.1, id.2, &id.3), data, id.0) {
            if let Some(code) = value.to_string() {
                //let script = replace_target_variables(code);
                if let Some(ast) = data.engine.compile_expression_with_scope(&mut data.scopes[instance_index], code.as_str()).ok() {
                    let r = data.engine.eval_ast_with_scope(&mut  data.scopes[instance_index], &ast);
                    if r.is_ok() {
                        data.ast.insert(id.clone(), ast);
                        return Some(r.unwrap());
                    } else {
                        println!("{:?}", r);
                    }
                }
            }
        }
    }

    None
}

/// Evaluates a numerical expression in the given instance.
pub fn eval_number_expression_instance(instance_index: usize, id: (BehaviorType, Uuid, Uuid, String), data: &mut RegionInstance) -> Option<f32> {
    if let Some(ast) = data.ast.get(&id) {
        let r = data.engine.eval_ast_with_scope::<Dynamic>(&mut  data.scopes[instance_index], ast);
        if r.is_ok() {
            let nn = r.unwrap();
            if let Some(n) = nn.as_float().ok() {
                return Some(n);
            }
            if let Some(n) = nn.as_int().ok() {
                return Some(n as f32);
            }
        } else {
            println!("{:?}", r);
        }
    } else {
        if let Some(value) = get_node_value((id.1, id.2, &id.3), data, id.0) {
            if let Some(code) = value.to_string() {
                if let Some(ast) = data.engine.compile_expression_with_scope(&mut  data.scopes[instance_index], code.as_str()).ok() {
                    let r = data.engine.eval_ast_with_scope::<Dynamic>(&mut  data.scopes[instance_index], &ast);
                    if r.is_ok() {
                        data.ast.insert(id.clone(), ast);
                        let nn = r.unwrap();
                        if let Some(n) = nn.as_float().ok() {
                            return Some(n);
                        }
                        if let Some(n) = nn.as_int().ok() {
                            return Some(n as f32);
                        }
                    } else {
                        println!("{:?}", r);
                    }
                }
            }
        }
    }
    None
}

/// Evaluates a dynamic script in the given instance.
pub fn eval_dynamic_script_instance(instance_index: usize, id: (BehaviorType, Uuid, Uuid, String), data: &mut RegionInstance) -> bool {
    if let Some(ast) = data.ast.get(&id) {
        let r = data.engine.eval_ast_with_scope::<Dynamic>(&mut  data.scopes[instance_index], ast);
        if r.is_ok() {
            return true
        } else {
            println!("{:?}", r);
        }
    } else {
        if let Some(value) = get_node_value((id.1, id.2, &id.3), data, id.0) {
            if let Some(code) = value.to_string() {
                if let Some(ast) = data.engine.compile_with_scope(&mut  data.scopes[instance_index], code.as_str()).ok() {
                    let r = data.engine.eval_ast_with_scope::<Dynamic>(&mut  data.scopes[instance_index], &ast);
                    if r.is_ok() {
                        data.ast.insert(id.clone(), ast);
                        return true
                    } else
                    if let Some(err) = r.err() {
                        data.script_errors.push(
                            ((id.1, id.2, id.3), (err.to_string(), None))
                        );
                    }
                }
            }
        }
    }

    false
}

/// Evaluates a dynamic script in the given instance.
pub fn eval_dynamic_script_instance_for_game_player_scope(_instance_index: usize, id: (BehaviorType, Uuid, Uuid, String), data: &mut RegionInstance, custom_scope: usize) -> bool {

    if let Some(ast) = data.ast.get(&id) {
        if let Some(custom_scope) = data.game_player_scopes.get_mut(&custom_scope) {

            let r = data.engine.eval_ast_with_scope::<Dynamic>(custom_scope, ast);
            if r.is_ok() {
                return true
            } else {
                println!("{:?}", r);
            }
        }
    } else {
        if let Some(value) = get_node_value((id.1, id.2, &id.3), data, id.0) {
            if let Some(script) = value.to_string() {
                if let Some(ast) = data.engine.compile_with_scope(data.game_player_scopes.get_mut(&custom_scope).unwrap(), script.as_str()).ok() {
                    let r = data.engine.eval_ast_with_scope::<Dynamic>(data.game_player_scopes.get_mut(&custom_scope).unwrap(), &ast);
                    if r.is_ok() {
                        data.ast.insert(id.clone(), ast);
                        return true
                    } else {
                        println!("{:?}", r);
                    }
                }
            }
        }
    }

    false
}
