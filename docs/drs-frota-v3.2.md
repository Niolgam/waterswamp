# Documento de Requisitos de Software (DRS)

## Módulo Frota — Plataforma de Gestão UFMT
### Universidade Federal de Mato Grosso

---

| Campo | Valor |
|-------|-------|
| **Versão** | 3.2 |
| **Data** | 20/04/2026 |
| **Status** | Em Revisão |
| **Autor original** | Equipe UFMT |
| **Revisado por** | Análise de Engenharia de Requisitos + Auditoria de Conformidade + Revisão Arquitetural |
| **Aprovado por** | *(Pendente)* |

### Histórico de Revisões

| Versão | Data | Descrição |
|--------|------|-----------|
| 1.0 | — | Versão inicial |
| 2.0 | 27/03/2026 | Decomposição atômica, LGPD, segurança, casos de uso expandidos, rastreabilidade, MoSCoW |
| 3.0 | 27/03/2026 | OCC, SoD, harmonização delegação/escalonamento, terceirizados, relatórios assíncronos, sinistros |
| 3.1 | 19/04/2026 | Validação de condutor até retorno, antecedência por finalidade em horas úteis, abastecimento/manutenção por contrato (CATMAT/CATSER), contingência de dispositivo, material informativo, TLS/GPS/responsável patrimonial como referência futura |
| 3.2 | 20/04/2026 | **Fundação de Integridade:** FSM formal de dois eixos para Veículo (operational_status + allocation_status), FSM de Viagem, protocolo OCC com resolução por criticidade, odômetro como série temporal imutável com hierarquia de confiança, idempotência por Request-ID, Pessimistic Locking para alocação, masking de dados sensíveis em logs, ciclo de vida de dados LGPD, TLS como gate obrigatório de deploy (não mais adiado), versionamento de entidades como RNF de dados |

---

## Sumário

1. Introdução
2. Escopo
3. Glossário, Definições e Siglas
4. **Fundação de Integridade** *(novo)*
5. Visão Geral da Arquitetura Funcional
6. Perfis de Acesso (Papéis)
7. Regras de Negócio (RN)
8. Requisitos Funcionais (RF)
9. Requisitos Não Funcionais (RNF)
10. Casos de Uso Expandidos
11. Matriz de Rastreabilidade
12. Priorização (MoSCoW) e Faseamento
13. Escopo Futuro Condicionado
14. Referências Normativas Futuras
15. Considerações Finais

---

## 1. Introdução

### 1.1. Propósito

Este documento especifica os requisitos de software para o Módulo Frota da Plataforma de Gestão da Universidade Federal de Mato Grosso (UFMT). Constitui a base autoritativa para desenvolvimento, testes, homologação e auditoria da solução.

### 1.2. Base Legal

| Norma | Descrição | Aplicação no Módulo |
|-------|-----------|---------------------|
| **Lei nº 9.327/1996** | Condução de veículo oficial | Credenciamento obrigatório por portaria do dirigente máximo |
| **Decreto nº 9.287/2018** | Utilização de veículos oficiais federais | Classificação, vedações, guarda e identificação |
| **IN SLTI/MPOG nº 3/2008** | Classificação, utilização e alienação de veículos oficiais | Registro obrigatório de condutor, origem, destino, horários, quilometragens |
| **Decreto nº 9.373/2018** | Alienação de bens móveis | Rito de desfazimento patrimonial |
| **Lei nº 13.709/2018 (LGPD)** | Proteção de dados pessoais | CNH, CPF, dados de jornada, localização |
| **CF Art. 150, VI, "a"** | Imunidade tributária recíproca | UFMT é imune ao IPVA (autarquia federal) |

### 1.3. Público-Alvo

Equipe de desenvolvimento, gestores de frota e administradores da UFMT, auditoria interna, órgãos de controle (CGU/TCU), usuários-chave para validação.

### 1.4. Convenções do Documento

