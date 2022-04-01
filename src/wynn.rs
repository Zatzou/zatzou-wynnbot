#![allow(
    non_snake_case,
    non_camel_case_types,
    dead_code,
    clippy::upper_case_acronyms
)]
//! Module for wynncraft type definitions

/// Module containing constants for wynncraft colors
pub mod color {
    use poise::serenity::utils::Color;

    pub const DEPRESSING_ITEM: Color = Color::from_rgb(170, 170, 170);
    pub const NORMAL_ITEM: Color = Color::from_rgb(255, 255, 255);
    pub const UNIQUE_ITEM: Color = Color::from_rgb(255, 255, 85);
    pub const RARE_ITEM: Color = Color::from_rgb(247, 82, 247);
    pub const LEGENDARY_ITEM: Color = Color::from_rgb(0, 255, 255);
    pub const FABLED_ITEM: Color = Color::from_rgb(255, 85, 85);
    pub const MYTHIC_ITEM: Color = Color::from_rgb(170, 0, 170);
    pub const SET_ITEM: Color = Color::from_rgb(40, 150, 24);
}

/// Module containing structs for holding world information such as territories
pub mod world {
    use std::collections::HashMap;

    use serde::Deserialize;

    /// Struct for the wynntils territories api
    #[derive(Deserialize, Clone)]
    pub struct Territories {
        pub territories: HashMap<String, Territory>,
    }

    /// Data off a single territory from the wynntils api
    #[derive(Deserialize, Clone)]
    pub struct Territory {
        pub territory: String,
        pub guild: String,
        pub guildPrefix: String,
        pub guildColor: Option<String>,
        pub acquired: String,
        // There is additional data we aren't capturing
        pub location: TerritoryLocation,
    }

    /// Struct representing the location of a territory
    ///
    /// Due to wynncraft/wynntils api fun these values can be all over the place
    #[derive(Deserialize, Clone, Copy)]
    pub struct TerritoryLocation {
        pub startX: f64,
        pub startZ: f64,
        pub endX: f64,
        pub endZ: f64,
    }
}

/// Item information and data
pub mod items {
    use std::{collections::BTreeMap, ops::RangeInclusive};

    use poise::serenity::utils::Color;
    use serde::Deserialize;

    use crate::wynn::color;

    /// All possible rarities of items
    #[derive(Debug, Deserialize, Clone)]
    pub enum Rarity {
        NORMAL,
        UNIQUE,
        RARE,
        LEGENDARY,
        FABLED,
        MYTHIC,
        SET,
    }

    /// Item types
    #[derive(Debug, Deserialize, Clone)]
    pub enum Type {
        SPEAR,
        WAND,
        BOW,
        DAGGER,
        RELIK,
        HELMET,
        CHESTPLATE,
        LEGGINGS,
        BOOTS,
        RING,
        BRACELET,
        NECKLACE,
    }

    /// all current wynncraft identifications
    #[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
    pub enum Identification {
        rawStrength,
        rawDexterity,
        rawIntelligence,
        rawDefence,
        rawAgility,
        attackSpeed,
        rawMainAttackNeutralDamage,
        mainAttackDamage,
        rawNeutralSpellDamage,
        rawSpellDamage,
        spellDamage,
        rawHealth,
        rawHealthRegen,
        healthRegen,
        lifeSteal,
        manaRegen,
        manaSteal,
        earthDamage,
        thunderDamage,
        waterDamage,
        fireDamage,
        airDamage,
        earthDefence,
        thunderDefence,
        waterDefence,
        fireDefence,
        airDefence,
        exploding,
        poison,
        thorns,
        reflection,
        walkSpeed,
        sprint,
        sprintRegen,
        rawJumpHeight,
        soulPointRegen,
        lootBonus,
        lootQuality,
        #[serde(alias = "stealing")]
        emeraldStealing,
        xpBonus,
        gatherXPBonus,
        gatherSpeed,
        raw1stSpellCost,
        #[serde(rename = "1stSpellCost")]
        SpellCost1,
        raw2ndSpellCost,
        #[serde(rename = "2ndSpellCost")]
        SpellCost2,
        raw3rdSpellCost,
        #[serde(rename = "3rdSpellCost")]
        SpellCost3,
        raw4thSpellCost,
        #[serde(rename = "4thSpellCost")]
        SpellCost4,
    }

