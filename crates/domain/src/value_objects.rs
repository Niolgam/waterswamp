use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt; // Importante para Display
use utoipa::ToSchema;

lazy_static! {
    static ref EMAIL_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();

    static ref USERNAME_REGEX: Regex = Regex::new(
        r"^[a-zA-Z0-9_-]{3,50}$"
    ).unwrap();

    // Location-related validation patterns
    static ref STATE_CODE_REGEX: Regex = Regex::new(r"^[A-Z]{2}$").unwrap();
    static ref LOCATION_NAME_REGEX: Regex = Regex::new(r"^[a-zA-ZÀ-ÿ0-9\s\-]{2,100}$").unwrap();

    // Warehouse-related validation patterns
    static ref MATERIAL_CODE_REGEX: Regex = Regex::new(r"^[0-9]{1,10}$").unwrap();
    static ref CATMAT_CODE_REGEX: Regex = Regex::new(r"^[0-9]{1,20}$").unwrap();
    static ref UNIT_OF_MEASURE_REGEX: Regex = Regex::new(r"^[a-zA-ZÀ-ÿ0-9\s\-]{1,50}$").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(try_from = "String")]
pub struct Email(String);

impl Email {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Implementar Display permite usar %payload.email nos logs
impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Email {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if EMAIL_REGEX.is_match(&value) {
            Ok(Self(value))
        } else {
            Err(format!("Email inválido: '{}'", value))
        }
    }
}

impl TryFrom<&str> for Email {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

// Implementar AsRef para facilitar uso onde se espera &str
impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(try_from = "String")]
pub struct Username(String);

impl Username {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// Implementar Display permite usar %user.username nos logs
impl fmt::Display for Username {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Username {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if USERNAME_REGEX.is_match(&value) {
            Ok(Self(value))
        } else {
            Err("Username inválido. Deve ter entre 3 e 50 caracteres alfanuméricos (incluindo _ e -).".to_string())
        }
    }
}

impl TryFrom<&str> for Username {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl AsRef<str> for Username {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ============================
// Location Value Objects
// ============================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(try_from = "String")]
pub struct StateCode(String);

impl StateCode {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StateCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for StateCode {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let upper = value.to_uppercase();
        if STATE_CODE_REGEX.is_match(&upper) {
            Ok(Self(upper))
        } else {
            Err("Código do estado inválido. Deve ter exatamente 2 letras maiúsculas (ex: SP, RJ).".to_string())
        }
    }
}

impl TryFrom<&str> for StateCode {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl AsRef<str> for StateCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
#[serde(try_from = "String")]
pub struct LocationName(String);

impl LocationName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for LocationName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for LocationName {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim().to_string();
        if LOCATION_NAME_REGEX.is_match(&trimmed) {
            Ok(Self(trimmed))
        } else {
            Err("Nome de localização inválido. Deve ter entre 2 e 100 caracteres (letras, números, espaços e hífens).".to_string())
        }
    }
}

impl TryFrom<&str> for LocationName {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl AsRef<str> for LocationName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ============================
// Warehouse Value Objects
// ============================

/// Material Code - Código numérico do grupo de material (1-10 dígitos)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(transparent)]
#[serde(try_from = "String")]
pub struct MaterialCode(String);

impl MaterialCode {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MaterialCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for MaterialCode {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim().to_string();
        if MATERIAL_CODE_REGEX.is_match(&trimmed) {
            Ok(Self(trimmed))
        } else {
            Err("Código de material inválido. Deve conter apenas números (1 a 10 dígitos).".to_string())
        }
    }
}

impl TryFrom<&str> for MaterialCode {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl AsRef<str> for MaterialCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// CATMAT Code - Código do material no sistema CATMAT do governo (opcional)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(transparent)]
#[serde(try_from = "String")]
pub struct CatmatCode(String);

impl CatmatCode {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CatmatCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for CatmatCode {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim().to_string();
        if CATMAT_CODE_REGEX.is_match(&trimmed) {
            Ok(Self(trimmed))
        } else {
            Err("Código CATMAT inválido. Deve conter apenas números (1 a 20 dígitos).".to_string())
        }
    }
}

impl TryFrom<&str> for CatmatCode {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl AsRef<str> for CatmatCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Unit of Measure - Unidade de medida do material (ex: Litro, Unidade, Kg)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[sqlx(transparent)]
#[serde(try_from = "String")]
pub struct UnitOfMeasure(String);

impl UnitOfMeasure {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UnitOfMeasure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for UnitOfMeasure {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim().to_string();
        if UNIT_OF_MEASURE_REGEX.is_match(&trimmed) && !trimmed.is_empty() {
            Ok(Self(trimmed))
        } else {
            Err("Unidade de medida inválida. Deve ter entre 1 e 50 caracteres.".to_string())
        }
    }
}

impl TryFrom<&str> for UnitOfMeasure {
    type Error = String;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl AsRef<str> for UnitOfMeasure {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
