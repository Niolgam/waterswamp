# SIGALM — Contrato de API: Funcionalidades DRS v2.0
## Novas Implementações (Branch: `claude/implement-drs-features-a1Bkn`)

> **Documento de referência para o frontend** sobre as funcionalidades implementadas a partir do DRS-SIGALM-UFMT v2.0.
> Complementa o `sigalm-api-contract.txt` e `glosas-e-refatoracao-triggers.md`.

---

## Sumário de Requisitos Implementados

| RF | Descrição | Endpoint(s) |
|----|-----------|-------------|
| RF-009 | Entrada Avulsa (doação/ajuste) | `POST /warehouses/{id}/entries` |
| RF-011 | Devolução de Requisição | `POST /warehouses/{id}/returns` |
| RF-013 | Status SUSPENDED + Start-Processing | `POST /requisitions/{id}/start-processing` |
| RF-014 | Atendimento (total/parcial) com justificativa | `POST /requisitions/{id}/fulfill` |
| RF-016 | Saída por Desfazimento/Baixa (SEI) | `POST /warehouses/{id}/disposals` |
| RF-017 | Saída por Ordem de Serviço | `POST /warehouses/{id}/manual-exits` |
| RF-018 | Transferência entre Almoxarifados (2 etapas) | Ver seção Transferências |
| — | Histórico de Movimentações | `GET /warehouses/{id}/movements` |

---

## 1. Máquina de Estados — Requisição (atualizada)

```
                    ┌─────────────────────────────────────────┐
                    │             REQUISIÇÃO                   │
                    └─────────────────────────────────────────┘

RASCUNHO (DRAFT)
    │ [Requisitante: submit]
    ▼
PENDENTE (PENDING) ──────────────────────────────────► SUSPENSA (SUSPENDED)
    │                                                       │
    │ [Gestor: approve]                    [Sistema: unidade bloqueada RN-004]
    ▼                                                       │
APROVADA (APPROVED)         [Sistema: unidade desbloqueada] │
    │                                       ◄───────────────┘
    │ [Gestor: start-processing]   ← NOVO
    ▼
EM PROCESSAMENTO (PROCESSING)  ← NOVO
    │
    ├──[Gestor: fulfill parcial]──► PARCIALMENTE ATENDIDA (PARTIALLY_FULFILLED) ──► (terminal)
    │
    └──[Gestor: fulfill total]───► ATENDIDA (FULFILLED) ──► (terminal)

PENDENTE/APROVADA/PROCESSANDO ──[Requisitante ou Gestor: cancel]──► CANCELADA (CANCELLED) ──► (terminal)
PENDENTE ──[Gestor: reject]──► REJEITADA (REJECTED)
REJEITADA ──[Requisitante: resubmit]──► RASCUNHO
```

### Transições por endpoint

| De | Para | Endpoint | Quem |
|----|------|----------|------|
| PENDING | APPROVED | `POST /requisitions/{id}/approve` | Gestor |
| PENDING | REJECTED | `POST /requisitions/{id}/reject` | Gestor |
| **APPROVED** | **PROCESSING** | `POST /requisitions/{id}/start-processing` | Gestor |
| **PROCESSING** | **FULFILLED** | `POST /requisitions/{id}/fulfill` | Gestor |
| **PROCESSING** | **PARTIALLY_FULFILLED** | `POST /requisitions/{id}/fulfill` | Gestor |
| PENDING | SUSPENDED | Sistema automático (RN-004) | — |
| SUSPENDED | PENDING | Sistema automático (RN-004) | — |
| DRAFT/PENDING/APPROVED | CANCELLED | `POST /requisitions/{id}/cancel` | Qualquer |

---

## 2. Novas Rotas de Requisição

### 2.1 Iniciar Processamento

**`POST /api/admin/requisitions/{id}/start-processing`**

Transiciona a requisição de `APPROVED` para `PROCESSING`. Indica que a separação física começou.

**Request Body:**
```json
{
  "notes": "Separação iniciada no corredor B" // opcional
}
```