    impl Identification {
        /// Get the name of the identification as a string
        pub fn name(&self) -> &str {
            match self {
                Identification::rawStrength => "Strength",
                Identification::rawDexterity => "Dexterity",
                Identification::rawIntelligence => "Intelligence",
                Identification::rawDefence => "Defence",
                Identification::rawAgility => "Agility",
                Identification::attackSpeed => "Attack Speed",
                Identification::rawMainAttackNeutralDamage => "Main Attack Neutral Damage",
                Identification::mainAttackDamage => "Main Attack Damage",
                Identification::rawNeutralSpellDamage => "Spell Damage",
                Identification::rawSpellDamage => "Spell Damage",
                Identification::spellDamage => "Spell Damage",
                Identification::rawHealth => "Health",
                Identification::rawHealthRegen => "Health Regen",
                Identification::healthRegen => "Health Regen",
                Identification::lifeSteal => "Life Steal",
                Identification::manaRegen => "Mana Regen",
                Identification::manaSteal => "Mana Steal",
                Identification::earthDamage => "Earth Damage",
                Identification::thunderDamage => "Thunder Damage",
                Identification::waterDamage => "Water Damage",
                Identification::fireDamage => "Fire Damage",
                Identification::airDamage => "Air Damage",
                Identification::earthDefence => "Earth Defence",
                Identification::thunderDefence => "Thunder Defence",
                Identification::waterDefence => "Water Defence",
                Identification::fireDefence => "Fire Defence",
                Identification::airDefence => "Air Defence",
                Identification::exploding => "Exploding",
                Identification::poison => "Poison",
                Identification::thorns => "Thorns",
                Identification::reflection => "Reflection",
                Identification::walkSpeed => "Walk Speed",
                Identification::sprint => "Sprint",
                Identification::sprintRegen => "Sprint Regen",
                Identification::rawJumpHeight => "Jump Height",
                Identification::soulPointRegen => "Soul Point Regen",
                Identification::lootBonus => "Loot Bonus",
                Identification::lootQuality => "Loot Quality",
                Identification::emeraldStealing => "Stealing",
                Identification::xpBonus => "XP Bonus",
                Identification::gatherXPBonus => "Gather XP Bonus",
                Identification::gatherSpeed => "Gather Speed",
                Identification::raw1stSpellCost => "1st Spell Cost",
                Identification::SpellCost1 => "1st Spell Cost",
                Identification::raw2ndSpellCost => "2nd Spell Cost",
                Identification::SpellCost2 => "2nd Spell Cost",
                Identification::raw3rdSpellCost => "3rd Spell Cost",
                Identification::SpellCost3 => "3rd Spell Cost",
                Identification::raw4thSpellCost => "4th Spell Cost",
                Identification::SpellCost4 => "4th Spell Cost",
            }
        }
    }

    #[derive(Debug, Deserialize, Clone, Copy)]
    pub enum AttackSpeed {
        SUPER_SLOW,
        VERY_SLOW,
        SLOW,
        NORMAL,
        FAST,
        VERY_FAST,
        SUPER_FAST,
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct ItemList {
        pub items: Vec<Item>,
        pub identificationOrder: IdentificationOrder,
    }

    /// Representation of a wynntils api item
    #[derive(Debug, Deserialize, Clone)]
    pub struct Item {
        /// Name of the item
        pub displayName: String,
        /// Rarity of the item
        pub tier: Rarity,
        /// number of powders on the item
        pub powderAmount: u8,
        /// Information about the item
        pub itemInfo: ItemInfo,
        /// Requirements to wear the item
        pub requirements: Requirements,
        /// Damage values of the item
        pub damageTypes: Option<DamageTypes>,
        /// Defence values of the item
        pub defenseTypes: Option<DefenseTypes>,
        /// Attack speed of the item
        pub attackSpeed: Option<AttackSpeed>,
        /// Statuses or the ids of the item
        pub statuses: BTreeMap<Identification, StatusId>,
        /// Wynnbuilder id for the item
        pub wynnBuilderID: Option<i32>,
    }

