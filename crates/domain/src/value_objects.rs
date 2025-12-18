use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt; // Importante para Display

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
