use crate::error_handling::Error::{
    InvalidSchema, MissingSchemaKey, NoneASCIICharacters, YamlParsingError,
};
use crate::error_handling::Result;
use crate::parser::regex_parser::parser::RegexParser;
use regex_syntax::ast::Ast;
use serde_yaml::Value;
use std::collections::{HashMap, HashSet};

pub struct TimestampSchema {
    regex: String,
    ast: Ast,
}

impl TimestampSchema {
    pub fn new(regex: String) -> Result<TimestampSchema> {
        let mut regex_parser = RegexParser::new();
        let ast = regex_parser.parse_into_ast(regex.as_str())?;
        Ok(Self { regex, ast })
    }

    pub fn get_regex(&self) -> &str {
        &self.regex
    }

    pub fn get_ast(&self) -> &Ast {
        &self.ast
    }
}

pub struct VarSchema {
    pub name: String,
    pub regex: String,
    pub ast: Ast,
}

impl VarSchema {
    pub fn new(name: String, regex: String) -> Result<VarSchema> {
        let mut regex_parser = RegexParser::new();
        let ast = regex_parser.parse_into_ast(regex.as_str())?;
        Ok(Self { name, regex, ast })
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_regex(&self) -> &str {
        &self.regex
    }

    pub fn get_ast(&self) -> &Ast {
        &self.ast
    }
}

pub enum Schema {
    Timestamp(TimestampSchema),
    Var(VarSchema),
}

impl Schema {
    pub fn get_ast(&self) -> &Ast {
        match self {
            Schema::Timestamp(schema) => schema.get_ast(),
            Schema::Var(schema) => schema.get_ast(),
        }
    }
}

pub struct ParsedSchema {
    pub schemas: Vec<Schema>,
    pub delimiters: HashSet<u8>,
}

impl ParsedSchema {
    pub fn get_schemas(&self) -> &Vec<Schema> {
        &self.schemas
    }

    pub fn has_delimiter(&self, delimiter: u8) -> bool {
        self.delimiters.contains(&delimiter)
    }
}

impl ParsedSchema {
    const TIMESTAMP_KEY: &'static str = "timestamp";
    const VAR_KEY: &'static str = "variables";
    const DELIMITER_EKY: &'static str = "delimiter";

    pub fn parse_from_str(yaml_content: &str) -> Result<ParsedSchema> {
        match Self::load_kv_pairs_from_yaml_content(yaml_content) {
            Ok(kv_pairs) => Self::load_from_kv_pairs(kv_pairs),
            Err(e) => Err(YamlParsingError(e)),
        }
    }

    fn get_key_value<'a>(
        kv_map: &'a HashMap<String, Value>,
        key: &'static str,
    ) -> Result<&'a Value> {
        kv_map.get(key).ok_or_else(|| MissingSchemaKey(key))
    }

    fn load_kv_pairs_from_yaml_content(
        yaml_content: &str,
    ) -> serde_yaml::Result<HashMap<String, Value>> {
        let kv_map_result: HashMap<String, Value> = serde_yaml::from_str(&yaml_content)?;
        Ok(kv_map_result)
    }

    fn load_from_kv_pairs(kv_pairs: HashMap<String, Value>) -> Result<Self> {
        let mut delimiters: HashSet<u8> = HashSet::new();
        let mut schemas: Vec<Schema> = Vec::new();

        // Handle timestamps
        let timestamps = Self::get_key_value(&kv_pairs, Self::TIMESTAMP_KEY)?;
        if let Value::Sequence(sequence) = timestamps {
            sequence.iter().try_for_each(|val| {
                if let Value::String(s) = val {
                    schemas.push(Schema::Timestamp(TimestampSchema::new(s.clone())?));
                    Ok(())
                } else {
                    Err(InvalidSchema)
                }
            })?;
        } else {
            return Err(InvalidSchema);
        }

        // Handle variables
        let vars = Self::get_key_value(&kv_pairs, Self::VAR_KEY)?;
        if let Value::Mapping(map) = vars {
            for (key, value) in map {
                match (key, value) {
                    (Value::String(name), Value::String(regex)) => {
                        schemas.push(Schema::Var(VarSchema::new(name.clone(), regex.clone())?));
                    }
                    _ => return Err(InvalidSchema),
                }
            }
        } else {
            return Err(InvalidSchema);
        }

        // Handle delimiter
        let delimiter = Self::get_key_value(&kv_pairs, Self::DELIMITER_EKY)?;
        if let Value::String(delimiter_str) = delimiter {
            for c in delimiter_str.chars() {
                if false == c.is_ascii() {
                    return Err(NoneASCIICharacters);
                }
                delimiters.insert(c as u8);
            }
        } else {
            return Err(InvalidSchema);
        }

        Ok((Self {
            delimiters,
            schemas,
        }))
    }
}