    impl Item {
        /// Gets the serenity color for the item's rarity
        pub fn get_color(&self) -> Color {
            match self.tier {
                Rarity::NORMAL => color::NORMAL_ITEM,
                Rarity::UNIQUE => color::UNIQUE_ITEM,
                Rarity::RARE => color::RARE_ITEM,
                Rarity::LEGENDARY => color::LEGENDARY_ITEM,
                Rarity::FABLED => color::FABLED_ITEM,
                Rarity::MYTHIC => color::MYTHIC_ITEM,
                Rarity::SET => color::SET_ITEM,
            }
        }

        /// Gets the item's rarity as a String
        pub fn get_rarity(&self) -> String {
            match self.tier {
                Rarity::NORMAL => String::from("Normal"),
                Rarity::UNIQUE => String::from("Unique"),
                Rarity::RARE => String::from("Rare"),
                Rarity::LEGENDARY => String::from("Legendary"),
                Rarity::FABLED => String::from("Fabled"),
                Rarity::MYTHIC => String::from("Mythic"),
                Rarity::SET => String::from("Set"),
            }
        }

        /// Gets the item's type as a String
        pub fn get_type(&self) -> String {
            match self.itemInfo.r#type {
                Type::SPEAR => String::from("Spear"),
                Type::WAND => String::from("Wand"),
                Type::BOW => String::from("Bow"),
                Type::DAGGER => String::from("Dagger"),
                Type::RELIK => String::from("Relik"),
                Type::HELMET => String::from("Helmet"),
                Type::CHESTPLATE => String::from("Chestplate"),
                Type::LEGGINGS => String::from("Leggings"),
                Type::BOOTS => String::from("Boots"),
                Type::RING => String::from("Ring"),
                Type::BRACELET => String::from("Bracelet"),
                Type::NECKLACE => String::from("Necklace"),
            }
        }

        /// Gets the attack speed of the item and formats it into a speed
        pub fn get_speed(&self) -> Option<String> {
            if let Some(speed) = self.attackSpeed {
                match speed {
                    AttackSpeed::SUPER_SLOW => Some(String::from("Super Slow")),
                    AttackSpeed::VERY_SLOW => Some(String::from("Very Slow")),
                    AttackSpeed::SLOW => Some(String::from("Slow Attack")),
                    AttackSpeed::NORMAL => Some(String::from("Normal")),
                    AttackSpeed::FAST => Some(String::from("Fast")),
                    AttackSpeed::VERY_FAST => Some(String::from("Very Fast")),
                    AttackSpeed::SUPER_FAST => Some(String::from("Super Fast")),
                }
            } else {
                None
            }
        }
    }

    #[derive(Debug, Deserialize, Clone)]
    pub struct ItemInfo {
        pub r#type: Type,
    }

    /// requirements of an item
    #[derive(Debug, Deserialize, Clone)]
    pub struct Requirements {
        pub level: Option<i32>,
        pub strength: Option<i32>,
        pub dexterity: Option<i32>,
        pub intelligence: Option<i32>,
        pub defense: Option<i32>,
        pub agility: Option<i32>,
    }

    /// damagetypes of the item
    #[derive(Debug, Deserialize, Clone)]
    pub struct DamageTypes {
        pub neutral: Option<String>,
        pub earth: Option<String>,
        pub thunder: Option<String>,
        pub water: Option<String>,
        pub fire: Option<String>,
        pub air: Option<String>,
    }

    /// defensetypes of the item
    #[derive(Debug, Deserialize, Clone)]
    pub struct DefenseTypes {
        pub health: Option<i32>,
        pub earth: Option<i32>,
        pub thunder: Option<i32>,
        pub water: Option<i32>,
        pub fire: Option<i32>,
        pub air: Option<i32>,
    }

