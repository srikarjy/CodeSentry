use std::collections::HashMap;
use tracing::info;

use crate::{
    error::{AnalysisError, AnalysisResult},
    types::Language,
};

pub mod javascript;
pub mod typescript;

pub struct ParserRegistry {
    parsers: HashMap<Language, Box<dyn Parser>>,
}

pub trait Parser: Send + Sync {
    fn language(&self) -> Language;
    fn parse(&self, content: &str) -> AnalysisResult<ParseResult>;
}

#[derive(Debug)]
pub struct ParseResult {
    pub language: Language,
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<ImportInfo>,
}

#[derive(Debug)]
pub struct FunctionInfo {
    pub name: String,
    pub line: u32,
    pub complexity: u32,
}

#[derive(Debug)]
pub struct ClassInfo {
    pub name: String,
    pub line: u32,
}

#[derive(Debug)]
pub struct ImportInfo {
    pub module: String,
    pub line: u32,
}

impl ParserRegistry {
    pub async fn new() -> AnalysisResult<Self> {
        let mut parsers: HashMap<Language, Box<dyn Parser>> = HashMap::new();
        
        // Register JavaScript parser
        let js_parser = javascript::JavaScriptParser::new()?;
        parsers.insert(Language::JavaScript, Box::new(js_parser));
        
        // Register TypeScript parser
        let ts_parser = typescript::TypeScriptParser::new()?;
        parsers.insert(Language::TypeScript, Box::new(ts_parser));
        
        info!("Parser registry initialized with {} parsers", parsers.len());
        
        Ok(Self { parsers })
    }

    pub fn get_parser(&self, language: &Language) -> Option<&dyn Parser> {
        self.parsers.get(language).map(|p| p.as_ref())
    }

    pub fn supported_languages(&self) -> Vec<Language> {
        self.parsers.keys().cloned().collect()
    }
}