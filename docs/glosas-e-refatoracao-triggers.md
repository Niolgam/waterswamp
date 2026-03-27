# Glosas de Notas Fiscais e Refatoração de Triggers → Rust

> **Audiência:** Equipe de front-end e integradores da API Waterswamp.
> **Data:** 2026-03-27

---

## Sumário

1. [O que mudou no backend](#1-o-que-mudou-no-backend)
2. [Novos endpoints — Ajustes de NF (Glosas)](#2-novos-endpoints--ajustes-de-nf-glosas)
3. [Endpoint atualizado — Adicionar item à requisição](#3-endpoint-atualizado--adicionar-item-à-requisição)
4. [Mudanças no comportamento existente](#4-mudanças-no-comportamento-existente)
5. [Novas tabelas do banco](#5-novas-tabelas-do-banco)
6. [Mudanças nos contratos JSON](#6-mudanças-nos-contratos-json)
7. [Exemplos de integração](#7-exemplos-de-integração)
8. [Erros esperados e seus códigos](#8-erros-esperados-e-seus-códigos)

---

## 1. O que mudou no backend

Esta release traz **duas mudanças arquiteturais** com impacto potencial para o front-end:

### 1.1 Novo módulo: Ajustes de Nota Fiscal (Glosas)

Uma nota fiscal com status `POSTED` pode agora receber **glosas** — registros de desconto/ajuste aplicados após o lançamento no estoque. Cada glosa pode:

- Ajustar a **quantidade** do item (gera movimentação `ADJUSTMENT_SUB` no estoque para itens `STOCKABLE`)
- Ajustar o **valor** financeiro (sem impacto no estoque)
- Ter múltiplos itens na mesma operação

### 1.2 Lógica de negócio migrada de triggers do banco para Rust

Os seguintes comportamentos **não mudam para o front-end** mas foram reimplementados no serviço Rust (mais previsíveis, com erros HTTP estruturados):

| Trigger removido | Comportamento equivalente agora em |
|---|---|
| `fn_auto_post_invoice` | `InvoiceService.post_invoice()` |
| `fn_process_stock_movement` | `StockMovementService.process_movement()` |
| `fn_manage_stock_reservation` | `RequisitionService.approve_requisition()` |
| `fn_capture_requisition_item_value` | `RequisitionService.add_item_to_requisition()` |
| `fn_update_invoice_totals` | `InvoiceService.create_invoice()` |
| `fn_update_requisition_total` | `RequisitionService.add_item_to_requisition()` |

**Impacto visível no front-end:** erros que antes chegavam como `500 Internal Server Error` com mensagem genérica de trigger PostgreSQL agora chegam como `400 Bad Request` com mensagem em português descrevendo a regra violada.

---

## 2. Novos endpoints — Ajustes de NF (Glosas)

Base URL: `/api/admin/invoices/{invoice_id}/adjustments`

Autenticação: `Authorization: Bearer <token>` com papel `admin`.

### 2.1 Listar ajustes de uma NF

```http
GET /api/admin/invoices/{invoice_id}/adjustments
```

**Resposta 200:**
```json
[
  {
    "id": "uuid",
    "invoice_id": "uuid",
    "reason": "Quantidade divergente na conferência física",
    "created_by": "uuid",
    "created_at": "2026-03-27T14:00:00Z",
    "items": [
      {
        "id": "uuid",
        "adjustment_id": "uuid",
        "invoice_item_id": "uuid",
        "catalog_item_name": "Caneta Esferográfica Azul",
        "adjusted_quantity": "2.0000",
        "adjusted_value": "20.0000",
        "notes": "Devolvido ao fornecedor",
        "created_at": "2026-03-27T14:00:00Z"
      }
    ]
  }
]
```

**Resposta quando não há ajustes:** `200 []` (array vazio)

**Erros:**
- `404` — NF não encontrada

---

### 2.2 Criar ajuste (glosa)

```http
POST /api/admin/invoices/{invoice_id}/adjustments
Content-Type: application/json
```

**Payload:**
```json
{
  "reason": "Quantidade divergente na conferência física",
  "items": [
    {
      "invoice_item_id": "uuid-do-item-da-nf",
      "adjusted_quantity": "2.0000",
      "adjusted_value": "20.0000",
      "notes": "Texto livre opcional"
    }
  ]
}
```

| Campo | Tipo | Obrigatório | Descrição |
|---|---|---|---|
| `reason` | string | **sim** | Motivo da glosa (não pode ser vazio) |
| `items` | array | **sim** | Ao menos 1 item |
| `items[].invoice_item_id` | UUID | **sim** | ID do item da NF (campo `id` em `/invoices/{id}/items`) |
| `items[].adjusted_quantity` | decimal | condicional | Quantidade glosada. Se omitido, assume 0 |
| `items[].adjusted_value` | decimal | condicional | Valor financeiro glosado. Se omitido, assume 0 |
| `items[].notes` | string | não | Observação livre por item |

> **Regra:** Cada item deve ter `adjusted_quantity > 0` **ou** `adjusted_value > 0`. Ambos zerados retorna `400`.

**Resposta 201:**
```json
{
  "id": "uuid",
  "invoice_id": "uuid",
  "reason": "Quantidade divergente na conferência física",
  "created_by": "uuid",
  "created_at": "2026-03-27T14:00:00Z",
  "items": [...]
}
```

**Erros:**
- `400 Bad Request` — NF não está em status `POSTED`
- `400 Bad Request` — `reason` vazio
- `400 Bad Request` — `items` vazio
- `400 Bad Request` — item com `adjusted_quantity = 0` e `adjusted_value = 0`
- `400 Bad Request` — `invoice_item_id` não pertence à NF indicada
- `404 Not Found` — NF não encontrada

---

## 3. Endpoint atualizado — Adicionar item à requisição

```http
POST /api/admin/requisitions/{requisition_id}/items
Content-Type: application/json
```

Este endpoint **existia antes**, mas agora aceita `POST` (antes era apenas `GET`).

**Payload:**
```json
{
  "catalog_item_id": "uuid",
  "requested_quantity": "5.0000",
  "justification": "Reposição de material"
}
```

| Campo | Tipo | Obrigatório |
|---|---|---|
| `catalog_item_id` | UUID | **sim** |
| `requested_quantity` | decimal | **sim** (deve ser > 0) |
| `justification` | string | não |

> O `unit_value` e `total_value` são preenchidos automaticamente pelo backend com base no `average_unit_value` do estoque do almoxarifado da requisição. Se não houver histórico de preço, o valor será `0.00`.

**Resposta 201:**
```json
{
  "id": "uuid",
  "requisition_id": "uuid",
  "catalog_item_id": "uuid",
  "requested_quantity": "5.0000",
  "approved_quantity": null,
  "fulfilled_quantity": "0.0000",
  "unit_value": "45.50",
  "total_value": "227.50",
  "justification": "Reposição de material",
  "cut_reason": null,
  "created_at": "2026-03-27T14:00:00Z",
  "updated_at": "2026-03-27T14:00:00Z"
}
```

**Erros:**
- `400 Bad Request` — Requisição não está em status `DRAFT` ou `PENDING`
- `404 Not Found` — Requisição não encontrada

---

## 4. Mudanças no comportamento existente

### 4.1 `POST /api/admin/invoices/{id}/post`

**Antes:** O lançamento no estoque ocorria via trigger do PostgreSQL. Se falhava, o erro chegava como `500` com mensagem de banco.

**Agora:** O lançamento é atômico na mesma transação da atualização de status. Erros de estoque (ex: item bloqueado) chegam como `400 Bad Request` com mensagem clara em português.

Comportamento esperado: **idêntico para o front-end**. A NF fica `POSTED` e o estoque é atualizado na mesma operação.

### 4.2 `POST /api/admin/invoices/{id}/cancel` em NF POSTED

**Antes:** O estorno do estoque ocorria via trigger.

**Agora:** O estorno é feito atomicamente: se o estorno falhar, o status não muda. O front-end não precisa tratar nenhum estado inconsistente.

### 4.3 `POST /api/admin/requisitions/{id}/approve`

**Antes:** As reservas de estoque (`stock_reservations`) eram criadas via trigger.

**Agora:** O serviço cria as reservas atomicamente junto com a aprovação. Se não houver estoque suficiente, retorna `400 Bad Request` com mensagem clara.

### 4.4 Campo `items[].adjusted_quantity` e `items[].adjusted_value` em itens de NF

Os itens retornados por `GET /api/admin/invoices/{id}/items` agora incluem dois campos adicionais de **leitura** que agregam as glosas existentes:

```json
{
  "id": "uuid",
  "catalog_item_id": "uuid",
  "quantity_raw": "10.0000",
  "unit_value_raw": "50.0000",
  "total_value": "500.0000",
  "adjusted_quantity": "2.0000",
  "adjusted_value": "100.0000",
  "adjustment_status": "PARTIAL"
}
```

| Campo novo | Tipo | Descrição |
|---|---|---|
| `adjusted_quantity` | decimal\|null | Soma de todas as quantidades glosadas para este item |
| `adjusted_value` | decimal\|null | Soma de todos os valores glosados para este item |
| `adjustment_status` | string\|null | `"PARTIAL"` quando há glosas, `null` quando não há |

Esses campos são `null` quando não existem glosas para o item.

---

## 5. Novas tabelas do banco

Duas novas tabelas foram adicionadas via migration `20260327000001`:

### `invoice_adjustments`
| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | UUID | PK |
| `invoice_id` | UUID | FK para `invoices` |
| `reason` | TEXT | Motivo da glosa |
| `created_by` | UUID | FK para `users` |
| `created_at` | TIMESTAMPTZ | |

### `invoice_adjustment_items`
| Coluna | Tipo | Descrição |
|---|---|---|
| `id` | UUID | PK |
| `adjustment_id` | UUID | FK para `invoice_adjustments` |
| `invoice_item_id` | UUID | FK para `invoice_items` |
| `adjusted_quantity` | DECIMAL | Quantidade glosada |
| `adjusted_value` | DECIMAL | Valor financeiro glosado |
| `notes` | TEXT | Observação livre |
| `created_at` | TIMESTAMPTZ | |

---

## 6. Mudanças nos contratos JSON

### 6.1 `RequisitionItemResponse` — campos renomeados/corrigidos

| Campo anterior | Campo atual | Tipo |
|---|---|---|
| `catmat_item_id: UUID?` | `catalog_item_id: UUID` | não-nullable |
| `catser_item_id: UUID?` | _(removido)_ | — |
| `unit_value: Decimal?` | `unit_value: Decimal` | não-nullable |
| `total_value: Decimal?` | `total_value: Decimal` | não-nullable |
| `fulfilled_quantity: Decimal?` | `fulfilled_quantity: Decimal` | não-nullable |

> **Ação requerida pelo front-end:** Se algum componente verificava `catmat_item_id || catser_item_id`, substituir por `catalog_item_id`. Se tratava `unit_value` como opcional, remover o fallback.

### 6.2 `InvoiceItemWithDetailsDto` — novos campos opcionais

Adicionados ao retorno de `GET /api/admin/invoices/{id}/items`:
- `adjusted_quantity?: Decimal` — soma das glosas de quantidade
- `adjusted_value?: Decimal` — soma das glosas de valor
- `adjustment_status?: string` — `"PARTIAL"` se há glosas

Esses campos são retrocompatíveis (null quando ausentes).

---

## 7. Exemplos de integração

### Fluxo completo de NF com glosa

```javascript
// 1. Lançar NF no estoque
await api.post(`/api/admin/invoices/${invoiceId}/post`)
// → status: "POSTED", estoque atualizado

// 2. Buscar itens para exibir ao usuário
const { items } = await api.get(`/api/admin/invoices/${invoiceId}/items`)
// items[0].adjusted_quantity === null (nenhuma glosa ainda)

// 3. Criar glosa
const adjustment = await api.post(`/api/admin/invoices/${invoiceId}/adjustments`, {
  reason: "Quantidade divergente na conferência física",
  items: [{
    invoice_item_id: items[0].id,
    adjusted_quantity: "2.0000",
    notes: "Devolvido ao fornecedor"
  }]
})
// → status 201, estoque reduzido em 2 unidades

// 4. Recarregar itens
const { items: updatedItems } = await api.get(`/api/admin/invoices/${invoiceId}/items`)
// updatedItems[0].adjusted_quantity === "2.0000"
// updatedItems[0].adjustment_status === "PARTIAL"

// 5. Listar todas as glosas da NF
const adjustments = await api.get(`/api/admin/invoices/${invoiceId}/adjustments`)
// → array com 1 entrada
```

### Adicionar item à requisição

```javascript
const item = await api.post(`/api/admin/requisitions/${reqId}/items`, {
  catalog_item_id: "uuid-do-item-catmat",
  requested_quantity: "5.0000",
  justification: "Reposição mensal"
})
// item.unit_value será preenchido automaticamente com o custo médio do estoque
// item.total_value = unit_value × requested_quantity
```

---

## 8. Erros esperados e seus códigos

| Situação | Código | Mensagem esperada |
|---|---|---|
| Glosa em NF que não é POSTED | `400` | `"Somente notas fiscais POSTED podem receber glosas"` |
| Motivo da glosa vazio | `400` | `"Motivo da glosa é obrigatório"` |
| Glosa sem itens | `400` | `"A glosa deve ter ao menos um item"` |
| Item da glosa com qty=0 e val=0 | `400` | `"Cada item de glosa deve ter quantidade ou valor ajustado > 0"` |
| Item da glosa não pertence à NF | `400` | `"Item X não pertence à nota fiscal Y"` |
| Estoque insuficiente para glosa de qtd | `400` | `"Saldo insuficiente. Disponível: X, Solicitado: Y"` |
| Estoque bloqueado (saídas) | `400` | `"Operação negada: O item está BLOQUEADO neste almoxarifado..."` |
| Adicionar item a requisição não-DRAFT/PENDING | `400` | `"Somente requisições DRAFT ou PENDING aceitam novos itens"` |
| NF não encontrada | `404` | `"Nota fiscal não encontrada"` |
| Requisição não encontrada | `404` | `"Requisição não encontrada"` |
| Token ausente | `401` | — |
| Token sem papel admin | `403` | — |

---

> **Dúvidas?** Consulte o backend-api-guide.md ou abra uma issue no repositório com a tag `frontend`.
