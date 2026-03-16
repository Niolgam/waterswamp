# Invoice API — Frontend Consumption Guide

This document describes every endpoint, request shape, response shape, and status-machine rule for the invoice module. It targets frontend developers consuming the Waterswamp REST API.

---

## Base URL

```
/api/admin/invoices
```

All endpoints require a valid JWT bearer token with admin role:

```http
Authorization: Bearer <access_token>
```

---

## Table of Contents

1. [Status Machine Overview](#1-status-machine-overview)
2. [Common Types (DTOs)](#2-common-types-dtos)
3. [Endpoints Reference](#3-endpoints-reference)
   - [List Invoices](#31-list-invoices)
   - [Create Invoice](#32-create-invoice)
   - [Get Invoice](#33-get-invoice)
   - [Update Invoice](#34-update-invoice)
   - [Delete Invoice](#35-delete-invoice)
   - [Get Invoice Items](#36-get-invoice-items)
   - [Start Checking](#37-start-checking)
   - [Finish Checking](#38-finish-checking)
   - [Post Invoice](#39-post-invoice)
   - [Reject Invoice](#310-reject-invoice)
   - [Cancel Invoice](#311-cancel-invoice)
4. [Error Responses](#4-error-responses)
5. [Full Workflow Example](#5-full-workflow-example)
6. [Filtering & Pagination](#6-filtering--pagination)
7. [Implementation Notes](#7-implementation-notes)

---

## 1. Status Machine Overview

```
                    ┌──────────────────────────────────────┐
                    │                                      │
             ┌──────▼──────┐                              │
   CREATE →  │   PENDING   │─── start-checking ──────►  CHECKING
             └──────┬──────┘                              │
                    │                                      ├─── reject ──► REJECTED
                    │ cancel                               │
                    ▼                                      ├─── finish-checking ──►  CHECKED
               CANCELLED                                   │                         │
                    ▲                                      │                         ├─── post ──► POSTED
                    │                                      │                         │
                    └───────── cancel (any state) ─────────┘                         └─── reject ──► REJECTED
```

| From      | Action            | To         | Guard                              |
|-----------|-------------------|------------|------------------------------------|
| PENDING   | start-checking    | CHECKING   | —                                  |
| PENDING   | cancel            | CANCELLED  | —                                  |
| CHECKING  | finish-checking   | CHECKED    | —                                  |
| CHECKING  | reject            | REJECTED   | `rejection_reason` required        |
| CHECKING  | cancel            | CANCELLED  | —                                  |
| CHECKED   | post              | POSTED     | DB trigger sets stock movements    |
| CHECKED   | reject            | REJECTED   | `rejection_reason` required        |
| CHECKED   | cancel            | CANCELLED  | —                                  |
| POSTED    | cancel            | CANCELLED  | DB trigger reverses stock movements|
| CANCELLED | —                 | (terminal) | Cannot cancel twice                |
| REJECTED  | —                 | (terminal) | Cannot transition out              |

**Edit (`PUT /{id}`) is only allowed while `status = PENDING`.**

**Delete (`DELETE /{id}`) is only allowed while `status IN (PENDING, REJECTED, CANCELLED)`.**

---

## 2. Common Types (DTOs)

### `InvoiceStatus` enum

All status values are uppercase strings.

```typescript
type InvoiceStatus =
  | 'PENDING'
  | 'CHECKING'
  | 'CHECKED'
  | 'POSTED'
  | 'REJECTED'
  | 'CANCELLED';
```

### `InvoiceWithDetailsDto`

Returned by all single-invoice endpoints (create, get, update, transitions).

```typescript
interface InvoiceWithDetailsDto {
  id: string;                         // UUID
  invoice_number: string;             // e.g. "NF 123"
  series: string | null;              // e.g. "1"
  access_key: string | null;          // 44-digit NFe key
  issue_date: string;                 // ISO 8601 datetime

  supplier_id: string;                // UUID
  supplier_name: string | null;       // joined from suppliers table

  warehouse_id: string;               // UUID
  warehouse_name: string | null;      // joined from warehouses table

  total_products: string;             // decimal, e.g. "100.00"
  total_freight: string;              // decimal
  total_discount: string;             // decimal
  total_value: string;                // = total_products + total_freight - total_discount

  status: InvoiceStatus;

  received_at: string | null;         // ISO 8601, set when → CHECKING
  received_by: string | null;         // UUID of user who started checking

  checked_at: string | null;          // ISO 8601, set when → CHECKED
  checked_by: string | null;          // UUID

  posted_at: string | null;           // ISO 8601, set when → POSTED
  posted_by: string | null;           // UUID

  commitment_number: string | null;   // e.g. "2026NE000123"
  purchase_order_number: string | null;
  contract_number: string | null;

  notes: string | null;
  rejection_reason: string | null;    // required when status = REJECTED
  pdf_url: string | null;
  xml_url: string | null;

  created_at: string;                 // ISO 8601
  updated_at: string;                 // ISO 8601
}
```

> **Note on decimals:** All monetary and quantity fields are serialized as strings (e.g. `"100.00"`) to preserve precision. Parse them with a decimal library — do **not** use JavaScript's native `parseFloat`.

### `InvoiceItemWithDetailsDto`

Returned by `GET /{id}/items`.

```typescript
interface InvoiceItemWithDetailsDto {
  id: string;                         // UUID
  invoice_id: string;                 // UUID

  catalog_item_id: string;            // UUID (catmat_items)
  catalog_item_name: string | null;   // joined from catmat_items

  unit_conversion_id: string | null;  // UUID, if conversion was applied
  unit_raw_id: string;                // UUID (units_of_measure)
  unit_raw_name: string | null;       // joined, e.g. "CAIXA"
  unit_raw_symbol: string | null;     // joined, e.g. "CX"

  // Classificação do material — herdada do PDM (Padrão Descritivo de Material)
  // Determina o que acontece com o item ao postar a NF:
  //   STOCKABLE  → gera movimentação de ENTRY no estoque do almoxarifado
  //   PERMANENT  → bem permanente (patrimônio) — sem impacto no estoque
  //   DIRECT_USE → consumo/uso direto — sem impacto no estoque
  material_classification: 'STOCKABLE' | 'PERMANENT' | 'DIRECT_USE';

  quantity_raw: string;               // decimal — quantity as on the paper invoice
  unit_value_raw: string;             // decimal — price per raw unit
  total_value: string;                // decimal — quantity_raw × unit_value_raw

  conversion_factor: string;          // decimal — snapshot at entry time
  quantity_base: string;              // decimal — GENERATED: quantity_raw × conversion_factor
  unit_value_base: string;            // decimal — GENERATED: unit_value_raw / conversion_factor

  ncm: string | null;                 // Nomenclatura Comum do Mercosul
  cfop: string | null;                // Código Fiscal de Operações e Prestações
  cest: string | null;                // Código Especificador da Substituição Tributária

  batch_number: string | null;        // lot/batch for traceability
  manufacturing_date: string | null;  // ISO date (YYYY-MM-DD)
  expiration_date: string | null;     // ISO date (YYYY-MM-DD)

  created_at: string;                 // ISO 8601
}
```

---

## 3. Endpoints Reference

### 3.1 List Invoices

```
GET /api/admin/invoices
```

#### Query Parameters

| Parameter      | Type           | Default | Description                                |
|----------------|----------------|---------|--------------------------------------------|
| `limit`        | integer        | 50      | Max items per page                         |
| `offset`       | integer        | 0       | Items to skip                              |
| `search`       | string         | —       | ILIKE on `invoice_number`, `access_key`, `supplier_name` |
| `status`       | InvoiceStatus  | —       | Filter by exact status                     |
| `supplier_id`  | UUID           | —       | Filter by supplier                         |
| `warehouse_id` | UUID           | —       | Filter by warehouse                        |

#### Response `200 OK`

```json
{
  "invoices": [ /* InvoiceWithDetailsDto[] */ ],
  "total": 42,
  "limit": 50,
  "offset": 0
}
```

#### Example

```http
GET /api/admin/invoices?status=PENDING&limit=20&offset=0
Authorization: Bearer <token>
```

---

### 3.2 Create Invoice

```
POST /api/admin/invoices
```

Creates a new invoice with `status = PENDING` and inserts all items atomically. The database trigger `fn_update_invoice_totals` automatically calculates `total_products` and `total_value` from the items.

#### Request Body

```typescript
interface CreateInvoicePayload {
  invoice_number: string;               // required, non-empty
  series?: string | null;
  access_key?: string | null;           // 44-char NFe key; must be unique if provided
  issue_date: string;                   // required, ISO 8601 datetime

  supplier_id: string;                  // required UUID
  warehouse_id: string;                 // required UUID

  total_freight?: string;               // decimal string, default "0"
  total_discount?: string;              // decimal string, default "0"

  commitment_number?: string | null;    // e.g. "2026NE000123"
  purchase_order_number?: string | null;
  contract_number?: string | null;

  notes?: string | null;
  pdf_url?: string | null;
  xml_url?: string | null;

  items: CreateInvoiceItemPayload[];    // required, must have >= 1 item
}

interface CreateInvoiceItemPayload {
  catalog_item_id: string;             // required UUID (catmat_items)
  unit_raw_id: string;                 // required UUID (units_of_measure)
  unit_conversion_id?: string | null;  // UUID (unit_conversions)

  quantity_raw: string;                // required decimal > 0
  unit_value_raw: string;              // required decimal >= 0
  conversion_factor?: string;          // decimal > 0, default "1.0"

  ncm?: string | null;                 // max 10 chars
  cfop?: string | null;                // max 4 chars
  cest?: string | null;                // max 7 chars

  batch_number?: string | null;
  manufacturing_date?: string | null;  // YYYY-MM-DD
  expiration_date?: string | null;     // YYYY-MM-DD; must be > manufacturing_date if both set
}
```

#### Responses

| Status | Meaning                                                      |
|--------|--------------------------------------------------------------|
| 201    | Invoice created — body is `InvoiceWithDetailsDto`            |
| 400    | Validation error (empty number, empty items, qty <= 0, etc.) |
| 409    | `access_key` already exists                                  |
| 401/403| Missing or insufficient auth                                 |

#### Example

```json
POST /api/admin/invoices
Content-Type: application/json
Authorization: Bearer <token>

{
  "invoice_number": "NF 2026/0042",
  "series": "1",
  "access_key": "35260300000000000000550010000000421000000019",
  "issue_date": "2026-03-16T14:30:00Z",
  "supplier_id": "550e8400-e29b-41d4-a716-446655440000",
  "warehouse_id": "660e8400-e29b-41d4-a716-446655440000",
  "total_freight": "50.00",
  "total_discount": "0.00",
  "commitment_number": "2026NE000042",
  "items": [
    {
      "catalog_item_id": "770e8400-e29b-41d4-a716-446655440000",
      "unit_raw_id": "880e8400-e29b-41d4-a716-446655440000",
      "quantity_raw": "10.0000",
      "unit_value_raw": "25.5000",
      "conversion_factor": "1.0",
      "ncm": "3004.90.69",
      "cfop": "1102",
      "batch_number": "LOTE2026A",
      "expiration_date": "2028-12-31"
    }
  ]
}
```

---

### 3.3 Get Invoice

```
GET /api/admin/invoices/:id
```

#### Path Parameters

| Parameter | Type | Description  |
|-----------|------|--------------|
| `id`      | UUID | Invoice ID   |

#### Responses

| Status | Meaning                            |
|--------|------------------------------------|
| 200    | Body is `InvoiceWithDetailsDto`    |
| 404    | Invoice not found                  |
| 401/403| Missing or insufficient auth       |

---

### 3.4 Update Invoice

```
PUT /api/admin/invoices/:id
```

**Only allowed when `status = PENDING`.** All fields are optional (partial update). Items are **not** modified through this endpoint — they are set at creation time.

#### Request Body

```typescript
interface UpdateInvoicePayload {
  invoice_number?: string | null;
  series?: string | null;
  access_key?: string | null;           // must remain unique if changed
  issue_date?: string | null;           // ISO 8601 datetime

  supplier_id?: string | null;          // UUID
  warehouse_id?: string | null;         // UUID

  total_freight?: string | null;        // decimal string
  total_discount?: string | null;       // decimal string

  commitment_number?: string | null;
  purchase_order_number?: string | null;
  contract_number?: string | null;

  notes?: string | null;
  pdf_url?: string | null;
  xml_url?: string | null;
}
```

#### Responses

| Status | Meaning                                         |
|--------|-------------------------------------------------|
| 200    | Updated — body is `InvoiceWithDetailsDto`       |
| 400    | Invoice is not PENDING                          |
| 404    | Invoice not found                               |
| 409    | `access_key` already used by another invoice    |
| 401/403| Missing or insufficient auth                    |

---

### 3.5 Delete Invoice

```
DELETE /api/admin/invoices/:id
```

**Only allowed when `status IN (PENDING, REJECTED, CANCELLED)`.** Cascades to invoice items via `ON DELETE CASCADE`.

#### Responses

| Status | Meaning                                                     |
|--------|-------------------------------------------------------------|
| 204    | Deleted successfully                                        |
| 400    | Cannot delete — status is CHECKING, CHECKED, or POSTED     |
| 404    | Invoice not found                                           |
| 401/403| Missing or insufficient auth                                |

---

### 3.6 Get Invoice Items

```
GET /api/admin/invoices/:id/items
```

Returns all line items for an invoice, with catalog item and unit names joined.

#### Responses

| Status | Meaning                                                |
|--------|--------------------------------------------------------|
| 200    | Body: `{ "items": InvoiceItemWithDetailsDto[] }`       |
| 404    | Invoice not found                                      |
| 401/403| Missing or insufficient auth                           |

---

### 3.7 Start Checking

```
POST /api/admin/invoices/:id/start-checking
```

Transitions: **PENDING → CHECKING**. Sets `received_at = NOW()` and `received_by = <current user>`.

#### Request Body

```json
{}
```

_(Body is accepted but ignored. Send an empty object.)_

#### Responses

| Status | Meaning                                         |
|--------|-------------------------------------------------|
| 200    | Body is `InvoiceWithDetailsDto` (status=CHECKING)|
| 400    | Invoice is not PENDING                          |
| 404    | Invoice not found                               |
| 401/403| Missing or insufficient auth                    |

---

### 3.8 Finish Checking

```
POST /api/admin/invoices/:id/finish-checking
```

Transitions: **CHECKING → CHECKED**. Sets `checked_at = NOW()` and `checked_by = <current user>`.

#### Request Body

```json
{}
```

#### Responses

| Status | Meaning                                          |
|--------|--------------------------------------------------|
| 200    | Body is `InvoiceWithDetailsDto` (status=CHECKED) |
| 400    | Invoice is not CHECKING                          |
| 404    | Invoice not found                                |
| 401/403| Missing or insufficient auth                     |

---

### 3.9 Post Invoice

```
POST /api/admin/invoices/:id/post
```

Transitions: **CHECKED → POSTED**. Sets `posted_at = NOW()` and `posted_by = <current user>`.

**Side effect:** The PostgreSQL trigger `fn_auto_post_invoice()` automatically creates `ENTRY` stock movements in `stock_movements` for each invoice item. No additional API call is required.

#### Request Body

```json
{}
```

#### Responses

| Status | Meaning                                        |
|--------|------------------------------------------------|
| 200    | Body is `InvoiceWithDetailsDto` (status=POSTED)|
| 400    | Invoice is not CHECKED                         |
| 404    | Invoice not found                              |
| 401/403| Missing or insufficient auth                   |

---

### 3.10 Reject Invoice

```
POST /api/admin/invoices/:id/reject
```

Transitions: **CHECKING → REJECTED** or **CHECKED → REJECTED**.

#### Request Body

```typescript
interface RejectInvoicePayload {
  rejection_reason: string;    // required, non-empty
}
```

#### Responses

| Status | Meaning                                          |
|--------|--------------------------------------------------|
| 200    | Body is `InvoiceWithDetailsDto` (status=REJECTED)|
| 400    | Invoice is not in CHECKING or CHECKED state, or rejection_reason is empty |
| 404    | Invoice not found                                |
| 401/403| Missing or insufficient auth                     |

#### Example

```json
{
  "rejection_reason": "Divergência na quantidade recebida: esperado 10 unidades, recebido 8"
}
```

---

### 3.11 Cancel Invoice

```
POST /api/admin/invoices/:id/cancel
```

Transitions: **any status → CANCELLED** (except CANCELLED itself).

**Side effect:** If the invoice was `POSTED`, the PostgreSQL trigger `fn_auto_post_invoice()` automatically creates `ADJUSTMENT_SUB` stock movements to reverse the stock entries. No additional API call is required.

#### Request Body

```json
{}
```

#### Responses

| Status | Meaning                                            |
|--------|----------------------------------------------------|
| 200    | Body is `InvoiceWithDetailsDto` (status=CANCELLED) |
| 400    | Invoice is already CANCELLED                       |
| 404    | Invoice not found                                  |
| 401/403| Missing or insufficient auth                       |

---

## 4. Error Responses

All errors return plain text with an HTTP status code:

```
HTTP/1.1 400 Bad Request
Content-Type: text/plain

Nota fiscal deve ter ao menos um item
```

| HTTP Status | Cause                                                    |
|-------------|----------------------------------------------------------|
| 400         | Validation error, invalid state transition               |
| 401         | Missing or expired JWT token                             |
| 403         | Authenticated but insufficient role (need admin)         |
| 404         | Resource not found                                       |
| 409         | Conflict — duplicate `access_key`                        |
| 500         | Unexpected server error                                  |

---

## 5. Full Workflow Example

```typescript
const BASE = '/api/admin/invoices';
const headers = { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' };

// Step 1 — Create
const invoice = await fetch(BASE, {
  method: 'POST',
  headers,
  body: JSON.stringify({
    invoice_number: 'NF 2026/0099',
    issue_date: new Date().toISOString(),
    supplier_id: '550e8400-...',
    warehouse_id: '660e8400-...',
    total_freight: '0.00',
    items: [{
      catalog_item_id: '770e8400-...',
      unit_raw_id: '880e8400-...',
      quantity_raw: '5.0000',
      unit_value_raw: '100.0000',
    }],
  }),
}).then(r => r.json());
// invoice.status === 'PENDING'

// Step 2 — Start checking (physical receipt)
await fetch(`${BASE}/${invoice.id}/start-checking`, {
  method: 'POST', headers, body: JSON.stringify({}),
});
// invoice.status === 'CHECKING'

// Step 3 — Finish checking (verification OK)
await fetch(`${BASE}/${invoice.id}/finish-checking`, {
  method: 'POST', headers, body: JSON.stringify({}),
});
// invoice.status === 'CHECKED'

// Step 4 — Post to inventory (stock movements created automatically)
const posted = await fetch(`${BASE}/${invoice.id}/post`, {
  method: 'POST', headers, body: JSON.stringify({}),
}).then(r => r.json());
// posted.status === 'POSTED'
// posted.posted_at is set
// stock_movements for each item are created by the database trigger
```

---

## 6. Filtering & Pagination

All list requests return `total`, `limit`, and `offset` at the root level to support pagination UIs:

```typescript
interface InvoicesListResponse {
  invoices: InvoiceWithDetailsDto[];
  total: number;     // total matching records (for page count calculation)
  limit: number;     // requested limit (echoed back)
  offset: number;    // requested offset (echoed back)
}
```

**Page calculation:**
```typescript
const totalPages = Math.ceil(total / limit);
const currentPage = Math.floor(offset / limit) + 1;
```

**Filter examples:**

```
# All pending invoices from a specific supplier
GET /api/admin/invoices?status=PENDING&supplier_id=<uuid>

# Search across number, access_key, and supplier name
GET /api/admin/invoices?search=NF+2026&limit=10

# Invoices from a specific warehouse, page 3 (with 20 per page)
GET /api/admin/invoices?warehouse_id=<uuid>&limit=20&offset=40
```

---

## 7. Implementation Notes

### Decimal values

All `Decimal` fields (`total_products`, `total_freight`, `total_discount`, `total_value`, `quantity_raw`, `unit_value_raw`, `quantity_base`, `unit_value_base`, `conversion_factor`) are serialized as **decimal strings** with fixed precision. Always use a decimal library when performing arithmetic on these values.

**Recommended libraries:**
- JavaScript/TypeScript: [`decimal.js`](https://mikemcl.github.io/decimal.js/) or [`big.js`](https://mikemcl.github.io/big.js/)
- Python: `decimal.Decimal`

### Auto-calculated fields

The following fields are **set by the database** and must not be sent in requests:

| Field            | Set by                            |
|------------------|-----------------------------------|
| `total_products` | Trigger `fn_update_invoice_totals` |
| `total_value`    | Trigger `fn_update_invoice_totals` |
| `quantity_base`  | `GENERATED ALWAYS AS` column      |
| `unit_value_base`| `GENERATED ALWAYS AS` column      |
| `received_at`    | `start-checking` endpoint         |
| `checked_at`     | `finish-checking` endpoint        |
| `posted_at`      | `post` endpoint                   |
| `created_at`     | DB default `NOW()`                |
| `updated_at`     | DB trigger `set_timestamp_invoices`|

### Stock movements and PDM classification

When an invoice is posted (`status → POSTED`), the database trigger `fn_auto_post_invoice()` processes each item according to its `material_classification` (inherited from the PDM):

| `material_classification` | Effect on posting                                  |
|---------------------------|----------------------------------------------------|
| `STOCKABLE`               | Creates `ENTRY` movement → updates `warehouse_stocks` (WAC recalculated) |
| `PERMANENT`               | No stock movement (future: patrimônio module)       |
| `DIRECT_USE`              | No stock movement                                   |

When the invoice is subsequently cancelled (`POSTED → CANCELLED`), the trigger creates `ADJUSTMENT_SUB` movements only for items that previously generated an `ENTRY` (i.e., `STOCKABLE` items). **The frontend does not need to make any separate request** to update inventory.

**UX recommendation — pre-flight posting summary:** Before the user clicks "Confirmar Lançamento", display a breakdown using `material_classification`:

```typescript
const items = await fetchInvoiceItems(invoiceId);

const summary = {
  stockable:  items.filter(i => i.material_classification === 'STOCKABLE'),
  permanent:  items.filter(i => i.material_classification === 'PERMANENT'),
  directUse:  items.filter(i => i.material_classification === 'DIRECT_USE'),
};

// Example message:
// "Ao confirmar: 5 itens entrarão no estoque, 2 serão registrados como patrimônio,
//  1 item é de uso direto e não gerará movimentação."
```

### Access key (`access_key`)

The 44-digit NFe access key is optional but, when provided, must be unique across all invoices. Use it to prevent accidental duplication of the same fiscal document.

### items are read-only after creation

Once an invoice is created, its items cannot be modified through the API (no item-level PATCH/PUT endpoint). To correct an item, the workflow is:

1. Cancel the invoice (if not POSTED)
2. Delete the cancelled invoice
3. Create a new invoice with the corrected items

Or, if the invoice is POSTED:

1. Cancel the invoice (stock is reversed automatically by the DB trigger)
2. Create a new invoice

### Authentication — role required

All invoice endpoints are under `/api/admin/` and protected by:
1. JWT authentication middleware (`mw_session_authenticate`)
2. Casbin RBAC authorization middleware (`mw_authorize`) — requires `admin` role

Users with the `user` role will receive `403 Forbidden`.
