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

/// Coordinates value object for geographic locations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

impl Coordinates {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, String> {
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(format!(
                "Invalid latitude: {}. Must be between -90 and 90",
                latitude
            ));
        }
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(format!(
                "Invalid longitude: {}. Must be between -180 and 180",
                longitude
            ));
        }
        Ok(Self {
            latitude,
            longitude,
        })
    }
}

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{}", self.latitude, self.longitude)
    }
}

// SQLx Type implementation for storing as TEXT in PostgreSQL
impl sqlx::Type<sqlx::Postgres> for Coordinates {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

// SQLx Encode implementation
impl sqlx::Encode<'_, sqlx::Postgres> for Coordinates {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = format!("{},{}", self.latitude, self.longitude);
        <String as sqlx::Encode<sqlx::Postgres>>::encode(s, buf)
    }
}

// SQLx Decode implementation
impl sqlx::Decode<'_, sqlx::Postgres> for Coordinates {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let s = <String as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        let parts: Vec<&str> = s.split(',').collect();
        if parts.len() != 2 {
            return Err("Invalid coordinates format".into());
        }
        let latitude = parts[0].parse::<f64>()?;
        let longitude = parts[1].parse::<f64>()?;
        Coordinates::new(latitude, longitude).map_err(|e| e.into())
    }
}
