use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    Insight,
}

impl EntityType {
    pub fn prefix(&self) -> &'static str {
        match self {
            EntityType::Pattern => "DP-",
            EntityType::Refactoring => "RF-",
            EntityType::Law => "LAW-",
            EntityType::Smell => "SMELL-",
            EntityType::Insight => "TK-",
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
            EntityType::Insight => write!(f, "insight"),
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
            "insight" => Ok(EntityType::Insight),
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
    DerivesFrom,
    AppliesTo,
    Supersedes,
    EnforcedBy,
    ViolatedBy,
}

impl fmt::Display for RelationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelationType::Solves => write!(f, "solves"),
            RelationType::SolvedBy => write!(f, "solved_by"),
            RelationType::Enforces => write!(f, "enforces"),
            RelationType::Violates => write!(f, "violates"),
            RelationType::RelatedTo => write!(f, "related_to"),
            RelationType::DerivesFrom => write!(f, "derives_from"),
            RelationType::AppliesTo => write!(f, "applies_to"),
            RelationType::Supersedes => write!(f, "supersedes"),
            RelationType::EnforcedBy => write!(f, "enforced_by"),
            RelationType::ViolatedBy => write!(f, "violated_by"),
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
            "derives_from" => Ok(RelationType::DerivesFrom),
            "applies_to" => Ok(RelationType::AppliesTo),
            "supersedes" => Ok(RelationType::Supersedes),
            "enforced_by" => Ok(RelationType::EnforcedBy),
            "violated_by" => Ok(RelationType::ViolatedBy),
            other => Err(format!("unknown relation type: {other}")),
        }
    }
}

impl RelationType {
    /// Return the inverse relation, if one exists.
    pub fn inverse_of(&self) -> Option<RelationType> {
        match self {
            RelationType::Solves => Some(RelationType::SolvedBy),
            RelationType::SolvedBy => Some(RelationType::Solves),
            RelationType::Enforces => Some(RelationType::EnforcedBy),
            RelationType::Violates => Some(RelationType::ViolatedBy),
            RelationType::ViolatedBy => Some(RelationType::Violates),
            RelationType::RelatedTo => None,
            RelationType::DerivesFrom => None,
            RelationType::AppliesTo => None,
            RelationType::Supersedes => None,
            RelationType::EnforcedBy => Some(RelationType::Enforces),
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
    TemporaryField,
    ParallelInheritance,
    Comments,
    DeadCode,
    InappropriateIntimacy,
    RefusedBequest,
    AlternativeClasses,
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
            SmellType::TemporaryField => "SMELL-08",
            SmellType::ParallelInheritance => "SMELL-15",
            SmellType::Comments => "SMELL-16",
            SmellType::DeadCode => "SMELL-17",
            SmellType::InappropriateIntimacy => "SMELL-19",
            SmellType::RefusedBequest => "SMELL-22",
            SmellType::AlternativeClasses => "SMELL-23",
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
            SmellType::TemporaryField => write!(f, "Temporary Field"),
            SmellType::ParallelInheritance => write!(f, "Parallel Inheritance Hierarchies"),
            SmellType::Comments => write!(f, "Comments"),
            SmellType::DeadCode => write!(f, "Dead Code"),
            SmellType::InappropriateIntimacy => write!(f, "Inappropriate Intimacy"),
            SmellType::RefusedBequest => write!(f, "Refused Bequest"),
            SmellType::AlternativeClasses => {
                write!(f, "Alternative Classes with Different Interfaces")
            }
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
            "Duplicate Code" | "DuplicateCode" | "duplicate_code" => Ok(SmellType::DuplicateCode),
            "Middle Man" | "MiddleMan" | "middle_man" => Ok(SmellType::MiddleMan),
            "Feature Envy" | "FeatureEnvy" | "feature_envy" => Ok(SmellType::FeatureEnvy),
            "Message Chains" | "MessageChains" | "message_chains" => Ok(SmellType::MessageChains),
            "God Object" | "GodObject" | "god_object" => Ok(SmellType::GodObject),
            "Temporary Field" | "TemporaryField" | "temporary_field" => {
                Ok(SmellType::TemporaryField)
            }
            "Parallel Inheritance Hierarchies" | "ParallelInheritance" | "parallel_inheritance" => {
                Ok(SmellType::ParallelInheritance)
            }
            "Comments" | "comments" => Ok(SmellType::Comments),
            "Dead Code" | "DeadCode" | "dead_code" => Ok(SmellType::DeadCode),
            "Inappropriate Intimacy" | "InappropriateIntimacy" | "inappropriate_intimacy" => {
                Ok(SmellType::InappropriateIntimacy)
            }
            "Refused Bequest" | "RefusedBequest" | "refused_bequest" => {
                Ok(SmellType::RefusedBequest)
            }
            "Alternative Classes with Different Interfaces"
            | "AlternativeClasses"
            | "alternative_classes" => Ok(SmellType::AlternativeClasses),
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

// ---------------------------------------------------------------------------
// User insight types (tacit knowledge layer)
// ---------------------------------------------------------------------------

/// Provenance metadata for a single user-entity relation link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkProvenance {
    pub source: String,
    pub score: f64,
    pub recorded_at: String,
}

/// A user-contributed insight linked to canonical knowledge graph entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEntity {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author: String,
    pub confidence: f64,
    pub evidence_count: u32,
    pub last_validated: String,
    pub tags: Vec<String>,
    pub relations: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub link_provenance: HashMap<String, LinkProvenance>,
    pub created_at: String,
    pub updated_at: String,
}

/// A link suggestion returned during insight creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightLink {
    pub entity_id: String,
    pub score: f64,
    pub link_type: InsightLinkType,
}