**Response `200 OK`:**
```json
{
  "id": "uuid",
  "requisition_number": "REQ-2026-000123",
  "status": "PROCESSING",
  "approved_by": "uuid",
  "approved_at": "2026-04-12T10:00:00Z",
  "updated_at": "2026-04-12T10:30:00Z"
}
```

**Erros:**
| Código | Mensagem |
|--------|----------|
| `400` | `Requisição não pode iniciar processamento: status atual é APPROVED. Esperado: APPROVED` — se status não for APPROVED |
| `404` | `Requisição não encontrada` |

---

### 2.2 Atender Requisição (Total ou Parcial)

**`POST /api/admin/requisitions/{id}/fulfill`**

Transiciona de `PROCESSING` para `FULFILLED` ou `PARTIALLY_FULFILLED`.

Gera movimentações `EXIT` no estoque para cada item atendido (via `StockMovementService`).

Libera as reservas de estoque (`stock_reservations`).

**⚠️ Regra RF-014:** quando `fulfilled_quantity < approved_quantity`, o campo `cut_reason` é **obrigatório**.

**Request Body:**
```json
{
  "items": [
    {
      "requisition_item_id": "uuid-item-1",
      "fulfilled_quantity": 5.000,
      "cut_reason": null              // obrigatório se qty < aprovada
    },
    {
      "requisition_item_id": "uuid-item-2",
      "fulfilled_quantity": 3.000,
      "cut_reason": "Saldo insuficiente para quantidade total aprovada"
    }
  ],
  "notes": "Retirada realizada pelo servidor João"  // opcional
}
```

**Response `200 OK`:**
```json
{
  "id": "uuid",
  "requisition_number": "REQ-2026-000123",
  "status": "PARTIALLY_FULFILLED",  // ou "FULFILLED"
  "fulfilled_by": "uuid-gestor",
  "fulfilled_at": "2026-04-12T11:00:00Z",
  "updated_at": "2026-04-12T11:00:00Z"
}
```

**Erros:**
| Código | Mensagem |
|--------|----------|
| `400` | `Requisição não pode ser atendida: status atual é APPROVED. Esperado: PROCESSING` |
| `400` | `Justificativa de corte é obrigatória para atendimento parcial do item {uuid}` |
| `400` | `Quantidade atendida (X) não pode exceder a aprovada (Y) para o item {uuid}` |
| `400` | `Saldo insuficiente. Disponível: X, Solicitado: Y` |

---

## 3. Novas Rotas de Almoxarifado — Entradas

> Todas estas rotas geram movimentações em `stock_movements` via `StockMovementService` com custo médio ponderado (CMP) e pessimistic locking.

### 3.1 Entrada Avulsa (RF-009)

**`POST /api/admin/warehouses/{id}/entries`**

Para doações, cessões, e ajustes de inventário por sobra.

**Request Body:**
```json
{
  "entry_type": "DONATION",              // ou "INVENTORY_ADJUSTMENT"
  "origin_description": "Doação FAPEMAT / CNPJ: 12.345.678/0001-99",
  "document_number": "DOAÇÃO/2026-042", // opcional
  "notes": "Material em bom estado de conservação",  // opcional
  "items": [
    {
      "catalog_item_id": "uuid-catmat-item",
      "unit_raw_id": "uuid-unidade",
      "unit_conversion_id": null,
      "quantity_raw": 10.0,
      "conversion_factor": 1.0,
      "unit_price_base": 25.50,
      "batch_number": "LOTE-2025-001",
      "expiration_date": "2027-06-30",   // null para itens sem validade
      "divergence_justification": null,  // obrigatório se preço divergir > 20% do custo médio
      "item_notes": null
    }
  ]
}
```

**Tipos de `entry_type`:**
| Valor | Tipo de Movimentação | Descrição |
|-------|---------------------|-----------|
| `DONATION` | `DONATION_IN` | Doação de terceiros |
| `INVENTORY_ADJUSTMENT` | `ADJUSTMENT_ADD` | Ajuste de inventário (sobra) |

**⚠️ Validação de preço divergente:** Se `unit_price_base` divergir mais de 20% do `average_unit_value` atual, o campo `divergence_justification` torna-se **obrigatório**.