- **DEVE/OBRIGATÓRIO**: Mandatório para a fase indicada.
- **DEVERIA**: Fortemente recomendado, negociável com justificativa.
- **PODE**: Desejável, conforme disponibilidade.
- Identificadores: formato `[MÓDULO]-[SEQ]` (ex: `RF-AST-01`). Requisitos atômicos.
- Prioridade: MoSCoW (Must/Should/Could/Won't).

---

## 2. Escopo

### 2.1. Escopo Incluído

- **Gestão patrimonial**: aquisição, registro, transferência, depreciação, desfazimento, sinistros.
- **Controle operacional**: abastecimentos (contrato/importação configurável), seguros, licenciamentos, acessórios.
- **Manutenção**: preventiva, corretiva, OS, inspeções (contrato/importação configurável).
- **Condutores**: credenciamento (Lei 9.327/96) de servidores e terceirizados, treinamentos, avaliação, jornada.
- **Viagens**: solicitação, aprovação, alocação, execução, prestação de contas, cancelamento, extensão, incidentes.
- **Rotas**: cadastro padrão, roteirização simplificada.
- **Multas**: registro, vinculação, recursos, ressarcimento.
- **Indicadores e relatórios**: dashboards, relatórios síncronos/assíncronos.
- **Catálogos**: combustíveis (CATMAT), serviços de manutenção (CATSER).
- **Importação de legados**: migração estruturada.
- **Integrações**: SouGov, SEI, Sistema de Patrimônio.

### 2.2. Escopo Excluído

- Estoque de posto próprio de combustível.
- Integração automática com DETRAN (escopo futuro).
- Aplicativo nativo mobile (PWA offline).
- Gestão financeira/orçamentária.
- Carsharing interno (escopo futuro condicionado — seção 13).
- Telemetria/rastreamento GPS (escopo futuro condicionado — seção 13).
- Gestão de pneus (escopo futuro — seção 13).
- Gestão de pedágios (não há trechos pedagiados nas rotas habituais da UFMT/MT; comprovantes registrados na prestação de contas quando ocorrem).

### 2.3. Dependências Externas

| Dependência | Tipo | Impacto se Indisponível |
|-------------|------|-------------------------|
| SouGov | Autenticação, dados de servidores, afastamentos | Cache local; delegação pode atrasar até próxima sincronização |
| SEI | Documentos formais | Documentos gerados localmente para inserção manual |
| Sistema de Patrimônio UFMT | Dados patrimoniais | Cadastro manual com conciliação posterior |
| CATMAT/CATSER | Referência de catálogos | Catálogo local sincronizado periodicamente |
| Conectividade (áreas rurais) | Operação PWA em campo | Offline com OCC + fila de resolução |

---

## 3. Glossário, Definições e Siglas

### 3.1. Termos do Domínio

| Termo | Definição |
|-------|-----------|
| **Condutor** | Pessoa credenciada conforme Lei 9.327/96 para conduzir veículos da frota. Servidor (com portaria) ou terceirizado (com contrato vigente). |
| **Credenciamento** | Ato administrativo formal (portaria publicada) que autoriza o servidor a conduzir veículos oficiais. Possui validade e requer CNH compatível vigente durante todo o período de uso. |
| **Solicitante** | Servidor que requisita veículo para finalidade institucional. |
| **Gestor de Frota** | Servidor designado por portaria para administrar operacionalmente a frota. |
| **Viagem** | Deslocamento institucional com veículo e condutor alocados, checklists obrigatórios e prestação de contas. |
| **Reserva** | Viagem aprovada com recursos alocados, ainda não iniciada. |
| **OS** | Ordem de Serviço — autoriza e registra manutenção preventiva ou corretiva. |
| **TCO** | Total Cost of Ownership — aquisição + operação + manutenção + depreciação − valor residual. |
| **Fornecedor de Abastecimento** | Empresa contratada para fornecimento de combustível. Motoristas usam cartão de abastecimento do contrato. |
| **Fornecedor de Manutenção** | Empresa contratada para serviços de manutenção veicular. |
| **operational_status** | Eixo 1 do estado do veículo: indica aptidão operacional. Valores: `ATIVO`, `MANUTENCAO`, `INDISPONIVEL`. |
| **allocation_status** | Eixo 2 do estado do veículo: indica vínculo a viagens. Valores: `LIVRE`, `RESERVADO`, `EM_USO`. |
| **OCC** | Optimistic Concurrency Control — versionamento de entidades para detectar conflitos de escrita concorrente. |
| **SoD** | Segregation of Duties — operações críticas exigem dois usuários distintos (propositor ≠ aprovador). |
| **Request-ID** | Identificador único gerado pelo cliente (UUID v4) que acompanha comandos de alteração de estado, garantindo idempotência. |
| **Odômetro Projetado** | Valor atual do hodômetro do veículo, derivado do último registro validado na série temporal de leituras. |
| **Registro de Quarentena** | Leitura de odômetro que apresenta divergência superior aos limites definidos; aceita no sistema mas não efetivada até revisão manual. |

### 3.2. Siglas

| Sigla | Significado |
|-------|-------------|
| CATMAT | Catálogo de Materiais do Governo Federal |
| CATSER | Catálogo de Serviços do Governo Federal |
| CRLV | Certificado de Registro e Licenciamento de Veículo |
| CNH | Carteira Nacional de Habilitação |
| CTB | Código de Trânsito Brasileiro |
| e-MAG | Modelo de Acessibilidade em Governo Eletrônico |
| FSM | Finite State Machine (Máquina de Estado Finito) |
| LGPD | Lei Geral de Proteção de Dados (Lei nº 13.709/2018) |
| MTBF | Mean Time Between Failures |
| MTTR | Mean Time To Repair |
| NF | Nota Fiscal |
| OCC | Optimistic Concurrency Control |
| OS | Ordem de Serviço |
| RBAC | Role-Based Access Control |
| RPO | Recovery Point Objective |
| RTO | Recovery Time Objective |
| SEI | Sistema Eletrônico de Informações |
| SoD | Segregation of Duties |
| SouGov | Sistema de Gestão de Pessoas do Governo Federal |
| TCO | Total Cost of Ownership |
| TTL | Time to Live (tempo de retenção de dados) |

### 3.3. Fórmulas e Regras de Cálculo

**TCO de Veículo:**
```
TCO = Custo_Aquisição
    + Σ(Custos_Combustível)
    + Σ(Custos_Manutenção)
    + Σ(Custos_Seguro_Licenciamento)
    + Σ(Depreciação_Acumulada)
    - Valor_Residual_Estimado
```
- Ausência de dados de aquisição: valor FIPE na data de incorporação.
- Depreciação linear: `Depreciação_Anual = (Valor_Aquisição − Valor_Residual_Mínimo) / Vida_Útil_Anos`.
- Valor Residual Estimado: `Valor_Aquisição − Depreciação_Acumulada`, piso R$ 0,00.

**Consumo Médio:**
```
Consumo_Médio (km/l) = (Odômetro_Atual − Odômetro_Abastecimento_Anterior) / Litros_Abastecidos
```
Onde `Odômetro_Atual` é sempre derivado do último registro **validado** da série temporal (seção 4.3).

**MTBF / MTTR:** Definições padrão. Veículos sem falhas no período: MTBF = tempo total do período.

### 3.4. Regra de Contagem de Horas Úteis

Para cálculo de antecedência mínima (RN07) e prazos operacionais:
- **Horário útil:** segunda a sexta, 08:00–18:00 (UTC-4, horário de Cuiabá).
- **Excluídos:** sábados, domingos, feriados nacionais e feriados locais cadastrados (RF-ADM-01).
- **Exemplo:** Solicitação feita sexta-feira às 19:00 para segunda-feira às 07:00 = 0 horas úteis.

---

## 4. Fundação de Integridade

Esta seção estabelece os contratos de comportamento do sistema ao nível de dados e concorrência. São pré-requisitos arquiteturais para todos os módulos: se a camada aqui descrita não for implementada, os requisitos funcionais acima dela não funcionam corretamente sob carga ou em cenários de conectividade intermitente.

---

### 4.1. Máquinas de Estado Finito (FSM)

#### 4.1.1. Entidade Veículo — Dois Eixos Independentes

A entidade `Veiculo` possui dois campos de estado com semânticas distintas e complementares, armazenados como colunas tipadas separadas na mesma tabela.

**Eixo 1 — `operational_status` (aptidão operacional):**

| Valor | Semântica |
|-------|-----------|
| `ATIVO` | Veículo mecanicamente apto, documentação em dia. |
| `MANUTENCAO` | Bloqueado por OS aberta ou alerta preventivo acionado. |
| `INDISPONIVEL` | Fora de operação — sinistro, processo de baixa, baixa definitiva. |

**Eixo 2 — `allocation_status` (vínculo a viagens):**

| Valor | Semântica |
|-------|-----------|
| `LIVRE` | No pátio, sem vínculo a viagens ativas ou futuras próximas. |
| `RESERVADO` | Vinculado a uma viagem aprovada aguardando check-out. |
| `EM_USO` | Em trânsito — check-out realizado, check-in pendente. |

**Regra de Bloqueio (Domain Rule — RN-FSM-01):**

Uma transição de `allocation_status` para `RESERVADO` ou `EM_USO` **só pode ocorrer** se `operational_status = ATIVO`. Qualquer tentativa de alocação de veículo com `operational_status ≠ ATIVO` DEVE ser rejeitada com HTTP 409 Conflict, body RFC 7807 com `type: "vehicle-not-allocatable"`.

**Tabela de combinações válidas:**

| `operational_status` | `allocation_status` permitido |
|---------------------|-------------------------------|
| `ATIVO` | `LIVRE`, `RESERVADO`, `EM_USO` |
| `MANUTENCAO` | `LIVRE` apenas |
| `INDISPONIVEL` | `LIVRE` apenas |

**Transições de `operational_status`:**

```
ATIVO      → MANUTENCAO    gatilho: abertura de OS (RF-MAN-03) ou acionamento de alerta preventivo
ATIVO      → INDISPONIVEL  gatilho: sinistro (RF-AST-12) ou início de processo de baixa (RF-AST-09)
MANUTENCAO → ATIVO         gatilho: conclusão de OS (RF-MAN-05), com allocation_status = LIVRE
INDISPONIVEL → ATIVO       gatilho: recuperação de sinistro (aprovação do Gestor de Frota)
INDISPONIVEL → [terminal]  gatilho: conclusão do processo de baixa (status BAIXADO — fora da FSM ativa)
```

**Transições de `allocation_status`:**

```
LIVRE      → RESERVADO  gatilho: aprovação de viagem e alocação de veículo (RF-VIG-04)
                         pré-condição: operational_status = ATIVO
RESERVADO  → EM_USO     gatilho: check-out (RF-VIG-09)
                         pré-condição: operational_status = ATIVO
EM_USO     → LIVRE      gatilho: check-in + odômetro validado (RF-VIG-11)
RESERVADO  → LIVRE      gatilho: cancelamento de viagem (RF-VIG-05) ou substituição de veículo (RF-VIG-08)
```

**Nota de implementação:** A validação das transições DEVE ser centralizada na camada de domínio do backend. Nenhum cliente (frontend, PWA, API externa) pode efetuar diretamente uma atualização de status — toda transição passa pela lógica de domínio que verifica as pré-condições. O SQL de atualização DEVE incluir a cláusula `WHERE operational_status = $1 AND allocation_status = $2 AND version = $3`; se `rows_affected = 0`, lançar 409 Conflict.

---

#### 4.1.2. Entidade Viagem — FSM

**Estados:**

| Estado | Semântica |
|--------|-----------|
| `SOLICITADA` | Solicitação criada, aguardando aprovação da chefia. |
| `APROVADA` | Chefia aprovou, aguardando alocação pelo Gestor de Frota. |
| `ALOCADA` | Veículo e condutor alocados, aguardando check-out. |
| `EM_CURSO` | Check-out realizado, viagem em andamento. |
| `AGUARDANDO_PC` | Check-in realizado, aguardando prestação de contas. |
| `CONCLUIDA` | Prestação de contas realizada. Terminal. |
| `CANCELADA` | Cancelada antes de `EM_CURSO`. Terminal. |
| `CONFLITO_MANUAL` | Viagem entrou em conflito de sincronização offline insolúvel automaticamente. Requer intervenção do Gestor de Frota. |

**Transições:**

```
SOLICITADA     → APROVADA         gatilho: aprovação da chefia (RF-VIG-02)
SOLICITADA     → CANCELADA        gatilho: rejeição ou cancelamento (RF-VIG-02, RF-VIG-05)
APROVADA       → ALOCADA          gatilho: alocação de veículo e condutor (RF-VIG-04)
                                   pré-condição: Veículo.operational_status = ATIVO
APROVADA       → CANCELADA        gatilho: cancelamento (RF-VIG-05)
ALOCADA        → EM_CURSO         gatilho: check-out (RF-VIG-09)
                                   pré-condição: Veículo.operational_status = ATIVO AND allocation_status = RESERVADO
ALOCADA        → CANCELADA        gatilho: cancelamento (RF-VIG-05)
EM_CURSO       → AGUARDANDO_PC    gatilho: check-in com odômetro validado (RF-VIG-11)
EM_CURSO       → CONFLITO_MANUAL  gatilho: conflito OCC irresolvível automaticamente (seção 4.2)
AGUARDANDO_PC  → CONCLUIDA        gatilho: prestação de contas confirmada (RF-VIG-12)
CONFLITO_MANUAL → CONCLUIDA       gatilho: resolução manual pelo Gestor de Frota (RF-ADM-06)
CONFLITO_MANUAL → CANCELADA       gatilho: decisão do Gestor de Frota na resolução do conflito
```

**Regra de Interdependência (RN-FSM-02):** A transição `ALOCADA → EM_CURSO` (check-out) DEVE verificar atomicamente, em uma única transação de banco, que:
1. `Viagem.status = ALOCADA`
2. `Veiculo.operational_status = ATIVO AND allocation_status = RESERVADO`
3. `Condutor.credenciamento_status = ATIVO AND cnh_validade > data_retorno_estimado`

Se qualquer condição falhar, a transição é rejeitada com 409 Conflict informando a condição violada.

---

#### 4.1.3. Entidade Condutor — FSM

**Estados:**

| Estado | Semântica |
|--------|-----------|
| `ATIVO` | Credenciamento válido, CNH válida, cursos em dia. Alocável. |
| `SUSPENSO` | Impedimento temporário (CNH vencida, curso vencido, contrato vencido). Não alocável. |
| `PENDENTE_VALIDACAO` | Cadastro criado sem validação SouGov confirmada. |
| `REVOGADO` | Credenciamento encerrado definitivamente. Terminal para o credenciamento vigente. |

**Transições:** conforme RF-CND-03 (seção 8, Módulo 4).

---

### 4.2. Protocolo OCC e Resolução de Conflitos

#### 4.2.1. Versionamento de Entidades

Toda tabela transacional crítica (`veiculos`, `viagens`, `condutores`, `leituras_hodometro`, `abastecimentos`, `ordens_servico`) DEVE possuir:
- Coluna `version INTEGER NOT NULL DEFAULT 1` — incrementada a cada escrita.
- Coluna `updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` — precisão de microssegundos.

Todo comando de atualização DEVE incluir:
```sql
UPDATE tabela
SET campo = $novo_valor, version = version + 1, updated_at = NOW()
WHERE id = $id AND version = $version_esperada;
```
Se `rows_affected = 0`: lançar HTTP 409 Conflict com body RFC 7807 `type: "optimistic-lock-failure"`.

#### 4.2.2. Classificação de Conflitos

| Classe | Definição | Resolução |
|--------|-----------|-----------|
| **Crítico — Bloqueante** | Dois atores tentam mudar o mesmo campo de estado para valores mutuamente exclusivos (ex: dois check-outs do mesmo veículo). | **Rejeita** o segundo conflito. Sistema gera alerta imediato ao Gestor de Frota. Viagem do segundo condutor → `CONFLITO_MANUAL`. A realidade física (quem está com a chave) prevalece; o sistema garante que os dados não colidam. |
| **Não-Crítico — Mergeable** | Atualização de campos descritivos independentes no mesmo registro (ex: observações do checklist). | **Last-Write-Wins** com base no `updated_at` mais recente. Log de auditoria preserva ambas as versões. |
| **Odômetro Divergente** | Nova leitura de odômetro com divergência acima dos limites (seção 4.3). | Registro aceito em **quarentena**. Não atualiza o `Odômetro_Projetado` do veículo até revisão manual (RF-ADM-06). |

#### 4.2.3. Cenário Crítico: Double Booking Offline

**Situação:** Condutor A e Condutor B, ambos offline, tentam iniciar viagem com o mesmo veículo (V1). No cache local de ambos, V1 estava `RESERVADO` para cada um deles.

**Resolução:**
1. O pacote de Condutor A chega primeiro ao servidor. Servidor executa a transição `RESERVADO → EM_USO` com validação de pré-condição e versão. Sucesso. `version` de V1 incrementada.
2. Condutor B sincroniza. Servidor detecta `version` divergente para V1. A pré-condição `allocation_status = RESERVADO AND version = $v_b` falha.
3. Sistema rejeita a transição de B com 409. Viagem de B → `CONFLITO_MANUAL`. Alerta imediato ao Gestor de Frota com detalhes de ambas as viagens.
4. Gestor resolve via RF-ADM-06: cancela a viagem de B ou realoca outro veículo, preservando ambas as versões no log de auditoria.

---

### 4.3. Odômetro como Série Temporal Imutável

#### 4.3.1. Modelo de Dados

O hodômetro não é um campo atualizável. É um **log cronológico de eventos de leitura**. A tabela `leituras_hodometro` é append-only:

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `id` | UUID | Identificador único da leitura |
| `veiculo_id` | UUID | FK para veículo |
| `valor_km` | NUMERIC(10,1) | Leitura em km |
| `fonte` | ENUM | Origem da leitura (ver hierarquia abaixo) |
| `referencia_id` | UUID | FK para viagem, abastecimento ou OS que originou a leitura |
| `coletado_em` | TIMESTAMPTZ | Momento real da coleta (pode ser retroativo) |
| `registrado_em` | TIMESTAMPTZ | Momento da gravação no servidor |
| `status` | ENUM | `VALIDADO`, `QUARENTENA`, `REJEITADO` |
| `request_id` | UUID | Idempotência (seção 4.4) |
| `version` | INTEGER | OCC (seção 4.2.1) |

O `Odômetro_Projetado` do veículo é sempre a projeção da leitura com maior `valor_km` entre os registros com `status = VALIDADO`.

#### 4.3.2. Hierarquia de Confiança das Fontes

Quando duas leituras para o mesmo veículo em instantes próximos divergirem, a fonte de maior peso prevalece:

| Peso | Fonte (`fonte`) | Descrição |
|------|-----------------|-----------|
| 1 (maior) | `CHECKIN_GESTOR` | Check-in validado e confirmado pelo Gestor de Frota |
| 2 | `CHECKIN_CONDUTOR` | Check-in preenchido pelo condutor no PWA |
| 3 | `CHECKOUT_CONDUTOR` | Check-out do condutor |
| 4 | `ABASTECIMENTO_IMPORTACAO` | Leitura proveniente de planilha do fornecedor contratado |
| 5 (menor) | `ABASTECIMENTO_MANUAL` | Registro manual pelo condutor |

Em caso de divergência entre fontes de pesos diferentes: a de maior peso define o `Odômetro_Projetado`; a de menor peso é gravada com `status = QUARENTENA` para auditoria.

#### 4.3.3. Regras de Validação de Leituras

| Condição | Ação |
|----------|------|
| `valor_km > Odômetro_Projetado_atual` | `status = VALIDADO`. Atualiza `Odômetro_Projetado`. |
| `valor_km < Odômetro_Projetado_atual` (regressão) | `status = QUARENTENA`. Alerta ao Gestor de Frota. Não atualiza projetado. |
| Diferença entre leituras incompatível com tempo da viagem (salto irreal: > velocidade média de 200 km/h para o intervalo de tempo) | `status = QUARENTENA`. Alerta ao Gestor de Frota. |
| Divergência < 1% do total de km da viagem entre duas fontes distintas | Aceitar fonte de maior peso. Registrar divergência menor no log de auditoria sem quarentena. |
| Divergência ≥ 1% | `status = QUARENTENA` para a fonte de menor peso. Intervenção manual. |

Registros em quarentena suspendem o cálculo de alertas de manutenção preventiva para o veículo até resolução.

---

### 4.4. Idempotência de Comandos

Todo comando que altera estado de entidade crítica (check-out, check-in, início/conclusão de OS, alocação de veículo) DEVE:

1. **Carregar um `Request-ID`** (UUID v4) gerado pelo cliente (PWA ou frontend) no cabeçalho HTTP `Idempotency-Key`.
2. O servidor armazena `(Request-ID, resultado)` em tabela de idempotência com TTL de 24 horas.
3. Se o servidor receber o mesmo `Request-ID` novamente (retry por falha de rede): retorna o resultado da primeira operação sem reprocessar a lógica de transição de estado.
4. Operações de importação em lote (abastecimentos, manutenções) usam o hash SHA-256 do arquivo como chave de idempotência. A re-importação do mesmo arquivo retorna o resultado da importação original.

**Nota de implementação:** O `Request-ID` é gerado pelo cliente, não pelo servidor. Isso garante que mesmo em cenários offline o cliente possa construir um ID único antes de ter conectividade, e reenviar sem risco de duplicação quando conectar.

---

### 4.5. Estratégia de Locking

O sistema utiliza dois mecanismos complementares conforme a criticidade da operação:

| Operação | Mecanismo | Justificativa |
|----------|-----------|---------------|
| **Alocação de veículo** (RF-VIG-04) | **Pessimistic Locking** — `SELECT ... FOR UPDATE NOWAIT` na linha do veículo | Operação de baixa frequência mas alto impacto. Previne double-booking sem overhead de retry. `NOWAIT` retorna 409 imediatamente se a linha estiver travada. |
| **Todas as outras escritas** | **Optimistic Locking** — `version` + `rows_affected = 0` → 409 | Escritas frequentes (checklists, abastecimentos, atualizações). OCC tem menor overhead em cenários de baixa contenção. |
| **Sincronização offline** | OCC com resolução por criticidade (seção 4.2.2) | Conflitos offline não podem usar Pessimistic Lock (sem conexão no momento da operação). |

**Dimensionamento:** Para uma frota de até 500 veículos, o Pessimistic Lock na alocação é viável sem degradação. A fila de pico esperada (≤ 5 alocações simultâneas) é absorvida pelo mecanismo sem necessidade de mensageria. A escalabilidade deve focar em índices e queries eficientes antes de dispersão arquitetural.

---

### 4.6. Ciclo de Vida de Dados e LGPD

#### 4.6.1. Classificação de Dados Pessoais

| Categoria | Dados | Base Legal (LGPD) | TTL / Retenção |
|-----------|-------|-------------------|----------------|
| **Identificação** | Nome, matrícula, CPF, e-mail | Execução de políticas públicas (Art. 7º, III) | Duração do vínculo + 5 anos |
| **Habilitação** | Número CNH, categoria, validade | Obrigação legal (Art. 7º, II) — Lei 9.327/96 | Duração do credenciamento + 5 anos |
| **Jornada** | Horários de saída/retorno, horas de direção | Execução de políticas públicas | 5 anos (tabela de temporalidade UFMT) |
| **Localização** (futuro — telemetria) | Coordenadas GPS durante viagem | Execução de políticas públicas + legítimo interesse | Granular (por minuto): 90 dias. Agregado (polígono de rota): 5 anos. Exceto quando vinculado a sindicância: prazo do processo. |
| **Infrações** | Multas, pontuação CNH | Obrigação legal | 5 anos após quitação/encerramento |

#### 4.6.2. Masking de Dados Sensíveis

- **Logs de aplicação:** CPF e número de CNH DEVEM ser mascarados antes de qualquer escrita em log (ex: `CPF: 123.456.***-**`, `CNH: ******.***`). Logs nunca conterão esses valores em texto claro.
- **APIs de baixo privilégio:** Endpoints acessíveis pelos papéis REQUESTER e DRIVER DEVEM retornar CPF e CNH mascarados. Papéis FLEET_MGR, AUDITOR, SYS_ADMIN recebem dados completos.
- **Exportações e relatórios:** Relatórios exportados para CSV/XLSX DEVEM aplicar masking para dados de CPF quando o exportador não for AUDITOR ou SYS_ADMIN.

#### 4.6.3. Anonimização Automática

Conforme RF-SEC-06: condutores desligados (servidores) ou com contrato encerrado (terceirizados) têm dados pessoais anonimizados após 2 anos. Campos anonimizados: CPF, telefone, endereço, foto da CNH. Preservados: matrícula (hash), registros de viagens (para auditoria histórica), dados agregados de desempenho.

---

### 4.7. Criptografia em Trânsito — Gate de Produção

A criptografia TLS é requisito obrigatório de implantação em produção, não um requisito opcional ou adiado.

**Especificação:**
- TLS 1.2 mínimo; TLS 1.3 recomendado.
- Certificados válidos emitidos por autoridade certificadora reconhecida (ex: Let's Encrypt, ICP-Brasil).
- HSTS com `max-age` mínimo de 1 ano.

**Gate de deploy:** O pipeline de CI/CD DEVE incluir verificação de certificado TLS como etapa obrigatória antes do deploy em produção. Deploy sem TLS válido DEVE ser bloqueado automaticamente.

**Responsabilidade:** A implementação de TLS é responsabilidade da equipe de infraestrutura da UFMT, não da equipe de desenvolvimento da aplicação. A equipe de desenvolvimento entrega a aplicação preparada para TLS; a infraestrutura provisiona os certificados. O prazo de implantação em produção é condicionado à disponibilidade do certificado.

---

## 5. Visão Geral da Arquitetura Funcional

| # | Módulo | Código | Descrição |
|---|--------|--------|-----------|
| 1 | Gestão de Ativos | AST | Ciclo de vida patrimonial, depreciação, sinistros |
| 2 | Gestão de Insumos | INS | Abastecimentos (contrato/importação), seguros, licenciamentos |
| 3 | Gestão de Manutenção | MAN | Preventiva, corretiva, OS, inspeções (contrato/importação) |
| 4 | Gestão de Condutores | CND | Credenciamento (Lei 9.327/96), treinamentos, avaliação, jornada |
| 5 | Gestão de Rotas | ROT | Rotas padrão, roteirização simplificada |
| 6 | Gestão de Viagens | VIG | Solicitação, aprovação, alocação, execução, prestação de contas |
| 7 | Gestão de Multas | MLT | Registro, vinculação, recursos, ressarcimento |
| 8 | Indicadores e Relatórios | IND | Dashboards, relatórios síncronos/assíncronos |
| 9 | Administração e Catálogos | ADM | Parametrização, catálogos CATMAT/CATSER, importação, notificações |
| T | Transversal: Segurança, Auditoria, LGPD | SEC | Autenticação, RBAC, SoD, auditoria, privacidade, FSM |

---

## 6. Perfis de Acesso (Papéis)

| Papel | Código | Descrição |
|-------|--------|-----------|
| Administrador do Sistema | SYS_ADMIN | Configuração técnica, parametrização. Sujeito a SoD. Acesso a dados completos (sem masking). |
| Gestor de Frota | FLEET_MGR | Gestão operacional, alocação, conflitos OCC, quarentena de odômetro. Acesso completo. |
| Gestor de Manutenção | MAINT_MGR | OS, importação de manutenções, solicitação corretiva. |
| Chefia de Departamento | DEPT_HEAD | Aprovação hierárquica, relatórios departamentais. Dados mascarados. |
| Condutor | DRIVER | Operação em campo: checklists, abastecimentos, avarias. Dados próprios sem masking; dados de outros: mascarados. |
| Solicitante | REQUESTER | Requisição, acompanhamento, cancelamento. Dados mascarados. |
| Gestor de Patrimônio | ASSET_MGR | Aquisições, transferências, baixas. |
| Auditor | AUDITOR | Consulta irrestrita somente-leitura. Segundo aprovador em SoD. Acesso a dados completos. |

### 6.1. Restrições de Segregação de Funções (SoD)

| Operação Crítica | Propositor | Aprovador | Restrição |
|------------------|-----------|-----------|-----------|
| Alteração de Chassi/Renavam | SYS_ADMIN | AUDITOR ou outro SYS_ADMIN | Propositor ≠ Aprovador |
| Correção de odômetro (quarentena → validado) | FLEET_MGR | SYS_ADMIN ou AUDITOR | Propositor ≠ Aprovador |
| Exclusão de log de auditoria | — | — | **Proibido para todos os papéis** |

---

## 7. Regras de Negócio (RN)

### FSM e Integridade de Estado

**RN-FSM-01 — Bloqueio de Alocação por Estado Operacional:** Transição de `allocation_status` para `RESERVADO` ou `EM_USO` exige `operational_status = ATIVO`. Violação retorna HTTP 409. Detalhe na seção 4.1.1.

**RN-FSM-02 — Atomicidade do Check-Out:** A transição `ALOCADA → EM_CURSO` verifica atomicamente as três condições da seção 4.1.2. Qualquer falha retorna 409 com indicação da condição violada.

**RN-FSM-03 — Manutenção Bloqueia Alocação:** Abertura de OS ou acionamento de alerta preventivo transiciona `operational_status → MANUTENCAO`. Se o veículo estava `RESERVADO` ou `EM_USO`, o Gestor de Frota é notificado imediatamente para resolução (RF-VIG-13). A transição de `allocation_status` para `LIVRE` ocorre apenas na conclusão da OS.

### Regras de Condutores e Credenciamento

**RN01 — Obrigatoriedade de Registro:** Toda utilização de veículo da frota DEVE ser registrada como viagem. Nenhum veículo sai sem registro ativo.

**RN02 — Validação de Condutor até Data de Retorno:** Alocação bloqueada se qualquer item vencer durante o período previsto (saída → retorno): credenciamento (portaria), CNH, cursos obrigatórios, contrato de terceirizado. Sistema indica o item impeditivo e a data de vencimento.

**RN03 — Limite de Jornada:** Máximo 8h contínuas ou 10h diárias. Viagens que excedam: segundo condutor obrigatório.

### Regras de Aprovação e Fluxo

**RN04 — Aprovação Hierárquica:** Toda viagem requer aprovação da chefia imediata antes da alocação.

**RN05 — Delegação de Aprovação:** Sistema sincroniza afastamentos do SouGov. Titular afastado com substituto ativo → roteamento direto ao substituto. Registrado no log.

**RN06 — Escalonamento por Inércia:** 2 dias úteis sem resposta do aprovador efetivo → escalonamento ao superior com checkbox de ciência obrigatório.

**RN07 — Antecedência Mínima por Finalidade (horas úteis):**

| Finalidade | Antecedência |
|------------|-------------|
| Atividade administrativa | 48h úteis |
| Aula de campo | 72h úteis |
| Evento institucional | 72h úteis |
| Viagem interestadual | 96h úteis |
| Emergência/Urgência | Sem mínimo — justificativa + aprovação direta do Gestor |

Valores configuráveis (RF-ADM-01).

### Regras de Agendamento e Disponibilidade

**RN08 — Conflito de Agendamento:** Impede alocação de veículo ou condutor com sobreposição. Margem configurável (padrão: 1h entre viagens).

**RN09 — Indisponibilidade por Status:** Veículos com `operational_status ≠ ATIVO` não são listados como disponíveis.

**RN10 — Suspensão Preventiva de Reservas:** Transição para `MANUTENCAO` ou `INDISPONIVEL`: sinalizar reservas futuras (≤ 30 dias), notificar envolvidos, resolução em 1 dia útil.

### Regras de Execução e Prestação de Contas

**RN11 — Prestação de Contas Obrigatória:** 3 dias úteis → alerta. 5 dias úteis → pendência no prontuário. Veículo só volta a `LIVRE` após check-in com odômetro validado.

**RN12 — Finalidade Institucional:** Destinos fora de locais institucionais cadastrados sinalizados para revisão.

### Regras de Auditoria e Segurança

**RN13 — Inviolabilidade do Log:** Logs não podem ser alterados/excluídos por nenhum perfil. Retenção mínima: 5 anos.

**RN14 — Desfazimento Patrimonial:** Rito Decreto 9.373/2018. Cada etapa vinculada ao SEI.

**RN15 — Ressarcimento de Multas:** Processo administrativo: notificação formal, 10 dias úteis para defesa, decisão fundamentada.

**RN16 — Dupla Custódia para Dados Críticos:** Alterações em Chassi/Renavam e resolução de quarentena de odômetro exigem dois usuários distintos (tabela SoD seção 6.1).

### Regras de Cálculo

**RN17 — TCO:** Recalculado a cada novo custo e mensalmente (depreciação). Fórmula: seção 3.3.

**RN18 — Alertas de Consumo:** Alerta quando desvio negativo > 15% da média semestral da categoria. Média recalculada mensalmente.

**RN19 — Projeção de Manutenção Preventiva:** Gatilho = menor entre km, tempo ou horas de uso. Alertas de manutenção preventiva são **suspensos** para veículos com odômetro em quarentena (seção 4.3.3) até resolução manual.

**RN20 — Depreciação:** Linear, por categoria, cessa ao atingir valor residual mínimo ou na baixa.

### Regras de Importação

**RN21 — Validação Pré-Commit:** Importações de abastecimento e manutenção: validação completa antes de efetivar. Cupons/NFs duplicados bloqueados. Commit explícito após revisão do Gestor.

### Regras de Sincronização Offline

**RN22 — OCC Offline:** Version token em toda entidade sincronizável. Conflitos → classificação por criticidade (seção 4.2.2). Ambas versões preservadas até resolução.

**RN23 — Bloqueio Local de Alocação Duplicada:** PWA armazena cache de alocações vigentes. Bloqueio preventivo se veículo constar alocado a outro condutor.

**RN24 — Contingência por Perda de Dispositivo:** Gestor preenche dados retroativamente via desktop com BO obrigatório e flag "Contingência" em todos os registros.

---

## 8. Requisitos Funcionais (RF)

### Módulo 1: Gestão de Ativos (AST)

**RF-AST-01 — Cadastrar Veículo** `[Must]`
- **Entradas obrigatórias:** Placa, Chassi, Renavam, Marca, Modelo, Ano Fab./Mod., Cor, Categoria (CTB), Tipo de Combustível (catálogo CATMAT — RF-ADM-07), Departamento, Número de Patrimônio.
- **Entradas opcionais:** Capacidade passageiros/carga, Potência, Dados de aquisição (NF, valor, data, processo).
- **Validações:** Chassi, Renavam e Placa únicos.
- **Saída:** Veículo com `operational_status = ATIVO`, `allocation_status = LIVRE`, `version = 1`. Depreciação configurada conforme categoria (RF-AST-11).
- **RNs:** RN20

**RF-AST-02 — Editar Dados de Veículo** `[Must]`
- **Editáveis livremente:** Cor, Departamento, Capacidade, Especificações.
- **Dupla custódia (RN16):** Chassi, Renavam, Placa.
- **Saída:** Versão incrementada. Log com diff anterior/posterior.

**RF-AST-03 — Consultar Veículos** `[Must]`
Filtros: placa, marca, modelo, `operational_status`, `allocation_status`, departamento, categoria, km. Exibe ambos os eixos de estado. Lista paginada com TCO atual e `Odômetro_Projetado`.

**RF-AST-04 — Gerenciar Anexos de Veículo** `[Must]`
Upload/visualização/substituição: CRLV, NF, Apólices, Laudos, Fotos. PDF/JPG/PNG (máx. 10MB). Versão histórica preservada.

**RF-AST-05 — Gerenciar Transições de Estado do Veículo** `[Must]`
Implementa a FSM da seção 4.1.1. Toda transição validada por pré-condições de estado + versão (OCC). Transições permitidas, pré-condições e gatilhos: conforme seção 4.1.1.
- Ao transicionar `operational_status → MANUTENCAO` ou `INDISPONIVEL`: dispara RN10 (RF-VIG-13).
- **RNs:** RN-FSM-01, RN09, RN10

**RF-AST-06 — Registrar Transferência de Departamento** `[Must]`
Entradas: Origem, Destino, Data efetiva, Motivo, Documento SEI. Histórico preservado.

**RF-AST-07 — Registrar Identificação Visual** `[Should]`
Adesivação: tipo, datas, fornecedor, fotos. Alerta 30 dias antes do vencimento.

**RF-AST-08 — Calcular Projeção de Substituição** `[Should]`
Ranking: idade, `Odômetro_Projetado`, TCO, valor residual, manutenções corretivas no último ano.

**RF-AST-09 — Iniciar Processo de Baixa** `[Must]`
Justificativa, Destino, Laudo (obrigatório). `operational_status → INDISPONIVEL`, `allocation_status → LIVRE`. Depreciação suspensa. Se viagens futuras: exige resolução (RN10). **RNs:** RN09, RN10, RN14, RN20

**RF-AST-10 — Registrar Etapas do Desfazimento** `[Must]`
Cada etapa com documento SEI. Conclusão: veículo sai da FSM ativa → base histórica. **RN:** RN14

**RF-AST-11 — Configurar e Calcular Depreciação** `[Must]`
Por categoria: vida útil, valor residual mínimo, método linear. Cálculo mensal. **RNs:** RN17, RN20

**RF-AST-12 — Registrar Sinistro** `[Must]`
Entradas: Tipo, Data/hora, Local, BO (obrigatório), Nº seguradora, Fotos. `operational_status → INDISPONIVEL`, `allocation_status → LIVRE`. Depreciação suspensa. Alerta seguradora se apólice vigente. Dispara RN10. **RNs:** RN09, RN10

### Módulo 2: Gestão de Insumos (INS)

**RF-INS-01 — Registrar Abastecimento Manual** `[Must]`
- **Entradas:** Veículo, Data, Hora, Local, Combustível (catálogo CATMAT), Quantidade, Valor Unitário, Valor Total, Leitura do Odômetro, Condutor, Cupom fiscal, Centro de custo.
- **Validações:** Leitura validada conforme seção 4.3.3. Veículo `operational_status = ATIVO`. Cupom único.
- **Saída:** Registro na série temporal `leituras_hodometro` (`fonte = ABASTECIMENTO_MANUAL`). Consumo médio e TCO recalculados se leitura validada.
- **RNs:** RN17, RN18, RN22

**RF-INS-02 — Registrar Abastecimento Retroativo** `[Must]`
Mesmas entradas de RF-INS-01 + justificativa. Aprovação do Gestor. Recálculo na ordem cronológica correta.

**RF-INS-03 — Resolver Quarentena de Odômetro** `[Must]`
Interface para o Gestor de Frota revisar leituras em quarentena. Ações: Validar (promove para `VALIDADO` após SoD — RN16) ou Rejeitar (mantém em quarentena com motivo). **RN:** RN16

**RF-INS-04 — Configurar Fornecedor de Abastecimento** `[Must]`
Razão social, CNPJ, Contrato, Vigência. Configuração de importação: formato (CSV/XLSX/TXT), encoding, separador, mapeamento de colunas, formato de data e numérico.

**RF-INS-05 — Importar Abastecimentos de Fornecedor** `[Must]`
Upload → mapeamento (RF-INS-04) → validação pré-commit (RN21) incluindo série temporal de odômetro → relatório → revisão → commit. `Request-ID` = SHA-256 do arquivo (idempotência — seção 4.4). **RNs:** RN17, RN18, RN21

**RF-INS-06 — Emitir Alertas de Consumo** `[Must]`
Alerta ao Gestor quando consumo desvia conforme RN18 ou gasto departamental excede limite. **RN:** RN18

**RF-INS-07 — Registrar Aplicação de Insumos** `[Must]`
Insumos consumíveis (óleos, filtros, baterias, fluidos — exceto pneus). Leitura de odômetro registrada na série temporal (`fonte = ABASTECIMENTO_MANUAL`). Próxima troca projetada. **RN:** RN19

**RF-INS-08 — Registrar Apólice de Seguro** `[Must]`
Seguradora, Nº apólice, Cobertura, Prêmio, Vigência, Franquia, Documento. Alerta 30 dias antes. TCO atualizado. Acionamento em sinistro.

**RF-INS-09 — Registrar Licenciamento e Taxas** `[Must]`
Taxa de licenciamento anual, taxa de vistoria (quando exigível). *(UFMT é imune ao IPVA — CF Art. 150 VI "a". DPVAT/SPVAT extinto.)* Entradas: Veículo, Tipo, Exercício, Valor, Comprovante. Alertas de vencimento.

### Módulo 3: Gestão de Manutenção (MAN)

**RF-MAN-01 — Configurar Plano Preventivo** `[Must]`
Por categoria: tipo de serviço (catálogo RF-ADM-08), intervalos (km, tempo, horas), checklist. **RN:** RN19

**RF-MAN-02 — Gerar OS Preventiva Automaticamente** `[Must]`
Ao atingir gatilho: OS `PROGRAMADA`, notificação ao Gestor de Manutenção. `operational_status → MANUTENCAO` ao confirmar a OS. **Suspensão:** Se `Odômetro_Projetado` em quarentena, alerta preventivo por km é suspenso (RN19). **RN:** RN19, RN-FSM-03

**RF-MAN-03 — Solicitar Manutenção Corretiva** `[Must]`
**Condutores, Gestores de Manutenção e Gestores de Frota** DEVEM poder abrir OS corretiva.
- Entradas: Veículo, Descrição, Urgência (Baixa/Média/Alta/Crítica), Imagens (≤ 5), Localização.
- Saída: OS `PENDENTE_AVALIACAO`. Urgência Crítica: `operational_status → MANUTENCAO` imediato + notificação ao Gestor de Frota.
- `Request-ID` obrigatório (idempotência). Disponível offline via PWA (OCC).

**RF-MAN-04 — Gerenciar Ciclo de Vida da OS** `[Must]`
FSM: `PROGRAMADA → PENDENTE_AVALIACAO → APROVADA → EM_EXECUCAO → AGUARDANDO_PECA → CONCLUIDA | CANCELADA`. Cada transição: responsável + timestamp + OCC (version). `operational_status` do veículo retorna a `ATIVO` somente na conclusão da OS (`CONCLUIDA`), com `allocation_status` restabelecido para `LIVRE`.

**RF-MAN-05 — Registrar Execução de OS** `[Must]`
Tipo (interna/externa), Oficina, Orçamentos, Peças (catálogo RF-ADM-08 + quantidade + valor), Mão de obra, Valor total, Datas, NF, Garantia. Leitura de odômetro na conclusão (fonte `ABASTECIMENTO_MANUAL`) registrada na série temporal. Para externas: justificativa de escolha quando múltiplos orçamentos. Saída: OS `CONCLUIDA`, TCO recalculado.

**RF-MAN-06 — Configurar Fornecedor de Manutenção** `[Must]`
Mesma mecânica de RF-INS-04: Razão social, CNPJ, Contrato, Vigência, tipos de serviço cobertos. Configuração de importação por fornecedor.

**RF-MAN-07 — Importar Manutenções de Fornecedor** `[Must]`
Upload → mapeamento → validação pré-commit (RN21) → relatório → confirmação → efetivação. OS criadas/concluídas automaticamente. Leituras de odômetro na série temporal. `Request-ID` = SHA-256 do arquivo. **RNs:** RN17, RN21

**RF-MAN-08 — Acionar Garantia** `[Should]`
OS vinculada à original, flag `GARANTIA`, prazo verificado automaticamente.

**RF-MAN-09 — Realizar Inspeção com Checklist** `[Must]`
Formulários configuráveis. Itens: Conforme/Não Conforme/N/A. Item reprovado → opção de gerar OS corretiva.

### Módulo 4: Gestão de Condutores (CND)

**RF-CND-01 — Cadastrar Condutor Servidor** `[Must]`
Vinculação SouGov (autopreenchimento). CNH (Número, Categoria, Validades). Número e data da portaria de credenciamento (Lei 9.327/96), validade. Status: `ATIVO`, tipo `SERVIDOR`. Fallback SouGov: flag `PENDENTE_VALIDACAO`.

**RF-CND-02 — Cadastrar Condutor Terceirizado** `[Must]`
CPF único, CNH, Empresa, Nº contrato, Vigência, Anexo contrato, Anexo CNH. Aprovação manual do Gestor. Status: `ATIVO`, tipo `TERCEIRIZADO`. Alerta 30 dias antes de vencimento. Suspensão automática ao vencer.

**RF-CND-03 — Gerenciar Status de Condutor** `[Must]`
FSM (seção 4.1.3). Suspensão automática: CNH vencida, curso obrigatório vencido, contrato vencido (terceirizados), portaria vencida. Notificação 30 dias antes.

**RF-CND-04 — Registrar Treinamento** `[Must]`
Cursos, carga horária, validade, certificado. Alerta 60 dias antes. **RN:** RN02

**RF-CND-05 — Registrar Avaliação de Desempenho** `[Should]`
Critérios configuráveis, notas, média ponderada, histórico.

**RF-CND-06 — Manter Prontuário** `[Must]`
Infrações, acidentes (responsabilidade apurada), advertências, pontuação CNH. Alerta ≥ 15 pontos.

**RF-CND-07 — Controlar Jornada de Direção** `[Must]`
Calculada a partir de check-out/check-in. Alertas: 7h contínuas (preventivo), 9h diárias (preventivo de limite). **RN:** RN03

### Módulo 5: Gestão de Rotas (ROT)

**RF-ROT-01 — Cadastrar Rota Padrão** `[Must]`
Nome, Origem, Destino, Pontos intermediários, Distância estimada, Tempo estimado, Orientações, Pontos de referência para locais não mapeados (aldeias, fazendas).

**RF-ROT-02 — Sugerir Roteirização Simplificada** `[Could]`
Ordenação por distância euclidiana com reordenação manual. Inserção de coordenadas para locais não mapeados.

### Módulo 6: Gestão de Viagens (VIG)

**RF-VIG-01 — Solicitar Viagem** `[Must]`
Entradas: Datas/horas (saída e retorno previstos), Origem, Destino(s), Rota padrão, Nº passageiros, Lista de passageiros, Carga, Finalidade (determina antecedência — RN07), Justificativa, Documentos. Autopreenchimento via SouGov. Antecedência calculada em horas úteis (seção 3.4). **RNs:** RN01, RN04, RN05, RN07, RN12

**RF-VIG-02 — Aprovar/Rejeitar Viagem** `[Must]`
Aprovador efetivo (titular ou substituto — RN05). Aprovação ou rejeição (justificativa obrigatória). Escalonamento (RN06): checkbox de ciência obrigatório. **RNs:** RN04, RN05, RN06

**RF-VIG-03 — Analisar Disponibilidade** `[Must]`
Filtra veículos por: `operational_status = ATIVO`, `allocation_status = LIVRE`, capacidade compatível, sem conflito de agendamento (RN08). Exibe ambos os eixos de estado na lista. **RNs:** RN08, RN09

**RF-VIG-04 — Alocar Veículo e Condutor** `[Must]`
- Implementa **Pessimistic Lock** (`SELECT ... FOR UPDATE NOWAIT`) na linha do veículo (seção 4.5).
- Valida FSM (RN-FSM-01): `operational_status = ATIVO`.
- Valida condutor até data de retorno (RN02).
- Valida conflito de agendamento (RN08).
- Transição atômica: `allocation_status → RESERVADO` com OCC (version).
- Se `NOWAIT` retornar lock collision → 409 com mensagem "Veículo sendo alocado simultaneamente. Tente novamente."
- **RNs:** RN-FSM-01, RN02, RN08

**RF-VIG-05 — Cancelar Viagem** `[Must]`
Cancelável em qualquer estado pré-execução. Motivo obrigatório. Libera `allocation_status → LIVRE`. Log de auditoria.

**RF-VIG-06 — Alterar Viagem Aprovada** `[Must]`
Alteração de data/destino → re-aprovação. Se nova data de retorno afeta validade de documentos do condutor (RN02): re-validação e alerta.

**RF-VIG-07 — Estender Duração de Viagem** `[Must]`
Nova data de retorno + justificativa. Verificação de conflito com próxima reserva. Re-validação de documentos do condutor (RN02). OCC offline.

**RF-VIG-08 — Substituir Veículo ou Condutor** `[Must]`
Novo recurso DEVE atender mesmos requisitos. Pessimistic Lock na alocação do novo veículo. Histórico de substituição preservado.

**RF-VIG-09 — Preencher Checklist de Saída** `[Must]`
Km inicial (inserido na série temporal como `fonte = CHECKOUT_CONDUTOR`), combustível, estado (checklist configurável), confirmação de chave. Transição atômica: `allocation_status → EM_USO` + `Viagem → EM_CURSO` com validação das três condições (RN-FSM-02). `Request-ID` obrigatório. Offline com OCC. **RNs:** RN11, RN-FSM-02, RN22

**RF-VIG-10 — Registrar Intercorrência** `[Must]`
Tipos: Acidente, Pane, Apreensão, Saúde, Clima, Outro. Descrição, Localização, Fotos, BO. Pane → OS automática. Sinistro → RF-AST-12. Acidente → prontuário do condutor. `Request-ID` obrigatório. Offline com OCC.

**RF-VIG-11 — Preencher Checklist de Retorno** `[Must]`
Km final inserido na série temporal (`fonte = CHECKIN_CONDUTOR`). Validação conforme seção 4.3.3: se validado, `allocation_status → LIVRE`. Se quarentena, `allocation_status` permanece `EM_USO` até resolução manual. Avarias → OS automática. `Request-ID` obrigatório. **RNs:** RN11, RN22

**RF-VIG-12 — Realizar Prestação de Contas** `[Must]`
Upload comprovantes, custo real vs. estimado, `Viagem → CONCLUIDA`. **RN:** RN11

**RF-VIG-13 — Suspender Reservas por Indisponibilidade** `[Must]`
Dashboard destacado com reservas afetadas (≤ 30 dias). Resolução em 1 dia útil. **RN:** RN10

**RF-VIG-14 — Preencher Dados por Contingência (Perda de Dispositivo)** `[Must]`
Gestor de Frota preenche retroativamente via desktop. BO obrigatório. Flag "Contingência" em todos os registros. Leituras de odômetro inseridas com `fonte = CHECKIN_GESTOR` (peso máximo na hierarquia). **RNs:** RN24

### Módulo 7: Gestão de Multas (MLT)

**RF-MLT-01 — Registrar Auto de Infração** `[Must]`
Vinculação automática via data/hora vs. viagens ativas. Se não identificável: direcionado ao Gestor de Frota para apuração manual.

**RF-MLT-02 — Notificar Condutor** `[Must]`
Notificação formal, data de ciência, prazo de defesa iniciado (RN15). **RN:** RN15

**RF-MLT-03 — Registrar Defesa/Recurso** `[Must]`
Texto, documentos, data de protocolo. Status → `EM_RECURSO`.

**RF-MLT-04 — Registrar Decisão e Ressarcimento** `[Must]`
Decisão, fundamentação, documento. Se responsável: processo de ressarcimento. **RN:** RN15

**RF-MLT-05 — Controlar Prazos** `[Must]`
Alertas 5 dias antes de: indicação de condutor, defesa, recurso, pagamento com desconto, vencimento.

### Módulo 8: Indicadores e Relatórios (IND)

**RF-IND-01 — Dashboard de Frota** `[Must]`
Cache 5 min. Exibe ambos os eixos de estado do veículo (`operational_status` e `allocation_status`). Adicionalmente: taxa de disponibilidade, idade média, TCO, valor contábil, leituras em quarentena pendentes, conflitos OCC pendentes. Filtros: Departamento, Período, Categoria.

**RF-IND-02 — Dashboard de Manutenção** `[Must]`
MTBF/MTTR, custo total, adesão preventiva, OS por status/antiguidade, Top 10 custo.

**RF-IND-03 — Dashboard de Consumo** `[Must]`
Km/l por veículo/categoria, custo por km, ranking eficiência, alertas ativos.

**RF-IND-04 — Dashboard de Viagens** `[Must]`
Nível de serviço, destinos frequentes, ocupação da frota, tempo médio solicitação→execução, prestações de contas pendentes, aprovações escalonadas.

**RF-IND-05 — Relatórios Síncronos** `[Must]`
Período ≤ 30 dias. PDF/XLSX/CSV. Dados de CPF/CNH mascarados conforme papel do exportador (seção 4.6.2).

**RF-IND-06 — Relatórios Assíncronos** `[Must]`
Período > 30 dias. Background worker, notificação, download (7 dias). Mesmo masking de RF-IND-05.

**RF-IND-07 — Agendar Relatórios Recorrentes** `[Could]`
Agendamento automático com envio por e-mail.

### Módulo 9: Administração e Catálogos (ADM)

**RF-ADM-01 — Parametrizar Sistema** `[Must]`
Inclui: antecedência por finalidade (tabela RN07), finalidades configuráveis, calendário de feriados (nacionais + locais), limites de consumo, intervalos preventivos, depreciação por categoria, prazos de aprovação, margem entre viagens, limites de jornada, prazos de prestação de contas, antecedências de alertas, TTL de relatórios assíncronos.

**RF-ADM-02 — Gerenciar Templates de Checklist** `[Must]`
Criar, editar, versionar por categoria. Itens: tipo resposta, obrigatoriedade.

**RF-ADM-03 — Importar Dados Legados** `[Must]`
XLSX por template. Validação → revisão → confirmação → rollback em 48h. Carga inicial de Chassi/Renavam dispensa SoD. Odômetros importados entram como `CHECKIN_GESTOR` na série temporal.

**RF-ADM-04 — Gerenciar Notificações** `[Must]`
Motor centralizado. Canais: in-app, e-mail. Tipos incluem: quarentena de odômetro pendente, conflito OCC pendente (estado `CONFLITO_MANUAL`). Templates editáveis.

**RF-ADM-05 — Gerenciar Acessórios** `[Should]`
Vincular equipamentos ao veículo (extintor, triângulo, etc.), validades, verificação no checklist.

**RF-ADM-06 — Resolver Conflitos de Sincronização e Quarentenas** `[Must]`
Interface unificada para o Gestor de Frota resolver:
- **Conflitos OCC:** Versões em conflito lado a lado, ações: aceitar servidor / aceitar dispositivo / mesclar. Ambas versões preservadas no log.
- **Quarentenas de odômetro:** Leitura em quarentena, valor atual projetado, ações: Validar (SoD — RN16) ou Rejeitar com motivo.
- Viagens em `CONFLITO_MANUAL`: Resolver alocação, realocando veículo ou cancelando. **RNs:** RN22, RN16

**RF-ADM-07 — Gerenciar Catálogo de Combustíveis** `[Must]`
Descrição, Código CATMAT, Centro de custo padrão, Status. Disponível em RF-INS-01 e RF-AST-01.

**RF-ADM-08 — Gerenciar Catálogo de Serviços de Manutenção** `[Must]`
Descrição específica (ex: "Troca pastilha de freio 2º eixo L.D."), Código CATSER (referência genérica), Categoria do serviço, Centro de custo padrão, Status.

**RF-ADM-09 — Gerar Material Informativo para Veículos** `[Should]`
PDF/adesivo: QR Code do PWA, procedimentos resumidos (saída, retorno, abastecimento, avaria), contato do setor de frota. Formatos: A5 (porta-luvas) ou adesivo (painel).

### Módulo Transversal: Segurança, Auditoria e LGPD (SEC)

**RF-SEC-01 — Autenticar Usuário** `[Must]`
SSO via SouGov. Fallback: autenticação local + modo contingência. Timeout 30 min. Sessão única por usuário (novo login invalida anterior).

**RF-SEC-02 — Autorizar por Papel (RBAC)** `[Must]`
Permissões declaradas por papel (seção 6). Múltiplos papéis com restrições SoD. Masking aplicado conforme papel (seção 4.6.2).

**RF-SEC-03 — Implementar Dupla Custódia (SoD)** `[Must]`
Conforme tabela SoD (seção 6.1). Propositor → justificativa → notificação → aprovador → efetivação. Log completo com ambos os atores. **RN:** RN16

**RF-SEC-04 — Registrar Trilha de Auditoria** `[Must]`
Para todas as escritas: usuário, timestamp UTC (μs), IP/dispositivo, ação, diff, version anterior/posterior, `Request-ID` da operação, participantes de SoD, flag de contingência, classificação de conflito OCC resolvido. **Masking obrigatório nos campos de log:** CPF e CNH nunca em texto claro. Inviolável. Retenção: 5 anos. **RN:** RN13

**RF-SEC-05 — Gerenciar Consentimento LGPD** `[Must]`
Termo de Uso + Política de Privacidade no primeiro acesso. Registra aceite com timestamp. Portal do titular: consulta, retificação, exportação de dados próprios.

**RF-SEC-06 — Anonimizar Dados de Desligados** `[Must]`
2 anos após desligamento/encerramento de contrato. Campos anonimizados: CPF, telefone, endereço, foto CNH. Preservados: matrícula (hash), viagens (auditoria), dados agregados. Irreversível + log.

---

## 9. Requisitos Não Funcionais (RNF)

### Integridade de Dados

**RNF-01 — Versionamento de Entidades** `[Must]`
Toda tabela transacional crítica (`veiculos`, `viagens`, `condutores`, `leituras_hodometro`, `abastecimentos`, `ordens_servico`) DEVE possuir colunas `version INTEGER NOT NULL DEFAULT 1` e `updated_at TIMESTAMPTZ NOT NULL`. Atualizações sem correspondência de versão DEVEM retornar HTTP 409 Conflict (body RFC 7807).

**RNF-02 — Idempotência de Comandos de Estado** `[Must]`
Comandos que alteram estado de entidades críticas DEVEM suportar `Idempotency-Key` (UUID v4). O servidor armazena `(key, resultado)` por 24h. Re-envio do mesmo `Idempotency-Key` retorna resultado original sem reprocessar. Importações em lote: idempotência por SHA-256 do arquivo.

**RNF-03 — Série Temporal de Odômetro** `[Must]`
A tabela `leituras_hodometro` é append-only. Nenhuma linha existente pode ser atualizada ou excluída. O `Odômetro_Projetado` é derivado (não armazenado diretamente), calculado como máximo valor validado. Índice obrigatório: `(veiculo_id, status, coletado_em DESC)`.

### Integração e Interoperabilidade

**RNF-04 — APIs de Integração** `[Must]`
APIs RESTful (OpenAPI 3.0) para: SouGov (SSO + afastamentos), SEI, Patrimônio UFMT, CATMAT/CATSER. Fallback por dependência (seção 2.3). Paginação cursor-based em todas as listagens (máx. 100/página). Respostas de erro no formato RFC 7807.

### Segurança

**RNF-05 — TLS em Produção (Gate Obrigatório)** `[Must]`
TLS 1.2 mínimo (1.3 recomendado). Certificado válido de autoridade reconhecida. HSTS `max-age` ≥ 1 ano. **O deploy em produção DEVE ser bloqueado pelo pipeline CI/CD se o certificado TLS não estiver configurado e válido.** Responsabilidade de provisão: equipe de infraestrutura UFMT.

**RNF-06 — Proteção contra Vulnerabilidades** `[Must]`
OWASP Top 10. Headers obrigatórios: CSP, HSTS, X-Content-Type-Options, X-Frame-Options. Rate limiting em autenticação e APIs.

**RNF-07 — Masking em Logs e APIs** `[Must]`
CPF e CNH NUNCA em texto claro em logs de aplicação. Mascaramento antes de qualquer escrita de log. APIs retornam dados mascarados para papéis REQUESTER, DRIVER, DEPT_HEAD. Exportações CSV/XLSX aplicam masking para exportadores sem papel AUDITOR ou SYS_ADMIN.

**RNF-08 — Criptografia em Repouso** `[Must]`
Dados sensíveis (CNH, CPF, dados de jornada) com AES-256 ou equivalente.

**RNF-09 — Gestão de Sessão** `[Must]`
Timeout 30 min (configurável). Sessão única. Token com entropia ≥ 128 bits. Invalidação no logout/troca de senha.

### Performance e Locking

**RNF-10 — Tempo de Resposta** `[Must]`
- Consultas simples: ≤ 2s (p95)
- Escrita: ≤ 3s (p95)
- Alocação com Pessimistic Lock: ≤ 500ms (p95) — `NOWAIT` retorna imediatamente em caso de colisão
- Relatórios síncronos (≤ 30 dias): ≤ 30s
- Relatórios assíncronos: background worker, máx. 10 min
- Dashboards: cache 5 min

**RNF-11 — Capacidade** `[Must]`
200 usuários simultâneos, 500 veículos, 2.000 condutores, 50.000 viagens/ano, 500GB anexos, 100 relatórios assíncronos simultâneos.

**RNF-12 — Índices Obrigatórios** `[Must]`
Além do índice de série temporal (RNF-03), o schema DEVE incluir índices em:
- `veiculos(operational_status, allocation_status)` — consultas de disponibilidade
- `viagens(status, data_saida_prevista)` — listagens por estado e período
- `condutores(status, cnh_validade)` — validações de alocação
- `leituras_hodometro(veiculo_id, status, coletado_em DESC)` — projeção de odômetro

### Disponibilidade e Recuperação

**RNF-13 — Disponibilidade** `[Must]`
99,5% uptime mensal (excluindo manutenção programada: domingos 02:00–06:00 UTC-4).

**RNF-14 — Backup e Recuperação** `[Must]`
RPO: 1h. RTO: 4h. Backup completo diário + incremental horário. Restore testado mensalmente. Logs de auditoria e série temporal de odômetro incluídos.

### Usabilidade

**RNF-15 — Usabilidade** `[Must]`
Solicitação padrão: ≤ 3 passos. Responsivo: desktop 1024px+, mobile 360px+.

**RNF-16 — PWA com OCC** `[Must]`
Offline: agenda, checklists, abastecimentos, avarias, intercorrências. OCC (seção 4.2). Bloqueio local de alocação duplicada (seção 4.2.3). Timestamp original preservado na sincronização. `Request-ID` gerado pelo cliente antes da conexão.

### Acessibilidade e Compatibilidade

**RNF-17 — Conformidade e-MAG** `[Must]`
e-MAG 3.1 + WCAG 2.1 nível AA (Decreto 5.296/2004, IN 1/2014).

**RNF-18 — Navegadores** `[Must]`
Chrome, Firefox, Edge (últimas 2 versões). Desktop: 1024×768. Mobile: 360×640.

### Observabilidade

**RNF-19 — Monitoramento** `[Should]`
Health check `/health`. Logging JSON estruturado (com masking). Métricas: tempo de resposta, taxa de erro, tamanho da fila de relatórios, conflitos OCC pendentes, leituras em quarentena. Alertas: indisponibilidade, erro > 1%, resposta > 5s.

---

## 10. Casos de Uso Expandidos

### UC01 — Solicitar e Aprovar Viagem

| Campo | Descrição |
|-------|-----------|
| **Atores** | Solicitante, Chefia (ou Substituto), Gestor de Frota, Condutor |
| **Pré-condições** | Solicitante autenticado. ≥ 1 veículo com `operational_status = ATIVO`. |
| **Pós-condições** | Viagem `ALOCADA`. Veículo `RESERVADO`. Todos notificados. |

**Fluxo Principal:**
1. Solicitante cria solicitação, seleciona finalidade institucional.
2. Sistema calcula antecedência em **horas úteis** (seção 3.4) conforme RN07. Se insuficiente: bloqueia ou exige justificativa de urgência.
3. Confirmação → `Viagem = SOLICITADA`. Aprovador identificado (RN05).
4. Aprovador aprova → `APROVADA`. Notificação Gestor de Frota.
5. Gestor abre análise de disponibilidade: filtra `operational_status = ATIVO AND allocation_status = LIVRE`.
6. Gestor seleciona veículo e condutor. Sistema executa Pessimistic Lock (`SELECT ... FOR UPDATE NOWAIT`).
7. Validação atômica: FSM (RN-FSM-01) + RN02 (condutor até data de retorno) + RN08 (conflito agendamento).
8. Se válido: `allocation_status → RESERVADO` com OCC. `Viagem → ALOCADA`. Notificações.

**Fluxos Alternativos:**
- **FA01 — Rejeição:** Justificativa → `CANCELADA` → notificação.
- **FA02 — Lock collision na alocação (passo 6):** Dois gestores tentam alocar o mesmo veículo simultaneamente. Segundo gestor recebe 409 "Veículo sendo alocado simultaneamente. Tente novamente." Espera < 500ms.
- **FA03 — Documento do condutor vence durante viagem (passo 7):** 409 com "CNH do condutor [Nome] vence em [Data], anterior ao retorno previsto [Data]." Gestor seleciona outro condutor.
- **FA04 — Escalonamento (RN06):** 2 dias úteis sem resposta → superior + checkbox de ciência obrigatório.
- **FA05 — SouGov offline:** Autopreenchimento manual + flag. Aprovador: cache de afastamentos; se vazio, roteia ao titular.

---

### UC02 — Executar Viagem

| Campo | Descrição |
|-------|-----------|
| **Atores** | Condutor, Gestor de Frota |
| **Pré-condições** | `Viagem = ALOCADA`. Veículo `operational_status = ATIVO, allocation_status = RESERVADO`. |
| **Pós-condições** | `Viagem = CONCLUIDA`. Veículo `LIVRE`. Odômetro atualizado na série temporal. |

**Fluxo Principal:**
1. Condutor acessa viagem no PWA → "Iniciar Viagem".
2. Preenche checklist de saída. Sistema gera `Request-ID` (UUID v4) localmente.
3. Ao submeter (online ou offline): validação atômica RN-FSM-02. Se online: `allocation_status → EM_USO`, `Viagem → EM_CURSO`. Se offline: registro local com OCC + `Request-ID`.
4. Condutor realiza deslocamento.
5. Preenche checklist de retorno. Informa km final.
6. Leitura km inserida na série temporal (`CHECKIN_CONDUTOR`). Validação seção 4.3.3.
7. Se validada: `allocation_status → LIVRE`, `Viagem → AGUARDANDO_PC`.
8. Se quarentena: `allocation_status` permanece `EM_USO`, alerta ao Gestor de Frota.
9. Prestação de contas → `CONCLUIDA`.

**Fluxos Alternativos:**
- **FA01 — Odômetro em quarentena (passo 7):** Gestor revisa via RF-ADM-06 com SoD. Ao validar: `allocation_status → LIVRE` liberado.
- **FA02 — Pane/Sinistro:** Intercorrência (RF-VIG-10) → OS ou RF-AST-12. Se OS crítica: `operational_status → MANUTENCAO`. Gestor resolve reservas via RF-VIG-13.
- **FA03 — Double booking offline (passo 3):** Condutor B recebe 409 ao sincronizar. `Viagem_B → CONFLITO_MANUAL`. Gestor resolve via RF-ADM-06.
- **FA04 — Perda de dispositivo:** Condutor informa o Gestor. Gestor preenche tudo via RF-VIG-14 com BO obrigatório. Km inserido como `CHECKIN_GESTOR` (peso máximo).

**Exceções:**
- **EX01 — Retry de check-out offline:** PWA reenvio com mesmo `Request-ID` → servidor retorna resultado da primeira operação sem reprocessar.

---

### UC03 — Registrar Abastecimento

| Campo | Descrição |
|-------|-----------|
| **Atores** | Gestor de Frota (importação), Condutor (manual) |
| **Pós-condições** | Leitura de odômetro na série temporal. Consumo médio e TCO recalculados (se leitura validada). |

**Fluxo Principal (importação — cenário mais comum):**
1. Fornecedor envia planilha periódica.
2. Gestor faz upload. `Request-ID` = SHA-256 do arquivo.
3. Sistema aplica mapeamento (RF-INS-04).
4. Validação pré-commit (RN21): campos, série temporal de odômetro, duplicidade de cupom.
5. Relatório: válidos, rejeitados (motivo por linha), divergências de odômetro (quarentena).
6. Gestor revisa, confirma → efetivação.

**Fluxo Alternativo (manual):**
- Condutor registra via PWA (RF-INS-01). Mesmo fluxo de validação, individual. `Request-ID` gerado pelo PWA.

**Nota operacional:** O fluxo principal de registro é a importação da planilha do fornecedor. O registro manual é complementar (emergência, posto fora da rede contratada). Condutores eventuais não são esperados para registrar abastecimentos manualmente.

---

### UC04 — Manutenção Corretiva

| Campo | Descrição |
|-------|-----------|
| **Atores** | Condutor, Gestor de Manutenção, Gestor de Frota |
| **Pós-condições** | OS `CONCLUIDA`. `operational_status → ATIVO`. TCO recalculado. Odômetro na série temporal. |

**Fluxo Principal:**
1. **Condutor, Gestor de Manutenção ou Gestor de Frota** abre OS (RF-MAN-03) com `Request-ID`.
2. Se Urgência Crítica: `operational_status → MANUTENCAO` imediato (dispara RN-FSM-03 + RN10).
3. Gestor de Manutenção avalia e aprova.
4. Execução → conclusão (RF-MAN-05). Leitura de odômetro na série temporal.
5. OS `CONCLUIDA` → `operational_status → ATIVO`, `allocation_status → LIVRE`.

**Fluxo Alternativo (importação):**
- Fornecedor envia planilha. Gestor importa via RF-MAN-07. OS criadas/concluídas automaticamente. Mesmo `Request-ID` = SHA-256 do arquivo.

---

### UC05 — Processo de Baixa
*(Fluxo conforme v3.1, estados atualizados para notação FSM)*

`operational_status → INDISPONIVEL`, `allocation_status → LIVRE` ao iniciar. Conclusão: veículo sai da FSM ativa → base histórica.

---

### UC06 — Registrar Sinistro
*(Fluxo conforme v3.1, estados atualizados para notação FSM)*

`operational_status → INDISPONIVEL`, `allocation_status → LIVRE`. Depreciação suspensa. Reservas futuras → RF-VIG-13.

---

## 11. Matriz de Rastreabilidade

### RN → RF

| RN | Requisitos Funcionais |
|----|----------------------|
| RN-FSM-01 | RF-AST-05, RF-VIG-03, RF-VIG-04 |
| RN-FSM-02 | RF-VIG-09 |
| RN-FSM-03 | RF-MAN-02, RF-MAN-03 |
| RN01 | RF-VIG-01 |
| RN02 | RF-VIG-04, RF-VIG-06, RF-VIG-07, RF-CND-01, RF-CND-02, RF-CND-03 |
| RN03 | RF-VIG-04, RF-CND-07, RF-ADM-01 |
| RN04 | RF-VIG-01, RF-VIG-02 |
| RN05 | RF-VIG-01, RF-VIG-02, RF-ADM-04 |
| RN06 | RF-VIG-02, RF-ADM-04, RF-ADM-01 |
| RN07 | RF-VIG-01, RF-ADM-01 |
| RN08 | RF-VIG-03, RF-VIG-04 |
| RN09 | RF-AST-05, RF-VIG-03 |
| RN10 | RF-AST-05, RF-AST-12, RF-VIG-13 |
| RN11 | RF-VIG-11, RF-VIG-12, RF-ADM-04 |
| RN12 | RF-VIG-01 |
| RN13 | RF-SEC-04 |
| RN14 | RF-AST-09, RF-AST-10 |
| RN15 | RF-MLT-01 a RF-MLT-05 |
| RN16 | RF-AST-02, RF-INS-03, RF-SEC-03, RF-ADM-06 |
| RN17 | RF-INS-01, RF-MAN-05, RF-AST-08, RF-AST-11 |
| RN18 | RF-INS-06, RF-ADM-01 |
| RN19 | RF-MAN-01, RF-MAN-02, RF-INS-07 |
| RN20 | RF-AST-01, RF-AST-08, RF-AST-09, RF-AST-11, RF-AST-12 |
| RN21 | RF-INS-05, RF-MAN-07 |
| RN22 | RF-INS-01, RF-VIG-09, RF-VIG-10, RF-VIG-11, RF-MAN-03, RF-ADM-06 |
| RN23 | RNF-16 (implementação PWA) |
| RN24 | RF-VIG-14 |

### RNF de Integridade → RF

| RNF | Requisitos Funcionais Impactados |
|-----|----------------------------------|
| RNF-01 (Versionamento) | Todos os RF de escrita (RF-AST, RF-INS, RF-MAN, RF-VIG, RF-CND) |
| RNF-02 (Idempotência) | RF-VIG-09, RF-VIG-10, RF-VIG-11, RF-MAN-03, RF-INS-05, RF-MAN-07 |
| RNF-03 (Série Temporal) | RF-INS-01, RF-INS-05, RF-INS-07, RF-MAN-05, RF-VIG-09, RF-VIG-11, RF-VIG-14, RF-ADM-03 |

### RF → UC

| Caso de Uso | RFs Principais |
|-------------|----------------|
| UC01 — Solicitar e Aprovar | RF-VIG-01 a 04, RF-SEC-01, RF-ADM-04 |
| UC02 — Executar Viagem | RF-VIG-05, RF-VIG-07 a 14, RF-INS-01, RF-ADM-06 |
| UC03 — Registrar Abastecimento | RF-INS-01 a 06, RF-ADM-06 |
| UC04 — Manutenção Corretiva | RF-MAN-03 a 09, RF-VIG-13 |
| UC05 — Processo de Baixa | RF-AST-09, RF-AST-10, RF-AST-05, RF-VIG-13 |
| UC06 — Registrar Sinistro | RF-AST-12, RF-VIG-10, RF-VIG-13, RF-INS-08, RF-ADM-06 |

---

## 12. Priorização (MoSCoW) e Faseamento

### Fase 1 — MVP (Meses 1–8)
**Must Have:**
A **Fundação de Integridade (seção 4) é pré-requisito não negociável** para todos os módulos funcionais da Fase 1. Deve ser implementada antes dos módulos acima dela.

- **Fundação:** RNF-01, RNF-02, RNF-03 (versionamento, idempotência, série temporal de odômetro)
- **AST:** RF-AST-01 a 06, 09, 10, 11, 12
- **INS:** RF-INS-01, 02, 03, 04, 05, 06, 07, 08, 09
- **MAN:** RF-MAN-01 a 07, 09
- **CND:** RF-CND-01 a 04, 06, 07
- **ROT:** RF-ROT-01
- **VIG:** RF-VIG-01 a 05, 09, 11, 12, 13, 14
- **MLT:** RF-MLT-01, 02, 05
- **IND:** RF-IND-01, 05
- **ADM:** RF-ADM-01 a 04, 06, 07, 08
- **SEC:** RF-SEC-01 a 05
- **RNFs:** RNF-01 a 19 (todos Must)

### Fase 2 — Consolidação (Meses 9–14)
**Should Have:**
- AST: RF-AST-07, 08
- MAN: RF-MAN-08
- CND: RF-CND-05
- VIG: RF-VIG-06, 07, 08, 10
- MLT: RF-MLT-03, 04
- IND: RF-IND-02, 03, 04, 06
- ADM: RF-ADM-05, 09
- SEC: RF-SEC-06
- RNFs: RNF-04 (SEI + Patrimônio), RNF-19

### Fase 3 — Evolução (Meses 15–18)
**Could Have:**
- ROT: RF-ROT-02
- IND: RF-IND-07
- Integração com DETRAN

---

## 13. Escopo Futuro Condicionado

### Carsharing Interno `[Won't nesta versão]`
**Pré-requisito:** Sistema de portaria com integração digital ou processo formal de custódia de chaves. Tratamento transitório: deslocamentos locais seguem fluxo padrão de viagem.

### Telemetria e Rastreamento GPS `[Won't nesta versão]`
**Pré-requisito:** Aquisição e instalação de rastreadores GPS, contrato com provedor de telemetria. Quando implementado, a tabela `leituras_hodometro` já suporta a fonte `TELEMETRIA` (a ser adicionada ao ENUM). A política de TTL de dados de geolocalização (90 dias granular, 5 anos agregado) está definida na seção 4.6.1.

### Gestão de Pneus `[Won't nesta versão]`
**Escopo futuro:** Cadastro individual por número de série/DOT, vinculação veículo↔posição↔pneu, rodízio, recapagem (até 2x por pneu), histórico de km por posição, integração com checklist.

### Outros Módulos Futuros

| Módulo | Descrição | Dependência |
|--------|-----------|-------------|
| Sinistros expandidos | Workflow completo com seguradora: perícia, indenização, reposição | Integração seguradoras |
| Contratos de locação | Controle de frota locada, km contratado vs. utilizado | Frota locada |
| Diárias e passagens | Vínculo com SCDP para viagens com diárias | Integração SCDP |
| Painel de Compliance | Indicadores de conformidade para auditores (checklists, preventivas, multas abertas) | Dados dos módulos existentes |

---

## 14. Referências Normativas Futuras

### Responsável Patrimonial Obrigatório

**Descrição:** Todo veículo ativo DEVERIA ter um responsável patrimonial designado formalmente (CPF de servidor — "fiel depositário"). Infrações sem condutor identificável seriam direcionadas ao responsável patrimonial.

**Adiamento:** Requer portaria interna da UFMT regulamentando a figura, discussão sobre responsabilidades e implicações jurídicas para os servidores designados.

**Tratamento atual:** Multas sem condutor identificável → apuração manual pelo Gestor de Frota (RF-MLT-01).

**Recomendação:** Incluir na próxima versão do DRS após publicação da portaria interna.

---

## 15. Considerações Finais

O DRS v3.2 consolida a Fundação de Integridade como camada estrutural do sistema, sem a qual os requisitos funcionais não se sustentam sob carga ou em cenários de conectividade intermitente.

**As decisões arquiteturais centrais são:**

A **FSM de dois eixos** (`operational_status` + `allocation_status`) elimina a ambiguidade entre aptidão operacional e vínculo a viagens. Um veículo em manutenção não pode ser alocado; um veículo ativo e livre é a única combinação que permite reserva. Isso é verificado atomicamente no banco de dados, não apenas na UI.

O **Pessimistic Lock** na alocação e **OCC** no restante é a escolha correta para a escala de 500 veículos. Kafka e EDA podem ser revisitados quando a volumetria justificar, mas a introdução prematura de mensageria aumenta custo operacional e curva de aprendizado sem benefício real neste momento.

O **odômetro como série temporal imutável** com hierarquia de confiança e quarentena torna o cálculo de consumo e os gatilhos de manutenção determinísticos. A regra de nunca sobrescrever uma leitura garante auditabilidade completa da história do veículo.

A **idempotência por `Request-ID`** resolve silenciosamente o problema mais comum do PWA offline: o retry após timeout. Sem isso, cada falha de rede gera um registro duplicado.

O **TLS como gate de produção**, e não como requisito adiado, transfere a responsabilidade corretamente — a aplicação é entregue preparada, a infraestrutura provisiona o certificado. O deploy é condicionado, não opcional.

### Fatores Críticos de Sucesso

1. **Implementar a Fundação antes dos módulos funcionais** — versionamento, série temporal e idempotência são infraestrutura, não feature.
2. **Schema de banco validado antes do primeiro deploy** — FSM como colunas tipadas, índices RNF-12, tabela `leituras_hodometro` append-only.
3. **Testes de concorrência obrigatórios** — cenário de double booking offline, lock collision na alocação simultânea, retry idempotente.
4. **TLS em produção** — pipeline CI/CD com verificação de certificado como gate.
5. **Calendário de feriados e catálogos** — cadastro antes do go-live (prazos de antecedência dependem disso).
6. **Material informativo nos veículos** — QR code do PWA para condutores eventuais.
7. **Testes de campo offline** — PWA em área sem cobertura, sincronização, quarentena de odômetro.

---

*Fim do Documento — Módulo Frota, Plataforma de Gestão UFMT — DRS v3.2*
