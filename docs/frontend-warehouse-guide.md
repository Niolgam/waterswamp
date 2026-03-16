# Guia Frontend — Módulo Warehouse (Almoxarifados e Estoques)

> **Versão:** 2026-03-16
> **Base URL:** `/api/admin/warehouses`
> **Autenticação:** Bearer JWT (admin obrigatório em todos os endpoints)

---

## Sumário

1. [Visão Geral do Módulo](#1-visão-geral-do-módulo)
2. [TypeScript Interfaces](#2-typescript-interfaces)
3. [Endpoints — Almoxarifados](#3-endpoints--almoxarifados)
4. [Endpoints — Estoques](#4-endpoints--estoques)
5. [Campos Calculados e Decimais](#5-campos-calculados-e-decimais)
6. [Comportamento do Estoque (Triggers)](#6-comportamento-do-estoque-triggers)
7. [Códigos de Status HTTP](#7-códigos-de-status-http)

---

## 1. Visão Geral do Módulo

O módulo **Warehouse** gerencia almoxarifados e seus estoques de itens de catálogo (CATMAT).

**Regra central:** `warehouse_stocks` **não é criado via API**. Os registros de estoque surgem automaticamente quando uma nota fiscal (invoice) é **postada** — o trigger `fn_auto_post_invoice()` gera movimentações que o trigger `fn_process_stock_movement()` converte em entradas de estoque com cálculo de Preço Médio Ponderado (PMC/WAC).

O frontend pode:
- Criar, editar, listar e excluir almoxarifados
- Consultar e filtrar o estoque de cada almoxarifado
- Atualizar parâmetros de controle (min/max/ponto de pedido/localização)
- Bloquear e desbloquear itens de estoque administrativamente

---

## 2. TypeScript Interfaces

```typescript
// =============================================
// Enums
// =============================================

export type WarehouseType = 'CENTRAL' | 'SECTOR';

// =============================================
// Almoxarifado (Warehouse)
// =============================================

export interface WarehouseWithDetailsDto {
  id: string;                         // UUID
  name: string;
  code: string;                       // Código único do almoxarifado
  warehouse_type: WarehouseType;
  city_id: string;                    // UUID
  city_name: string | null;
  state_abbreviation: string | null;
  responsible_user_id: string | null;  // UUID — usuário responsável
  responsible_unit_id: string | null;  // UUID — unidade organizacional responsável
  allows_transfers: boolean;
  is_budgetary: boolean;
  address: string | null;
  phone: string | null;
  email: string | null;
  is_active: boolean;
  created_at: string;                 // ISO 8601
  updated_at: string;                 // ISO 8601
}

export interface CreateWarehousePayload {
  name: string;                       // Obrigatório
  code: string;                       // Obrigatório, único no sistema
  warehouse_type: WarehouseType;      // Obrigatório
  city_id: string;                    // UUID, obrigatório
  responsible_user_id?: string;
  responsible_unit_id?: string;
  allows_transfers?: boolean;         // Default: true
  is_budgetary?: boolean;             // Default: false
  address?: string;
  phone?: string;
  email?: string;
}

export interface UpdateWarehousePayload {
  name?: string;
  code?: string;
  warehouse_type?: WarehouseType;
  city_id?: string;
  responsible_user_id?: string;
  responsible_unit_id?: string;
  allows_transfers?: boolean;
  is_budgetary?: boolean;
  address?: string;
  phone?: string;
  email?: string;
  is_active?: boolean;
}

export interface WarehousesListResponse {
  warehouses: WarehouseWithDetailsDto[];
  total: number;
  limit: number;
  offset: number;
}

// =============================================
// Estoque (Warehouse Stock)
// =============================================

export interface WarehouseStockWithDetailsDto {
  id: string;                           // UUID
  warehouse_id: string;                 // UUID
  warehouse_name: string | null;
  catalog_item_id: string;              // UUID
  catalog_item_name: string | null;
  catalog_item_code: string | null;
  unit_symbol: string | null;           // Ex: "UNID", "KG", "L"
  unit_name: string | null;

  // Quantidades — sempre string (Decimal serializado)
  quantity: string;                     // Estoque bruto
  reserved_quantity: string;            // Reservado para pedidos pendentes
  available_quantity: string;           // quantity - reserved_quantity (0 se bloqueado)
  average_unit_value: string;           // Preço médio ponderado (PMC)
  total_value: string;                  // quantity * average_unit_value

  // Parâmetros de controle
  min_stock: string | null;
  max_stock: string | null;
  reorder_point: string | null;
  resupply_days: number | null;         // Lead time em dias
  location: string | null;             // Ex: "Corredor A"
  secondary_location: string | null;

  // Bloqueio administrativo
  is_blocked: boolean;
  block_reason: string | null;
  blocked_at: string | null;           // ISO 8601
  blocked_by: string | null;           // UUID do usuário que bloqueou

  // Última movimentação (cache)
  last_entry_at: string | null;        // ISO 8601
  last_exit_at: string | null;
  last_inventory_at: string | null;

  created_at: string;
  updated_at: string;
}

export interface WarehouseStockDto {
  id: string;
  warehouse_id: string;
  catalog_item_id: string;
  quantity: string;
  reserved_quantity: string;
  average_unit_value: string;
  min_stock: string | null;
  max_stock: string | null;
  reorder_point: string | null;
  resupply_days: number | null;
  location: string | null;
  secondary_location: string | null;
  is_blocked: boolean;
  block_reason: string | null;
  blocked_at: string | null;
  blocked_by: string | null;
  last_entry_at: string | null;
  last_exit_at: string | null;
  last_inventory_at: string | null;
  created_at: string;
  updated_at: string;
}

export interface UpdateStockParamsPayload {
  min_stock?: string;           // Decimal como string
  max_stock?: string;
  reorder_point?: string;
  resupply_days?: number;
  location?: string;
  secondary_location?: string;
}

export interface BlockStockPayload {
  block_reason: string;         // Obrigatório, não pode ser vazio
}

export interface WarehouseStocksListResponse {
  stocks: WarehouseStockWithDetailsDto[];
  total: number;
  limit: number;
  offset: number;
}
```

---

## 3. Endpoints — Almoxarifados

### 3.1 Listar Almoxarifados

```
GET /api/admin/warehouses
```

**Query Parameters:**

| Parâmetro       | Tipo           | Descrição                              |
|-----------------|----------------|----------------------------------------|
| `limit`         | integer        | Registros por página (default: 50)     |
| `offset`        | integer        | Pular N registros (default: 0)         |
| `search`        | string         | Busca por nome, código ou cidade       |
| `warehouse_type`| `CENTRAL\|SECTOR` | Filtrar por tipo                    |
| `city_id`       | UUID           | Filtrar por cidade                     |
| `is_active`     | boolean        | `true` ou `false`                      |

**Exemplo de request:**

```bash
GET /api/admin/warehouses?search=central&is_active=true&limit=20&offset=0
Authorization: Bearer <token>
```

**Response `200 OK`:**

```json
{
  "warehouses": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "name": "Almoxarifado Central",
      "code": "ALM-CENTRAL-01",
      "warehouse_type": "CENTRAL",
      "city_id": "...",
      "city_name": "São Paulo",
      "state_abbreviation": "SP",
      "responsible_user_id": null,
      "responsible_unit_id": "...",
      "allows_transfers": true,
      "is_budgetary": true,
      "address": "Rua das Flores, 123",
      "phone": null,
      "email": null,
      "is_active": true,
      "created_at": "2026-01-10T09:00:00Z",
      "updated_at": "2026-03-16T14:30:00Z"
    }
  ],
  "total": 1,
  "limit": 20,
  "offset": 0
}
```

---

### 3.2 Criar Almoxarifado

```
POST /api/admin/warehouses
```

**Request Body:**

```json
{
  "name": "Almoxarifado Setor Norte",
  "code": "ALM-NORTE-01",
  "warehouse_type": "SECTOR",
  "city_id": "550e8400-e29b-41d4-a716-446655440001",
  "allows_transfers": true,
  "is_budgetary": false
}
```

**Response `201 Created`:** `WarehouseWithDetailsDto`

**Erros possíveis:**
- `409 Conflict` — código já existe no sistema

---

### 3.3 Obter Almoxarifado por ID

```
GET /api/admin/warehouses/{id}
```

**Response `200 OK`:** `WarehouseWithDetailsDto`

**Erros:** `404 Not Found`

---

### 3.4 Atualizar Almoxarifado

```
PUT /api/admin/warehouses/{id}
```

Todos os campos são opcionais. Apenas os campos enviados são atualizados (COALESCE no SQL).

**Request Body:**

```json
{
  "name": "Almoxarifado Central Reformado",
  "is_active": false
}
```

**Response `200 OK`:** `WarehouseWithDetailsDto`

**Erros possíveis:**
- `404 Not Found` — almoxarifado não existe
- `409 Conflict` — o novo código já pertence a outro almoxarifado

---

### 3.5 Excluir Almoxarifado

```
DELETE /api/admin/warehouses/{id}
```

> **Atenção:** A exclusão só é possível se o almoxarifado não tiver `warehouse_stocks` associados (FK `ON DELETE CASCADE` — registros de estoque serão excluídos junto). Se houver invoices referenciando o almoxarifado, a exclusão falhará com `409`.

**Response `204 No Content`** (sem corpo)

**Erros:** `404 Not Found`

---

## 4. Endpoints — Estoques

### 4.1 Listar Estoque do Almoxarifado

```
GET /api/admin/warehouses/{warehouse_id}/stocks
```

**Query Parameters:**

| Parâmetro   | Tipo    | Descrição                                |
|-------------|---------|------------------------------------------|
| `limit`     | integer | Default: 50                              |
| `offset`    | integer | Default: 0                               |
| `search`    | string  | Busca por nome/código do item ou localização |
| `is_blocked`| boolean | `true` = apenas bloqueados, `false` = apenas liberados |

**Response `200 OK`:**

```json
{
  "stocks": [
    {
      "id": "...",
      "warehouse_id": "...",
      "warehouse_name": "Almoxarifado Central",
      "catalog_item_id": "...",
      "catalog_item_name": "Caneta Esferográfica Azul",
      "catalog_item_code": "4490.17.001",
      "unit_symbol": "UNID",
      "unit_name": "Unidade",
      "quantity": "150.000",
      "reserved_quantity": "10.000",
      "available_quantity": "140.000",
      "average_unit_value": "2.5000",
      "total_value": "375.0000",
      "min_stock": "20.000",
      "max_stock": "500.000",
      "reorder_point": "50.000",
      "resupply_days": 7,
      "location": "Corredor B, Prateleira 2",
      "secondary_location": null,
      "is_blocked": false,
      "block_reason": null,
      "blocked_at": null,
      "blocked_by": null,
      "last_entry_at": "2026-03-10T08:00:00Z",
      "last_exit_at": null,
      "last_inventory_at": null,
      "created_at": "2026-01-15T10:00:00Z",
      "updated_at": "2026-03-10T08:00:00Z"
    }
  ],
  "total": 1,
  "limit": 50,
  "offset": 0
}
```

**Erros:** `404 Not Found` (almoxarifado não existe)

---

### 4.2 Obter Estoque por ID

```
GET /api/admin/warehouses/stocks/{stock_id}
```

**Response `200 OK`:** `WarehouseStockWithDetailsDto`

**Erros:** `404 Not Found`

---

### 4.3 Atualizar Parâmetros de Controle do Estoque

```
PATCH /api/admin/warehouses/stocks/{stock_id}
```

Atualiza parâmetros operacionais (não afeta quantidade, preço ou movimentação).

**Request Body:**

```json
{
  "min_stock": "20.000",
  "max_stock": "500.000",
  "reorder_point": "50.000",
  "resupply_days": 14,
  "location": "Corredor A, Prateleira 1",
  "secondary_location": "Depósito auxiliar"
}
```

**Response `200 OK`:** `WarehouseStockDto` (sem joins)

**Erros:** `404 Not Found`

---

### 4.4 Bloquear Estoque

```
POST /api/admin/warehouses/stocks/{stock_id}/block
```

Bloqueia o item de estoque administrativamente. Um item bloqueado tem `available_quantity = 0` (independente da quantidade real), impedindo novas reservas.

**Request Body:**

```json
{
  "block_reason": "Produto em quarentena — aguardando análise de qualidade"
}
```

**Response `200 OK`:** `WarehouseStockDto`

```json
{
  "id": "...",
  "is_blocked": true,
  "block_reason": "Produto em quarentena — aguardando análise de qualidade",
  "blocked_at": "2026-03-16T15:00:00Z",
  "blocked_by": "uuid-do-admin",
  ...
}
```

**Erros:**
- `400 Bad Request` — estoque já está bloqueado
- `400 Bad Request` — `block_reason` vazio
- `404 Not Found`

---

### 4.5 Desbloquear Estoque

```
POST /api/admin/warehouses/stocks/{stock_id}/unblock
```

Sem corpo de request.

**Response `200 OK`:** `WarehouseStockDto`

```json
{
  "id": "...",
  "is_blocked": false,
  "block_reason": null,
  "blocked_at": null,
  "blocked_by": null,
  ...
}
```

**Erros:**
- `400 Bad Request` — estoque não está bloqueado
- `404 Not Found`

---

## 5. Campos Calculados e Decimais

### Decimais como String

Todos os campos monetários e de quantidade são serializados como **string** no JSON para preservar precisão:

```typescript
// Correto: usar biblioteca de decimal para operações
import Decimal from 'decimal.js';

const qty = new Decimal(stock.quantity);       // "150.000"
const price = new Decimal(stock.average_unit_value); // "2.5000"
const total = qty.mul(price);                  // 375.0000

// Para exibição
const formatted = new Decimal(stock.total_value).toFixed(2); // "375.00"
```

### Campos Calculados pelo Backend

| Campo | Fórmula | Nota |
|-------|---------|------|
| `available_quantity` | `quantity - reserved_quantity` (ou 0 se bloqueado) | Calculado na query |
| `total_value` | `quantity * average_unit_value` | Calculado na query |
| `average_unit_value` | PMC — atualizado via trigger em cada entrada | Gerenciado automaticamente |

### Alerta de Estoque Baixo

```typescript
// Verificar se o item está abaixo do ponto de pedido
function isLowStock(stock: WarehouseStockWithDetailsDto): boolean {
  if (!stock.reorder_point) return false;
  return new Decimal(stock.available_quantity).lte(new Decimal(stock.reorder_point));
}

// Verificar se está abaixo do estoque mínimo (crítico)
function isCriticalStock(stock: WarehouseStockWithDetailsDto): boolean {
  if (!stock.min_stock) return false;
  return new Decimal(stock.quantity).lt(new Decimal(stock.min_stock));
}
```

---

## 6. Comportamento do Estoque (Triggers)

O estoque é **gerenciado automaticamente** pelo banco de dados via triggers:

```
Invoice POSTED
    └─► trigger fn_auto_post_invoice()
            └─► INSERT INTO stock_movements (tipo ENTRY)
                    └─► trigger fn_process_stock_movement()
                            └─► UPSERT warehouse_stocks
                                  · Atualiza quantity
                                  · Recalcula average_unit_value (WAC)
                                  · Atualiza last_entry_at
```

**Implicações para o frontend:**

1. **Não existe** `POST /warehouses/{id}/stocks` — o estoque nasce quando uma invoice é postada.
2. O `available_quantity` de um item bloqueado é sempre **zero**, mesmo que `quantity > 0`.
3. A coluna `reserved_quantity` é reservada para integrações futuras com requisições — atualmente sempre 0.
4. Ao **cancelar** uma invoice postada, o trigger reverte automaticamente as entradas de estoque.

---

## 7. Códigos de Status HTTP

| Status | Significado |
|--------|-------------|
| `200 OK` | Operação bem-sucedida (GET, PUT, PATCH, POST de transição) |
| `201 Created` | Almoxarifado criado com sucesso |
| `204 No Content` | Almoxarifado excluído |
| `400 Bad Request` | Erro de validação (nome/código vazio, bloquear já bloqueado, etc.) |
| `401 Unauthorized` | Token ausente ou inválido |
| `403 Forbidden` | Usuário não tem role admin |
| `404 Not Found` | Almoxarifado ou estoque não encontrado |
| `409 Conflict` | Código do almoxarifado já existe |

---

## Exemplo: Tela de Estoque com Indicadores

```typescript
async function loadWarehouseStocks(warehouseId: string) {
  const response = await fetch(
    `/api/admin/warehouses/${warehouseId}/stocks?limit=50&offset=0`,
    { headers: { Authorization: `Bearer ${token}` } }
  );
  const data: WarehouseStocksListResponse = await response.json();

  return data.stocks.map(stock => ({
    ...stock,
    isLow: stock.reorder_point
      ? new Decimal(stock.available_quantity).lte(stock.reorder_point)
      : false,
    isCritical: stock.min_stock
      ? new Decimal(stock.quantity).lt(stock.min_stock)
      : false,
    formattedTotalValue: `R$ ${new Decimal(stock.total_value).toFixed(2)}`,
  }));
}
```

---

## Exemplo: Criar e Listar Almoxarifados

```typescript
// Criar almoxarifado
const response = await fetch('/api/admin/warehouses', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    Authorization: `Bearer ${token}`,
  },
  body: JSON.stringify({
    name: 'Almoxarifado Setor Sul',
    code: 'ALM-SUL-01',
    warehouse_type: 'SECTOR',
    city_id: cityId,
    allows_transfers: true,
    is_budgetary: false,
  }),
});

if (response.status === 409) {
  // Código já existe — mostrar erro ao usuário
}

// Listar almoxarifados ativos do tipo CENTRAL
const list = await fetch(
  '/api/admin/warehouses?warehouse_type=CENTRAL&is_active=true&limit=20',
  { headers: { Authorization: `Bearer ${token}` } }
);
const { warehouses, total }: WarehousesListResponse = await list.json();
```

---

## Relação com o Módulo de Invoice

O `warehouse_id` de uma invoice referencia um almoxarifado deste módulo. Ao postar uma invoice, o estoque do almoxarifado correspondente é atualizado automaticamente.

Consulte também: [`docs/frontend-invoice-guide.md`](./frontend-invoice-guide.md) — para entender o fluxo completo de entrada de materiais.
