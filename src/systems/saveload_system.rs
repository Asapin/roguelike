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

use crate::components::{DefenseBonus, Equippable, Equipped, MeleePowerBonus, WantsToUnequipItem};
use crate::level::{MAP_HEIGHT, MAP_WIDTH};
use crate::{
    components::{
        AreaOfEffect, BlocksTile, CombatStats, Confusion, Consumable, InBackpack, InflictsDamage,
        Item, Monster, Name, Player, Position, ProvidesHealing, Ranged, Renderable,
        SerializationHelper, SerializeMe, SufferDamage, Viewshed, WantsToDropItem, WantsToMelee,
        WantsToPickupItem, WantsToUseItem,
    },
    map::Map,
};

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
    let map_copy = ecs.get_mut::<Map>().unwrap().clone();
    let save_helper = ecs
        .create_entity()
        .with(SerializationHelper { map: map_copy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

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
            SerializationHelper,
            Equippable,
            Equipped,
            MeleePowerBonus,
            DefenseBonus,
            WantsToUnequipItem
        );
    }

    // Clean up
    ecs.delete_entity(save_helper).expect("Crash on cleanup");
}

pub fn load_game(ecs: &mut World) {
    {
        let mut to_delete = Vec::new();
        for e in ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete {
            ecs.delete_entity(del).expect("Deletion failed");
        }
    }

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
            SerializationHelper,
            Equippable,
            Equipped,
            MeleePowerBonus,
            DefenseBonus,
            WantsToUnequipItem
        );
    }

    let mut delete_me: Option<Entity> = None;
    {
        let entities = ecs.entities();
        let helper = ecs.read_storage::<SerializationHelper>();
        let player = ecs.read_storage::<Player>();
        let position = ecs.read_storage::<Position>();
        for (e, h) in (&entities, &helper).join() {
            let mut world_map = ecs.write_resource::<Map>();
            *world_map = h.map.clone();
            world_map.tile_content = vec![Vec::new(); MAP_WIDTH as usize * MAP_HEIGHT as usize];
            delete_me = Some(e);
        }

        for (e, _p, pos) in (&entities, &player, &position).join() {
            let mut player_pos = ecs.write_resource::<rltk::Point>();
            *player_pos = Point::new(pos.x, pos.y);
            let mut player_resource = ecs.write_resource::<Entity>();
            *player_resource = e;
        }
    }

    ecs.delete_entity(delete_me.unwrap())
        .expect("Unable to delete helper");
}
