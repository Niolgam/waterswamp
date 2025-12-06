use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

use crate::value_objects::Coordinates;

/// Campus entity - represents a university campus
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Campus {
    pub id: Uuid,
    pub name: String,
    pub acronym: String,
    pub city_id: Uuid,
    pub coordinates: Coordinates,
    pub address: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// DTO for creating a Campus
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateCampusDto {
    #[validate(length(min = 3, max = 200, message = "Name must be between 3 and 200 characters"))]
    pub name: String,

    #[validate(length(min = 2, max = 10, message = "Acronym must be between 2 and 10 characters"))]
    pub acronym: String,

    pub city_id: Uuid,

    pub coordinates: CoordinatesDto,

    #[validate(length(min = 10, max = 500, message = "Address must be between 10 and 500 characters"))]
    pub address: String,
}

/// DTO for updating a Campus
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateCampusDto {
    #[validate(length(min = 3, max = 200, message = "Name must be between 3 and 200 characters"))]
    pub name: Option<String>,

    #[validate(length(min = 2, max = 10, message = "Acronym must be between 2 and 10 characters"))]
    pub acronym: Option<String>,

    pub city_id: Option<Uuid>,

    pub coordinates: Option<CoordinatesDto>,

    #[validate(length(min = 10, max = 500, message = "Address must be between 10 and 500 characters"))]
    pub address: Option<String>,
}

/// DTO for Coordinates (used in JSON requests)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinatesDto {
    pub latitude: f64,
    pub longitude: f64,
}

impl From<CoordinatesDto> for Coordinates {
    fn from(dto: CoordinatesDto) -> Self {
        // Validation will be done in the service layer
        Coordinates {
            latitude: dto.latitude,
            longitude: dto.longitude,
        }
    }
}

impl From<Coordinates> for CoordinatesDto {
    fn from(coord: Coordinates) -> Self {
        Self {
            latitude: coord.latitude,
            longitude: coord.longitude,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_campus_dto_validation() {
        let city_id = Uuid::new_v4();
        let valid_dto = CreateCampusDto {
            name: "Central Campus".to_string(),
            acronym: "CC".to_string(),
            city_id,
            coordinates: CoordinatesDto {
                latitude: -23.5505,
                longitude: -46.6333,
            },
            address: "Av. Paulista, 1000".to_string(),
        };

        assert!(valid_dto.validate().is_ok());
    }

    #[test]
    fn test_create_campus_dto_validation_fails() {
        let city_id = Uuid::new_v4();
        // Name too short
        let invalid_dto = CreateCampusDto {
            name: "CC".to_string(), // Only 2 characters
            acronym: "CC".to_string(),
            city_id,
            coordinates: CoordinatesDto {
                latitude: -23.5505,
                longitude: -46.6333,
            },
            address: "Av. Paulista, 1000".to_string(),
        };

        assert!(invalid_dto.validate().is_err());
    }

    #[test]
    fn test_coordinates_dto_conversion() {
        let dto = CoordinatesDto {
            latitude: -23.5505,
            longitude: -46.6333,
        };

        let coord: Coordinates = dto.clone().into();
        assert_eq!(coord.latitude, dto.latitude);
        assert_eq!(coord.longitude, dto.longitude);

        let dto_back: CoordinatesDto = coord.into();
        assert_eq!(dto_back.latitude, dto.latitude);
        assert_eq!(dto_back.longitude, dto.longitude);
    }
}
