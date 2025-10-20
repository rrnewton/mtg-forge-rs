//! Strongly-typed wrappers for game concepts
//!
//! This module provides newtypes to prevent type confusion and make the code
//! more self-documenting. Instead of using bare Strings for different concepts,
//! we wrap them in distinct types that cannot be mixed up.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Card subtype (creature type, artifact type, land type, etc.)
///
/// Examples: "Goblin", "Warrior", "Equipment", "Island"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Subtype(String);

impl Subtype {
    pub fn new(s: impl Into<String>) -> Self {
        Subtype(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Subtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Subtype {
    fn from(s: String) -> Self {
        Subtype(s)
    }
}

impl From<&str> for Subtype {
    fn from(s: &str) -> Self {
        Subtype(s.to_string())
    }
}

/// Counter type with RGB color information for display
///
/// Represents all official MTG counter types from the comprehensive Java implementation.
/// Each counter has an optional display name and RGB color for UI rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CounterType {
    // Power/Toughness Modifiers
    M1M1,      // -1/-1
    P1P1,      // +1/+1
    M0M1,      // -0/-1
    M0M2,      // -0/-2
    M1M0,      // -1/-0
    M2M1,      // -2/-1
    M2M2,      // -2/-2
    P0P1,      // +0/+1
    P0P2,      // +0/+2
    P1P0,      // +1/+0
    P1P2,      // +1/+2
    P2P0,      // +2/+0
    P2P2,      // +2/+2

    // Planeswalker
    Loyalty,

    // Alphabetical Counter Types
    Acorn,
    Aegis,
    Age,
    Aim,
    Arrow,
    Arrowhead,
    Awakening,
    Bait,
    Blaze,
    Blessing,
    Blight,
    Blood,
    Bloodline,
    Bloodstain,
    Bore,
    Bounty,
    Brain,
    Bribery,
    Brick,
    Burden,
    Cage,
    Carrion,
    Cell,
    Charge,
    Chorus,
    Coin,
    Collection,
    Component,
    Contested,
    Corpse,
    Corruption,
    Croak,
    Credit,
    Crystal,
    Cube,
    Currency,
    Death,
    Defense,
    Delay,
    Depletion,
    Descent,
    Despair,
    Devotion,
    Discovery,
    Divinity,
    Doom,
    Dread,
    Dream,
    Duty,
    Echo,
    Egg,
    Elixir,
    Ember,
    Eon,
    Eruption,
    Exposure,
    Eyeball,
    Eyestalk,
    Everything,
    Fade,
    Fate,
    Feather,
    Feeding,
    Fellowship,
    Fetch,
    Filibuster,
    Film,
    Finality,
    Fire,
    Flame,
    Flavor,
    Flood,
    Foreshadow,
    Fungus,
    Funk,
    Fury,
    Fuse,
    Gem,
    Ghostform,
    Glyph,
    Gold,
    Growth,
    Harmony,
    Hatching,
    Hatchling,
    Healing,
    Hit,
    Hone,
    Hope,
    Hoofprint,
    Hour,
    Hourglass,
    Hunger,
    Husk,
    Ice,
    Impostor,
    Incarnation,
    Incubation,
    Ingredient,
    Infection,
    Influence,
    Ingenuity,
    Intel,
    Intervention,
    Invitation,
    Isolation,
    Javelin,
    Judgment,
    Ki,
    Kick,
    Knowledge,
    Landmark,
    Level,
    Loot,
    Lore,
    Luck,
    Manabond,
    Magnet,
    Mana,
    Manifestation,
    Mannequin,
    Matrix,
    Memory,
    Midway,
    Mine,
    Mining,
    Mire,
    Music,
    Muster,
    Necrodermis,
    Net,
    Nest,
    Oil,
    Omen,
    Ore,
    Page,
    Pain,
    Paralyzation,
    Petal,
    Petrification,
    Pin,
    Plague,
    Plot,
    Pressure,
    Phylactery,
    Phyresis,
    Point,
    Polyp,
    Possession,
    Prey,
    Pupa,
    Quest,
    Rally,
    Release,
    Reprieve,
    Rejection,
    Rev,
    Revival,
    Ribbon,
    Ritual,
    Rope,
    Rust,
    Scream,
    Scroll,
    Shell,
    Shield,
    Shred,
    Silver,
    Skewer,
    Sleep,
    Slumber,
    Sleight,
    Slime,
    Soul,
    Soot,
    Spite,
    Spore,
    Stash,
    Storage,
    Story,
    Strife,
    Study,
    Stun,
    Supply,
    Takeover,
    Task,
    Theft,
    Tide,
    Time,
    Tower,
    Training,
    Trap,
    Treasure,
    Unity,
    Unlock,
    Valor,
    Velocity,
    Verse,
    Vitality,
    Vortex,
    Voyage,
    Wage,
    Winch,
    Wind,
    Wish,
    Wreck,

    // Player Counters
    Energy,
    Experience,
    Poison,
    Rad,
    Ticket,
}

impl CounterType {
    /// Get the display name for this counter (as shown on cards)
    pub fn display_name(&self) -> &'static str {
        match self {
            CounterType::M1M1 => "-1/-1",
            CounterType::P1P1 => "+1/+1",
            CounterType::M0M1 => "-0/-1",
            CounterType::M0M2 => "-0/-2",
            CounterType::M1M0 => "-1/-0",
            CounterType::M2M1 => "-2/-1",
            CounterType::M2M2 => "-2/-2",
            CounterType::P0P1 => "+0/+1",
            CounterType::P0P2 => "+0/+2",
            CounterType::P1P0 => "+1/+0",
            CounterType::P1P2 => "+1/+2",
            CounterType::P2P0 => "+2/+0",
            CounterType::P2P2 => "+2/+2",
            CounterType::Loyalty => "LOYAL",
            CounterType::Acorn => "ACORN",
            CounterType::Aegis => "AEGIS",
            CounterType::Age => "AGE",
            CounterType::Aim => "AIM",
            CounterType::Arrow => "ARROW",
            CounterType::Arrowhead => "ARWHD",
            CounterType::Awakening => "AWAKE",
            CounterType::Bait => "BAIT",
            CounterType::Blaze => "BLAZE",
            CounterType::Blessing => "BLESS",
            CounterType::Blight => "BLGHT",
            CounterType::Blood => "BLOOD",
            CounterType::Bloodline => "BLDLN",
            CounterType::Bloodstain => "BLDST",
            CounterType::Bore => "BORE",
            CounterType::Bounty => "BOUNT",
            CounterType::Brain => "BRAIN",
            CounterType::Bribery => "BRIBE",
            CounterType::Brick => "BRICK",
            CounterType::Burden => "BURDEN",
            CounterType::Cage => "CAGE",
            CounterType::Carrion => "CRRON",
            CounterType::Cell => "CELL",
            CounterType::Charge => "CHARG",
            CounterType::Chorus => "CHRUS",
            CounterType::Coin => "COIN",
            CounterType::Collection => "CLLCT",
            CounterType::Component => "COMPN",
            CounterType::Contested => "CONTES",
            CounterType::Corpse => "CRPSE",
            CounterType::Corruption => "CRPTN",
            CounterType::Croak => "CROAK",
            CounterType::Credit => "CRDIT",
            CounterType::Crystal => "CRYST",
            CounterType::Cube => "CUBE",
            CounterType::Currency => "CURR",
            CounterType::Death => "DEATH",
            CounterType::Defense => "DEF",
            CounterType::Delay => "DELAY",
            CounterType::Depletion => "DPLT",
            CounterType::Descent => "DESCT",
            CounterType::Despair => "DESPR",
            CounterType::Devotion => "DEVOT",
            CounterType::Discovery => "DISCO",
            CounterType::Divinity => "DVNTY",
            CounterType::Doom => "DOOM",
            CounterType::Dread => "DREAD",
            CounterType::Dream => "DREAM",
            CounterType::Duty => "DUTY",
            CounterType::Echo => "ECHO",
            CounterType::Egg => "EGG",
            CounterType::Elixir => "ELIXR",
            CounterType::Ember => "EMBER",
            CounterType::Eon => "EON",
            CounterType::Eruption => "ERUPTION",
            CounterType::Exposure => "EXPOSURE",
            CounterType::Eyeball => "EYE",
            CounterType::Eyestalk => "EYES",
            CounterType::Everything => "EVRY",
            CounterType::Fade => "FADE",
            CounterType::Fate => "FATE",
            CounterType::Feather => "FTHR",
            CounterType::Feeding => "FEED",
            CounterType::Fellowship => "FLWS",
            CounterType::Fetch => "FETCH",
            CounterType::Filibuster => "FLBTR",
            CounterType::Film => "FILM",
            CounterType::Finality => "FINAL",
            CounterType::Fire => "FIRE",
            CounterType::Flame => "FLAME",
            CounterType::Flavor => "FLAVOR",
            CounterType::Flood => "FLOOD",
            CounterType::Foreshadow => "FRSHD",
            CounterType::Fungus => "FNGUS",
            CounterType::Funk => "FUNK",
            CounterType::Fury => "FURY",
            CounterType::Fuse => "FUSE",
            CounterType::Gem => "GEM",
            CounterType::Ghostform => "GHSTF",
            CounterType::Glyph => "GLYPH",
            CounterType::Gold => "GOLD",
            CounterType::Growth => "GRWTH",
            CounterType::Harmony => "HRMNY",
            CounterType::Hatching => "HATCH",
            CounterType::Hatchling => "HTCHL",
            CounterType::Healing => "HEAL",
            CounterType::Hit => "HIT",
            CounterType::Hone => "HONE",
            CounterType::Hope => "HOPE",
            CounterType::Hoofprint => "HOOF",
            CounterType::Hour => "HOUR",
            CounterType::Hourglass => "HRGLS",
            CounterType::Hunger => "HUNGR",
            CounterType::Husk => "HUSK",
            CounterType::Ice => "ICE",
            CounterType::Impostor => "IMPO",
            CounterType::Incarnation => "INCRN",
            CounterType::Incubation => "INCBT",
            CounterType::Ingredient => "INGRD",
            CounterType::Infection => "INFCT",
            CounterType::Influence => "INFL",
            CounterType::Ingenuity => "INGTY",
            CounterType::Intel => "INTEL",
            CounterType::Intervention => "INTRV",
            CounterType::Invitation => "INVIT",
            CounterType::Isolation => "ISOLT",
            CounterType::Javelin => "JAVLN",
            CounterType::Judgment => "JUDGM",
            CounterType::Ki => "KI",
            CounterType::Kick => "KICK",
            CounterType::Knowledge => "KNOWL",
            CounterType::Landmark => "LNMRK",
            CounterType::Level => "LEVEL",
            CounterType::Loot => "LOOT",
            CounterType::Lore => "LORE",
            CounterType::Luck => "LUCK",
            CounterType::Manabond => "MANA",
            CounterType::Magnet => "MAGNT",
            CounterType::Mana => "MANA",
            CounterType::Manifestation => "MNFST",
            CounterType::Mannequin => "MANQN",
            CounterType::Matrix => "MATRX",
            CounterType::Memory => "MEMRY",
            CounterType::Midway => "MDWAY",
            CounterType::Mine => "MINE",
            CounterType::Mining => "MINNG",
            CounterType::Mire => "MIRE",
            CounterType::Music => "MUSIC",
            CounterType::Muster => "MUSTR",
            CounterType::Necrodermis => "NECRO",
            CounterType::Net => "NET",
            CounterType::Nest => "NEST",
            CounterType::Oil => "OIL",
            CounterType::Omen => "OMEN",
            CounterType::Ore => "ORE",
            CounterType::Page => "PAGE",
            CounterType::Pain => "PAIN",
            CounterType::Paralyzation => "PRLYZ",
            CounterType::Petal => "PETAL",
            CounterType::Petrification => "PETRI",
            CounterType::Pin => "PIN",
            CounterType::Plague => "PLGUE",
            CounterType::Plot => "PLOT",
            CounterType::Pressure => "PRESS",
            CounterType::Phylactery => "PHYLA",
            CounterType::Phyresis => "PHYRE",
            CounterType::Point => "POINT",
            CounterType::Polyp => "POLYP",
            CounterType::Possession => "POSSN",
            CounterType::Prey => "PREY",
            CounterType::Pupa => "PUPA",
            CounterType::Quest => "QUEST",
            CounterType::Rally => "RALLY",
            CounterType::Release => "RELEASE",
            CounterType::Reprieve => "REPR",
            CounterType::Rejection => "REJECT",
            CounterType::Rev => "REV",
            CounterType::Revival => "REVIVL",
            CounterType::Ribbon => "RIBBON",
            CounterType::Ritual => "RITUAL",
            CounterType::Rope => "ROPE",
            CounterType::Rust => "RUST",
            CounterType::Scream => "SCREM",
            CounterType::Scroll => "SCRLL",
            CounterType::Shell => "SHELL",
            CounterType::Shield => "SHLD",
            CounterType::Shred => "SHRED",
            CounterType::Silver => "SILVER",
            CounterType::Skewer => "SKEWER",
            CounterType::Sleep => "SLEEP",
            CounterType::Slumber => "SLMBR",
            CounterType::Sleight => "SLGHT",
            CounterType::Slime => "SLIME",
            CounterType::Soul => "SOUL",
            CounterType::Soot => "SOOT",
            CounterType::Spite => "SPITE",
            CounterType::Spore => "SPORE",
            CounterType::Stash => "STASH",
            CounterType::Storage => "STORG",
            CounterType::Story => "STORY",
            CounterType::Strife => "STRFE",
            CounterType::Study => "STUDY",
            CounterType::Stun => "STUN",
            CounterType::Supply => "SPPLY",
            CounterType::Takeover => "TKVR",
            CounterType::Task => "TASK",
            CounterType::Theft => "THEFT",
            CounterType::Tide => "TIDE",
            CounterType::Time => "TIME",
            CounterType::Tower => "TOWER",
            CounterType::Training => "TRAIN",
            CounterType::Trap => "TRAP",
            CounterType::Treasure => "TRSUR",
            CounterType::Unity => "UNITY",
            CounterType::Unlock => "UNLCK",
            CounterType::Valor => "VALOR",
            CounterType::Velocity => "VELO",
            CounterType::Verse => "VERSE",
            CounterType::Vitality => "VITAL",
            CounterType::Vortex => "VORTX",
            CounterType::Voyage => "VOYAGE",
            CounterType::Wage => "WAGE",
            CounterType::Winch => "WINCH",
            CounterType::Wind => "WIND",
            CounterType::Wish => "WISH",
            CounterType::Wreck => "WRECK",
            CounterType::Energy => "ENRGY",
            CounterType::Experience => "EXP",
            CounterType::Poison => "POISN",
            CounterType::Rad => "RAD",
            CounterType::Ticket => "TICKET",
        }
    }

    /// Get the RGB color for this counter (for UI display)
    /// Returns (red, green, blue) where each component is 0-255
    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            CounterType::M1M1 | CounterType::M0M1 | CounterType::M0M2
            | CounterType::M1M0 | CounterType::M2M1 | CounterType::M2M2 => (255, 110, 106),
            CounterType::P1P1 | CounterType::P0P1 | CounterType::P0P2
            | CounterType::P1P0 | CounterType::P1P2 | CounterType::P2P0 | CounterType::P2P2 => (96, 226, 23),
            CounterType::Loyalty => (198, 198, 198),
            CounterType::Acorn => (139, 69, 19),
            CounterType::Aegis => (207, 207, 207),
            CounterType::Age => (255, 137, 57),
            CounterType::Aim => (255, 180, 0),
            CounterType::Arrow => (237, 195, 0),
            CounterType::Arrowhead => (230, 191, 167),
            CounterType::Awakening => (0, 231, 79),
            CounterType::Bait => (120, 100, 60),
            CounterType::Blaze => (255, 124, 82),
            CounterType::Blessing => (251, 0, 94),
            CounterType::Blight => (130, 115, 160),
            CounterType::Blood => (255, 108, 111),
            CounterType::Bloodline => (224, 44, 44),
            CounterType::Bloodstain => (224, 44, 44),
            CounterType::Bore => (98, 47, 34),
            CounterType::Bounty => (255, 158, 0),
            CounterType::Brain => (197, 62, 212),
            CounterType::Bribery => (172, 201, 235),
            CounterType::Brick => (226, 192, 164),
            CounterType::Burden => (135, 62, 35),
            CounterType::Cage => (155, 155, 155),
            CounterType::Carrion => (255, 163, 222),
            CounterType::Cell => (90, 10, 95),
            CounterType::Charge => (246, 192, 0),
            CounterType::Chorus => (0, 192, 246),
            CounterType::Coin => (255, 215, 0),
            CounterType::Collection => (255, 215, 0),
            CounterType::Component => (224, 160, 48),
            CounterType::Contested => (255, 76, 2),
            CounterType::Corpse => (230, 186, 209),
            CounterType::Corruption => (210, 121, 210),
            CounterType::Croak => (155, 255, 5),
            CounterType::Credit => (188, 197, 234),
            CounterType::Crystal => (255, 85, 206),
            CounterType::Cube => (148, 219, 0),
            CounterType::Currency => (223, 200, 0),
            CounterType::Death => (255, 108, 110),
            CounterType::Defense => (164, 23, 32),
            CounterType::Delay => (102, 206, 255),
            CounterType::Depletion => (185, 201, 208),
            CounterType::Descent => (175, 35, 40),
            CounterType::Despair => (238, 186, 187),
            CounterType::Devotion => (255, 111, 255),
            CounterType::Discovery => (12, 230, 100),
            CounterType::Divinity => (0, 233, 255),
            CounterType::Doom => (255, 104, 118),
            CounterType::Dread => (205, 170, 240),
            CounterType::Dream => (190, 189, 255),
            CounterType::Duty => (232, 245, 245),
            CounterType::Echo => (225, 180, 255),
            CounterType::Egg => (255, 245, 195),
            CounterType::Elixir => (81, 221, 175),
            CounterType::Ember => (247, 52, 43),
            CounterType::Eon => (23, 194, 255),
            CounterType::Eruption => (255, 124, 124),
            CounterType::Exposure => (50, 180, 30),
            CounterType::Eyeball => (184, 202, 201),
            CounterType::Eyestalk => (184, 202, 201),
            CounterType::Everything => (255, 255, 255),
            CounterType::Fade => (159, 209, 192),
            CounterType::Fate => (255, 164, 226),
            CounterType::Feather => (195, 202, 165),
            CounterType::Feeding => (245, 21, 5),
            CounterType::Fellowship => (255, 255, 255),
            CounterType::Fetch => (180, 235, 52),
            CounterType::Filibuster => (255, 179, 119),
            CounterType::Film => (255, 255, 255),
            CounterType::Finality => (255, 255, 255),
            CounterType::Fire => (240, 30, 35),
            CounterType::Flame => (255, 143, 43),
            CounterType::Flavor => (208, 152, 97),
            CounterType::Flood => (0, 203, 255),
            CounterType::Foreshadow => (144, 99, 207),
            CounterType::Fungus => (121, 219, 151),
            CounterType::Funk => (215, 24, 222),
            CounterType::Fury => (255, 120, 89),
            CounterType::Fuse => (255, 122, 85),
            CounterType::Gem => (255, 99, 251),
            CounterType::Ghostform => (223, 0, 254),
            CounterType::Glyph => (184, 202, 199),
            CounterType::Gold => (248, 191, 0),
            CounterType::Growth => (87, 226, 32),
            CounterType::Harmony => (0, 230, 155),
            CounterType::Hatching => (204, 255, 204),
            CounterType::Hatchling => (201, 199, 186),
            CounterType::Healing => (255, 166, 236),
            CounterType::Hit => (255, 245, 195),
            CounterType::Hone => (51, 227, 255),
            CounterType::Hope => (232, 245, 245),
            CounterType::Hoofprint => (233, 189, 170),
            CounterType::Hour => (198, 197, 210),
            CounterType::Hourglass => (0, 215, 255),
            CounterType::Hunger => (255, 91, 149),
            CounterType::Husk => (227, 212, 173),
            CounterType::Ice => (0, 239, 255),
            CounterType::Impostor => (173, 194, 255),
            CounterType::Incarnation => (247, 206, 64),
            CounterType::Incubation => (40, 210, 25),
            CounterType::Ingredient => (180, 50, 145),
            CounterType::Infection => (0, 230, 66),
            CounterType::Influence => (201, 99, 212),
            CounterType::Ingenuity => (67, 186, 205),
            CounterType::Intel => (80, 250, 180),
            CounterType::Intervention => (205, 203, 105),
            CounterType::Invitation => (205, 0, 26),
            CounterType::Isolation => (250, 190, 0),
            CounterType::Javelin => (180, 206, 172),
            CounterType::Judgment => (249, 220, 52),
            CounterType::Ki => (190, 189, 255),
            CounterType::Kick => (255, 255, 240),
            CounterType::Knowledge => (0, 115, 255),
            CounterType::Landmark => (186, 28, 28),
            CounterType::Level => (60, 222, 185),
            CounterType::Loot => (255, 215, 0),
            CounterType::Lore => (209, 198, 161),
            CounterType::Luck => (185, 174, 255),
            CounterType::Manabond => (0, 255, 0),
            CounterType::Magnet => (198, 197, 210),
            CounterType::Mana => (0, 237, 152),
            CounterType::Manifestation => (104, 225, 8),
            CounterType::Mannequin => (206, 199, 162),
            CounterType::Matrix => (183, 174, 255),
            CounterType::Memory => (174, 183, 255),
            CounterType::Midway => (84, 101, 207),
            CounterType::Mine => (255, 100, 127),
            CounterType::Mining => (184, 201, 207),
            CounterType::Mire => (153, 209, 199),
            CounterType::Music => (255, 138, 255),
            CounterType::Muster => (235, 196, 0),
            CounterType::Necrodermis => (80, 209, 250),
            CounterType::Net => (0, 221, 251),
            CounterType::Nest => (80, 80, 50),
            CounterType::Oil => (99, 102, 106),
            CounterType::Omen => (255, 178, 120),
            CounterType::Ore => (200, 201, 163),
            CounterType::Page => (218, 195, 162),
            CounterType::Pain => (255, 108, 111),
            CounterType::Paralyzation => (220, 201, 0),
            CounterType::Petal => (255, 162, 216),
            CounterType::Petrification => (185, 201, 208),
            CounterType::Pin => (194, 196, 233),
            CounterType::Plague => (94, 226, 25),
            CounterType::Plot => (255, 172, 133),
            CounterType::Pressure => (255, 164, 159),
            CounterType::Phylactery => (117, 219, 153),
            CounterType::Phyresis => (125, 97, 128),
            CounterType::Point => (153, 255, 130),
            CounterType::Polyp => (236, 185, 198),
            CounterType::Possession => (60, 65, 85),
            CounterType::Prey => (240, 0, 0),
            CounterType::Pupa => (0, 223, 203),
            CounterType::Quest => (251, 189, 0),
            CounterType::Rally => (25, 230, 225),
            CounterType::Release => (200, 210, 50),
            CounterType::Reprieve => (240, 120, 50),
            CounterType::Rejection => (212, 235, 242),
            CounterType::Rev => (255, 108, 111),
            CounterType::Revival => (130, 230, 50),
            CounterType::Ribbon => (233, 245, 232),
            CounterType::Ritual => (155, 17, 30),
            CounterType::Rope => (239, 223, 187),
            CounterType::Rust => (255, 181, 116),
            CounterType::Scream => (0, 220, 255),
            CounterType::Scroll => (206, 199, 162),
            CounterType::Shell => (190, 207, 111),
            CounterType::Shield => (202, 198, 186),
            CounterType::Shred => (255, 165, 152),
            CounterType::Silver => (192, 192, 192),
            CounterType::Skewer => (202, 192, 156),
            CounterType::Sleep => (178, 192, 255),
            CounterType::Slumber => (178, 205, 255),
            CounterType::Sleight => (185, 174, 255),
            CounterType::Slime => (101, 220, 163),
            CounterType::Soul => (243, 190, 247),
            CounterType::Soot => (211, 194, 198),
            CounterType::Spite => (0, 218, 255),
            CounterType::Spore => (122, 218, 150),
            CounterType::Stash => (248, 191, 0),
            CounterType::Storage => (255, 177, 121),
            CounterType::Story => (180, 72, 195),
            CounterType::Strife => (255, 89, 223),
            CounterType::Study => (226, 192, 165),
            CounterType::Stun => (226, 192, 165),
            CounterType::Supply => (70, 105, 60),
            CounterType::Takeover => (63, 49, 191),
            CounterType::Task => (191, 63, 49),
            CounterType::Theft => (255, 176, 125),
            CounterType::Tide => (0, 212, 187),
            CounterType::Time => (255, 121, 255),
            CounterType::Tower => (0, 239, 255),
            CounterType::Training => (220, 201, 0),
            CounterType::Trap => (255, 121, 86),
            CounterType::Treasure => (255, 184, 0),
            CounterType::Unity => (242, 156, 255),
            CounterType::Unlock => (222, 146, 205),
            CounterType::Valor => (252, 250, 222),
            CounterType::Velocity => (255, 95, 138),
            CounterType::Verse => (0, 237, 155),
            CounterType::Vitality => (255, 94, 142),
            CounterType::Vortex => (142, 200, 255),
            CounterType::Voyage => (38, 150, 137),
            CounterType::Wage => (242, 190, 106),
            CounterType::Winch => (208, 195, 203),
            CounterType::Wind => (0, 236, 255),
            CounterType::Wish => (255, 85, 206),
            CounterType::Wreck => (208, 55, 255),
            // Player counters default to white
            CounterType::Energy | CounterType::Experience | CounterType::Poison
            | CounterType::Rad | CounterType::Ticket => (255, 255, 255),
        }
    }

    /// Parse a counter type from a string
    ///
    /// Handles the special cases for power/toughness counters like "+1/+1" -> P1P1
    pub fn from_str(s: &str) -> Option<Self> {
        // Replace special characters for power/toughness counters
        let normalized = s
            .replace('/', "")
            .replace('+', "P")
            .replace('-', "M")
            .to_uppercase();

        match normalized.as_str() {
            "M1M1" => Some(CounterType::M1M1),
            "P1P1" => Some(CounterType::P1P1),
            "M0M1" => Some(CounterType::M0M1),
            "M0M2" => Some(CounterType::M0M2),
            "M1M0" => Some(CounterType::M1M0),
            "M2M1" => Some(CounterType::M2M1),
            "M2M2" => Some(CounterType::M2M2),
            "P0P1" => Some(CounterType::P0P1),
            "P0P2" => Some(CounterType::P0P2),
            "P1P0" => Some(CounterType::P1P0),
            "P1P2" => Some(CounterType::P1P2),
            "P2P0" => Some(CounterType::P2P0),
            "P2P2" => Some(CounterType::P2P2),
            "LOYAL" | "LOYALTY" => Some(CounterType::Loyalty),
            "ACORN" => Some(CounterType::Acorn),
            "AEGIS" => Some(CounterType::Aegis),
            "AGE" => Some(CounterType::Age),
            "AIM" => Some(CounterType::Aim),
            "ARROW" => Some(CounterType::Arrow),
            "ARWHD" | "ARROWHEAD" => Some(CounterType::Arrowhead),
            "AWAKE" | "AWAKENING" => Some(CounterType::Awakening),
            "BAIT" => Some(CounterType::Bait),
            "BLAZE" => Some(CounterType::Blaze),
            "BLESS" | "BLESSING" => Some(CounterType::Blessing),
            "BLGHT" | "BLIGHT" => Some(CounterType::Blight),
            "BLOOD" => Some(CounterType::Blood),
            "BLDLN" | "BLOODLINE" => Some(CounterType::Bloodline),
            "BLDST" | "BLOODSTAIN" => Some(CounterType::Bloodstain),
            "BORE" => Some(CounterType::Bore),
            "BOUNT" | "BOUNTY" => Some(CounterType::Bounty),
            "BRAIN" => Some(CounterType::Brain),
            "BRIBE" | "BRIBERY" => Some(CounterType::Bribery),
            "BRICK" => Some(CounterType::Brick),
            "BURDEN" => Some(CounterType::Burden),
            "CAGE" => Some(CounterType::Cage),
            "CRRON" | "CARRION" => Some(CounterType::Carrion),
            "CELL" => Some(CounterType::Cell),
            "CHARG" | "CHARGE" => Some(CounterType::Charge),
            "CHRUS" | "CHORUS" => Some(CounterType::Chorus),
            "COIN" => Some(CounterType::Coin),
            "CLLCT" | "COLLECTION" => Some(CounterType::Collection),
            "COMPN" | "COMPONENT" => Some(CounterType::Component),
            "CONTES" | "CONTESTED" => Some(CounterType::Contested),
            "CRPSE" | "CORPSE" => Some(CounterType::Corpse),
            "CRPTN" | "CORRUPTION" => Some(CounterType::Corruption),
            "CROAK" => Some(CounterType::Croak),
            "CRDIT" | "CREDIT" => Some(CounterType::Credit),
            "CRYST" | "CRYSTAL" => Some(CounterType::Crystal),
            "CUBE" => Some(CounterType::Cube),
            "CURR" | "CURRENCY" => Some(CounterType::Currency),
            "DEATH" => Some(CounterType::Death),
            "DEF" | "DEFENSE" => Some(CounterType::Defense),
            "DELAY" => Some(CounterType::Delay),
            "DPLT" | "DEPLETION" => Some(CounterType::Depletion),
            "DESCT" | "DESCENT" => Some(CounterType::Descent),
            "DESPR" | "DESPAIR" => Some(CounterType::Despair),
            "DEVOT" | "DEVOTION" => Some(CounterType::Devotion),
            "DISCO" | "DISCOVERY" => Some(CounterType::Discovery),
            "DVNTY" | "DIVINITY" => Some(CounterType::Divinity),
            "DOOM" => Some(CounterType::Doom),
            "DREAD" => Some(CounterType::Dread),
            "DREAM" => Some(CounterType::Dream),
            "DUTY" => Some(CounterType::Duty),
            "ECHO" => Some(CounterType::Echo),
            "EGG" => Some(CounterType::Egg),
            "ELIXR" | "ELIXIR" => Some(CounterType::Elixir),
            "EMBER" => Some(CounterType::Ember),
            "EON" => Some(CounterType::Eon),
            "ERUPTION" => Some(CounterType::Eruption),
            "EXPOSURE" => Some(CounterType::Exposure),
            "EYE" | "EYEBALL" => Some(CounterType::Eyeball),
            "EYES" | "EYESTALK" => Some(CounterType::Eyestalk),
            "EVRY" | "EVERYTHING" => Some(CounterType::Everything),
            "FADE" => Some(CounterType::Fade),
            "FATE" => Some(CounterType::Fate),
            "FTHR" | "FEATHER" => Some(CounterType::Feather),
            "FEED" | "FEEDING" => Some(CounterType::Feeding),
            "FLWS" | "FELLOWSHIP" => Some(CounterType::Fellowship),
            "FETCH" => Some(CounterType::Fetch),
            "FLBTR" | "FILIBUSTER" => Some(CounterType::Filibuster),
            "FILM" => Some(CounterType::Film),
            "FINAL" | "FINALITY" => Some(CounterType::Finality),
            "FIRE" => Some(CounterType::Fire),
            "FLAME" => Some(CounterType::Flame),
            "FLAVOR" => Some(CounterType::Flavor),
            "FLOOD" => Some(CounterType::Flood),
            "FRSHD" | "FORESHADOW" => Some(CounterType::Foreshadow),
            "FNGUS" | "FUNGUS" => Some(CounterType::Fungus),
            "FUNK" => Some(CounterType::Funk),
            "FURY" => Some(CounterType::Fury),
            "FUSE" => Some(CounterType::Fuse),
            "GEM" => Some(CounterType::Gem),
            "GHSTF" | "GHOSTFORM" => Some(CounterType::Ghostform),
            "GLYPH" => Some(CounterType::Glyph),
            "GOLD" => Some(CounterType::Gold),
            "GRWTH" | "GROWTH" => Some(CounterType::Growth),
            "HRMNY" | "HARMONY" => Some(CounterType::Harmony),
            "HATCH" | "HATCHING" => Some(CounterType::Hatching),
            "HTCHL" | "HATCHLING" => Some(CounterType::Hatchling),
            "HEAL" | "HEALING" => Some(CounterType::Healing),
            "HIT" => Some(CounterType::Hit),
            "HONE" => Some(CounterType::Hone),
            "HOPE" => Some(CounterType::Hope),
            "HOOF" | "HOOFPRINT" => Some(CounterType::Hoofprint),
            "HOUR" => Some(CounterType::Hour),
            "HRGLS" | "HOURGLASS" => Some(CounterType::Hourglass),
            "HUNGR" | "HUNGER" => Some(CounterType::Hunger),
            "HUSK" => Some(CounterType::Husk),
            "ICE" => Some(CounterType::Ice),
            "IMPO" | "IMPOSTOR" => Some(CounterType::Impostor),
            "INCRN" | "INCARNATION" => Some(CounterType::Incarnation),
            "INCBT" | "INCUBATION" => Some(CounterType::Incubation),
            "INGRD" | "INGREDIENT" => Some(CounterType::Ingredient),
            "INFCT" | "INFECTION" => Some(CounterType::Infection),
            "INFL" | "INFLUENCE" => Some(CounterType::Influence),
            "INGTY" | "INGENUITY" => Some(CounterType::Ingenuity),
            "INTEL" => Some(CounterType::Intel),
            "INTRV" | "INTERVENTION" => Some(CounterType::Intervention),
            "INVIT" | "INVITATION" => Some(CounterType::Invitation),
            "ISOLT" | "ISOLATION" => Some(CounterType::Isolation),
            "JAVLN" | "JAVELIN" => Some(CounterType::Javelin),
            "JUDGM" | "JUDGMENT" => Some(CounterType::Judgment),
            "KI" => Some(CounterType::Ki),
            "KICK" => Some(CounterType::Kick),
            "KNOWL" | "KNOWLEDGE" => Some(CounterType::Knowledge),
            "LNMRK" | "LANDMARK" => Some(CounterType::Landmark),
            "LEVEL" => Some(CounterType::Level),
            "LOOT" => Some(CounterType::Loot),
            "LORE" => Some(CounterType::Lore),
            "LUCK" => Some(CounterType::Luck),
            "MAGNT" | "MAGNET" => Some(CounterType::Magnet),
            "MANA" | "MANABOND" => Some(CounterType::Mana),
            "MNFST" | "MANIFESTATION" => Some(CounterType::Manifestation),
            "MANQN" | "MANNEQUIN" => Some(CounterType::Mannequin),
            "MATRX" | "MATRIX" => Some(CounterType::Matrix),
            "MEMRY" | "MEMORY" => Some(CounterType::Memory),
            "MDWAY" | "MIDWAY" => Some(CounterType::Midway),
            "MINE" => Some(CounterType::Mine),
            "MINNG" | "MINING" => Some(CounterType::Mining),
            "MIRE" => Some(CounterType::Mire),
            "MUSIC" => Some(CounterType::Music),
            "MUSTR" | "MUSTER" => Some(CounterType::Muster),
            "NECRO" | "NECRODERMIS" => Some(CounterType::Necrodermis),
            "NET" => Some(CounterType::Net),
            "NEST" => Some(CounterType::Nest),
            "OIL" => Some(CounterType::Oil),
            "OMEN" => Some(CounterType::Omen),
            "ORE" => Some(CounterType::Ore),
            "PAGE" => Some(CounterType::Page),
            "PAIN" => Some(CounterType::Pain),
            "PRLYZ" | "PARALYZATION" => Some(CounterType::Paralyzation),
            "PETAL" => Some(CounterType::Petal),
            "PETRI" | "PETRIFICATION" => Some(CounterType::Petrification),
            "PIN" => Some(CounterType::Pin),
            "PLGUE" | "PLAGUE" => Some(CounterType::Plague),
            "PLOT" => Some(CounterType::Plot),
            "PRESS" | "PRESSURE" => Some(CounterType::Pressure),
            "PHYLA" | "PHYLACTERY" => Some(CounterType::Phylactery),
            "PHYRE" | "PHYRESIS" => Some(CounterType::Phyresis),
            "POINT" => Some(CounterType::Point),
            "POLYP" => Some(CounterType::Polyp),
            "POSSN" | "POSSESSION" => Some(CounterType::Possession),
            "PREY" => Some(CounterType::Prey),
            "PUPA" => Some(CounterType::Pupa),
            "QUEST" => Some(CounterType::Quest),
            "RALLY" => Some(CounterType::Rally),
            "RELEASE" => Some(CounterType::Release),
            "REPR" | "REPRIEVE" => Some(CounterType::Reprieve),
            "REJECT" | "REJECTION" => Some(CounterType::Rejection),
            "REV" => Some(CounterType::Rev),
            "REVIVL" | "REVIVAL" => Some(CounterType::Revival),
            "RIBBON" => Some(CounterType::Ribbon),
            "RITUAL" => Some(CounterType::Ritual),
            "ROPE" => Some(CounterType::Rope),
            "RUST" => Some(CounterType::Rust),
            "SCREM" | "SCREAM" => Some(CounterType::Scream),
            "SCRLL" | "SCROLL" => Some(CounterType::Scroll),
            "SHELL" => Some(CounterType::Shell),
            "SHLD" | "SHIELD" => Some(CounterType::Shield),
            "SHRED" => Some(CounterType::Shred),
            "SILVER" => Some(CounterType::Silver),
            "SKEWER" => Some(CounterType::Skewer),
            "SLEEP" => Some(CounterType::Sleep),
            "SLMBR" | "SLUMBER" => Some(CounterType::Slumber),
            "SLGHT" | "SLEIGHT" => Some(CounterType::Sleight),
            "SLIME" => Some(CounterType::Slime),
            "SOUL" => Some(CounterType::Soul),
            "SOOT" => Some(CounterType::Soot),
            "SPITE" => Some(CounterType::Spite),
            "SPORE" => Some(CounterType::Spore),
            "STASH" => Some(CounterType::Stash),
            "STORG" | "STORAGE" => Some(CounterType::Storage),
            "STORY" => Some(CounterType::Story),
            "STRFE" | "STRIFE" => Some(CounterType::Strife),
            "STUDY" => Some(CounterType::Study),
            "STUN" => Some(CounterType::Stun),
            "SPPLY" | "SUPPLY" => Some(CounterType::Supply),
            "TKVR" | "TAKEOVER" => Some(CounterType::Takeover),
            "TASK" => Some(CounterType::Task),
            "THEFT" => Some(CounterType::Theft),
            "TIDE" => Some(CounterType::Tide),
            "TIME" => Some(CounterType::Time),
            "TOWER" => Some(CounterType::Tower),
            "TRAIN" | "TRAINING" => Some(CounterType::Training),
            "TRAP" => Some(CounterType::Trap),
            "TRSUR" | "TREASURE" => Some(CounterType::Treasure),
            "UNITY" => Some(CounterType::Unity),
            "UNLCK" | "UNLOCK" => Some(CounterType::Unlock),
            "VALOR" => Some(CounterType::Valor),
            "VELO" | "VELOCITY" => Some(CounterType::Velocity),
            "VERSE" => Some(CounterType::Verse),
            "VITAL" | "VITALITY" => Some(CounterType::Vitality),
            "VORTX" | "VORTEX" => Some(CounterType::Vortex),
            "VOYAGE" => Some(CounterType::Voyage),
            "WAGE" => Some(CounterType::Wage),
            "WINCH" => Some(CounterType::Winch),
            "WIND" => Some(CounterType::Wind),
            "WISH" => Some(CounterType::Wish),
            "WRECK" => Some(CounterType::Wreck),
            "ENRGY" | "ENERGY" => Some(CounterType::Energy),
            "EXP" | "EXPERIENCE" => Some(CounterType::Experience),
            "POISN" | "POISON" => Some(CounterType::Poison),
            "RAD" => Some(CounterType::Rad),
            "TICKET" => Some(CounterType::Ticket),
            _ => None,
        }
    }

    /// Check if this is a player counter (not a permanent counter)
    pub fn is_player_counter(&self) -> bool {
        matches!(
            self,
            CounterType::Energy
                | CounterType::Experience
                | CounterType::Poison
                | CounterType::Rad
                | CounterType::Ticket
        )
    }

    /// Check if this modifies power/toughness
    pub fn is_power_toughness_modifier(&self) -> bool {
        matches!(
            self,
            CounterType::M1M1
                | CounterType::P1P1
                | CounterType::M0M1
                | CounterType::M0M2
                | CounterType::M1M0
                | CounterType::M2M1
                | CounterType::M2M2
                | CounterType::P0P1
                | CounterType::P0P2
                | CounterType::P1P0
                | CounterType::P1P2
                | CounterType::P2P0
                | CounterType::P2P2
        )
    }

    /// Get the power/toughness modification amount (if applicable)
    /// Returns (power_mod, toughness_mod)
    pub fn power_toughness_mod(&self) -> Option<(i32, i32)> {
        match self {
            CounterType::M1M1 => Some((-1, -1)),
            CounterType::P1P1 => Some((1, 1)),
            CounterType::M0M1 => Some((0, -1)),
            CounterType::M0M2 => Some((0, -2)),
            CounterType::M1M0 => Some((-1, 0)),
            CounterType::M2M1 => Some((-2, -1)),
            CounterType::M2M2 => Some((-2, -2)),
            CounterType::P0P1 => Some((0, 1)),
            CounterType::P0P2 => Some((0, 2)),
            CounterType::P1P0 => Some((1, 0)),
            CounterType::P1P2 => Some((1, 2)),
            CounterType::P2P0 => Some((2, 0)),
            CounterType::P2P2 => Some((2, 2)),
            _ => None,
        }
    }
}