/// Whether a link was auto-detected or manually suggested.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InsightLinkType {
    Auto,
    Suggested,
    Manual,
}

impl fmt::Display for InsightLinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InsightLinkType::Auto => write!(f, "auto"),
            InsightLinkType::Suggested => write!(f, "suggested"),
            InsightLinkType::Manual => write!(f, "manual"),
        }
    }
}

/// Correlation score between two user insights.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationScore {
    pub insight_id: String,
    pub semantic: f64,
    pub graph_proximity: f64,
    pub temporal: f64,
    pub combined: f64,
}

/// Duplicate detection result during insight creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateCandidate {
    pub insight_id: String,
    pub overlap: OverlapKind,
    pub note: String,
}

/// How two insights overlap.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OverlapKind {
    Exact,
    Partial,
    None,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- SmellType: new variants id() -------------------------------------

    #[test]
    fn smell_temporary_field_id() {
        assert_eq!(SmellType::TemporaryField.id(), "SMELL-08");
    }

    #[test]
    fn smell_parallel_inheritance_id() {
        assert_eq!(SmellType::ParallelInheritance.id(), "SMELL-15");
    }

    #[test]
    fn smell_comments_id() {
        assert_eq!(SmellType::Comments.id(), "SMELL-16");
    }

    #[test]
    fn smell_dead_code_id() {
        assert_eq!(SmellType::DeadCode.id(), "SMELL-17");
    }

    #[test]
    fn smell_inappropriate_intimacy_id() {
        assert_eq!(SmellType::InappropriateIntimacy.id(), "SMELL-19");
    }

    #[test]
    fn smell_refused_bequest_id() {
        assert_eq!(SmellType::RefusedBequest.id(), "SMELL-22");
    }

    #[test]
    fn smell_alternative_classes_id() {
        assert_eq!(SmellType::AlternativeClasses.id(), "SMELL-23");
    }

    // -- SmellType: new variants Display ----------------------------------

    #[test]
    fn smell_temporary_field_display() {
        assert_eq!(format!("{}", SmellType::TemporaryField), "Temporary Field");
    }

    #[test]
    fn smell_parallel_inheritance_display() {
        assert_eq!(
            format!("{}", SmellType::ParallelInheritance),
            "Parallel Inheritance Hierarchies"
        );
    }

    #[test]
    fn smell_comments_display() {
        assert_eq!(format!("{}", SmellType::Comments), "Comments");
    }

    #[test]
    fn smell_dead_code_display() {
        assert_eq!(format!("{}", SmellType::DeadCode), "Dead Code");
    }

    #[test]
    fn smell_inappropriate_intimacy_display() {
        assert_eq!(
            format!("{}", SmellType::InappropriateIntimacy),
            "Inappropriate Intimacy"
        );
    }

    #[test]
    fn smell_refused_bequest_display() {
        assert_eq!(format!("{}", SmellType::RefusedBequest), "Refused Bequest");
    }

    #[test]
    fn smell_alternative_classes_display() {
        assert_eq!(
            format!("{}", SmellType::AlternativeClasses),
            "Alternative Classes with Different Interfaces"
        );
    }

    // -- SmellType: new variants FromStr (all three forms) ----------------

    #[test]
    fn smell_temporary_field_from_str() {
        assert_eq!(
            "Temporary Field".parse::<SmellType>().unwrap(),
            SmellType::TemporaryField
        );
        assert_eq!(
            "TemporaryField".parse::<SmellType>().unwrap(),
            SmellType::TemporaryField
        );
        assert_eq!(
            "temporary_field".parse::<SmellType>().unwrap(),
            SmellType::TemporaryField
        );
    }

    #[test]
    fn smell_parallel_inheritance_from_str() {
        assert_eq!(
            "Parallel Inheritance Hierarchies"
                .parse::<SmellType>()
                .unwrap(),
            SmellType::ParallelInheritance
        );
        assert_eq!(
            "ParallelInheritance".parse::<SmellType>().unwrap(),
            SmellType::ParallelInheritance
        );
        assert_eq!(
            "parallel_inheritance".parse::<SmellType>().unwrap(),
            SmellType::ParallelInheritance
        );
    }

    #[test]
    fn smell_comments_from_str() {
        assert_eq!(
            "Comments".parse::<SmellType>().unwrap(),
            SmellType::Comments
        );
        assert_eq!(
            "comments".parse::<SmellType>().unwrap(),
            SmellType::Comments
        );
    }

    #[test]
    fn smell_dead_code_from_str() {
        assert_eq!(
            "Dead Code".parse::<SmellType>().unwrap(),
            SmellType::DeadCode
        );
        assert_eq!(
            "DeadCode".parse::<SmellType>().unwrap(),
            SmellType::DeadCode
        );
        assert_eq!(
            "dead_code".parse::<SmellType>().unwrap(),
            SmellType::DeadCode
        );
    }

    #[test]
    fn smell_inappropriate_intimacy_from_str() {
        assert_eq!(
            "Inappropriate Intimacy".parse::<SmellType>().unwrap(),
            SmellType::InappropriateIntimacy
        );
        assert_eq!(
            "InappropriateIntimacy".parse::<SmellType>().unwrap(),
            SmellType::InappropriateIntimacy
        );
        assert_eq!(
            "inappropriate_intimacy".parse::<SmellType>().unwrap(),
            SmellType::InappropriateIntimacy
        );
    }

    #[test]
    fn smell_refused_bequest_from_str() {
        assert_eq!(
            "Refused Bequest".parse::<SmellType>().unwrap(),
            SmellType::RefusedBequest
        );
        assert_eq!(
            "RefusedBequest".parse::<SmellType>().unwrap(),
            SmellType::RefusedBequest
        );
        assert_eq!(
            "refused_bequest".parse::<SmellType>().unwrap(),
            SmellType::RefusedBequest
        );
    }

    #[test]
    fn smell_alternative_classes_from_str() {
        assert_eq!(
            "Alternative Classes with Different Interfaces"
                .parse::<SmellType>()
                .unwrap(),
            SmellType::AlternativeClasses
        );
        assert_eq!(
            "AlternativeClasses".parse::<SmellType>().unwrap(),
            SmellType::AlternativeClasses
        );
        assert_eq!(
            "alternative_classes".parse::<SmellType>().unwrap(),
            SmellType::AlternativeClasses
        );
    }

    // -- RelationType: new variants Display/FromStr -----------------------

    #[test]
    fn relation_enforced_by_display() {
        assert_eq!(format!("{}", RelationType::EnforcedBy), "enforced_by");
    }

    #[test]
    fn relation_violated_by_display() {
        assert_eq!(format!("{}", RelationType::ViolatedBy), "violated_by");
    }

    #[test]
    fn relation_enforced_by_from_str() {
        assert_eq!(
            "enforced_by".parse::<RelationType>().unwrap(),
            RelationType::EnforcedBy
        );
    }

    #[test]
    fn relation_violated_by_from_str() {
        assert_eq!(
            "violated_by".parse::<RelationType>().unwrap(),
            RelationType::ViolatedBy
        );
    }

    // -- RelationType::inverse_of -----------------------------------------

    #[test]
    fn inverse_of_solves() {
        assert_eq!(
            RelationType::Solves.inverse_of(),
            Some(RelationType::SolvedBy)
        );
    }

    #[test]
    fn inverse_of_solved_by() {
        assert_eq!(
            RelationType::SolvedBy.inverse_of(),
            Some(RelationType::Solves)
        );
    }

    #[test]
    fn inverse_of_enforces() {
        assert_eq!(
            RelationType::Enforces.inverse_of(),
            Some(RelationType::EnforcedBy)
        );
    }

    #[test]
    fn inverse_of_violates() {
        assert_eq!(
            RelationType::Violates.inverse_of(),
            Some(RelationType::ViolatedBy)
        );
    }

    #[test]
    fn inverse_of_enforced_by() {
        assert_eq!(
            RelationType::EnforcedBy.inverse_of(),
            Some(RelationType::Enforces)
        );
    }

    #[test]
    fn inverse_of_violated_by() {
        assert_eq!(
            RelationType::ViolatedBy.inverse_of(),
            Some(RelationType::Violates)
        );
    }

    #[test]
    fn inverse_of_related_to_is_none() {
        assert_eq!(RelationType::RelatedTo.inverse_of(), None);
    }

    #[test]
    fn inverse_of_derives_from_is_none() {
        assert_eq!(RelationType::DerivesFrom.inverse_of(), None);
    }

    #[test]
    fn inverse_of_applies_to_is_none() {
        assert_eq!(RelationType::AppliesTo.inverse_of(), None);
    }

    #[test]
    fn inverse_of_supersedes_is_none() {
        assert_eq!(RelationType::Supersedes.inverse_of(), None);
    }

    // -- SmellType: total count is 23 -------------------------------------

    #[test]
    fn smell_type_count_is_23() {
        // Ensures all 23 variants are accounted for by spot-checking each new one
        let all_ids: Vec<&str> = vec![
            SmellType::LongMethod.id(),
            SmellType::LongParameterList.id(),
            SmellType::PrimitiveObsession.id(),
            SmellType::LargeClass.id(),
            SmellType::DataClumps.id(),
            SmellType::SwitchStatements.id(),
            SmellType::DataClass.id(),
            SmellType::TemporaryField.id(),
            SmellType::ShotgunSurgery.id(),
            SmellType::DivergentChange.id(),
            SmellType::LazyClass.id(),
            SmellType::SpeculativeGenerality.id(),
            SmellType::DuplicateCode.id(),
            SmellType::MiddleMan.id(),
            SmellType::ParallelInheritance.id(),
            SmellType::Comments.id(),
            SmellType::DeadCode.id(),
            SmellType::FeatureEnvy.id(),
            SmellType::InappropriateIntimacy.id(),
            SmellType::MessageChains.id(),
            SmellType::GodObject.id(),
            SmellType::RefusedBequest.id(),
            SmellType::AlternativeClasses.id(),
        ];
        assert_eq!(all_ids.len(), 23);
    }
}