**Response `201 Created`:**
```json
{
  "movements_created": 1,
  "entry_type": "Donation",
  "origin_description": "Doação FAPEMAT / CNPJ: 12.345.678/0001-99",
  "warehouse_id": "uuid-almoxarifado"
}
```

**Erros:**
| Código | Mensagem |
|--------|----------|
| `400` | `Origem da entrada avulsa é obrigatória` |
| `400` | `Variação de preço > 20% em relação ao custo médio. Informe uma justificativa.` |
| `400` | `Operação negada: O item está BLOQUEADO neste almoxarifado.` |

---

### 3.2 Devolução de Requisição (RF-011)

**`POST /api/admin/warehouses/{id}/returns`**

Para reincorporar itens de uma requisição já atendida ao estoque.

**Request Body:**
```json
{
  "requisition_id": "uuid-requisicao-original",
  "notes": "Material devolvido sem uso — projeto cancelado",
  "items": [
    {
      "catalog_item_id": "uuid-catmat-item",
      "unit_raw_id": "uuid-unidade",
      "unit_conversion_id": null,
      "quantity_raw": 3.0,
      "conversion_factor": 1.0,
      "batch_number": "LOTE-2025-001",
      "expiration_date": "2027-06-30",
      "item_notes": "Estado: conservado"
    }
  ]
}
```

**⚠️ Regras:**
- A requisição referenciada deve ter status `FULFILLED` ou `PARTIALLY_FULFILLED`
- Gera movimentação `RETURN` — usa o custo médio atual (não o preço original)
- O `document_number` é gerado automaticamente como `DEV/{numero_requisicao}`

**Response `201 Created`:**
```json
{
  "movements_created": 1,
  "requisition_id": "uuid-requisicao-original",
  "warehouse_id": "uuid-almoxarifado"
}
```

**Erros:**
| Código | Mensagem |
|--------|----------|
| `400` | `Devolução só é permitida para requisições FULFILLED ou PARTIALLY_FULFILLED.` |
| `404` | `Requisição não encontrada` |

---

## 4. Novas Rotas de Almoxarifado — Saídas

### 4.1 Saída por Desfazimento/Baixa (RF-016)

**`POST /api/admin/warehouses/{id}/disposals`**

Para materiais vencidos, inservíveis ou destinados a desfazimento legal.

**⚠️ Todos os campos são obrigatórios (RN-005):**
- `justification` — razão do desfazimento
- `sei_process_number` — número do processo SEI (RF-039)
- `technical_opinion_url` — URL do Parecer Técnico em PDF

**Request Body:**
```json
{
  "justification": "Produto com validade expirada em 01/03/2026. Inutilizável.",
  "sei_process_number": "23108.012345/2026-07",
  "technical_opinion_url": "https://sei.ufmt.br/documentos/parecer-tecnico-2026-042.pdf",
  "notes": "Descarte conforme normas ambientais ABNT",
  "items": [
    {
      "catalog_item_id": "uuid-catmat-item",
      "unit_raw_id": "uuid-unidade",
      "unit_conversion_id": null,
      "quantity_raw": 5.0,
      "conversion_factor": 1.0,
      "batch_number": "LOTE-2024-003",
      "item_notes": "Lote vencido em 01/03/2026"
    }
  ]
}
```

**Formato do `sei_process_number` (RF-039):**
- Padrão: `NNNNN.NNNNNN/YYYY-NN`
- Exemplos válidos: `23108.012345/2026-07`, `00001.000001/2026-01`

**Response `201 Created`:**
```json
{
  "movements_created": 1,
  "sei_process_number": "23108.012345/2026-07",
  "warehouse_id": "uuid-almoxarifado"
}
```

**Erros:**
| Código | Mensagem |
|--------|----------|
| `400` | `Justificativa é obrigatória para desfazimento (RN-005)` |
| `400` | `URL do Parecer Técnico é obrigatória para desfazimento (RN-005/RF-016)` |
| `400` | `Número de processo SEI inválido. Formato esperado: NNNNN.NNNNNN/YYYY-NN` |
| `400` | `Saldo insuficiente. Disponível: X, Solicitado: Y` |

---

