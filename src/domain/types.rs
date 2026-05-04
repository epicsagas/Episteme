use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ---------------------------------------------------------------------------
// Entity type enumeration
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum EntityType {
    Pattern,
    Refactoring,
    Law,
    Smell,
}

impl EntityType {
    pub fn prefix(&self) -> &'static str {
        match self {
            EntityType::Pattern => "DP-",
            EntityType::Refactoring => "RF-",
            EntityType::Law => "LAW-",
            EntityType::Smell => "SMELL-",
        }
    }
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityType::Pattern => write!(f, "pattern"),
            EntityType::Refactoring => write!(f, "refactoring"),
            EntityType::Law => write!(f, "law"),
            EntityType::Smell => write!(f, "smell"),
        }
    }
}

impl FromStr for EntityType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pattern" => Ok(EntityType::Pattern),
            "refactoring" => Ok(EntityType::Refactoring),
            "law" => Ok(EntityType::Law),
            "smell" => Ok(EntityType::Smell),
            other => Err(format!("unknown entity type: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Relation type enumeration
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationType {
    Solves,
    SolvedBy,
    Enforces,
    Violates,
    RelatedTo,
}

impl fmt::Display for RelationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelationType::Solves => write!(f, "solves"),
            RelationType::SolvedBy => write!(f, "solved_by"),
            RelationType::Enforces => write!(f, "enforces"),
            RelationType::Violates => write!(f, "violates"),
            RelationType::RelatedTo => write!(f, "related_to"),
        }
    }
}

impl FromStr for RelationType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "solves" => Ok(RelationType::Solves),
            "solved_by" => Ok(RelationType::SolvedBy),
            "enforces" => Ok(RelationType::Enforces),
            "violates" => Ok(RelationType::Violates),
            "related_to" => Ok(RelationType::RelatedTo),
            other => Err(format!("unknown relation type: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Smell type enumeration
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SmellType {
    LongMethod,
    LongParameterList,
    PrimitiveObsession,
    LargeClass,
    DataClumps,
    SwitchStatements,
    DataClass,
    ShotgunSurgery,
    DivergentChange,
    LazyClass,
    SpeculativeGenerality,
    DuplicateCode,
    MiddleMan,
    FeatureEnvy,
    MessageChains,
    GodObject,
}

impl SmellType {
    pub fn id(&self) -> &'static str {
        match self {
            SmellType::LongMethod => "SMELL-01",
            SmellType::LongParameterList => "SMELL-02",
            SmellType::PrimitiveObsession => "SMELL-03",
            SmellType::LargeClass => "SMELL-04",
            SmellType::DataClumps => "SMELL-05",
            SmellType::SwitchStatements => "SMELL-06",
            SmellType::DataClass => "SMELL-07",
            SmellType::ShotgunSurgery => "SMELL-09",
            SmellType::DivergentChange => "SMELL-10",
            SmellType::LazyClass => "SMELL-11",
            SmellType::SpeculativeGenerality => "SMELL-12",
            SmellType::DuplicateCode => "SMELL-13",
            SmellType::MiddleMan => "SMELL-14",
            SmellType::FeatureEnvy => "SMELL-18",
            SmellType::MessageChains => "SMELL-20",
            SmellType::GodObject => "SMELL-21",
        }
    }
}

impl fmt::Display for SmellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmellType::LongMethod => write!(f, "Long Method"),
            SmellType::LongParameterList => write!(f, "Long Parameter List"),
            SmellType::PrimitiveObsession => write!(f, "Primitive Obsession"),
            SmellType::LargeClass => write!(f, "Large Class"),
            SmellType::DataClumps => write!(f, "Data Clumps"),
            SmellType::SwitchStatements => write!(f, "Switch Statements"),
            SmellType::DataClass => write!(f, "Data Class"),
            SmellType::ShotgunSurgery => write!(f, "Shotgun Surgery"),
            SmellType::DivergentChange => write!(f, "Divergent Change"),
            SmellType::LazyClass => write!(f, "Lazy Class"),
            SmellType::SpeculativeGenerality => write!(f, "Speculative Generality"),
            SmellType::DuplicateCode => write!(f, "Duplicate Code"),
            SmellType::MiddleMan => write!(f, "Middle Man"),
            SmellType::FeatureEnvy => write!(f, "Feature Envy"),
            SmellType::MessageChains => write!(f, "Message Chains"),
            SmellType::GodObject => write!(f, "God Object"),
        }
    }
}