impl fmt::Display for CounterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Card name (distinct from other string types)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardName(String);

impl CardName {
    pub fn new(s: impl Into<String>) -> Self {
        CardName(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_lowercase(&self) -> String {
        self.0.to_lowercase()
    }
}

impl fmt::Display for CardName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for CardName {
    fn from(s: String) -> Self {
        CardName(s)
    }
}

impl From<&str> for CardName {
    fn from(s: &str) -> Self {
        CardName(s.to_string())
    }
}

/// Player name (distinct from other string types)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerName(String);

impl PlayerName {
    pub fn new(s: impl Into<String>) -> Self {
        PlayerName(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PlayerName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for PlayerName {
    fn from(s: String) -> Self {
        PlayerName(s)
    }
}

impl From<&str> for PlayerName {
    fn from(s: &str) -> Self {
        PlayerName(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subtype() {
        let subtype = Subtype::new("Goblin");
        assert_eq!(subtype.as_str(), "Goblin");
        assert_eq!(subtype.to_string(), "Goblin");
    }

    #[test]
    fn test_counter_type() {
        let counter = CounterType::P1P1;
        assert_eq!(counter.display_name(), "+1/+1");
        assert_eq!(counter.to_string(), "+1/+1");
        assert_eq!(counter.color(), (96, 226, 23));
        assert!(counter.is_power_toughness_modifier());
        assert_eq!(counter.power_toughness_mod(), Some((1, 1)));
    }

    #[test]
    fn test_counter_type_parsing() {
        assert_eq!(CounterType::from_str("+1/+1"), Some(CounterType::P1P1));
        assert_eq!(CounterType::from_str("-1/-1"), Some(CounterType::M1M1));
        assert_eq!(CounterType::from_str("charge"), Some(CounterType::Charge));
        assert_eq!(CounterType::from_str("CHARG"), Some(CounterType::Charge));
        assert_eq!(CounterType::from_str("loyalty"), Some(CounterType::Loyalty));
        assert_eq!(CounterType::from_str("LOYAL"), Some(CounterType::Loyalty));
        assert_eq!(CounterType::from_str("poison"), Some(CounterType::Poison));
        assert_eq!(CounterType::from_str("unknown"), None);
    }

    #[test]
    fn test_player_counters() {
        assert!(CounterType::Energy.is_player_counter());
        assert!(CounterType::Poison.is_player_counter());
        assert!(!CounterType::P1P1.is_player_counter());
        assert!(!CounterType::Charge.is_player_counter());
    }

    #[test]
    fn test_power_toughness_mods() {
        assert_eq!(CounterType::M1M1.power_toughness_mod(), Some((-1, -1)));
        assert_eq!(CounterType::P2P2.power_toughness_mod(), Some((2, 2)));
        assert_eq!(CounterType::P1P0.power_toughness_mod(), Some((1, 0)));
        assert_eq!(CounterType::M0M2.power_toughness_mod(), Some((0, -2)));
        assert_eq!(CounterType::Charge.power_toughness_mod(), None);
    }

    #[test]
    fn test_card_name() {
        let name = CardName::new("Lightning Bolt");
        assert_eq!(name.as_str(), "Lightning Bolt");
    }

    #[test]
    fn test_player_name() {
        let name = PlayerName::new("Alice");
        assert_eq!(name.as_str(), "Alice");
    }
}