### 4.2 Saída por Ordem de Serviço / Manual (RF-017)

**`POST /api/admin/warehouses/{id}/manual-exits`**

Para saídas por Ordem de Serviço (OS) ou outros motivos manuais que não passam pelo fluxo de requisição.

**Request Body:**
```json
{
  "document_number": "OS-2026-00456",
  "justification": "Manutenção preventiva do laboratório de informática",
  "notes": "Materiais para Bloco A, sala 201",
  "items": [
    {
      "catalog_item_id": "uuid-catmat-item",
      "unit_raw_id": "uuid-unidade",
      "unit_conversion_id": null,
      "quantity_raw": 2.0,
      "conversion_factor": 1.0,
      "batch_number": null,
      "item_notes": "Utilização imediata"
    }
  ]
}
```

**Response `201 Created`:**
```json
{
  "movements_created": 1,
  "document_number": "OS-2026-00456",
  "warehouse_id": "uuid-almoxarifado"
}
```

---

## 5. Transferências entre Almoxarifados (RF-018)

O fluxo de transferência é **em duas etapas** com pessimistic locking (RN-011).

### Máquina de Estados da Transferência

```
           [Gestor Origem]
PENDENTE ◄─── initiate ─── (criado)
    │
    ├──[Gestor Destino: confirm]──► CONFIRMADA (terminal)
    │
    ├──[Gestor Destino: reject] ──► REJEITADA  (terminal — estoque restaurado)
    │
    ├──[Gestor Origem: cancel]  ──► CANCELADA  (terminal — estoque restaurado)
    │
    └──[timeout automático]     ──► EXPIRADA   (terminal — estoque restaurado)
```

### 5.1 Iniciar Transferência (Passo 1)

**`POST /api/admin/warehouses/{source_id}/transfers`**

O gestor do almoxarifado de origem inicia a transferência. O sistema gera movimentação `TRANSFER_OUT` na origem (deduzindo o estoque imediatamente via pessimistic lock).

**Request Body:**
```json
{
  "destination_warehouse_id": "uuid-almox-destino",
  "expires_in_hours": 72,          // opcional — prazo para confirmação (null = sem prazo)
  "notes": "Transferência para reposição emergencial do Setor de Biologia",
  "items": [
    {
      "catalog_item_id": "uuid-catmat-item",
      "unit_raw_id": "uuid-unidade",
      "unit_conversion_id": null,
      "quantity_raw": 20.0,
      "conversion_factor": 1.0,
      "batch_number": "LOTE-2025-001",
      "expiration_date": "2027-03-31",
      "notes": null
    }
  ]
}
```

**Response `201 Created`:**
```json
{
  "transfer": {
    "id": "uuid-transfer",
    "transfer_number": "TRF-2026-000001",
    "source_warehouse_id": "uuid-origem",
    "source_warehouse_name": "Almoxarifado Central",
    "destination_warehouse_id": "uuid-destino",
    "destination_warehouse_name": "Almoxarifado Setorial — Biologia",
    "status": "PENDING",
    "notes": "Transferência para reposição emergencial do Setor de Biologia",
    "initiated_by": "uuid-gestor",
    "initiated_at": "2026-04-12T09:00:00Z",
    "expires_at": "2026-04-15T09:00:00Z",
    "created_at": "2026-04-12T09:00:00Z",
    "updated_at": "2026-04-12T09:00:00Z"
  },
  "items": [
    {
      "id": "uuid-transfer-item",
      "transfer_id": "uuid-transfer",
      "catalog_item_id": "uuid-catmat-item",
      "catalog_item_name": "Papel A4 75g/m2",
      "quantity_requested": 20.0,
      "quantity_confirmed": null,
      "unit_raw_id": "uuid-unidade",
      "unit_symbol": "resma",
      "conversion_factor": 1.0,
      "batch_number": "LOTE-2025-001",
      "expiration_date": "2027-03-31",
      "source_movement_id": "uuid-movement-out"
    }
  ]
}
```

---

### 5.2 Confirmar Recebimento (Passo 2a)

**`POST /api/admin/transfers/{id}/confirm`**

