use async_trait::async_trait;
use chrono::NaiveDate;
use domain::{
    errors::RepositoryError,
    models::vehicle::*,
    ports::vehicle::*,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

// ============================
// Vehicle Category Repository
// ============================

pub struct VehicleCategoryRepository {
    pool: PgPool,
}

impl VehicleCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleCategoryRepositoryPort for VehicleCategoryRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleCategoryDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleCategoryDto>("SELECT * FROM vehicle_categories WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_categories WHERE name = $1")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_name_excluding(&self, name: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_categories WHERE name = $1 AND id != $2")
            .bind(name)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, name: &str, description: Option<&str>, is_active: bool) -> Result<VehicleCategoryDto, RepositoryError> {
        sqlx::query_as::<_, VehicleCategoryDto>(
            "INSERT INTO vehicle_categories (name, description, is_active) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(name)
        .bind(description)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, name: Option<&str>, description: Option<&str>, is_active: Option<bool>) -> Result<VehicleCategoryDto, RepositoryError> {
        sqlx::query_as::<_, VehicleCategoryDto>(
            r#"
            UPDATE vehicle_categories
            SET name = COALESCE($2, name),
                description = CASE WHEN $3::TEXT IS NOT NULL THEN $3 ELSE description END,
                is_active = COALESCE($4, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM vehicle_categories WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleCategoryDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let items = sqlx::query_as::<_, VehicleCategoryDto>(
            r#"
            SELECT * FROM vehicle_categories
            WHERE ($1::TEXT IS NULL OR name ILIKE $1)
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vehicle_categories WHERE ($1::TEXT IS NULL OR name ILIKE $1)"
        )
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((items, total))
    }
}

// ============================
// Vehicle Make Repository
// ============================

pub struct VehicleMakeRepository {
    pool: PgPool,
}

impl VehicleMakeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleMakeRepositoryPort for VehicleMakeRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleMakeDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleMakeDto>("SELECT * FROM vehicle_makes WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_makes WHERE name = $1")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_name_excluding(&self, name: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_makes WHERE name = $1 AND id != $2")
            .bind(name)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, name: &str) -> Result<VehicleMakeDto, RepositoryError> {
        sqlx::query_as::<_, VehicleMakeDto>(
            "INSERT INTO vehicle_makes (name) VALUES ($1) RETURNING *"
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, name: Option<&str>, is_active: Option<bool>) -> Result<VehicleMakeDto, RepositoryError> {
        sqlx::query_as::<_, VehicleMakeDto>(
            r#"
            UPDATE vehicle_makes
            SET name = COALESCE($2, name),
                is_active = COALESCE($3, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM vehicle_makes WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleMakeDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let items = sqlx::query_as::<_, VehicleMakeDto>(
            r#"
            SELECT * FROM vehicle_makes
            WHERE ($1::TEXT IS NULL OR name ILIKE $1)
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vehicle_makes WHERE ($1::TEXT IS NULL OR name ILIKE $1)"
        )
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((items, total))
    }
}

// ============================
// Vehicle Model Repository
// ============================

pub struct VehicleModelRepository {
    pool: PgPool,
}

impl VehicleModelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleModelRepositoryPort for VehicleModelRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleModelDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleModelDto>("SELECT * FROM vehicle_models WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_name_in_make(&self, name: &str, make_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_models WHERE name = $1 AND make_id = $2")
            .bind(name)
            .bind(make_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_name_in_make_excluding(&self, name: &str, make_id: Uuid, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_models WHERE name = $1 AND make_id = $2 AND id != $3")
            .bind(name)
            .bind(make_id)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, make_id: Uuid, name: &str) -> Result<VehicleModelDto, RepositoryError> {
        sqlx::query_as::<_, VehicleModelDto>(
            "INSERT INTO vehicle_models (make_id, name) VALUES ($1, $2) RETURNING *"
        )
        .bind(make_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, name: Option<&str>, is_active: Option<bool>) -> Result<VehicleModelDto, RepositoryError> {
        sqlx::query_as::<_, VehicleModelDto>(
            r#"
            UPDATE vehicle_models
            SET name = COALESCE($2, name),
                is_active = COALESCE($3, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM vehicle_models WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>, make_id: Option<Uuid>) -> Result<(Vec<VehicleModelDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let items = sqlx::query_as::<_, VehicleModelDto>(
            r#"
            SELECT * FROM vehicle_models
            WHERE ($1::TEXT IS NULL OR name ILIKE $1)
              AND ($2::UUID IS NULL OR make_id = $2)
            ORDER BY name
            LIMIT $3 OFFSET $4
            "#
        )
        .bind(&search_pattern)
        .bind(make_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM vehicle_models
            WHERE ($1::TEXT IS NULL OR name ILIKE $1)
              AND ($2::UUID IS NULL OR make_id = $2)
            "#
        )
        .bind(&search_pattern)
        .bind(make_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((items, total))
    }
}

// ============================
// Vehicle Color Repository
// ============================

pub struct VehicleColorRepository {
    pool: PgPool,
}

impl VehicleColorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleColorRepositoryPort for VehicleColorRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleColorDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleColorDto>("SELECT * FROM vehicle_colors WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_colors WHERE name = $1")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_name_excluding(&self, name: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_colors WHERE name = $1 AND id != $2")
            .bind(name)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, name: &str, hex_code: Option<&str>) -> Result<VehicleColorDto, RepositoryError> {
        sqlx::query_as::<_, VehicleColorDto>(
            "INSERT INTO vehicle_colors (name, hex_code) VALUES ($1, $2) RETURNING *"
        )
        .bind(name)
        .bind(hex_code)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, name: Option<&str>, hex_code: Option<&str>, is_active: Option<bool>) -> Result<VehicleColorDto, RepositoryError> {
        sqlx::query_as::<_, VehicleColorDto>(
            r#"
            UPDATE vehicle_colors
            SET name = COALESCE($2, name),
                hex_code = CASE WHEN $3::TEXT IS NOT NULL THEN $3 ELSE hex_code END,
                is_active = COALESCE($4, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(name)
        .bind(hex_code)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM vehicle_colors WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleColorDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let items = sqlx::query_as::<_, VehicleColorDto>(
            r#"
            SELECT * FROM vehicle_colors
            WHERE ($1::TEXT IS NULL OR name ILIKE $1)
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vehicle_colors WHERE ($1::TEXT IS NULL OR name ILIKE $1)"
        )
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((items, total))
    }
}

// ============================
// Vehicle Fuel Type Repository
// ============================

pub struct VehicleFuelTypeRepository {
    pool: PgPool,
}

impl VehicleFuelTypeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleFuelTypeRepositoryPort for VehicleFuelTypeRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleFuelTypeDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleFuelTypeDto>("SELECT * FROM vehicle_fuel_types WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_fuel_types WHERE name = $1")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_name_excluding(&self, name: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_fuel_types WHERE name = $1 AND id != $2")
            .bind(name)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, name: &str) -> Result<VehicleFuelTypeDto, RepositoryError> {
        sqlx::query_as::<_, VehicleFuelTypeDto>(
            "INSERT INTO vehicle_fuel_types (name) VALUES ($1) RETURNING *"
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, name: Option<&str>, is_active: Option<bool>) -> Result<VehicleFuelTypeDto, RepositoryError> {
        sqlx::query_as::<_, VehicleFuelTypeDto>(
            r#"
            UPDATE vehicle_fuel_types
            SET name = COALESCE($2, name),
                is_active = COALESCE($3, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM vehicle_fuel_types WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleFuelTypeDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let items = sqlx::query_as::<_, VehicleFuelTypeDto>(
            r#"
            SELECT * FROM vehicle_fuel_types
            WHERE ($1::TEXT IS NULL OR name ILIKE $1)
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vehicle_fuel_types WHERE ($1::TEXT IS NULL OR name ILIKE $1)"
        )
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((items, total))
    }
}

// ============================
// Vehicle Transmission Type Repository
// ============================

pub struct VehicleTransmissionTypeRepository {
    pool: PgPool,
}

impl VehicleTransmissionTypeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleTransmissionTypeRepositoryPort for VehicleTransmissionTypeRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleTransmissionTypeDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleTransmissionTypeDto>("SELECT * FROM vehicle_transmission_types WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_name(&self, name: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_transmission_types WHERE name = $1")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_name_excluding(&self, name: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicle_transmission_types WHERE name = $1 AND id != $2")
            .bind(name)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(&self, name: &str) -> Result<VehicleTransmissionTypeDto, RepositoryError> {
        sqlx::query_as::<_, VehicleTransmissionTypeDto>(
            "INSERT INTO vehicle_transmission_types (name) VALUES ($1) RETURNING *"
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(&self, id: Uuid, name: Option<&str>, is_active: Option<bool>) -> Result<VehicleTransmissionTypeDto, RepositoryError> {
        sqlx::query_as::<_, VehicleTransmissionTypeDto>(
            r#"
            UPDATE vehicle_transmission_types
            SET name = COALESCE($2, name),
                is_active = COALESCE($3, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(name)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM vehicle_transmission_types WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(&self, limit: i64, offset: i64, search: Option<String>) -> Result<(Vec<VehicleTransmissionTypeDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let items = sqlx::query_as::<_, VehicleTransmissionTypeDto>(
            r#"
            SELECT * FROM vehicle_transmission_types
            WHERE ($1::TEXT IS NULL OR name ILIKE $1)
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM vehicle_transmission_types WHERE ($1::TEXT IS NULL OR name ILIKE $1)"
        )
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((items, total))
    }
}

// ============================
// Vehicle Repository
// ============================

pub struct VehicleRepository {
    pool: PgPool,
}

impl VehicleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleRepositoryPort for VehicleRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleDto>("SELECT * FROM vehicles WHERE id = $1 AND is_deleted = false")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<VehicleWithDetailsDto>, RepositoryError> {
        let result = sqlx::query(
            r#"
            SELECT
                v.id, v.license_plate, v.chassis_number, v.renavam, v.engine_number,
                v.category_id, vc.name as category_name,
                v.make_id, vm.name as make_name,
                v.model_id, vmod.name as model_name,
                v.color_id, vcol.name as color_name,
                v.fuel_type_id, vft.name as fuel_type_name,
                v.transmission_type_id, vtt.name as transmission_type_name,
                v.manufacture_year, v.model_year,
                v.passenger_capacity, v.load_capacity_kg,
                v.engine_displacement, v.horsepower,
                v.acquisition_type, v.acquisition_date, v.purchase_value,
                v.patrimony_number, v.department_id,
                v.status,
                v.created_at, v.updated_at
            FROM vehicles v
            JOIN vehicle_categories vc ON v.category_id = vc.id
            JOIN vehicle_makes vm ON v.make_id = vm.id
            JOIN vehicle_models vmod ON v.model_id = vmod.id
            JOIN vehicle_colors vcol ON v.color_id = vcol.id
            JOIN vehicle_fuel_types vft ON v.fuel_type_id = vft.id
            LEFT JOIN vehicle_transmission_types vtt ON v.transmission_type_id = vtt.id
            WHERE v.id = $1 AND v.is_deleted = false
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(result.map(|r| VehicleWithDetailsDto {
            id: r.get("id"),
            license_plate: r.get("license_plate"),
            chassis_number: r.get("chassis_number"),
            renavam: r.get("renavam"),
            engine_number: r.get("engine_number"),
            category_id: r.get("category_id"),
            category_name: r.get("category_name"),
            make_id: r.get("make_id"),
            make_name: r.get("make_name"),
            model_id: r.get("model_id"),
            model_name: r.get("model_name"),
            color_id: r.get("color_id"),
            color_name: r.get("color_name"),
            fuel_type_id: r.get("fuel_type_id"),
            fuel_type_name: r.get("fuel_type_name"),
            transmission_type_id: r.get("transmission_type_id"),
            transmission_type_name: r.get("transmission_type_name"),
            manufacture_year: r.get("manufacture_year"),
            model_year: r.get("model_year"),
            passenger_capacity: r.get("passenger_capacity"),
            load_capacity_kg: r.get("load_capacity_kg"),
            engine_displacement: r.get("engine_displacement"),
            horsepower: r.get("horsepower"),
            acquisition_type: r.get("acquisition_type"),
            acquisition_date: r.get("acquisition_date"),
            purchase_value: r.get("purchase_value"),
            patrimony_number: r.get("patrimony_number"),
            department_id: r.get("department_id"),
            status: r.get("status"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }))
    }

    async fn exists_by_license_plate(&self, plate: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE license_plate = $1 AND is_deleted = false")
            .bind(plate)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_license_plate_excluding(&self, plate: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE license_plate = $1 AND id != $2 AND is_deleted = false")
            .bind(plate)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_chassis(&self, chassis: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE chassis_number = $1 AND is_deleted = false")
            .bind(chassis)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_chassis_excluding(&self, chassis: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE chassis_number = $1 AND id != $2 AND is_deleted = false")
            .bind(chassis)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_renavam(&self, renavam: &str) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE renavam = $1 AND is_deleted = false")
            .bind(renavam)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn exists_by_renavam_excluding(&self, renavam: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vehicles WHERE renavam = $1 AND id != $2 AND is_deleted = false")
            .bind(renavam)
            .bind(exclude_id)
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(count > 0)
    }

    async fn create(
        &self,
        license_plate: &str,
        chassis_number: &str,
        renavam: &str,
        engine_number: Option<&str>,
        category_id: Uuid,
        make_id: Uuid,
        model_id: Uuid,
        color_id: Uuid,
        fuel_type_id: Uuid,
        transmission_type_id: Option<Uuid>,
        manufacture_year: i32,
        model_year: i32,
        passenger_capacity: Option<i32>,
        load_capacity_kg: Option<Decimal>,
        engine_displacement: Option<i32>,
        horsepower: Option<i32>,
        acquisition_type: AcquisitionType,
        acquisition_date: Option<NaiveDate>,
        purchase_value: Option<Decimal>,
        patrimony_number: Option<&str>,
        department_id: Option<Uuid>,
        status: VehicleStatus,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDto, RepositoryError> {
        sqlx::query_as::<_, VehicleDto>(
            r#"
            INSERT INTO vehicles (
                license_plate, chassis_number, renavam, engine_number,
                category_id, make_id, model_id, color_id, fuel_type_id, transmission_type_id,
                manufacture_year, model_year,
                passenger_capacity, load_capacity_kg, engine_displacement, horsepower,
                acquisition_type, acquisition_date, purchase_value,
                patrimony_number, department_id, status, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)
            RETURNING *
            "#
        )
        .bind(license_plate)
        .bind(chassis_number)
        .bind(renavam)
        .bind(engine_number)
        .bind(category_id)
        .bind(make_id)
        .bind(model_id)
        .bind(color_id)
        .bind(fuel_type_id)
        .bind(transmission_type_id)
        .bind(manufacture_year)
        .bind(model_year)
        .bind(passenger_capacity)
        .bind(load_capacity_kg)
        .bind(engine_displacement)
        .bind(horsepower)
        .bind(acquisition_type)
        .bind(acquisition_date)
        .bind(purchase_value)
        .bind(patrimony_number)
        .bind(department_id)
        .bind(status)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        license_plate: Option<&str>,
        chassis_number: Option<&str>,
        renavam: Option<&str>,
        engine_number: Option<&str>,
        category_id: Option<Uuid>,
        make_id: Option<Uuid>,
        model_id: Option<Uuid>,
        color_id: Option<Uuid>,
        fuel_type_id: Option<Uuid>,
        transmission_type_id: Option<Uuid>,
        manufacture_year: Option<i32>,
        model_year: Option<i32>,
        passenger_capacity: Option<i32>,
        load_capacity_kg: Option<Decimal>,
        engine_displacement: Option<i32>,
        horsepower: Option<i32>,
        acquisition_type: Option<AcquisitionType>,
        acquisition_date: Option<NaiveDate>,
        purchase_value: Option<Decimal>,
        patrimony_number: Option<&str>,
        department_id: Option<Uuid>,
        status: Option<VehicleStatus>,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleDto, RepositoryError> {
        sqlx::query_as::<_, VehicleDto>(
            r#"
            UPDATE vehicles
            SET license_plate = COALESCE($2, license_plate),
                chassis_number = COALESCE($3, chassis_number),
                renavam = COALESCE($4, renavam),
                engine_number = CASE WHEN $5::TEXT IS NOT NULL THEN $5 ELSE engine_number END,
                category_id = COALESCE($6, category_id),
                make_id = COALESCE($7, make_id),
                model_id = COALESCE($8, model_id),
                color_id = COALESCE($9, color_id),
                fuel_type_id = COALESCE($10, fuel_type_id),
                transmission_type_id = CASE WHEN $11::UUID IS NOT NULL THEN $11 ELSE transmission_type_id END,
                manufacture_year = COALESCE($12, manufacture_year),
                model_year = COALESCE($13, model_year),
                passenger_capacity = CASE WHEN $14::INT IS NOT NULL THEN $14 ELSE passenger_capacity END,
                load_capacity_kg = CASE WHEN $15::NUMERIC IS NOT NULL THEN $15 ELSE load_capacity_kg END,
                engine_displacement = CASE WHEN $16::INT IS NOT NULL THEN $16 ELSE engine_displacement END,
                horsepower = CASE WHEN $17::INT IS NOT NULL THEN $17 ELSE horsepower END,
                acquisition_type = COALESCE($18, acquisition_type),
                acquisition_date = CASE WHEN $19::DATE IS NOT NULL THEN $19 ELSE acquisition_date END,
                purchase_value = CASE WHEN $20::NUMERIC IS NOT NULL THEN $20 ELSE purchase_value END,
                patrimony_number = CASE WHEN $21::TEXT IS NOT NULL THEN $21 ELSE patrimony_number END,
                department_id = CASE WHEN $22::UUID IS NOT NULL THEN $22 ELSE department_id END,
                status = COALESCE($23, status),
                updated_by = $24,
                updated_at = NOW()
            WHERE id = $1 AND is_deleted = false
            RETURNING *
            "#
        )
        .bind(id)
        .bind(license_plate)
        .bind(chassis_number)
        .bind(renavam)
        .bind(engine_number)
        .bind(category_id)
        .bind(make_id)
        .bind(model_id)
        .bind(color_id)
        .bind(fuel_type_id)
        .bind(transmission_type_id)
        .bind(manufacture_year)
        .bind(model_year)
        .bind(passenger_capacity)
        .bind(load_capacity_kg)
        .bind(engine_displacement)
        .bind(horsepower)
        .bind(acquisition_type)
        .bind(acquisition_date)
        .bind(purchase_value)
        .bind(patrimony_number)
        .bind(department_id)
        .bind(status)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn soft_delete(&self, id: Uuid, deleted_by: Option<Uuid>) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            "UPDATE vehicles SET is_deleted = true, deleted_at = NOW(), deleted_by = $2, updated_at = NOW() WHERE id = $1 AND is_deleted = false"
        )
        .bind(id)
        .bind(deleted_by)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn restore(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            "UPDATE vehicles SET is_deleted = false, deleted_at = NULL, deleted_by = NULL, updated_at = NOW() WHERE id = $1 AND is_deleted = true"
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        status: Option<VehicleStatus>,
        category_id: Option<Uuid>,
        make_id: Option<Uuid>,
        fuel_type_id: Option<Uuid>,
        department_id: Option<Uuid>,
        include_deleted: bool,
    ) -> Result<(Vec<VehicleWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let records = sqlx::query(
            r#"
            SELECT
                v.id, v.license_plate, v.chassis_number, v.renavam, v.engine_number,
                v.category_id, vc.name as category_name,
                v.make_id, vm.name as make_name,
                v.model_id, vmod.name as model_name,
                v.color_id, vcol.name as color_name,
                v.fuel_type_id, vft.name as fuel_type_name,
                v.transmission_type_id, vtt.name as transmission_type_name,
                v.manufacture_year, v.model_year,
                v.passenger_capacity, v.load_capacity_kg,
                v.engine_displacement, v.horsepower,
                v.acquisition_type, v.acquisition_date, v.purchase_value,
                v.patrimony_number, v.department_id,
                v.status,
                v.created_at, v.updated_at
            FROM vehicles v
            JOIN vehicle_categories vc ON v.category_id = vc.id
            JOIN vehicle_makes vm ON v.make_id = vm.id
            JOIN vehicle_models vmod ON v.model_id = vmod.id
            JOIN vehicle_colors vcol ON v.color_id = vcol.id
            JOIN vehicle_fuel_types vft ON v.fuel_type_id = vft.id
            LEFT JOIN vehicle_transmission_types vtt ON v.transmission_type_id = vtt.id
            WHERE ($1::BOOLEAN OR v.is_deleted = false)
              AND ($2::TEXT IS NULL OR v.license_plate ILIKE $2 OR v.chassis_number ILIKE $2 OR v.renavam ILIKE $2 OR vm.name ILIKE $2 OR vmod.name ILIKE $2)
              AND ($3::vehicle_status_enum IS NULL OR v.status = $3)
              AND ($4::UUID IS NULL OR v.category_id = $4)
              AND ($5::UUID IS NULL OR v.make_id = $5)
              AND ($6::UUID IS NULL OR v.fuel_type_id = $6)
              AND ($7::UUID IS NULL OR v.department_id = $7)
            ORDER BY v.updated_at DESC
            LIMIT $8 OFFSET $9
            "#
        )
        .bind(include_deleted)
        .bind(&search_pattern)
        .bind(&status)
        .bind(category_id)
        .bind(make_id)
        .bind(fuel_type_id)
        .bind(department_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let vehicles: Vec<VehicleWithDetailsDto> = records.into_iter().map(|r| VehicleWithDetailsDto {
            id: r.get("id"),
            license_plate: r.get("license_plate"),
            chassis_number: r.get("chassis_number"),
            renavam: r.get("renavam"),
            engine_number: r.get("engine_number"),
            category_id: r.get("category_id"),
            category_name: r.get("category_name"),
            make_id: r.get("make_id"),
            make_name: r.get("make_name"),
            model_id: r.get("model_id"),
            model_name: r.get("model_name"),
            color_id: r.get("color_id"),
            color_name: r.get("color_name"),
            fuel_type_id: r.get("fuel_type_id"),
            fuel_type_name: r.get("fuel_type_name"),
            transmission_type_id: r.get("transmission_type_id"),
            transmission_type_name: r.get("transmission_type_name"),
            manufacture_year: r.get("manufacture_year"),
            model_year: r.get("model_year"),
            passenger_capacity: r.get("passenger_capacity"),
            load_capacity_kg: r.get("load_capacity_kg"),
            engine_displacement: r.get("engine_displacement"),
            horsepower: r.get("horsepower"),
            acquisition_type: r.get("acquisition_type"),
            acquisition_date: r.get("acquisition_date"),
            purchase_value: r.get("purchase_value"),
            patrimony_number: r.get("patrimony_number"),
            department_id: r.get("department_id"),
            status: r.get("status"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM vehicles v
            JOIN vehicle_makes vm ON v.make_id = vm.id
            JOIN vehicle_models vmod ON v.model_id = vmod.id
            WHERE ($1::BOOLEAN OR v.is_deleted = false)
              AND ($2::TEXT IS NULL OR v.license_plate ILIKE $2 OR v.chassis_number ILIKE $2 OR v.renavam ILIKE $2 OR vm.name ILIKE $2 OR vmod.name ILIKE $2)
              AND ($3::vehicle_status_enum IS NULL OR v.status = $3)
              AND ($4::UUID IS NULL OR v.category_id = $4)
              AND ($5::UUID IS NULL OR v.make_id = $5)
              AND ($6::UUID IS NULL OR v.fuel_type_id = $6)
              AND ($7::UUID IS NULL OR v.department_id = $7)
            "#
        )
        .bind(include_deleted)
        .bind(&search_pattern)
        .bind(&status)
        .bind(category_id)
        .bind(make_id)
        .bind(fuel_type_id)
        .bind(department_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((vehicles, total))
    }

    async fn search_autocomplete(&self, query: &str, limit: i64) -> Result<Vec<VehicleDto>, RepositoryError> {
        let pattern = format!("%{}%", query);
        sqlx::query_as::<_, VehicleDto>(
            r#"
            SELECT * FROM vehicles
            WHERE is_deleted = false
              AND (license_plate ILIKE $1 OR chassis_number ILIKE $1 OR renavam ILIKE $1 OR patrimony_number ILIKE $1)
            ORDER BY license_plate
            LIMIT $2
            "#
        )
        .bind(pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}

// ============================
// Vehicle Document Repository
// ============================

pub struct VehicleDocumentRepository {
    pool: PgPool,
}

impl VehicleDocumentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleDocumentRepositoryPort for VehicleDocumentRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleDocumentDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleDocumentDto>("SELECT * FROM vehicle_documents WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn create(
        &self,
        vehicle_id: Uuid,
        document_type: DocumentType,
        file_name: &str,
        file_path: &str,
        file_size: i64,
        mime_type: &str,
        description: Option<&str>,
        uploaded_by: Option<Uuid>,
    ) -> Result<VehicleDocumentDto, RepositoryError> {
        sqlx::query_as::<_, VehicleDocumentDto>(
            r#"
            INSERT INTO vehicle_documents (vehicle_id, document_type, file_name, file_path, file_size, mime_type, description, uploaded_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING *
            "#
        )
        .bind(vehicle_id)
        .bind(document_type)
        .bind(file_name)
        .bind(file_path)
        .bind(file_size)
        .bind(mime_type)
        .bind(description)
        .bind(uploaded_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM vehicle_documents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_by_vehicle(&self, vehicle_id: Uuid) -> Result<Vec<VehicleDocumentDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleDocumentDto>(
            "SELECT * FROM vehicle_documents WHERE vehicle_id = $1 ORDER BY created_at DESC"
        )
        .bind(vehicle_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}

// ============================
// Vehicle Status History Repository
// ============================

pub struct VehicleStatusHistoryRepository {
    pool: PgPool,
}

impl VehicleStatusHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleStatusHistoryRepositoryPort for VehicleStatusHistoryRepository {
    async fn create(
        &self,
        vehicle_id: Uuid,
        old_status: Option<VehicleStatus>,
        new_status: VehicleStatus,
        reason: Option<&str>,
        changed_by: Option<Uuid>,
    ) -> Result<VehicleStatusHistoryDto, RepositoryError> {
        sqlx::query_as::<_, VehicleStatusHistoryDto>(
            r#"
            INSERT INTO vehicle_status_history (vehicle_id, old_status, new_status, reason, changed_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(vehicle_id)
        .bind(old_status)
        .bind(new_status)
        .bind(reason)
        .bind(changed_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list_by_vehicle(&self, vehicle_id: Uuid) -> Result<Vec<VehicleStatusHistoryDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleStatusHistoryDto>(
            "SELECT * FROM vehicle_status_history WHERE vehicle_id = $1 ORDER BY created_at DESC"
        )
        .bind(vehicle_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
