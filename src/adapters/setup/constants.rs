use crate::domain::types::{Category, EntityType};

use std::collections::HashMap;
use std::sync::LazyLock;

pub static ENTITY_TYPES: LazyLock<Vec<EntityType>> = LazyLock::new(|| {
    vec![
        EntityType::Pattern,
        EntityType::Refactoring,
        EntityType::Law,
        EntityType::Smell,
    ]
});

pub static ENTITY_PREFIXES: LazyLock<HashMap<EntityType, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(EntityType::Pattern, "DP-");
    m.insert(EntityType::Refactoring, "RF-");
    m.insert(EntityType::Law, "LAW-");
    m.insert(EntityType::Smell, "SMELL-");
    m
});

pub static CATEGORIES: LazyLock<HashMap<u8, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert(Category::Teams.id(), "teams");
    m.insert(Category::Planning.id(), "planning");
    m.insert(Category::Architecture.id(), "architecture");
    m.insert(Category::Quality.id(), "quality");
    m.insert(Category::Scalability.id(), "scalability");
    m.insert(Category::Design.id(), "design");
    m.insert(Category::Decisions.id(), "decisions");
    m
});

pub const SIMILARITY_THRESHOLD: f64 = 0.5;
pub const DEFAULT_SEARCH_LIMIT: usize = 5;
pub const MAX_SEARCH_LIMIT: usize = 20;
pub const MAX_TOKENS_PER_RESPONSE: usize = 500;

pub const EMBEDDING_MODEL: &str = "all-MiniLM-L6-v2";
pub const EMBEDDING_DIMENSIONS: usize = 384;
pub const OPENAI_EMBED_DIM: usize = 1536;
pub const OPENAI_EMBED_MODEL: &str = "text-embedding-3-small";

pub const MAX_CODE_BYTES: usize = 500_000;
pub const MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024;
