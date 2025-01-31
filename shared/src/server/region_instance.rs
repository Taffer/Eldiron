use super::prelude::*;
use crate::prelude::*;
use crate::server::{REGIONS, UPDATES};
use theframework::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegionInstance {
    pub id: Uuid,

    sandbox: TheCodeSandbox,

    #[serde(skip)]
    characters: FxHashMap<Uuid, TheCodePackage>,

    #[serde(skip)]
    characters_instances: FxHashMap<Uuid, TheCodePackage>,

    /// For fast lookups an array of (character_instance_id, character_id) tuples.
    #[serde(skip)]
    characters_ids: Vec<(Uuid, Uuid)>,

    redraw_ms: u32,
    tick_ms: u32,
}

impl Default for RegionInstance {
    fn default() -> Self {
        Self::new()
    }
}

impl RegionInstance {
    pub fn new() -> Self {
        let sandbox = TheCodeSandbox::new();

        Self {
            id: Uuid::nil(),

            sandbox,

            characters: FxHashMap::default(),
            characters_instances: FxHashMap::default(),
            characters_ids: vec![],

            redraw_ms: 1000 / 30,
            tick_ms: 250,
        }
    }

    /// Sets up the region instance.
    pub fn setup(&mut self, id: Uuid, project: &Project) {
        self.id = id;
        self.sandbox.id = id;

        self.tick_ms = project.tick_ms;
        self.redraw_ms = 1000 / project.target_fps;
    }

    /// Tick. Compute the next frame.
    pub fn tick(&mut self) {
        // We iterate over all character instances and execute their main function
        // as well as the main function of their character template.
        for (instance_id, character_id) in &mut self.characters_ids {
            self.sandbox.clear_runtime_states();
            self.sandbox
                .aliases
                .insert("self".to_string(), *instance_id);

            // if let Some(instance) = self.characters_instances.get_mut(instance_id) {
            //     instance.execute("main".to_string(), &mut self.sandbox);
            // }

            if let Some(instance) = self.characters.get_mut(character_id) {
                instance.execute("main".to_string(), &mut self.sandbox);

                // println!(
                //     "instance_id: {}, debug {:?}",
                //     character_id,
                //     self.sandbox.get_codegrid_debug_module(*character_id)
                // );
            }
        }
    }

