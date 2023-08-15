use std::fs::{self, File};
use std::path::Path;

use rltk::Point;
use specs::error::NoError;
use specs::saveload::SimpleMarkerAllocator;
use specs::{
    saveload::{DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker},
    Builder, World, WorldExt,
};
use specs::{Entity, Join};

use crate::components::{
    DefenseBonus, EntityMoved, EntryTrigger, Equippable, Equipped, GameLogSerializationHelper,
    Hidden, HungerClock, Lifetime, MeleePowerBonus, Particle, ProvidesFood, SingleActivation,
    WantsToUnequipItem,
};
use crate::gamelog::GameLog;
use crate::level::{MAP_HEIGHT, MAP_WIDTH};
use crate::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InBackpack, InflictsDamage,
        Item, MapSerializationHelper, Monster, Name, Player, Position, ProvidesHealing, Ranged,
        Renderable, SerializeMe, SufferDamage, Viewshed, WantsToDropItem, WantsToMelee,
        WantsToPickupItem, WantsToUseItem,
    },
    map::Map,
};

use super::particle_system::ParticleBuilder;

const SAVE_FILE: &str = "./savegame.json";

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

macro_rules! deserialize_individually {
    ($ecs:expr, $de:expr, $data:expr, $( $type:ty),*) => {
        $(
        DeserializeComponents::<NoError, _>::deserialize(
            &mut ( &mut $ecs.write_storage::<$type>(), ),
            &mut $data.0, // entities
            &mut $data.1, // marker
            &mut $data.2, // allocater
            &mut $de,
        )
        .unwrap();
        )*
    };
}

pub fn does_save_exist() -> bool {
    Path::new(SAVE_FILE).exists()
}

pub fn delete_save() {
    let _ = std::fs::remove_file(SAVE_FILE);
}

pub fn save_game(ecs: &mut World) {
    // Create helper
    let mut save_helpers = Vec::new();
    {
        let map_copy = ecs.get_mut::<Map>().unwrap().clone();
        let save_helper = ecs
            .create_entity()
            .with(MapSerializationHelper { map: map_copy })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        save_helpers.push(save_helper);
    }
    {
        let gamelog_copy = ecs.get_mut::<GameLog>().unwrap().clone();
        let save_helper = ecs
            .create_entity()
            .with(GameLogSerializationHelper {
                gamelog: gamelog_copy,
            })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        save_helpers.push(save_helper);
    }
    // Actually serialize
    {
        let data = (
            ecs.entities(),
            ecs.read_storage::<SimpleMarker<SerializeMe>>(),
        );

        let writer = File::create(SAVE_FILE).unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(
            ecs,
            serializer,
            data,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            MapSerializationHelper,
            GameLogSerializationHelper,
            Equippable,
            Equipped,
            MeleePowerBonus,
            DefenseBonus,
            WantsToUnequipItem,
            Lifetime,
            Particle,
            HungerClock,
            ProvidesFood,
            Hidden,
            EntryTrigger,
            EntityMoved,
            SingleActivation
        );
    }

    // Clean up
    for save_helper in save_helpers {
        ecs.delete_entity(save_helper).expect("Crash on cleanup");
    }
}

pub fn load_game(ecs: &mut World) {
    ecs.delete_all();
    let data = fs::read_to_string(SAVE_FILE).unwrap();
    let mut de = serde_json::Deserializer::from_str(&data);

    {
        let mut d = (
            &mut ecs.entities(),
            &mut ecs.write_storage::<SimpleMarker<SerializeMe>>(),
            &mut ecs.write_resource::<SimpleMarkerAllocator<SerializeMe>>(),
        );
        deserialize_individually!(
            ecs,
            de,
            d,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
            SufferDamage,
            WantsToMelee,
            Item,
            Consumable,
            Ranged,
            InflictsDamage,
            AreaOfEffect,
            Confusion,
            ProvidesHealing,
            InBackpack,
            WantsToPickupItem,
            WantsToUseItem,
            WantsToDropItem,
            MapSerializationHelper,
            GameLogSerializationHelper,
            Equippable,
            Equipped,
            MeleePowerBonus,
            DefenseBonus,
            WantsToUnequipItem,
            Lifetime,
            Particle,
            HungerClock,
            ProvidesFood,
            Hidden,
            EntryTrigger,
            EntityMoved,
            SingleActivation
        );
    }

    let mut helpers_to_delete = Vec::new();
    let mut loaded_map = None;
    let mut loaded_gamelog = None;
    {
        let entities = ecs.entities();
        let map_helper = ecs.read_storage::<MapSerializationHelper>();
        for (e, h) in (&entities, &map_helper).join() {
            let mut map = h.map.clone();
            map.tile_content = vec![Vec::new(); MAP_WIDTH as usize * MAP_HEIGHT as usize];
            loaded_map = Some(map);
            helpers_to_delete.push(e);
        }

        let log_helper = ecs.read_storage::<GameLogSerializationHelper>();
        for (e, h) in (&entities, &log_helper).join() {
            let gamelog = h.gamelog.clone();
            loaded_gamelog = Some(gamelog);
            helpers_to_delete.push(e);
        }

        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e, _p, pos) in (&entities, &player, &position).join() {
            let mut player_pos = ecs.write_resource::<rltk::Point>();
            *player_pos = Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }

    for delete_entity in helpers_to_delete {
        ecs.delete_entity(delete_entity)
            .expect("Unable to delete helper");
    }
    ecs.insert(loaded_map.unwrap());
    ecs.insert(loaded_gamelog.unwrap());
    ecs.insert(ParticleBuilder::new());
}