    /// Type of id
    #[derive(Debug, Deserialize, Clone, Copy)]
    pub enum StatusType {
        PERCENTAGE,
        INTEGER,
        FOUR_SECONDS,
        THREE_SECONDS,
        TIER,
    }

    /// Struct containing a single id for an item.
    ///
    /// This format is intended for deserialisation and does not contain the actual id type.
    #[derive(Debug, Deserialize, Clone)]
    pub struct StatusId {
        pub r#type: StatusType,
        pub isFixed: bool,
        pub baseValue: i32,
    }

    /// Struct for holding the order of identifications as defied by the wynntils api
    #[derive(Debug, Deserialize, Clone)]
    pub struct IdentificationOrder {
        pub order: BTreeMap<Identification, i32>,
        pub groups: Vec<String>,
        pub inverted: Vec<Identification>,
    }

    /// Groups for ids this should probably not be hardcoded but neither should many other things here
    pub const IDGROUPS: [RangeInclusive<i32>; 9] = [
        1..=5,
        6..=11,
        12..=17,
        18..=22,
        23..=27,
        28..=31,
        32..=35,
        36..=42,
        43..=50,
    ];

    /// Powder types
    pub enum Powders {
        EARTH,
        THUNDER,
        WATER,
        FIRE,
        AIR,
    }

    impl Powders {
        pub fn from_i32(n: i32) -> Self {
            match n {
                0 => Powders::EARTH,
                1 => Powders::THUNDER,
                2 => Powders::WATER,
                3 => Powders::FIRE,
                4 => Powders::AIR,
                _ => Powders::AIR,
            }
        }
    }
}

/// Module for gather information
pub mod Gather {
    use cached::proc_macro::cached;
    use serde::Deserialize;
    use tracing::info;

    /// Helper function to get and cache the gatherspots from the wynntils api
    #[cached(time = 3600, result = true)]
    pub async fn get_gatherspots() -> Result<GatherSpots, reqwest::Error> {
        info!("Getting new gathering data from wynntils");
        let client = crate::get_reqwest_client()?;

        let gather: GatherSpots = client
            .get("https://athena.wynntils.com/cache/get/gatheringSpots")
            .send()
            .await?
            .json()
            .await?;
        
        Ok(gather)
    }

    #[derive(Deserialize, Clone)]
    pub struct GatherSpots {
        pub woodCutting: Vec<GatherSpot>,
        pub mining: Vec<GatherSpot>,
        pub farming: Vec<GatherSpot>,
        pub fishing: Vec<GatherSpot>,
    }

    #[derive(Clone, Deserialize)]
    pub struct GatherSpot {
        pub reliability: i32,
        pub location: Location,
        pub r#type: String,
    }

    #[derive(Clone, Deserialize)]
    pub struct Location {
        pub x: f64,
        pub z: f64,
    }
}

/// Module for server related things
pub mod Servers {
    use std::collections::HashMap;

    use cached::proc_macro::cached;
    use serde::Deserialize;
    use tracing::info;

    /// Function for getting the current servers with their uptimes from the wynntils api
    #[cached(time = 300, result = true)]
    pub async fn get_servers() -> Result<Vec<ParsedServer>, reqwest::Error> {
        info!("Getting new server data from wynntils");
        let client = crate::get_reqwest_client()?;

        let servers: ServerList = client
            .get("https://athena.wynntils.com/cache/get/serverList")
            .send()
            .await?
            .json()
            .await?;
        
        let mut parsed = Vec::new();
        for (k, v) in servers.servers.into_iter() {
            parsed.push(ParsedServer {
                name: k,
                started: v.firstSeen,
                players: v.players,
            })
        }
        Ok(parsed)
    }

    #[derive(Clone, Deserialize)]
    pub struct ServerList {
        pub servers: HashMap<String, Server>,
    }

    #[derive(Clone, Deserialize)]
    #[allow(non_snake_case)]
    pub struct Server {
        pub firstSeen: i64,
        pub players: Vec<String>,
    }

    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct ParsedServer {
        pub name: String,
        pub started: i64,
        pub players: Vec<String>,
    }
}
