use async_trait::async_trait;
use domain::{
    errors::RepositoryError,
    models::requisition::*,
    ports::requisition::*,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db_utils::map_db_error;

// ============================================================================
// Requisition Repository
// ============================================================================

pub struct RequisitionRepository {
    pool: PgPool,
}

impl RequisitionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RequisitionRepositoryPort for RequisitionRepository {
    async fn set_audit_context(
        &self,
        user_id: Uuid,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
    ) -> Result<(), RepositoryError> {
        sqlx::query("SELECT fn_set_audit_context($1, $2, $3)")
            .bind(user_id)
            .bind(ip_address)
            .bind(user_agent)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<RequisitionDto>, RepositoryError> {
        sqlx::query_as::<_, RequisitionDto>("SELECT * FROM requisitions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_by_number(&self, number: &str) -> Result<Option<RequisitionDto>, RepositoryError> {
        sqlx::query_as::<_, RequisitionDto>(
            "SELECT * FROM requisitions WHERE requisition_number = $1",
        )
        .bind(number)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn approve(
        &self,
        id: Uuid,
        approved_by: Uuid,
        notes: Option<&str>,
    ) -> Result<RequisitionDto, RepositoryError> {
        sqlx::query_as::<_, RequisitionDto>(
            r#"
            UPDATE requisitions SET
                status = 'APPROVED',
                approved_by = $2,
                approved_at = NOW(),
                internal_notes = COALESCE($3, internal_notes),
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(approved_by)
        .bind(notes)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn reject(
        &self,
        id: Uuid,
        rejected_by: Uuid,
        reason: &str,
    ) -> Result<RequisitionDto, RepositoryError> {
        sqlx::query_as::<_, RequisitionDto>(
            r#"
            UPDATE requisitions SET
                status = 'REJECTED',
                approved_by = $2,
                approved_at = NOW(),
                rejection_reason = $3,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(rejected_by)
        .bind(reason)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn cancel(
        &self,
        id: Uuid,
        reason: &str,
        user_id: Uuid,
    ) -> Result<serde_json::Value, RepositoryError> {
        let result: serde_json::Value =
            sqlx::query_scalar("SELECT fn_cancel_requisition($1, $2, $3)")
                .bind(id)
                .bind(reason)
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?;
        Ok(result)
    }

    async fn rollback(
        &self,
        id: Uuid,
        history_id: Uuid,
        reason: &str,
        user_id: Uuid,
    ) -> Result<serde_json::Value, RepositoryError> {
        let result: serde_json::Value =
            sqlx::query_scalar("SELECT fn_rollback_requisition($1, $2, $3, $4)")
                .bind(id)
                .bind(history_id)
                .bind(reason)
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_error)?;
        Ok(result)
    }

    async fn get_history(
        &self,
        id: Uuid,
        limit: i64,
    ) -> Result<Vec<RequisitionHistoryEntry>, RepositoryError> {
        sqlx::query_as::<_, RequisitionHistoryEntry>(
            r#"
            SELECT
                history_id,
                operation,
                status_before,
                status_after,
                changed_fields,
                performed_at,
                performed_by,
                performed_by_name,
                reason,
                summary
            FROM vw_requisition_audit_trail
            WHERE requisition_id = $1
            ORDER BY performed_at DESC
            LIMIT $2
            "#,
        )
        .bind(id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn get_rollback_points(
        &self,
        id: Uuid,
        limit: i32,
    ) -> Result<Vec<RollbackPoint>, RepositoryError> {
        sqlx::query_as::<_, RollbackPoint>(
            r#"
            SELECT
                rh.id AS history_id,
                rh.operation::TEXT AS operation,
                rh.status_after,
                rh.performed_at,
                rh.performed_by_name,
                rh.changed_fields,
                CASE
                    WHEN rh.is_rollback_point = FALSE THEN FALSE
                    WHEN rh.operation IN ('DELETE', 'SOFT_DELETE') THEN FALSE
                    WHEN EXISTS (
                        SELECT 1 FROM stock_movements sm
                        WHERE sm.requisition_id = $1
                          AND sm.movement_date > rh.performed_at
                    ) THEN FALSE
                    ELSE TRUE
                END AS can_rollback,
                CASE
                    WHEN rh.is_rollback_point = FALSE THEN 'Estado marcado como não-reversível'
                    WHEN rh.operation IN ('DELETE', 'SOFT_DELETE') THEN 'Não é possível reverter para estado de exclusão'
                    WHEN EXISTS (
                        SELECT 1 FROM stock_movements sm
                        WHERE sm.requisition_id = $1
                          AND sm.movement_date > rh.performed_at
                    ) THEN 'Existem movimentações de estoque posteriores a este ponto'
                    ELSE NULL
                END AS rollback_blocked_reason
            FROM requisition_history rh
            WHERE rh.requisition_id = $1
              AND rh.data_after IS NOT NULL
            ORDER BY rh.performed_at DESC
            LIMIT $2
            "#,
        )
        .bind(id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn list(
        &self,
        limit: i64,
        offset: i64,
        status: Option<RequisitionStatus>,
        requester_id: Option<Uuid>,
        warehouse_id: Option<Uuid>,
    ) -> Result<(Vec<RequisitionDto>, i64), RepositoryError> {
        let requisitions = sqlx::query_as::<_, RequisitionDto>(
            r#"
            SELECT * FROM requisitions
            WHERE ($1::requisition_status_enum IS NULL OR status = $1)
              AND ($2::UUID IS NULL OR requester_id = $2)
              AND ($3::UUID IS NULL OR warehouse_id = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(status)
        .bind(requester_id)
        .bind(warehouse_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;

        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM requisitions
            WHERE ($1::requisition_status_enum IS NULL OR status = $1)
              AND ($2::UUID IS NULL OR requester_id = $2)
              AND ($3::UUID IS NULL OR warehouse_id = $3)
            "#,
        )
        .bind(status)
        .bind(requester_id)
        .bind(warehouse_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok((requisitions, total))
    }
}

// ============================================================================
// Requisition Item Repository
// ============================================================================

pub struct RequisitionItemRepository {
    pool: PgPool,
}

impl RequisitionItemRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RequisitionItemRepositoryPort for RequisitionItemRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<RequisitionItemDto>, RepositoryError> {
        sqlx::query_as::<_, RequisitionItemDto>("SELECT * FROM requisition_items WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_error)
    }

    async fn find_by_requisition_id(
        &self,
        requisition_id: Uuid,
    ) -> Result<Vec<RequisitionItemDto>, RepositoryError> {
        sqlx::query_as::<_, RequisitionItemDto>(
            r#"
            SELECT * FROM requisition_items
            WHERE requisition_id = $1
              AND deleted_at IS NULL
            ORDER BY created_at
            "#,
        )
        .bind(requisition_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)
    }

    async fn soft_delete(
        &self,
        id: Uuid,
        deleted_by: Uuid,
        reason: &str,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            UPDATE requisition_items SET
                deleted_at = NOW(),
                deleted_by = $2,
                deletion_reason = $3,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(deleted_by)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(())
    }

    async fn restore(&self, id: Uuid) -> Result<RequisitionItemDto, RepositoryError> {
        sqlx::query_as::<_, RequisitionItemDto>(
            r#"
            UPDATE requisition_items SET
                deleted_at = NULL,
                deleted_by = NULL,
                deletion_reason = NULL,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)
    }
}
