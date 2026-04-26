use crate::errors::ServiceError;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{postgres::Postgres, PgPool, Transaction};
use uuid::Uuid;

// ============================================================================
// Tipos públicos
// ============================================================================

/// Tipo de movimentação — espelha `stock_movement_type_enum` no PostgreSQL
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StockMovementType {
    Entry,
    Exit,
    Loss,
    Return,
    TransferIn,
    TransferOut,
    AdjustmentAdd,
    AdjustmentSub,
    DonationIn,
    DonationOut,
}

impl StockMovementType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StockMovementType::Entry => "ENTRY",
            StockMovementType::Exit => "EXIT",
            StockMovementType::Loss => "LOSS",
            StockMovementType::Return => "RETURN",
            StockMovementType::TransferIn => "TRANSFER_IN",
            StockMovementType::TransferOut => "TRANSFER_OUT",
            StockMovementType::AdjustmentAdd => "ADJUSTMENT_ADD",
            StockMovementType::AdjustmentSub => "ADJUSTMENT_SUB",
            StockMovementType::DonationIn => "DONATION_IN",
            StockMovementType::DonationOut => "DONATION_OUT",
        }
    }

    pub fn is_entry(&self) -> bool {
        matches!(
            self,
            StockMovementType::Entry
                | StockMovementType::Return
                | StockMovementType::TransferIn
                | StockMovementType::AdjustmentAdd
                | StockMovementType::DonationIn
        )
    }
}

/// Parâmetros para processar uma movimentação de estoque
#[derive(Debug, Clone)]
pub struct ProcessMovementInput {
    pub warehouse_id: Uuid,
    pub catalog_item_id: Uuid,
    pub movement_type: StockMovementType,
    pub unit_raw_id: Uuid,
    pub unit_conversion_id: Option<Uuid>,
    pub quantity_raw: Decimal,
    pub conversion_factor: Decimal,
    pub quantity_base: Decimal,
    pub unit_price_base: Decimal,
    pub invoice_id: Option<Uuid>,
    pub invoice_item_id: Option<Uuid>,
    pub requisition_id: Option<Uuid>,
    pub requisition_item_id: Option<Uuid>,
    pub related_warehouse_id: Option<Uuid>,
    pub document_number: Option<String>,
    pub notes: Option<String>,
    pub user_id: Uuid,
    pub batch_number: Option<String>,
    pub expiration_date: Option<NaiveDate>,
    pub divergence_justification: Option<String>,
}

// ============================================================================
// Serviço
// ============================================================================

pub struct StockMovementService {
    #[allow(dead_code)]
    pool: PgPool,
}