O gestor do almoxarifado de destino confirma o recebimento. Gera movimentação `TRANSFER_IN` no destino. Suporta recebimento parcial.

**Request Body:**
```json
{
  "items": [
    {
      "transfer_item_id": "uuid-transfer-item",
      "quantity_confirmed": 18.0   // pode ser <= quantity_requested
    }
  ],
  "notes": "Recebido 18 resmas. 2 resmas danificadas em trânsito."
}
```

**Response `200 OK`:**
```json
{
  "transfer": {
    "id": "uuid-transfer",
    "status": "CONFIRMED",
    "confirmed_by": "uuid-gestor-destino",
    "confirmed_at": "2026-04-13T14:00:00Z"
  },
  "items": [...]
}
```

**Erros:**
| Código | Mensagem |
|--------|----------|
| `400` | `Transferência não pode ser confirmada. Status atual: CONFIRMED` |
| `400` | `Transferência expirada. Cancele e inicie uma nova.` |
| `400` | `Quantidade confirmada (20) não pode exceder a solicitada (18)` |

---

### 5.3 Rejeitar Transferência (Passo 2b)

**`POST /api/admin/transfers/{id}/reject`**

O gestor do destino rejeita o recebimento. O sistema gera uma movimentação compensatória `TRANSFER_IN` na **origem** para restaurar o estoque.

**Request Body:**
```json
{
  "rejection_reason": "Material com embalagem danificada. Não conforme com especificações."
}
```

**Response `200 OK`:**
```json
{
  "transfer": {
    "id": "uuid-transfer",
    "status": "REJECTED",
    "rejected_by": "uuid-gestor-destino",
    "rejected_at": "2026-04-13T14:00:00Z",
    "rejection_reason": "Material com embalagem danificada."
  }
}
```

---

### 5.4 Cancelar Transferência (pela origem)

**`POST /api/admin/transfers/{id}/cancel`**

O gestor da origem cancela uma transferência pendente. Gera movimentação compensatória `TRANSFER_IN` na origem.

**Request Body:**
```json
{
  "cancellation_reason": "Transferência solicitada por engano. Item não disponível para envio."
}
```

**Response `200 OK`:**
```json
{
  "transfer": {
    "id": "uuid-transfer",
    "status": "CANCELLED",
    "cancelled_by": "uuid-gestor-origem",
    "cancelled_at": "2026-04-12T11:00:00Z",
    "cancellation_reason": "Transferência solicitada por engano."
  }
}
```

---

### 5.5 Listar Transferências

**`GET /api/admin/transfers`**

**Query Parameters:**
| Parâmetro | Tipo | Descrição |
|-----------|------|-----------|
| `limit` | int | Máximo de registros (padrão: 20) |
| `offset` | int | Paginação |
| `source_warehouse_id` | UUID | Filtrar por almoxarifado de origem |
| `destination_warehouse_id` | UUID | Filtrar por almoxarifado de destino |
| `status` | string | `PENDING`, `CONFIRMED`, `REJECTED`, `CANCELLED`, `EXPIRED` |

**Response `200 OK`:**
```json
{
  "data": [...],
  "total": 42,
  "limit": 20,
  "offset": 0
}
```

---

### 5.6 Detalhes de Transferência

**`GET /api/admin/transfers/{id}`**

Retorna a transferência com todos os itens e movimentações associadas.

---

## 6. Histórico de Movimentações

**`GET /api/admin/warehouses/{id}/movements`**

Retorna o histórico completo de movimentações de estoque de um almoxarifado, incluindo entradas, saídas, transferências, ajustes e devoluções.

**Query Parameters:**
| Parâmetro | Tipo | Descrição |
|-----------|------|-----------|
| `limit` | int | Máximo de registros (padrão: 50) |
| `offset` | int | Paginação |
| `catalog_item_id` | UUID | Filtrar por item do catálogo |
| `movement_type` | string | Filtrar por tipo (ver tabela abaixo) |