    /// Create an instance from json.
    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap_or_default()
    }

    /// Convert the instance to json.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }

    /// Sets the debugging mode.
    pub fn set_debug_mode(&mut self, debug_mode: bool) {
        self.sandbox.debug_mode = debug_mode;
    }

    /// Returns the debug module (if any) for the given module_id.
    pub fn get_module_debug_module(&self, id: Uuid) -> TheDebugModule {
        self.sandbox.get_module_debug_module(id)
    }

    /// Returns the debug module (if any) for the given codegrid_id.
    pub fn get_codegrid_debug_module(&self, id: Uuid) -> TheDebugModule {
        self.sandbox.get_codegrid_debug_module(id)
    }

    /// Draws this instance into the given buffer.
    pub fn draw(
        &mut self,
        buffer: &mut TheRGBABuffer,
        tiledrawer: &TileDrawer,
        anim_counter: &usize,
        ctx: &mut TheContext,
        server_ctx: &ServerContext,
    ) {
        let delta = self.redraw_ms as f32 / self.tick_ms as f32;

        if let Some(region) = REGIONS.read().unwrap().get(&self.id) {
            let grid_size = region.grid_size as f32;

            tiledrawer.draw_region(buffer, region, anim_counter, ctx);

            if let Some(update) = UPDATES.write().unwrap().get_mut(&self.id) {
                for (id, character) in &mut update.characters{

                    let draw_pos = if let Some((start, end)) = &mut character.moving {

                        // pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
                        //     let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
                        //     t * t * (3.0 - 2.0 * t)
                        // }

                        let sum = (delta + character.move_delta).clamp(0.0, 1.0);
                        //let d = smoothstep(0.0, 1.0, sum);//.clamp(0.0, 1.0);
                        let d = if sum < 0.5 {
                            2.0 * sum * sum
                        } else {
                            1.0 - (-2.0 * sum + 2.0).powi(2) / 2.0
                        };
                        let x = start.x * (1.0 - d) + end.x * d;
                        let y = start.y * (1.0 - d) + end.y * d;
                        character.move_delta = sum;
                        vec2i((x * grid_size).round() as i32, (y * grid_size).round() as i32)
                    } else {
                        vec2i((character.position.x * grid_size) as i32, (character.position.y * grid_size) as i32)
                    };

                    //println!("moving: {:?}", draw_pos);

                    if !tiledrawer.draw_tile_at_pixel(
                        draw_pos,
                        buffer,
                        character.tile_id,
                        anim_counter,
                        ctx,
                    ) {
                        if let Some(found_id) =
                            tiledrawer.get_tile_id_by_name(character.tile_name.clone())
                        {
                            character.tile_id = found_id;
                            tiledrawer.draw_tile_at_pixel(
                                draw_pos,
                                buffer,
                                found_id,
                                anim_counter,
                                ctx,
                            );
                        } else {
                            //println!("RegionInstance::draw: Tile not found: {}", name);
                        }
                    }

                    if Some(*id) == server_ctx.curr_character_instance {
                        tiledrawer.draw_tile_outline_at_pixel(
                            draw_pos,
                            buffer,
                            WHITE,
                            ctx,
                        );
                    } else if Some(*id) == server_ctx.curr_character {
                        tiledrawer.draw_tile_outline_at_pixel(
                            draw_pos,
                            buffer,
                            [128, 128, 128, 255],
                            ctx,
                        );
                    }
                }
            }
            /*
            for c in self.sandbox.objects.values_mut() {
                if let Some(TheValue::Position(p)) = c.get(&"position".into()).cloned() {
                    if let Some(TheValue::Tile(name, id)) = c.get_mut(&"tile".into()) {
                        //println!("p {:?} s {:?}", p, name);

                        if !tiledrawer.draw_tile(
                            vec2i(p.x as i32, p.y as i32),
                            buffer,
                            region.grid_size,
                            *id,
                            anim_counter,
                            ctx,
                        ) {
                            if let Some(found_id) = tiledrawer.get_tile_id_by_name(name.clone()) {
                                *id = found_id;
                                tiledrawer.draw_tile(
                                    vec2i(p.x as i32, p.y as i32),
                                    buffer,
                                    region.grid_size,
                                    found_id,
                                    anim_counter,
                                    ctx,
                                );
                            } else {
                                //println!("RegionInstance::draw: Tile not found: {}", name);
                            }
                        }
                    }
                }

                if Some(c.id) == server_ctx.curr_character_instance {
                    if let Some(TheValue::Position(p)) = c.get(&"position".into()) {
                        tiledrawer.draw_tile_outline(
                            vec2i(p.x as i32, p.y as i32),
                            buffer,
                            region.grid_size,
                            WHITE,
                            ctx,
                        );
                    }
                } else if Some(c.id) == server_ctx.curr_character {
                    if let Some(TheValue::Position(p)) = c.get(&"position".into()) {
                        tiledrawer.draw_tile_outline(
                            vec2i(p.x as i32, p.y as i32),
                            buffer,
                            region.grid_size,
                            [128, 128, 128, 255],
                            ctx,
                        );
                    }
                }
            }*/
        }
    }

    /// Insert a (TheCodePackage) to the region.
    pub fn insert_character(&mut self, mut character: TheCodePackage) {
        // We collect all instances of this character and execute the init function on them.
        let mut instance_ids = vec![];
        for o in self.sandbox.objects.values() {
            if o.package_id == character.id {
                instance_ids.push(o.id);
            }
        }

        for id in instance_ids {
            self.sandbox.clear_runtime_states();
            self.sandbox.aliases.insert("self".to_string(), id);
            character.execute("init".to_string(), &mut self.sandbox);

            if let Some(inst) = self.characters_instances.get_mut(&id) {
                inst.execute("init".to_string(), &mut self.sandbox);
            }
        }

        self.characters.insert(character.id, character);
    }

    /// Adds a character instance to the region.
    pub fn add_character_instance(&mut self, mut character: Character) -> Option<Uuid> {
        let mut package = TheCodePackage::new();
        package.id = character.id;

        let mut module_id = None;

        let mut compiler = TheCompiler::new();

        for grid in character.instance.grids.values_mut() {
            let rc = compiler.compile(grid);
            if let Ok(mut module) = rc {
                module.name = grid.name.clone();
                println!(
                    "RegionInstance::add_character_instance: Compiled grid module: {}",
                    grid.name
                );
                module_id = Some(module.id);
                package.insert_module(module.name.clone(), module);
            } else {
                println!(
                    "RegionInstance::add_character_instance: Failed to compile grid: {}",
                    grid.name
                );
            }
        }

        let mut o = TheCodeObject::new();
        o.id = character.id;

        self.sandbox.clear_runtime_states();
        self.sandbox.aliases.insert("self".to_string(), o.id);

        if let Some(template) = self.characters.get_mut(&character.character_id) {
            o.package_id = template.id;
            self.sandbox.add_object(o);
            template.execute("init".to_string(), &mut self.sandbox);
        }

        package.execute("init".to_string(), &mut self.sandbox);

        // Add the character to the update struct.
        if let Some(object) = self.sandbox.objects.get_mut(&character.id) {
            let mut character_update = CharacterUpdate::new();
            if let Some(TheValue::Position(p)) = object.get(&"position".into()) {
                character_update.position = vec2f(p.x, p.y);
            }
            if let Some(TheValue::Text(t)) = object.get(&"name".into()) {
                character_update.name = t.clone();
            }
            if let Some(TheValue::Tile(name, id)) = object.get_mut(&"tile".into()) {
                character_update.tile_name = name.clone();
                character_update.tile_id = *id;
            }

            if let Some(update) = UPDATES.write().unwrap().get_mut(&self.id) {
                update.characters.insert(character.id, character_update);
            }
        }

        self.characters_ids
            .push((character.id, character.character_id));
        self.characters_instances.insert(package.id, package);

        module_id
    }

    /// Updates a character instance.
    pub fn update_character_instance_bundle(&mut self, character: Uuid, mut bundle: TheCodeBundle) {
        if let Some(existing_package) = self.characters_instances.get_mut(&character) {
            let mut package = TheCodePackage::new();

            let mut compiler = TheCompiler::new();

            for grid in bundle.grids.values_mut() {
                let rc = compiler.compile(grid);
                if let Ok(mut module) = rc {
                    module.name = grid.name.clone();
                    println!(
                        "RegionInstance::add_character_instance: Compiled grid module: {}",
                        grid.name
                    );
                    package.insert_module(module.name.clone(), module);
                } else {
                    println!(
                        "RegionInstance::add_character_instance: Failed to compile grid: {}",
                        grid.name
                    );
                }
            }

            self.sandbox.clear_runtime_states();
            self.sandbox.aliases.insert("self".to_string(), character);

            package.execute("init".to_string(), &mut self.sandbox);

            *existing_package = package;
        }
    }

    /// Removes the given character instance from the region.
    pub fn remove_character_instance(&mut self, character: Uuid) {
        self.characters_instances.remove(&character);
        self.characters_ids
            .retain(|(instance_id, _)| *instance_id != character);
        self.sandbox.objects.remove(&character);
    }

    /// Returns the character instance id and the character id for the character at the given position.
    pub fn get_character_at(&self, pos: Vec2i) -> Option<(Uuid, Uuid)> {
        for c in self.sandbox.objects.values() {
            if let Some(TheValue::Position(p)) = c.get(&"position".into()).cloned() {
                if vec2i(p.x as i32, p.y as i32) == pos {
                    for (instance_id, character_id) in &self.characters_ids {
                        if *instance_id == c.id {
                            return Some((*instance_id, *character_id));
                        }
                    }
                }
            }
        }

        None
    }

    /// Returns the value of the given character instance property along with its character id.
    pub fn get_character_property(
        &self,
        character_id: Uuid,
        property: String,
    ) -> Option<(TheValue, Uuid)> {
        for (id, c) in &self.sandbox.objects {
            if *id == character_id {
                if let Some(value) = c.get(&property).cloned() {
                    for (instance_id, character_id) in &self.characters_ids {
                        if *instance_id == c.id {
                            return Some((value.clone(), *character_id));
                        }
                    }
                }
            }
        }

        None
    }

    /// Returns the object of the given character instance property along with its character id.
    pub fn get_character_object(&self, character_id: Uuid) -> Option<(TheCodeObject, Uuid)> {
        for (id, c) in &self.sandbox.objects {
            if *id == character_id {
                for (instance_id, character_id) in &self.characters_ids {
                    if *instance_id == c.id {
                        return Some((c.clone(), *character_id));
                    }
                }
            }
        }

        None
    }
}