impl StockMovementService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Processa uma movimentação de estoque dentro de uma transação existente.
    ///
    /// Replica toda a lógica de `fn_process_stock_movement`:
    ///  1. Verifica se o item é STOCKABLE
    ///  2. Captura saldo atual com lock pessimista (SELECT … FOR UPDATE)
    ///  3. Valida bloqueio administrativo
    ///  4. Verifica divergência de preço (busca threshold em system_settings)
    ///  5. Calcula novo saldo e custo médio ponderado
    ///  6. Insere em `stock_movements` com snapshots completos
    ///  7. Faz UPSERT em `warehouse_stocks`
    pub async fn process_movement(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        mut input: ProcessMovementInput,
    ) -> Result<(), ServiceError> {
        // ── 1. Verificar se o item é STOCKABLE ────────────────────────────────
        let is_stockable: bool = sqlx::query_scalar(
            r#"SELECT (COALESCE(pdm.material_classification, 'STOCKABLE')::TEXT = 'STOCKABLE')
               FROM catmat_items ci
               LEFT JOIN catmat_pdms pdm ON pdm.id = ci.pdm_id
               WHERE ci.id = $1"#,
        )
        .bind(input.catalog_item_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?
        .unwrap_or(false);

        if !is_stockable {
            // Item não estocável (ex: serviço) — registra movimento sem afetar saldo
            sqlx::query(
                r#"INSERT INTO stock_movements (
                    warehouse_id, catalog_item_id, movement_type,
                    unit_raw_id, unit_conversion_id,
                    quantity_raw, conversion_factor, quantity_base,
                    unit_price_base, total_value,
                    balance_before, balance_after, average_before, average_after,
                    invoice_id, invoice_item_id, requisition_id, requisition_item_id,
                    related_warehouse_id, document_number, notes, user_id, batch_number,
                    expiration_date, divergence_justification
                ) VALUES (
                    $1,$2,$3::stock_movement_type_enum,$4,$5,$6,$7,$8,$9,$10,
                    0,0,0,0,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21
                )"#,
            )
            .bind(input.warehouse_id)
            .bind(input.catalog_item_id)
            .bind(input.movement_type.as_str())
            .bind(input.unit_raw_id)
            .bind(input.unit_conversion_id)
            .bind(input.quantity_raw)
            .bind(input.conversion_factor)
            .bind(input.quantity_base)
            .bind(input.unit_price_base)
            .bind(input.quantity_base * input.unit_price_base)
            .bind(input.invoice_id)
            .bind(input.invoice_item_id)
            .bind(input.requisition_id)
            .bind(input.requisition_item_id)
            .bind(input.related_warehouse_id)
            .bind(input.document_number.as_deref())
            .bind(input.notes.as_deref())
            .bind(input.user_id)
            .bind(input.batch_number.as_deref())
            .bind(input.expiration_date)
            .bind(input.divergence_justification.as_deref())
            .execute(&mut **tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
            return Ok(());
        }

        // ── 1.5. Validar quantidade discreta×contínua (RF-004) ───────────────
        // Consulta allows_fractions da unidade informada na movimentação.
        let allows_fractions: Option<bool> = sqlx::query_scalar(
            "SELECT allows_fractions FROM units_of_measure WHERE id = $1",
        )
        .bind(input.unit_raw_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        if let Some(false) = allows_fractions {
            if input.quantity_base.fract() != Decimal::ZERO {
                return Err(ServiceError::BadRequest(format!(
                    "Item indivisível (unidade discreta): quantidade fracionada não é permitida. \
                     Informado: {} (RF-004).",
                    input.quantity_base
                )));
            }
        }

        // ── 2. Capturar saldo atual com lock pessimista ───────────────────────
        let stock_row: Option<(Decimal, Decimal, bool, Option<String>)> = sqlx::query_as(
            r#"SELECT quantity, average_unit_value, is_blocked, block_reason
               FROM warehouse_stocks
               WHERE warehouse_id = $1 AND catalog_item_id = $2
               FOR UPDATE"#,
        )
        .bind(input.warehouse_id)
        .bind(input.catalog_item_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        let (curr_qty, curr_avg, is_blocked, block_reason) =
            stock_row.unwrap_or((Decimal::ZERO, Decimal::ZERO, false, None));

        // ── 3. Verificar bloqueio administrativo ─────────────────────────────
        if is_blocked && !input.movement_type.is_entry() {
            return Err(ServiceError::BadRequest(format!(
                "Operação negada: O item está BLOQUEADO neste almoxarifado. Motivo: {}",
                block_reason.as_deref().unwrap_or("não informado")
            )));
        }

        // ── 4. Verificar divergência de preço (entradas avulsas) ──────────────
        let mut requires_review = false;
        if matches!(
            input.movement_type,
            StockMovementType::AdjustmentAdd | StockMovementType::DonationIn
        ) && curr_avg > Decimal::ZERO
            && input.unit_price_base > Decimal::ZERO
        {
            let threshold: Decimal = sqlx::query_scalar(
                "SELECT (value::TEXT)::DECIMAL FROM system_settings WHERE key = 'inventory.price_divergence_threshold'",
            )
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?
            .unwrap_or(Decimal::new(20, 2)); // 0.20 = 20%

            let diff = (input.unit_price_base - curr_avg).abs() / curr_avg;
            if diff > threshold {
                requires_review = true;
                if input
                    .divergence_justification
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .is_empty()
                {
                    return Err(ServiceError::BadRequest(format!(
                        "Variação de preço > {}% em relação ao custo médio. Informe uma justificativa.",
                        threshold * Decimal::new(100, 0)
                    )));
                }
            }
        }

        // ── 5. Calcular novos saldos ──────────────────────────────────────────
        let (new_qty, new_avg, final_price, final_total) = if input.movement_type.is_entry() {
            let qty_after = curr_qty + input.quantity_base;
            let avg_after = if qty_after > Decimal::ZERO && input.unit_price_base > Decimal::ZERO {
                (curr_qty * curr_avg + input.quantity_base * input.unit_price_base) / qty_after
            } else if qty_after > Decimal::ZERO {
                curr_avg
            } else {
                input.unit_price_base
            };
            let total = input.quantity_base * input.unit_price_base;
            (qty_after, avg_after, input.unit_price_base, total)
        } else {
            // Saída
            if input.quantity_base > curr_qty {
                return Err(ServiceError::BadRequest(format!(
                    "Saldo insuficiente. Disponível: {}, Solicitado: {}",
                    curr_qty, input.quantity_base
                )));
            }
            let qty_after = curr_qty - input.quantity_base;
            // Saída usa custo médio atual (contabilidade)
            let exit_price = curr_avg;
            let exit_total = input.quantity_base * curr_avg;
            (qty_after, curr_avg, exit_price, exit_total)
        };

        // Atualiza preço para saídas (custo médio)
        input.unit_price_base = final_price;

        // ── 6. Inserir em stock_movements ─────────────────────────────────────
        sqlx::query(
            r#"INSERT INTO stock_movements (
                warehouse_id, catalog_item_id, movement_type,
                unit_raw_id, unit_conversion_id,
                quantity_raw, conversion_factor, quantity_base,
                unit_price_base, total_value,
                balance_before, balance_after, average_before, average_after,
                invoice_id, invoice_item_id, requisition_id, requisition_item_id,
                related_warehouse_id, document_number, notes, user_id, batch_number,
                expiration_date, divergence_justification, requires_review
            ) VALUES (
                $1,$2,$3::stock_movement_type_enum,$4,$5,
                $6,$7,$8,$9,$10,
                $11,$12,$13,$14,
                $15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26
            )"#,
        )
        .bind(input.warehouse_id)
        .bind(input.catalog_item_id)
        .bind(input.movement_type.as_str())
        .bind(input.unit_raw_id)
        .bind(input.unit_conversion_id)
        .bind(input.quantity_raw)
        .bind(input.conversion_factor)
        .bind(input.quantity_base)
        .bind(final_price)
        .bind(final_total)
        .bind(curr_qty) // balance_before
        .bind(new_qty) // balance_after
        .bind(curr_avg) // average_before
        .bind(new_avg) // average_after
        .bind(input.invoice_id)
        .bind(input.invoice_item_id)
        .bind(input.requisition_id)
        .bind(input.requisition_item_id)
        .bind(input.related_warehouse_id)
        .bind(input.document_number.as_deref())
        .bind(input.notes.as_deref())
        .bind(input.user_id)
        .bind(input.batch_number.as_deref())
        .bind(input.expiration_date)
        .bind(input.divergence_justification.as_deref())
        .bind(requires_review)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // ── 7. UPSERT em warehouse_stocks ─────────────────────────────────────
        let is_entry_movement = input.movement_type.is_entry();
        sqlx::query(
            r#"INSERT INTO warehouse_stocks (
                warehouse_id, catalog_item_id, quantity, average_unit_value,
                last_entry_at, last_exit_at, updated_at
            ) VALUES (
                $1, $2, $3, $4,
                CASE WHEN $5 THEN NOW() ELSE NULL END,
                CASE WHEN NOT $5 THEN NOW() ELSE NULL END,
                NOW()
            )
            ON CONFLICT (warehouse_id, catalog_item_id)
            DO UPDATE SET
                quantity = EXCLUDED.quantity,
                average_unit_value = EXCLUDED.average_unit_value,
                last_entry_at = COALESCE(EXCLUDED.last_entry_at, warehouse_stocks.last_entry_at),
                last_exit_at = COALESCE(EXCLUDED.last_exit_at, warehouse_stocks.last_exit_at),
                updated_at = NOW()"#,
        )
        .bind(input.warehouse_id)
        .bind(input.catalog_item_id)
        .bind(new_qty)
        .bind(new_avg)
        .bind(is_entry_movement)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        // ── 8. UPSERT em warehouse_batch_stocks (RF-021 FEFO) ─────────────────
        // Only when a batch_number is provided — maintains per-batch inventory.
        if let Some(ref bn) = input.batch_number {
            let delta = if is_entry_movement {
                input.quantity_base
            } else {
                -input.quantity_base
            };
            let cost = if is_entry_movement && input.unit_price_base > Decimal::ZERO {
                input.unit_price_base
            } else {
                curr_avg
            };
            sqlx::query(
                r#"INSERT INTO warehouse_batch_stocks
                    (warehouse_id, catalog_item_id, batch_number, expiration_date, quantity, unit_cost)
                   VALUES ($1, $2, $3, $4, GREATEST(0, $5), $6)
                   ON CONFLICT (warehouse_id, catalog_item_id, batch_number)
                   DO UPDATE SET
                    quantity = GREATEST(0, warehouse_batch_stocks.quantity + $5),
                    unit_cost = CASE
                        WHEN $5 > 0 AND $6 > 0 THEN
                            (warehouse_batch_stocks.quantity * warehouse_batch_stocks.unit_cost
                             + $5 * $6)
                            / NULLIF(warehouse_batch_stocks.quantity + $5, 0)
                        ELSE warehouse_batch_stocks.unit_cost
                    END,
                    expiration_date = COALESCE(warehouse_batch_stocks.expiration_date, $4),
                    updated_at = NOW()"#,
            )
            .bind(input.warehouse_id)
            .bind(input.catalog_item_id)
            .bind(bn)
            .bind(input.expiration_date)
            .bind(delta)
            .bind(cost)
            .execute(&mut **tx)
            .await
            .map_err(|e| ServiceError::Internal(e.to_string()))?;
        }

        Ok(())
    }

    /// Processa todos os itens de uma NF como movimentações de ENTRY.
    /// Chamado por InvoiceService::post_invoice() dentro de uma transação.
    pub async fn process_invoice_entry(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        invoice_id: Uuid,
        warehouse_id: Uuid,
        invoice_number: &str,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        #[derive(sqlx::FromRow)]
        struct InvoiceItemRow {
            id: Uuid,
            catalog_item_id: Uuid,
            unit_raw_id: Uuid,
            unit_conversion_id: Option<Uuid>,
            quantity_raw: Decimal,
            conversion_factor: Decimal,
            quantity_base: Decimal,
            unit_value_base: Decimal,
            #[allow(dead_code)]
            total_value: Decimal,
            batch_number: Option<String>,
            expiration_date: Option<NaiveDate>,
        }

        let items = sqlx::query_as::<_, InvoiceItemRow>(
            "SELECT id, catalog_item_id, unit_raw_id, unit_conversion_id,
                    quantity_raw, conversion_factor, quantity_base, unit_value_base,
                    total_value, batch_number, expiration_date
             FROM invoice_items WHERE invoice_id = $1",
        )
        .bind(invoice_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        for item in items {
            self.process_movement(
                tx,
                ProcessMovementInput {
                    warehouse_id,
                    catalog_item_id: item.catalog_item_id,
                    movement_type: StockMovementType::Entry,
                    unit_raw_id: item.unit_raw_id,
                    unit_conversion_id: item.unit_conversion_id,
                    quantity_raw: item.quantity_raw,
                    conversion_factor: item.conversion_factor,
                    quantity_base: item.quantity_base,
                    unit_price_base: item.unit_value_base,
                    invoice_id: Some(invoice_id),
                    invoice_item_id: Some(item.id),
                    requisition_id: None,
                    requisition_item_id: None,
                    related_warehouse_id: None,
                    document_number: Some(invoice_number.to_string()),
                    notes: None,
                    user_id,
                    batch_number: item.batch_number,
                    expiration_date: item.expiration_date,
                    divergence_justification: None,
                },
            )
            .await?;
        }
        Ok(())
    }

    /// Reverte todas as entradas de uma NF já lançada (ADJUSTMENT_SUB).
    /// Chamado por InvoiceService::cancel_invoice() dentro de uma transação.
    pub async fn reverse_invoice_entry(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        invoice_id: Uuid,
        invoice_number: &str,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        #[derive(sqlx::FromRow)]
        struct EntryMovementRow {
            warehouse_id: Uuid,
            catalog_item_id: Uuid,
            unit_raw_id: Uuid,
            unit_conversion_id: Option<Uuid>,
            quantity_raw: Decimal,
            conversion_factor: Decimal,
            quantity_base: Decimal,
            unit_price_base: Decimal,
            batch_number: Option<String>,
            expiration_date: Option<NaiveDate>,
            invoice_item_id: Option<Uuid>,
        }

        let movements = sqlx::query_as::<_, EntryMovementRow>(
            r#"SELECT warehouse_id, catalog_item_id, unit_raw_id, unit_conversion_id,
                      quantity_raw, conversion_factor, quantity_base, unit_price_base,
                      batch_number, expiration_date, invoice_item_id
               FROM stock_movements
               WHERE invoice_id = $1 AND movement_type = 'ENTRY'"#,
        )
        .bind(invoice_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(|e| ServiceError::Internal(e.to_string()))?;

        for mv in movements {
            self.process_movement(
                tx,
                ProcessMovementInput {
                    warehouse_id: mv.warehouse_id,
                    catalog_item_id: mv.catalog_item_id,
                    movement_type: StockMovementType::AdjustmentSub,
                    unit_raw_id: mv.unit_raw_id,
                    unit_conversion_id: mv.unit_conversion_id,
                    quantity_raw: mv.quantity_raw,
                    conversion_factor: mv.conversion_factor,
                    quantity_base: mv.quantity_base,
                    unit_price_base: mv.unit_price_base,
                    invoice_id: Some(invoice_id),
                    invoice_item_id: mv.invoice_item_id,
                    requisition_id: None,
                    requisition_item_id: None,
                    related_warehouse_id: None,
                    document_number: Some(format!("ESTORNO NF {}", invoice_number)),
                    notes: Some(
                        "Estorno automático — NF revertida de POSTED para CANCELLED".to_string(),
                    ),
                    user_id,
                    batch_number: mv.batch_number,
                    expiration_date: mv.expiration_date,
                    divergence_justification: None,
                },
            )
            .await?;
        }
        Ok(())
    }

    /// Gera movimentações ADJUSTMENT_SUB para itens de ajuste (glosa).
    /// Chamado por InvoiceAdjustmentService dentro de uma transação.
    #[allow(clippy::too_many_arguments)]
    pub async fn process_adjustment_sub(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        warehouse_id: Uuid,
        catalog_item_id: Uuid,
        unit_raw_id: Uuid,
        unit_conversion_id: Option<Uuid>,
        quantity_base: Decimal,
        invoice_id: Uuid,
        invoice_item_id: Uuid,
        document_number: &str,
        notes: Option<&str>,
        user_id: Uuid,
    ) -> Result<(), ServiceError> {
        self.process_movement(
            tx,
            ProcessMovementInput {
                warehouse_id,
                catalog_item_id,
                movement_type: StockMovementType::AdjustmentSub,
                unit_raw_id,
                unit_conversion_id,
                quantity_raw: quantity_base,
                conversion_factor: Decimal::ONE,
                quantity_base,
                unit_price_base: Decimal::ZERO, // será sobrescrito com custo médio atual
                invoice_id: Some(invoice_id),
                invoice_item_id: Some(invoice_item_id),
                requisition_id: None,
                requisition_item_id: None,
                related_warehouse_id: None,
                document_number: Some(document_number.to_string()),
                notes: notes.map(str::to_owned),
                user_id,
                batch_number: None,
                expiration_date: None,
                divergence_justification: None,
            },
        )
        .await
    }
}
