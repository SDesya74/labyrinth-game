use bevy::{prelude::*, ecs::DynamicBundle};


use std::{time::Duration, collections::{hash_map::Values, HashMap}};

use lab_core::stage;


mod systems;

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut AppBuilder) {        
        app
            .add_resource(SpriteLibrary::new())
            .add_startup_system_to_stage(stage::INIT, crate::systems::load_world_sprites_system.system());
    }
}


#[derive(Clone, Properties, Debug, Default)]
pub struct SpriteInfo {
    pub name:  String,
    pub atlas_sprite : u32,
    pub atlas_handle : Handle<TextureAtlas>,
    pub height: u32,
    pub width: u32
}

impl SpriteInfo {
    pub fn size(&self) -> Vec2 {
        return Vec2::new(self.width as f32, self.height as f32);
    }
}

pub struct Letter;
pub struct StationaryLetter;

pub struct SpriteLibrary {
    library: HashMap<String, SpriteInfo>
}

impl SpriteLibrary {
    pub fn new () -> SpriteLibrary {
        SpriteLibrary {
            library : HashMap::new()
        }
    }

    pub fn len(&self) -> usize {
        self.library.len()
    }

   
    pub fn iter(&mut self) -> Values<'_, String, SpriteInfo> {
        self.library.values()
    }
    pub fn add(&mut self,sprite: SpriteInfo){
        self.library.insert(sprite.name.to_string(), sprite);
    }

    pub fn get(&self, name : &str) -> Option<&SpriteInfo> {
        self.library.get(name)
    }

    pub fn width_for_char(&self,c : char) -> f32 {
        match c {
            'i' => 16.,
            'm' => 20.,
            _ => 14.
        }
    }

    pub fn make_string(&self, st : String, mut location : Vec3) -> Vec<(Vec3,SpriteInfo)> {
        let mut sprites_for_string = Vec::<(Vec3, SpriteInfo)>::new();
        
        for c in st.to_lowercase().chars().into_iter() {
            if c == ' ' || c == '_' {
                *location.x_mut() += 8.;
                continue;
            }
            if let Some(sprite) = self.get(&format!("{}", c)){
                sprites_for_string.push((location.clone(), sprite.clone()));
                *location.x_mut() += self.width_for_char(c);
            } else {
                println!("Couldn't find sprite for letter {}", c);
            }
        }
        //*location.z_mut() = 10.;

        sprites_for_string
    }

    pub fn catalog_sprites(
        &mut self,
        asset_server: &AssetServer, 
        assets: &mut ResMut<Assets<Texture>>,     
        texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
        filename : &str, 
        labels: &[&'static str], 
        dim : (usize,usize)) {
        let texture_handle = asset_server
        .load_sync(
            assets,
            filename,
        )
        .unwrap();
    
        let texture = assets.get(&texture_handle).unwrap();
    
        let texture_atlas = TextureAtlas::from_grid(texture_handle, texture.size, dim.0, dim.1);
    
        let size = texture_atlas.size / Vec2::new(dim.0 as f32, dim.1 as f32);
        
        let texture_atlas_handle = texture_atlases.add(texture_atlas);

        labels
        .iter()
        .enumerate()
        .for_each(|(i,s)| self.add(SpriteInfo::new(s.to_string(),  i as u32, texture_atlas_handle.clone(), size.x() as u32, size.y() as u32)));
        
    }

    pub fn write_despawning_text(&self,  
        mut commands : &mut Commands,
        st : String, 
        duration : Duration, 
        location : Vec3){

        for c in self.make_string(st, location).into_iter() {     
            commands.spawn(c.1.to_components(c.0, Scale(1.)))
                .with_bundle((StationaryLetter,lab_core::Despawn,Timer::new(duration, false)));
        };
    }
    pub fn write_text(&self,  
        mut commands : &mut Commands,
        st : String,  
        location : Vec3) -> Vec<Entity>
    {
       
        self.make_string(st, location).into_iter().map(move |c| {            
            commands.spawn(c.1.to_components(c.0, Scale(1.))).current_entity().unwrap()
        }).collect()
        
    }
    pub fn place_despawning_sprite(&self,  
        mut commands :&mut Commands,
        name : String, 
        scale : Scale, 
        duration : Duration, 
        location : Vec3,
        bundle : impl DynamicBundle + Send + Sync + 'static) -> Entity {
                
        commands.spawn(self.get(&name).unwrap().to_components(location, scale))
            .with(lab_core::Despawn)
            .with(Timer::new(duration, true))
            .with_bundle(bundle)
            .current_entity().unwrap()
    }
}

impl SpriteInfo {
    pub fn new (name : String, sprite_idx: u32, handle: Handle<TextureAtlas>, width: u32, height: u32) -> SpriteInfo {
         return SpriteInfo {
             name: name.to_string(),
             atlas_sprite: sprite_idx,
             atlas_handle: handle,
             width,
             height
         }
    }

    pub fn to_components(&self, loc : Vec3, scale: Scale) -> SpriteSheetComponents {
        SpriteSheetComponents {
            translation: Translation::new(loc.x(), loc.y(), loc.z()),
            scale: scale,
            draw: Draw { is_visible: true, is_transparent: true, ..Default::default() },
            sprite: TextureAtlasSprite::new(self.atlas_sprite),
            texture_atlas: self.atlas_handle.clone(),
            ..Default::default()
        }
    }
}

pub struct MoveAnimation {
    pub up: Vec<SpriteInfo>,
    pub down: Vec<SpriteInfo>,
    pub left: Vec<SpriteInfo>,
    pub right: Vec<SpriteInfo>,
    pub count: usize
}

impl Default for MoveAnimation {
    fn default() -> Self {
        MoveAnimation {
            up: Vec::new(),
            down: Vec::new(),
            left: Vec::new(),
            right: Vec::new(),
            count: 0
        }
    }
}


#[derive(Debug, Properties, Default, Clone)]
pub struct TileAnimation {
    pub states: Vec<SpriteInfo>,
    pub count: usize
}