**Tipos de `movement_type`:**
| Valor | Descrição |
|-------|-----------|
| `ENTRY` | Entrada por Nota Fiscal |
| `EXIT` | Saída por Requisição ou OS |
| `LOSS` | Perda (desfazimento, quebra, vencimento) |
| `RETURN` | Devolução ao almoxarifado |
| `TRANSFER_IN` | Recebimento via transferência |
| `TRANSFER_OUT` | Envio via transferência |
| `ADJUSTMENT_ADD` | Ajuste de inventário (sobra) |
| `ADJUSTMENT_SUB` | Ajuste de inventário (falta/glosa) |
| `DONATION_IN` | Entrada por doação |
| `DONATION_OUT` | Saída por doação |

**Response `200 OK`:**
```json
{
  "data": [
    {
      "id": "uuid-movement",
      "warehouse_id": "uuid-almoxarifado",
      "warehouse_name": "Almoxarifado Central",
      "catalog_item_id": "uuid-item",
      "catalog_item_name": "Papel A4 75g/m2",
      "catalog_item_code": "357036",
      "movement_type": "ENTRY",
      "movement_date": "2026-04-12T09:15:00Z",
      "quantity_raw": 10.0,
      "quantity_base": 50.0,
      "unit_price_base": "25.5000",
      "total_value": "1275.00",
      "balance_before": "100.000",
      "balance_after": "150.000",
      "average_before": "24.8000",
      "average_after": "25.1333",
      "invoice_id": "uuid-nf",
      "requisition_id": null,
      "related_warehouse_id": null,
      "related_warehouse_name": null,
      "document_number": "NF-2026-00042",
      "notes": null,
      "batch_number": "LOTE-2025-001",
      "expiration_date": "2027-03-31",
      "requires_review": false,
      "user_id": "uuid-usuario",
      "user_name": "jsilva",
      "created_at": "2026-04-12T09:15:00Z"
    }
  ],
  "total": 87,
  "limit": 50,
  "offset": 0,
  "warehouse_id": "uuid-almoxarifado"
}
```

---

## 7. Status de Requisição — Valor SUSPENDED (novo)

O status `SUSPENDED` é adicionado ao enum `RequisitionStatus`:

```
"SUSPENDED" — Requisição suspensa automaticamente quando a unidade
              organizacional da requisição recebe status "BLOQUEADO" (RN-004).
              Volta para PENDING quando a unidade é desbloqueada.
```

**Impacto no frontend:**
- Exibir badge visual diferenciado (ex: cor laranja/âmbar) para requisições suspensas
- Exibir mensagem de aviso: "Requisição suspensa — unidade organizacional bloqueada"
- Não exibir ações de aprovação/rejeição para requisições SUSPENDED

---

## 8. Comportamento do Estoque por Tipo de Movimentação

| Tipo | Incrementa saldo? | Custo usado | Efeito em `warehouse_stocks` |
|------|------------------|-------------|------------------------------|
| `ENTRY` | ✅ sim | Preço informado → recalcula CMP | `last_entry_at` |
| `DONATION_IN` | ✅ sim | Preço informado → recalcula CMP | `last_entry_at` |
| `ADJUSTMENT_ADD` | ✅ sim | Preço informado → recalcula CMP | `last_entry_at` |
| `RETURN` | ✅ sim | Custo médio atual (sem alterar CMP) | `last_entry_at` |
| `TRANSFER_IN` | ✅ sim | Custo médio da origem (preservado) | `last_entry_at` |
| `EXIT` | ❌ diminui | Custo médio atual (contabilidade) | `last_exit_at` |
| `LOSS` | ❌ diminui | Custo médio atual | `last_exit_at` |
| `TRANSFER_OUT` | ❌ diminui | Custo médio atual | `last_exit_at` |
| `ADJUSTMENT_SUB` | ❌ diminui | Custo médio atual | `last_exit_at` |
| `DONATION_OUT` | ❌ diminui | Custo médio atual | `last_exit_at` |

**CMP = Custo Médio Ponderado** (recalculado a cada entrada com preço > 0):
```
CMP_novo = (qty_atual × CMP_atual + qty_entrada × preço_entrada) / (qty_atual + qty_entrada)
```

---

## 9. Regras de Segurança e Autorização

Todas as rotas requerem autenticação JWT Bearer.

