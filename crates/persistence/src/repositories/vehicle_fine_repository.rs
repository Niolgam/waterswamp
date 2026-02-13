use async_trait::async_trait;
use chrono::{DateTime, Utc};
use domain::{
    errors::RepositoryError,
    models::vehicle_fine::*,
    ports::vehicle_fine::*,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::db_utils::map_db_error;

// ============================
// Vehicle Fine Type Repository
// ============================

pub struct VehicleFineTypeRepository {
    pool: PgPool,
}

impl VehicleFineTypeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleFineTypeRepositoryPort for VehicleFineTypeRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleFineTypeDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleFineTypeDto>("SELECT * FROM vehicle_fine_types WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn exists_by_code(&self, code: &str) -> Result<bool, RepositoryError> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM vehicle_fine_types WHERE code = $1)"
        )
        .bind(code)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(result)
    }

    async fn exists_by_code_excluding(&self, code: &str, exclude_id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM vehicle_fine_types WHERE code = $1 AND id != $2)"
        )
        .bind(code)
        .bind(exclude_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(result)
    }

    async fn create(
        &self,
        code: &str,
        description: &str,
        severity: &FineSeverity,
        points: i32,
        fine_amount: Decimal,
        created_by: Option<Uuid>,
    ) -> Result<VehicleFineTypeDto, RepositoryError> {
        sqlx::query_as::<_, VehicleFineTypeDto>(
            r#"INSERT INTO vehicle_fine_types (code, description, severity, points, fine_amount, created_by, updated_by)
               VALUES ($1, $2, $3, $4, $5, $6, $6)
               RETURNING *"#,
        )
        .bind(code)
        .bind(description)
        .bind(severity)
        .bind(points)
        .bind(fine_amount)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        code: Option<&str>,
        description: Option<&str>,
        severity: Option<&FineSeverity>,
        points: Option<i32>,
        fine_amount: Option<Decimal>,
        is_active: Option<bool>,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleFineTypeDto, RepositoryError> {
        sqlx::query_as::<_, VehicleFineTypeDto>(
            r#"UPDATE vehicle_fine_types SET
                code = COALESCE($2, code),
                description = COALESCE($3, description),
                severity = COALESCE($4, severity),
                points = COALESCE($5, points),
                fine_amount = COALESCE($6, fine_amount),
                is_active = COALESCE($7, is_active),
                updated_by = COALESCE($8, updated_by)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(code)
        .bind(description)
        .bind(severity)
        .bind(points)
        .bind(fine_amount)
        .bind(is_active)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, RepositoryError> {
        let result = sqlx::query("DELETE FROM vehicle_fine_types WHERE id = $1")
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
        severity: Option<FineSeverity>,
        is_active: Option<bool>,
    ) -> Result<(Vec<VehicleFineTypeDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let items = sqlx::query_as::<_, VehicleFineTypeDto>(
            r#"SELECT * FROM vehicle_fine_types
               WHERE ($1::TEXT IS NULL OR description ILIKE $1 OR code ILIKE $1)
                 AND ($2::fine_severity_enum IS NULL OR severity = $2)
                 AND ($3::BOOL IS NULL OR is_active = $3)
               ORDER BY code ASC
               LIMIT $4 OFFSET $5"#,
        )
        .bind(&search_pattern)
        .bind(&severity)
        .bind(is_active)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM vehicle_fine_types
               WHERE ($1::TEXT IS NULL OR description ILIKE $1 OR code ILIKE $1)
                 AND ($2::fine_severity_enum IS NULL OR severity = $2)
                 AND ($3::BOOL IS NULL OR is_active = $3)"#,
        )
        .bind(&search_pattern)
        .bind(&severity)
        .bind(is_active)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((items, total))
    }
}

// ============================
// Vehicle Fine Repository
// ============================

pub struct VehicleFineRepository {
    pool: PgPool,
}

impl VehicleFineRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleFineRepositoryPort for VehicleFineRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<VehicleFineDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleFineDto>("SELECT * FROM vehicle_fines WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_with_details_by_id(&self, id: Uuid) -> Result<Option<VehicleFineWithDetailsDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleFineWithDetailsDto>(
            r#"SELECT vf.id, vf.vehicle_id, v.license_plate AS vehicle_license_plate,
                      vf.fine_type_id, ft.code AS fine_type_code, ft.description AS fine_type_description,
                      ft.severity AS fine_type_severity, ft.points AS fine_type_points,
                      vf.supplier_id, s.legal_name AS supplier_name,
                      vf.driver_id, d.full_name AS driver_name,
                      vf.auto_number, vf.fine_date, vf.notification_date, vf.due_date,
                      vf.location, vf.sei_process_number,
                      vf.fine_amount, vf.discount_amount, vf.paid_amount, vf.payment_date,
                      vf.status, vf.notes, vf.is_deleted,
                      vf.created_at, vf.updated_at
               FROM vehicle_fines vf
               LEFT JOIN vehicles v ON v.id = vf.vehicle_id
               LEFT JOIN vehicle_fine_types ft ON ft.id = vf.fine_type_id
               LEFT JOIN suppliers s ON s.id = vf.supplier_id
               LEFT JOIN drivers d ON d.id = vf.driver_id
               WHERE vf.id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn create(
        &self,
        vehicle_id: Uuid,
        fine_type_id: Uuid,
        supplier_id: Uuid,
        driver_id: Option<Uuid>,
        auto_number: Option<&str>,
        fine_date: DateTime<Utc>,
        notification_date: Option<DateTime<Utc>>,
        due_date: DateTime<Utc>,
        location: Option<&str>,
        sei_process_number: Option<&str>,
        fine_amount: Decimal,
        discount_amount: Option<Decimal>,
        notes: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<VehicleFineDto, RepositoryError> {
        sqlx::query_as::<_, VehicleFineDto>(
            r#"INSERT INTO vehicle_fines (vehicle_id, fine_type_id, supplier_id, driver_id,
                                          auto_number, fine_date, notification_date, due_date,
                                          location, sei_process_number,
                                          fine_amount, discount_amount, notes,
                                          created_by, updated_by)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $14)
               RETURNING *"#,
        )
        .bind(vehicle_id)
        .bind(fine_type_id)
        .bind(supplier_id)
        .bind(driver_id)
        .bind(auto_number)
        .bind(fine_date)
        .bind(notification_date)
        .bind(due_date)
        .bind(location)
        .bind(sei_process_number)
        .bind(fine_amount)
        .bind(discount_amount)
        .bind(notes)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update(
        &self,
        id: Uuid,
        vehicle_id: Option<Uuid>,
        fine_type_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        auto_number: Option<&str>,
        fine_date: Option<DateTime<Utc>>,
        notification_date: Option<DateTime<Utc>>,
        due_date: Option<DateTime<Utc>>,
        location: Option<&str>,
        sei_process_number: Option<&str>,
        fine_amount: Option<Decimal>,
        discount_amount: Option<Decimal>,
        paid_amount: Option<Decimal>,
        payment_date: Option<DateTime<Utc>>,
        notes: Option<&str>,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleFineDto, RepositoryError> {
        sqlx::query_as::<_, VehicleFineDto>(
            r#"UPDATE vehicle_fines SET
                vehicle_id = COALESCE($2, vehicle_id),
                fine_type_id = COALESCE($3, fine_type_id),
                supplier_id = COALESCE($4, supplier_id),
                driver_id = COALESCE($5, driver_id),
                auto_number = COALESCE($6, auto_number),
                fine_date = COALESCE($7, fine_date),
                notification_date = COALESCE($8, notification_date),
                due_date = COALESCE($9, due_date),
                location = COALESCE($10, location),
                sei_process_number = COALESCE($11, sei_process_number),
                fine_amount = COALESCE($12, fine_amount),
                discount_amount = COALESCE($13, discount_amount),
                paid_amount = COALESCE($14, paid_amount),
                payment_date = COALESCE($15, payment_date),
                notes = COALESCE($16, notes),
                updated_by = COALESCE($17, updated_by)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(vehicle_id)
        .bind(fine_type_id)
        .bind(supplier_id)
        .bind(driver_id)
        .bind(auto_number)
        .bind(fine_date)
        .bind(notification_date)
        .bind(due_date)
        .bind(location)
        .bind(sei_process_number)
        .bind(fine_amount)
        .bind(discount_amount)
        .bind(paid_amount)
        .bind(payment_date)
        .bind(notes)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn update_status(
        &self,
        id: Uuid,
        status: &FineStatus,
        updated_by: Option<Uuid>,
    ) -> Result<VehicleFineDto, RepositoryError> {
        sqlx::query_as::<_, VehicleFineDto>(
            r#"UPDATE vehicle_fines SET status = $2, updated_by = COALESCE($3, updated_by)
               WHERE id = $1
               RETURNING *"#,
        )
        .bind(id)
        .bind(status)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn soft_delete(&self, id: Uuid, deleted_by: Option<Uuid>) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            r#"UPDATE vehicle_fines SET is_deleted = true, deleted_at = NOW(), deleted_by = $2
               WHERE id = $1 AND is_deleted = false"#,
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
            r#"UPDATE vehicle_fines SET is_deleted = false, deleted_at = NULL, deleted_by = NULL
               WHERE id = $1 AND is_deleted = true"#,
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
        vehicle_id: Option<Uuid>,
        fine_type_id: Option<Uuid>,
        supplier_id: Option<Uuid>,
        driver_id: Option<Uuid>,
        status: Option<FineStatus>,
        search: Option<String>,
        include_deleted: bool,
    ) -> Result<(Vec<VehicleFineWithDetailsDto>, i64), RepositoryError> {
        let search_pattern = search.map(|s| format!("%{}%", s));

        let items = sqlx::query_as::<_, VehicleFineWithDetailsDto>(
            r#"SELECT vf.id, vf.vehicle_id, v.license_plate AS vehicle_license_plate,
                      vf.fine_type_id, ft.code AS fine_type_code, ft.description AS fine_type_description,
                      ft.severity AS fine_type_severity, ft.points AS fine_type_points,
                      vf.supplier_id, s.legal_name AS supplier_name,
                      vf.driver_id, d.full_name AS driver_name,
                      vf.auto_number, vf.fine_date, vf.notification_date, vf.due_date,
                      vf.location, vf.sei_process_number,
                      vf.fine_amount, vf.discount_amount, vf.paid_amount, vf.payment_date,
                      vf.status, vf.notes, vf.is_deleted,
                      vf.created_at, vf.updated_at
               FROM vehicle_fines vf
               LEFT JOIN vehicles v ON v.id = vf.vehicle_id
               LEFT JOIN vehicle_fine_types ft ON ft.id = vf.fine_type_id
               LEFT JOIN suppliers s ON s.id = vf.supplier_id
               LEFT JOIN drivers d ON d.id = vf.driver_id
               WHERE (vf.is_deleted = false OR $1::BOOL)
                 AND ($2::UUID IS NULL OR vf.vehicle_id = $2)
                 AND ($3::UUID IS NULL OR vf.fine_type_id = $3)
                 AND ($4::UUID IS NULL OR vf.supplier_id = $4)
                 AND ($5::UUID IS NULL OR vf.driver_id = $5)
                 AND ($6::fine_status_enum IS NULL OR vf.status = $6)
                 AND ($7::TEXT IS NULL OR vf.auto_number ILIKE $7 OR vf.sei_process_number ILIKE $7 OR v.license_plate ILIKE $7)
               ORDER BY vf.due_date DESC
               LIMIT $8 OFFSET $9"#,
        )
        .bind(include_deleted)
        .bind(vehicle_id)
        .bind(fine_type_id)
        .bind(supplier_id)
        .bind(driver_id)
        .bind(&status)
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM vehicle_fines vf
               LEFT JOIN vehicles v ON v.id = vf.vehicle_id
               WHERE (vf.is_deleted = false OR $1::BOOL)
                 AND ($2::UUID IS NULL OR vf.vehicle_id = $2)
                 AND ($3::UUID IS NULL OR vf.fine_type_id = $3)
                 AND ($4::UUID IS NULL OR vf.supplier_id = $4)
                 AND ($5::UUID IS NULL OR vf.driver_id = $5)
                 AND ($6::fine_status_enum IS NULL OR vf.status = $6)
                 AND ($7::TEXT IS NULL OR vf.auto_number ILIKE $7 OR vf.sei_process_number ILIKE $7 OR v.license_plate ILIKE $7)"#,
        )
        .bind(include_deleted)
        .bind(vehicle_id)
        .bind(fine_type_id)
        .bind(supplier_id)
        .bind(driver_id)
        .bind(&status)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((items, total))
    }
}

// ============================
// Vehicle Fine Status History Repository
// ============================

pub struct VehicleFineStatusHistoryRepository {
    pool: PgPool,
}

impl VehicleFineStatusHistoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VehicleFineStatusHistoryRepositoryPort for VehicleFineStatusHistoryRepository {
    async fn create(
        &self,
        vehicle_fine_id: Uuid,
        old_status: Option<FineStatus>,
        new_status: FineStatus,
        reason: Option<&str>,
        changed_by: Option<Uuid>,
    ) -> Result<VehicleFineStatusHistoryDto, RepositoryError> {
        sqlx::query_as::<_, VehicleFineStatusHistoryDto>(
            r#"INSERT INTO vehicle_fine_status_history (vehicle_fine_id, old_status, new_status, reason, changed_by)
               VALUES ($1, $2, $3, $4, $5)
               RETURNING *"#,
        )
        .bind(vehicle_fine_id)
        .bind(old_status)
        .bind(new_status)
        .bind(reason)
        .bind(changed_by)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list_by_fine(&self, vehicle_fine_id: Uuid) -> Result<Vec<VehicleFineStatusHistoryDto>, RepositoryError> {
        sqlx::query_as::<_, VehicleFineStatusHistoryDto>(
            "SELECT * FROM vehicle_fine_status_history WHERE vehicle_fine_id = $1 ORDER BY created_at DESC"
        )
        .bind(vehicle_fine_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