impl FromStr for SmellType {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "Long Method" | "LongMethod" | "long_method" => Ok(SmellType::LongMethod),
            "Long Parameter List" | "LongParameterList" | "long_parameter_list" => {
                Ok(SmellType::LongParameterList)
            }
            "Primitive Obsession" | "PrimitiveObsession" | "primitive_obsession" => {
                Ok(SmellType::PrimitiveObsession)
            }
            "Large Class" | "LargeClass" | "large_class" => Ok(SmellType::LargeClass),
            "Data Clumps" | "DataClumps" | "data_clumps" => Ok(SmellType::DataClumps),
            "Switch Statements" | "SwitchStatements" | "switch_statements" => {
                Ok(SmellType::SwitchStatements)
            }
            "Data Class" | "DataClass" | "data_class" => Ok(SmellType::DataClass),
            "Shotgun Surgery" | "ShotgunSurgery" | "shotgun_surgery" => {
                Ok(SmellType::ShotgunSurgery)
            }
            "Divergent Change" | "DivergentChange" | "divergent_change" => {
                Ok(SmellType::DivergentChange)
            }
            "Lazy Class" | "LazyClass" | "lazy_class" => Ok(SmellType::LazyClass),
            "Speculative Generality" | "SpeculativeGenerality" | "speculative_generality" => {
                Ok(SmellType::SpeculativeGenerality)
            }
            "Duplicate Code" | "DuplicateCode" | "duplicate_code" => {
                Ok(SmellType::DuplicateCode)
            }
            "Middle Man" | "MiddleMan" | "middle_man" => Ok(SmellType::MiddleMan),
            "Feature Envy" | "FeatureEnvy" | "feature_envy" => Ok(SmellType::FeatureEnvy),
            "Message Chains" | "MessageChains" | "message_chains" => {
                Ok(SmellType::MessageChains)
            }
            "God Object" | "GodObject" | "god_object" => Ok(SmellType::GodObject),
            other => Err(format!("unknown smell type: {other}")),
        }
    }
}

// ---------------------------------------------------------------------------
// Category enumeration
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Category {
    Teams,
    Planning,
    Architecture,
    Quality,
    Scalability,
    Design,
    Decisions,
}

impl Category {
    pub fn id(&self) -> u8 {
        match self {
            Category::Teams => 1,
            Category::Planning => 2,
            Category::Architecture => 3,
            Category::Quality => 4,
            Category::Scalability => 5,
            Category::Design => 6,
            Category::Decisions => 7,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Teams => write!(f, "teams"),
            Category::Planning => write!(f, "planning"),
            Category::Architecture => write!(f, "architecture"),
            Category::Quality => write!(f, "quality"),
            Category::Scalability => write!(f, "scalability"),
            Category::Design => write!(f, "design"),
            Category::Decisions => write!(f, "decisions"),
        }
    }
}

impl FromStr for Category {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "teams" => Ok(Category::Teams),
            "planning" => Ok(Category::Planning),
            "architecture" => Ok(Category::Architecture),
            "quality" => Ok(Category::Quality),
            "scalability" => Ok(Category::Scalability),
            "design" => Ok(Category::Design),
            "decisions" => Ok(Category::Decisions),
            other => Err(format!("unknown category: {other}")),
        }
    }
}

impl From<u8> for Category {
    fn from(id: u8) -> Self {
        match id {
            1 => Category::Teams,
            2 => Category::Planning,
            3 => Category::Architecture,
            4 => Category::Quality,
            5 => Category::Scalability,
            6 => Category::Design,
            7 => Category::Decisions,
            _ => panic!("unknown category id: {id}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Entity (knowledge graph node)
// ---------------------------------------------------------------------------

/// An entity in the knowledge graph (design pattern, refactoring, law, or smell).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Entity {
    pub id: String,

    #[serde(default)]
    pub r#type: String,

    #[serde(default)]
    pub title: String,

    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub category: String,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub relations: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub context: HashMap<String, Vec<String>>,

    #[serde(default)]
    pub file_path: String,

    /// Source metadata -- may be a string, object, or null depending on the
    /// data version. Stored as a generic JSON value for forward compatibility.
    #[serde(default)]
    pub source: serde_json::Value,
}

// ---------------------------------------------------------------------------
// Graph types
// ---------------------------------------------------------------------------

/// A directed, typed edge between two entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from_id: String,
    pub to_id: String,
    pub relation_type: String,
}

/// Complete one-hop neighborhood of an entity (outgoing + incoming).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neighborhood {
    pub entity: Entity,
    pub outgoing: HashMap<String, Vec<String>>,
    pub incoming: HashMap<String, Vec<String>>,
}

/// An entity that both enforces and violates the same principle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub entity_id: String,
    pub title: String,
    pub conflicts: Vec<String>,
}

/// Aggregate statistics about the loaded graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub total_entities: usize,
    pub total_edges: usize,
    pub by_type: HashMap<String, usize>,
    pub entities_with_relations: usize,
    pub avg_edges_per_entity: f64,
}
