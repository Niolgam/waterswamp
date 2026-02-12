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

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<VehicleModelWithDetailsDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleModelWithDetailsDto>(
            r#"
            SELECT
                m.id, m.make_id, mk.name AS make_name,
                m.category_id, cat.name AS category_name,
                m.name,
                m.passenger_capacity, m.engine_displacement, m.horsepower,
                m.load_capacity,
                m.avg_consumption_min, m.avg_consumption_max, m.avg_consumption_target,
                m.is_active, m.created_at, m.updated_at
            FROM vehicle_models m
            JOIN vehicle_makes mk ON m.make_id = mk.id
            LEFT JOIN vehicle_categories cat ON m.category_id = cat.id
            WHERE m.id = $1
            "#
        )
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

    async fn create(
        &self,
        make_id: Uuid,
        category_id: Option<Uuid>,
        name: &str,
        passenger_capacity: Option<i32>,
        engine_displacement: Option<i32>,
        horsepower: Option<i32>,
        load_capacity: Option<Decimal>,
        avg_consumption_min: Option<Decimal>,
        avg_consumption_max: Option<Decimal>,
        avg_consumption_target: Option<Decimal>,
    ) -> Result<VehicleModelDto, RepositoryError> {
        sqlx::query_as::<_, VehicleModelDto>(
            "INSERT INTO vehicle_models (make_id, category_id, name, passenger_capacity, engine_displacement, horsepower, load_capacity, avg_consumption_min, avg_consumption_max, avg_consumption_target) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) RETURNING *"
        )
        .bind(make_id)
        .bind(category_id)
        .bind(name)
        .bind(passenger_capacity)
        .bind(engine_displacement)
        .bind(horsepower)
        .bind(load_capacity)
        .bind(avg_consumption_min)
        .bind(avg_consumption_max)
        .bind(avg_consumption_target)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        category_id: Option<Uuid>,
        passenger_capacity: Option<i32>,
        engine_displacement: Option<i32>,
        horsepower: Option<i32>,
        load_capacity: Option<Decimal>,
        avg_consumption_min: Option<Decimal>,
        avg_consumption_max: Option<Decimal>,
        avg_consumption_target: Option<Decimal>,
        is_active: Option<bool>,
    ) -> Result<VehicleModelDto, RepositoryError> {
        sqlx::query_as::<_, VehicleModelDto>(
            r#"
            UPDATE vehicle_models
            SET name = COALESCE($2, name),
                category_id = CASE WHEN $3::UUID IS NOT NULL THEN $3 ELSE category_id END,
                passenger_capacity = CASE WHEN $4::INT IS NOT NULL THEN $4 ELSE passenger_capacity END,
                engine_displacement = CASE WHEN $5::INT IS NOT NULL THEN $5 ELSE engine_displacement END,
                horsepower = CASE WHEN $6::INT IS NOT NULL THEN $6 ELSE horsepower END,
                load_capacity = CASE WHEN $7::NUMERIC IS NOT NULL THEN $7 ELSE load_capacity END,
                avg_consumption_min = CASE WHEN $8::NUMERIC IS NOT NULL THEN $8 ELSE avg_consumption_min END,
                avg_consumption_max = CASE WHEN $9::NUMERIC IS NOT NULL THEN $9 ELSE avg_consumption_max END,
                avg_consumption_target = CASE WHEN $10::NUMERIC IS NOT NULL THEN $10 ELSE avg_consumption_target END,
                is_active = COALESCE($11, is_active),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#
        )
        .bind(id)
        .bind(name)
        .bind(category_id)
        .bind(passenger_capacity)
        .bind(engine_displacement)
        .bind(horsepower)
        .bind(load_capacity)
        .bind(avg_consumption_min)
        .bind(avg_consumption_max)
        .bind(avg_consumption_target)
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

    async fn list(&self, limit: i64, offset: i64, search: Option<String>, make_id: Option<Uuid>) -> Result<(Vec<VehicleModelWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));
        let items = sqlx::query_as::<_, VehicleModelWithDetailsDto>(
            r#"
            SELECT
                m.id, m.make_id, mk.name AS make_name,
                m.category_id, cat.name AS category_name,
                m.name,
                m.passenger_capacity, m.engine_displacement, m.horsepower,
                m.load_capacity,
                m.avg_consumption_min, m.avg_consumption_max, m.avg_consumption_target,
                m.is_active, m.created_at, m.updated_at
            FROM vehicle_models m
            JOIN vehicle_makes mk ON m.make_id = mk.id
            LEFT JOIN vehicle_categories cat ON m.category_id = cat.id
            WHERE ($1::TEXT IS NULL OR m.name ILIKE $1)
              AND ($2::UUID IS NULL OR m.make_id = $2)
            ORDER BY m.name
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
            SELECT COUNT(*) FROM vehicle_models m
            WHERE ($1::TEXT IS NULL OR m.name ILIKE $1)
              AND ($2::UUID IS NULL OR m.make_id = $2)
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
                v.injection_pump, v.gearbox, v.differential,
                v.model_id, m.name as model_name,
                mk.name as make_name,
                cat.name as category_name,
                v.color_id, vcol.name as color_name,
                v.fuel_type_id, vft.name as fuel_type_name,
                v.transmission_type_id, vtt.name as transmission_type_name,
                v.manufacture_year, v.model_year,
                v.fleet_code, v.cost_sharing, v.initial_mileage, v.fuel_tank_capacity,
                v.acquisition_type, v.acquisition_date, v.purchase_value,
                v.patrimony_number, v.department_id,
                v.status, v.notes,
                v.last_odometer_km, v.last_odometer_date, v.last_fueling_id,
                v.created_at, v.updated_at
            FROM vehicles v
            JOIN vehicle_models m ON v.model_id = m.id
            JOIN vehicle_makes mk ON m.make_id = mk.id
            LEFT JOIN vehicle_categories cat ON m.category_id = cat.id
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
            injection_pump: r.get("injection_pump"),
            gearbox: r.get("gearbox"),
            differential: r.get("differential"),
            model_id: r.get("model_id"),
            model_name: r.get("model_name"),
            make_name: r.get("make_name"),
            category_name: r.get("category_name"),
            color_id: r.get("color_id"),
            color_name: r.get("color_name"),
            fuel_type_id: r.get("fuel_type_id"),
            fuel_type_name: r.get("fuel_type_name"),
            transmission_type_id: r.get("transmission_type_id"),
            transmission_type_name: r.get("transmission_type_name"),
            manufacture_year: r.get("manufacture_year"),
            model_year: r.get("model_year"),
            fleet_code: r.get("fleet_code"),
            cost_sharing: r.get("cost_sharing"),
            initial_mileage: r.get("initial_mileage"),
            fuel_tank_capacity: r.get("fuel_tank_capacity"),
            acquisition_type: r.get("acquisition_type"),
            acquisition_date: r.get("acquisition_date"),
            purchase_value: r.get("purchase_value"),
            patrimony_number: r.get("patrimony_number"),
            department_id: r.get("department_id"),
            status: r.get("status"),
            notes: r.get("notes"),
            last_odometer_km: r.get("last_odometer_km"),
            last_odometer_date: r.get("last_odometer_date"),
            last_fueling_id: r.get("last_fueling_id"),
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
        injection_pump: Option<&str>,
        gearbox: Option<&str>,
        differential: Option<&str>,
        model_id: Uuid,
        color_id: Uuid,
        fuel_type_id: Uuid,
        transmission_type_id: Option<Uuid>,
        manufacture_year: i32,
        model_year: i32,
        fleet_code: Option<&str>,
        cost_sharing: bool,
        initial_mileage: Option<Decimal>,
        fuel_tank_capacity: Option<Decimal>,
        acquisition_type: AcquisitionType,
        acquisition_date: Option<NaiveDate>,
        purchase_value: Option<Decimal>,
        patrimony_number: Option<&str>,
        department_id: Option<Uuid>,
        status: VehicleStatus,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleDto, RepositoryError> {
        sqlx::query_as::<_, VehicleDto>(
            r#"
            INSERT INTO vehicles (
                license_plate, chassis_number, renavam, engine_number,
                injection_pump, gearbox, differential,
                model_id, color_id, fuel_type_id, transmission_type_id,
                manufacture_year, model_year,
                fleet_code, cost_sharing, initial_mileage, fuel_tank_capacity,
                acquisition_type, acquisition_date, purchase_value,
                patrimony_number, department_id, status, notes, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25)
            RETURNING *
            "#
        )
        .bind(license_plate)
        .bind(chassis_number)
        .bind(renavam)
        .bind(engine_number)
        .bind(injection_pump)
        .bind(gearbox)
        .bind(differential)
        .bind(model_id)
        .bind(color_id)
        .bind(fuel_type_id)
        .bind(transmission_type_id)
        .bind(manufacture_year)
        .bind(model_year)
        .bind(fleet_code)
        .bind(cost_sharing)
        .bind(initial_mileage)
        .bind(fuel_tank_capacity)
        .bind(acquisition_type)
        .bind(acquisition_date)
        .bind(purchase_value)
        .bind(patrimony_number)
        .bind(department_id)
        .bind(status)
        .bind(notes)
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
        injection_pump: Option<&str>,
        gearbox: Option<&str>,
        differential: Option<&str>,
        model_id: Option<Uuid>,
        color_id: Option<Uuid>,
        fuel_type_id: Option<Uuid>,
        transmission_type_id: Option<Uuid>,
        manufacture_year: Option<i32>,
        model_year: Option<i32>,
        fleet_code: Option<&str>,
        cost_sharing: Option<bool>,
        initial_mileage: Option<Decimal>,
        fuel_tank_capacity: Option<Decimal>,
        acquisition_type: Option<AcquisitionType>,
        acquisition_date: Option<NaiveDate>,
        purchase_value: Option<Decimal>,
        patrimony_number: Option<&str>,
        department_id: Option<Uuid>,
        status: Option<VehicleStatus>,
        notes: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleDto, RepositoryError> {
        sqlx::query_as::<_, VehicleDto>(
            r#"
            UPDATE vehicles
            SET license_plate = COALESCE($2, license_plate),
                chassis_number = COALESCE($3, chassis_number),
                renavam = COALESCE($4, renavam),
                engine_number = CASE WHEN $5::TEXT IS NOT NULL THEN $5 ELSE engine_number END,
                injection_pump = CASE WHEN $6::TEXT IS NOT NULL THEN $6 ELSE injection_pump END,
                gearbox = CASE WHEN $7::TEXT IS NOT NULL THEN $7 ELSE gearbox END,
                differential = CASE WHEN $8::TEXT IS NOT NULL THEN $8 ELSE differential END,
                model_id = COALESCE($9, model_id),
                color_id = COALESCE($10, color_id),
                fuel_type_id = COALESCE($11, fuel_type_id),
                transmission_type_id = CASE WHEN $12::UUID IS NOT NULL THEN $12 ELSE transmission_type_id END,
                manufacture_year = COALESCE($13, manufacture_year),
                model_year = COALESCE($14, model_year),
                fleet_code = CASE WHEN $15::TEXT IS NOT NULL THEN $15 ELSE fleet_code END,
                cost_sharing = COALESCE($16, cost_sharing),
                initial_mileage = CASE WHEN $17::NUMERIC IS NOT NULL THEN $17 ELSE initial_mileage END,
                fuel_tank_capacity = CASE WHEN $18::NUMERIC IS NOT NULL THEN $18 ELSE fuel_tank_capacity END,
                acquisition_type = COALESCE($19, acquisition_type),
                acquisition_date = CASE WHEN $20::DATE IS NOT NULL THEN $20 ELSE acquisition_date END,
                purchase_value = CASE WHEN $21::NUMERIC IS NOT NULL THEN $21 ELSE purchase_value END,
                patrimony_number = CASE WHEN $22::TEXT IS NOT NULL THEN $22 ELSE patrimony_number END,
                department_id = CASE WHEN $23::UUID IS NOT NULL THEN $23 ELSE department_id END,
                status = COALESCE($24, status),
                notes = CASE WHEN $25::TEXT IS NOT NULL THEN $25 ELSE notes END,
                updated_by = $26,
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
        .bind(injection_pump)
        .bind(gearbox)
        .bind(differential)
        .bind(model_id)
        .bind(color_id)
        .bind(fuel_type_id)
        .bind(transmission_type_id)
        .bind(manufacture_year)
        .bind(model_year)
        .bind(fleet_code)
        .bind(cost_sharing)
        .bind(initial_mileage)
        .bind(fuel_tank_capacity)
        .bind(acquisition_type)
        .bind(acquisition_date)
        .bind(purchase_value)
        .bind(patrimony_number)
        .bind(department_id)
        .bind(status)
        .bind(notes)
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
        model_id: Option<Uuid>,
        fuel_type_id: Option<Uuid>,
        department_id: Option<Uuid>,
        include_deleted: bool,
    ) -> Result<(Vec<VehicleWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let records = sqlx::query(
            r#"
            SELECT
                v.id, v.license_plate, v.chassis_number, v.renavam, v.engine_number,
                v.injection_pump, v.gearbox, v.differential,
                v.model_id, m.name as model_name,
                mk.name as make_name,
                cat.name as category_name,
                v.color_id, vcol.name as color_name,
                v.fuel_type_id, vft.name as fuel_type_name,
                v.transmission_type_id, vtt.name as transmission_type_name,
                v.manufacture_year, v.model_year,
                v.fleet_code, v.cost_sharing, v.initial_mileage, v.fuel_tank_capacity,
                v.acquisition_type, v.acquisition_date, v.purchase_value,
                v.patrimony_number, v.department_id,
                v.status, v.notes,
                v.last_odometer_km, v.last_odometer_date, v.last_fueling_id,
                v.created_at, v.updated_at
            FROM vehicles v
            JOIN vehicle_models m ON v.model_id = m.id
            JOIN vehicle_makes mk ON m.make_id = mk.id
            LEFT JOIN vehicle_categories cat ON m.category_id = cat.id
            JOIN vehicle_colors vcol ON v.color_id = vcol.id
            JOIN vehicle_fuel_types vft ON v.fuel_type_id = vft.id
            LEFT JOIN vehicle_transmission_types vtt ON v.transmission_type_id = vtt.id
            WHERE ($1::BOOLEAN OR v.is_deleted = false)
              AND ($2::TEXT IS NULL OR v.license_plate ILIKE $2 OR v.chassis_number ILIKE $2 OR v.renavam ILIKE $2 OR mk.name ILIKE $2 OR m.name ILIKE $2)
              AND ($3::vehicle_status_enum IS NULL OR v.status = $3)
              AND ($4::UUID IS NULL OR v.model_id = $4)
              AND ($5::UUID IS NULL OR v.fuel_type_id = $5)
              AND ($6::UUID IS NULL OR v.department_id = $6)
            ORDER BY v.updated_at DESC
            LIMIT $7 OFFSET $8
            "#
        )
        .bind(include_deleted)
        .bind(&search_pattern)
        .bind(&status)
        .bind(model_id)
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
            injection_pump: r.get("injection_pump"),
            gearbox: r.get("gearbox"),
            differential: r.get("differential"),
            model_id: r.get("model_id"),
            model_name: r.get("model_name"),
            make_name: r.get("make_name"),
            category_name: r.get("category_name"),
            color_id: r.get("color_id"),
            color_name: r.get("color_name"),
            fuel_type_id: r.get("fuel_type_id"),
            fuel_type_name: r.get("fuel_type_name"),
            transmission_type_id: r.get("transmission_type_id"),
            transmission_type_name: r.get("transmission_type_name"),
            manufacture_year: r.get("manufacture_year"),
            model_year: r.get("model_year"),
            fleet_code: r.get("fleet_code"),
            cost_sharing: r.get("cost_sharing"),
            initial_mileage: r.get("initial_mileage"),
            fuel_tank_capacity: r.get("fuel_tank_capacity"),
            acquisition_type: r.get("acquisition_type"),
            acquisition_date: r.get("acquisition_date"),
            purchase_value: r.get("purchase_value"),
            patrimony_number: r.get("patrimony_number"),
            department_id: r.get("department_id"),
            status: r.get("status"),
            notes: r.get("notes"),
            last_odometer_km: r.get("last_odometer_km"),
            last_odometer_date: r.get("last_odometer_date"),
            last_fueling_id: r.get("last_fueling_id"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        }).collect();

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM vehicles v
            JOIN vehicle_models m ON v.model_id = m.id
            JOIN vehicle_makes mk ON m.make_id = mk.id
            WHERE ($1::BOOLEAN OR v.is_deleted = false)
              AND ($2::TEXT IS NULL OR v.license_plate ILIKE $2 OR v.chassis_number ILIKE $2 OR v.renavam ILIKE $2 OR mk.name ILIKE $2 OR m.name ILIKE $2)
              AND ($3::vehicle_status_enum IS NULL OR v.status = $3)
              AND ($4::UUID IS NULL OR v.model_id = $4)
              AND ($5::UUID IS NULL OR v.fuel_type_id = $5)
              AND ($6::UUID IS NULL OR v.department_id = $6)
            "#
        )
        .bind(include_deleted)
        .bind(&search_pattern)
        .bind(&status)
        .bind(model_id)
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
            SELECT v.* FROM vehicles v
            JOIN vehicle_models m ON v.model_id = m.id
            JOIN vehicle_makes mk ON m.make_id = mk.id
            WHERE v.is_deleted = false
              AND (v.license_plate ILIKE $1 OR v.chassis_number ILIKE $1 OR v.renavam ILIKE $1 OR v.patrimony_number ILIKE $1 OR mk.name ILIKE $1 OR m.name ILIKE $1)
            ORDER BY v.license_plate
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