| Endpoint | Papel mínimo sugerido |
|----------|-----------------------|
| `POST /requisitions/{id}/start-processing` | `gestor_almoxarifado` |
| `POST /requisitions/{id}/fulfill` | `gestor_almoxarifado` |
| `POST /warehouses/{id}/entries` | `gestor_central` |
| `POST /warehouses/{id}/returns` | `gestor_almoxarifado` |
| `POST /warehouses/{id}/disposals` | `gestor_central` |
| `POST /warehouses/{id}/manual-exits` | `gestor_almoxarifado` |
| `POST /warehouses/{id}/transfers` | `gestor_central` |
| `POST /transfers/{id}/confirm` | `gestor_almoxarifado` (destino) |
| `POST /transfers/{id}/reject` | `gestor_almoxarifado` (destino) |
| `POST /transfers/{id}/cancel` | `gestor_central` (origem) |
| `GET /warehouses/{id}/movements` | `gestor_almoxarifado` |

---

## 10. Exemplos de Fluxo Completo

### Fluxo completo de requisição

```
1. Requisitante cria:    POST /requisitions      → DRAFT
2. Requisitante submete: [adiciona itens, muda status manualmente ou via campo]
3. Gestor aprova:        POST /requisitions/{id}/approve         → APPROVED
4. Gestor inicia:        POST /requisitions/{id}/start-processing → PROCESSING
5. Gestor atende:        POST /requisitions/{id}/fulfill          → FULFILLED
   └─ Gera EXIT em stock_movements para cada item
   └─ Libera stock_reservations
```

### Fluxo de transferência com confirmação parcial

```
1. Central inicia:      POST /warehouses/{central_id}/transfers     → TRF-2026-000001 (PENDING)
   └─ Gera TRANSFER_OUT no central (estoque deduzido imediatamente)

2. Setorial confirma:   POST /transfers/{id}/confirm (qty_confirmed = 18, solicitado = 20)
   └─ Gera TRANSFER_IN no setorial (estoque acrescido)
   └─ Status → CONFIRMED
   └─ movements ficam linked via related_movement_id
```

### Fluxo de desfazimento

```
1. Gestor identifica lote vencido via GET /warehouses/{id}/movements?movement_type=ENTRY
2. Anexa Parecer Técnico e obtém número de processo SEI
3. Gestor efetiva: POST /warehouses/{id}/disposals
   └─ Valida formato SEI (regex: \d{5}\.\d{6}/\d{4}-\d{2})
   └─ Gera LOSS no stock_movements
   └─ Notas incluem: SEI, justificativa e URL do Parecer
```

---

## 11. Compatibilidade com Funcionalidades Anteriores

Esta implementação **não altera** nenhum endpoint existente. É compatível com:
- `glosas-e-refatoracao-triggers.md` (InvoiceAdjustmentService, StockMovementService)
- `sigalm-api-contract.txt` (todos os endpoints documentados anteriormente)
- `frontend-warehouse-guide.md` e `frontend-invoice-guide.md`

---

## 12. Requisitos DRS NÃO Implementados (Escopo Futuro)

| RF | Descrição | Motivo |
|----|-----------|--------|
| RF-010 | Entrada por Devolução de OS | Módulo de OS não implementado |
| RF-019 | Inventário Periódico e Rotativo | Módulo complexo independente |
| RF-020 | Política FEFO/FIFO automática | Requer `stock_batches` por lote no `warehouse_stocks` |
| RF-021/RF-022 | Alertas de vencimento + bloqueio automático | Requer job assíncrono periódico |
| RF-025 | Integração Comprasnet | Integração externa — requer credenciais |
| RF-026 | Alertas de estoque mínimo | Requer job diário |
| RF-029 | Notificações email de requisição | Disponível via EmailService — falta wiring |
| RF-036 | Importação CSV/XLSX | Feature independente de alta complexidade |
| RF-040 | Assinatura eletrônica (Gov.br) | Integração externa com Gov.br Assinatura |

---

> **Gerado em:** 2026-04-12
> **Branch:** `claude/implement-drs-features-a1Bkn`
> **Referência DRS:** DRS-SIGALM-UFMT v2.0 (27/03/2026)
