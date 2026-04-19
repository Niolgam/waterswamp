# Documento de Requisitos de Software (DRS)

## Sistema de Gestão de Frota — SIGFROTA
### Universidade Federal de Mato Grosso (UFMT)

---

| Campo | Valor |
|-------|-------|
| **Versão** | 3.0 |
| **Data** | 27/03/2026 |
| **Status** | Em Revisão |
| **Autor original** | Equipe UFMT |
| **Revisado por** | Análise de Engenharia de Requisitos + Auditoria de Conformidade |
| **Aprovado por** | *(Pendente)* |

### Histórico de Revisões

| Versão | Data | Autor | Descrição |
|--------|------|-------|-----------|
| 1.0 | — | Equipe UFMT | Versão inicial do DRS |
| 2.0 | 27/03/2026 | Revisão técnica | Decomposição atômica de requisitos, adição de LGPD/segurança/acessibilidade, expansão de casos de uso, resolução de conflitos, novos fluxos operacionais, matriz de rastreabilidade, priorização MoSCoW |
| 3.0 | 27/03/2026 | Auditoria + Consolidação | Substituição de last-write-wins por OCC, harmonização RN05/RN06 (delegação/escalonamento), dupla custódia para dados críticos (SoD), responsável patrimonial para multas, condutores terceirizados, relatórios assíncronos, ciência obrigatória em escalonamento, depreciação patrimonial no TCO, fluxo de sinistro/roubo, Carsharing movido para Won't com pré-requisito institucional |

---

## Sumário

1. Introdução
2. Escopo
3. Glossário, Definições e Siglas
4. Visão Geral da Arquitetura Funcional
5. Perfis de Acesso (Papéis)
6. Regras de Negócio (RN)
7. Requisitos Funcionais (RF)
8. Requisitos Não Funcionais (RNF)
9. Casos de Uso Expandidos
10. Matriz de Rastreabilidade
11. Priorização (MoSCoW) e Faseamento
12. Escopo Futuro Condicionado
13. Considerações Finais

---

## 1. Introdução

### 1.1. Propósito

Este documento especifica os requisitos de software para o SIGFROTA — Sistema de Gestão de Frota da Universidade Federal de Mato Grosso (UFMT). Constitui a base autoritativa para desenvolvimento, testes, homologação e auditoria da solução.

### 1.2. Público-Alvo

- Equipe de desenvolvimento
- Gestores de frota e administradores da UFMT
- Auditoria interna e órgãos de controle (CGU/TCU)
- Usuários-chave para validação

### 1.3. Convenções do Documento

- **DEVE/OBRIGATÓRIO**: Requisito mandatório para a fase indicada.
- **DEVERIA**: Requisito fortemente recomendado, pode ser negociado com justificativa.
- **PODE**: Requisito desejável, implementado conforme disponibilidade.
- Cada requisito possui um identificador único no formato `[MÓDULO]-[SEQ]` (ex: `RF-AST-01`).
- Requisitos são atômicos: cada ID cobre uma única capacidade testável.
- Prioridade segue classificação MoSCoW (Must/Should/Could/Won't nesta versão).

---

## 2. Escopo

### 2.1. Escopo Incluído

O SIGFROTA abrangerá o ciclo de vida completo da frota institucional da UFMT:

- **Gestão patrimonial**: aquisição, registro, transferência, depreciação, desfazimento e baixa de veículos, incluindo sinistros (roubo/furto, perda total).
- **Controle operacional**: abastecimentos, pedágios, seguros, licenciamentos, acessórios.
- **Manutenção**: planos preventivos, solicitações corretivas, ordens de serviço, inspeções.
- **Condutores**: credenciamento de servidores e terceirizados, treinamentos, avaliação, controle de jornada.
- **Viagens**: solicitação, aprovação hierárquica, alocação, execução, prestação de contas, cancelamento, extensão, incidentes.
- **Rotas**: cadastro de rotas padrão, roteirização simplificada, auditoria de trajeto (quando rastreamento disponível).
- **Multas**: registro, vinculação ao condutor ou responsável patrimonial, recursos, ressarcimento.
- **Indicadores e relatórios**: dashboards operacionais, relatórios gerenciais (síncronos e assíncronos), exportações.
- **Importação de dados legados**: migração estruturada da base histórica.
- **Integrações**: SouGov (RH/autenticação), SEI (documentos), Sistema de Patrimônio, APIs de telemetria (prontidão).

### 2.2. Escopo Excluído

- Gestão de combustível em nível de estoque de posto próprio (caso a UFMT possua).
- Integração com DETRAN para consulta automática de multas (escopo futuro).
- Aplicativo nativo mobile (será PWA responsivo com capacidade offline).
- Gestão financeira/orçamentária (o sistema fornece dados de custo mas não executa pagamentos).
- **Carsharing interno** — movido para escopo futuro condicionado (ver seção 12).

### 2.3. Dependências Externas

| Dependência | Tipo | Impacto se Indisponível |
|-------------|------|-------------------------|
| SouGov | Autenticação, dados de servidores e sincronização de afastamentos | Sistema opera em modo degradado com cache local de dados previamente sincronizados. Delegações de aprovação podem atrasar até próxima sincronização. |
| SEI | Geração de documentos formais | Documentos gerados localmente para posterior inserção manual no SEI |
| Sistema de Patrimônio UFMT | Dados patrimoniais de veículos | Cadastro manual com conciliação posterior |
| Conectividade Internet (áreas rurais) | Operação do PWA em campo | PWA opera offline com sincronização via Concorrência Otimista (OCC) e fila de resolução de conflitos |

---

## 3. Glossário, Definições e Siglas

### 3.1. Termos do Domínio

| Termo | Definição |
|-------|-----------|
| **Condutor** | Pessoa devidamente credenciada no SIGFROTA, autorizada a conduzir veículos da frota institucional. Pode ser: (a) servidor público federal lotado na UFMT (validado via SouGov), ou (b) motorista terceirizado vinculado por contrato de prestação de serviço (validado manualmente). Termo oficial utilizado em todo o sistema. |
| **Solicitante** | Servidor que requisita o uso de veículo da frota para finalidade institucional. Pode ou não ser o condutor da viagem. |
| **Gestor de Frota** | Servidor designado por portaria para administrar operacionalmente a frota, incluindo alocação de veículos, aprovação de manutenções e gestão de condutores. Equivale ao papel "Administrador de Frota" em contexto de sistema. |
| **Responsável Patrimonial** | Servidor formalmente designado como fiel depositário do veículo, geralmente a chefia do departamento custodiante ou o Gestor de Frota local. Assume responsabilidade por infrações e ocorrências quando não há condutor identificável (veículo estacionado, fora de viagem ativa). Todo veículo DEVE ter um responsável patrimonial vinculado. |
| **Viagem** | Deslocamento institucional efetivamente realizado ou em execução, com veículo e condutor alocados, checklists preenchidos e prestação de contas obrigatória. |
| **Reserva** | Solicitação aprovada de uso de veículo para período futuro, com recursos alocados mas viagem ainda não iniciada. |
| **Ordem de Serviço (OS)** | Documento formal que autoriza e registra a execução de serviço de manutenção em veículo da frota, preventivo ou corretivo. |
| **TCO** | Total Cost of Ownership — Custo Total de Propriedade. Soma de custos de aquisição, operação (combustível, pedágios, seguros), manutenção (preventiva e corretiva) e depreciação, deduzido o valor residual estimado, ao longo da vida útil do veículo. |
| **Checklist** | Formulário estruturado e configurável de verificação, utilizado em inspeções, saídas e retornos de viagem. |
| **Concorrência Otimista (OCC)** | Estratégia de resolução de conflitos para operações offline. Cada registro carrega um version token. Ao sincronizar, se o token do servidor divergir do token local, o registro entra em fila de conflito para resolução manual pelo Gestor de Frota, em vez de sobrescrever silenciosamente. |
| **Dupla Custódia (SoD)** | Princípio de Segregação de Funções: operações críticas exigem ação de dois usuários distintos com papéis diferentes. Quem propõe a alteração não pode ser quem a aprova. |

### 3.2. Siglas

| Sigla | Significado |
|-------|-------------|
| CRLV | Certificado de Registro e Licenciamento de Veículo |
| CNH | Carteira Nacional de Habilitação |
| CTB | Código de Trânsito Brasileiro |
| DPIA | Data Protection Impact Assessment (Avaliação de Impacto à Proteção de Dados) |
| e-MAG | Modelo de Acessibilidade em Governo Eletrônico |
| LGPD | Lei Geral de Proteção de Dados (Lei nº 13.709/2018) |
| MTBF | Mean Time Between Failures (Tempo Médio Entre Falhas) |
| MTTR | Mean Time To Repair (Tempo Médio de Reparo) |
| NF | Nota Fiscal |
| OCC | Optimistic Concurrency Control (Controle de Concorrência Otimista) |
| OS | Ordem de Serviço |
| RBAC | Role-Based Access Control (Controle de Acesso Baseado em Papéis) |
| ROI | Return on Investment (Retorno sobre o Investimento) |
| RPO | Recovery Point Objective (Ponto máximo aceitável de perda de dados) |
| RTO | Recovery Time Objective (Tempo máximo aceitável para restauração) |
| SEI | Sistema Eletrônico de Informações |
| SoD | Segregation of Duties (Segregação de Funções) |
| SouGov | Sistema de Gestão de Pessoas do Governo Federal |
| TCO | Total Cost of Ownership (Custo Total de Propriedade) |

### 3.3. Fórmulas e Regras de Cálculo

**TCO de Veículo:**
```
TCO = Custo_Aquisição
    + Σ(Custos_Combustível)
    + Σ(Custos_Manutenção)
    + Σ(Custos_Seguro_Licenciamento)
    + Σ(Custos_Pedágio)
    + Σ(Depreciação_Acumulada)
    - Valor_Residual_Estimado
```
- Período de cálculo: desde a aquisição até a data corrente (ou data de baixa).
- Quando dados de aquisição estiverem ausentes (veículos legados), utiliza-se valor de referência FIPE na data de incorporação ao patrimônio.
- **Depreciação:** Calculada conforme taxa anual configurável por categoria (RF-AST-11). Método padrão: depreciação linear. Exemplo: veículo adquirido por R$ 100.000 com vida útil de 10 anos → depreciação anual de R$ 10.000.
- **Valor Residual Estimado:** `Valor_Aquisição - Depreciação_Acumulada`, com piso de R$ 0,00 (valor residual não pode ser negativo).

**Consumo Médio:**
```
Consumo_Médio (km/l) = (Odômetro_Atual - Odômetro_Abastecimento_Anterior) / Litros_Abastecidos
```

**MTBF:**
```
MTBF = Tempo_Total_Operação / Número_de_Falhas_no_Período
```
- Para veículos sem falhas registradas no período, MTBF = tempo total do período.

**MTTR:**
```
MTTR = Σ(Tempo_Indisponibilidade_por_Manutenção_Corretiva) / Número_de_Manutenções_Corretivas
```

---

## 4. Visão Geral da Arquitetura Funcional

O SIGFROTA está organizado em **9 módulos funcionais** e **1 módulo transversal**:

| # | Módulo | Código | Descrição |
|---|--------|--------|-----------|
| 1 | Gestão de Ativos | AST | Ciclo de vida patrimonial dos veículos, depreciação, sinistros |
| 2 | Gestão de Insumos | INS | Abastecimentos, pedágios, seguros, licenciamentos |
| 3 | Gestão de Manutenção | MAN | Preventiva, corretiva, ordens de serviço, inspeções |
| 4 | Gestão de Condutores | CND | Credenciamento (servidores e terceirizados), treinamentos, avaliação, jornada |
| 5 | Gestão de Rotas | ROT | Rotas padrão, roteirização, auditoria de trajeto |
| 6 | Gestão de Viagens | VIG | Solicitação, aprovação, execução, prestação de contas |
| 7 | Gestão de Multas | MLT | Registro, vinculação, recursos, ressarcimento |
| 8 | Indicadores e Relatórios | IND | Dashboards, relatórios síncronos e assíncronos, exportações |
| 9 | Administração e Importação | ADM | Parametrização, importação legada, notificações |
| T | Transversal: Segurança, Auditoria, LGPD | SEC | Autenticação, autorização, auditoria, privacidade, SoD |

**Módulo removido do escopo ativo:** Carsharing Interno (CSH) — ver seção 12.

---

## 5. Perfis de Acesso (Papéis)

O sistema implementa RBAC com os seguintes papéis. Um usuário pode acumular mais de um papel, respeitando restrições de Segregação de Funções (SoD).

| Papel | Código | Descrição |
|-------|--------|-----------|
| Administrador do Sistema | SYS_ADMIN | Configuração técnica e parametrização. Sujeito a SoD para alterações em dados críticos. |
| Gestor de Frota | FLEET_MGR | Gestão operacional completa da frota: alocar veículos, aprovar OS, gerenciar condutores, resolver conflitos de sincronização, visualizar todos os módulos. |
| Gestor de Manutenção | MAINT_MGR | Gestão específica de manutenções: criar/aprovar OS, registrar execuções, gerenciar oficinas. |
| Chefia de Departamento | DEPT_HEAD | Aprovação hierárquica de solicitações do seu departamento, visualizar relatórios departamentais. |
| Condutor | DRIVER | Operação de viagens e registros em campo: visualizar agenda, preencher checklists, registrar abastecimentos, reportar avarias. Aplicável a servidores e terceirizados credenciados. |
| Solicitante | REQUESTER | Requisição de veículos: criar solicitações, acompanhar status, cancelar próprias solicitações. |
| Gestor de Patrimônio | ASSET_MGR | Gestão patrimonial de veículos: registrar aquisições, transferências, processos de baixa, designar responsável patrimonial. |
| Auditor | AUDITOR | Consulta irrestrita somente-leitura: todos os dados, logs de auditoria e relatórios. Pode ser segundo aprovador em operações de dupla custódia. |

### 5.1. Restrições de Segregação de Funções (SoD)

| Operação Crítica | Papel Propositor | Papel Aprovador | Restrição |
|------------------|-----------------|-----------------|-----------|
| Alteração de Chassi/Renavam | SYS_ADMIN | AUDITOR ou outro SYS_ADMIN | Propositor ≠ Aprovador |
| Correção de odômetro | FLEET_MGR | SYS_ADMIN ou AUDITOR | Propositor ≠ Aprovador |
| Exclusão/inativação de log de auditoria | — | — | **Proibido para todos os papéis** |
| Alteração de responsável patrimonial | ASSET_MGR ou FLEET_MGR | DEPT_HEAD do departamento destino | Propositor ≠ Aprovador |

---

## 6. Regras de Negócio (RN)

### Regras Gerais

**RN01 — Obrigatoriedade de Registro:** Toda utilização de veículo da frota institucional DEVE ser registrada no SIGFROTA como viagem, incluindo deslocamentos locais, aulas de campo e qualquer outro uso. Nenhum veículo pode sair do estacionamento sem registro ativo no sistema.

**RN02 — Validação de Condutor:** Um condutor só pode ser alocado a uma viagem se, cumulativamente: (a) possuir credenciamento ativo no SIGFROTA, (b) CNH válida com categoria compatível com o veículo, (c) nenhum curso obrigatório vencido, (d) pontuação da CNH abaixo do limite suspensivo. Aplica-se igualmente a servidores e terceirizados credenciados.

**RN03 — Aprovação Hierárquica de Viagens:** Toda solicitação de viagem requer aprovação da chefia imediata do departamento solicitante antes da alocação de recursos.

**RN04 — Delegação de Aprovação:** A delegação de competência de aprovação segue a seguinte lógica:
- (a) O sistema sincroniza proativamente os afastamentos do SouGov (férias, licenças, viagens) para identificar aprovadores ausentes.
- (b) Se o aprovador titular possui substituto legal ativo cadastrado no SouGov na data da solicitação, o sistema roteia diretamente ao substituto, sem espera.
- (c) A delegação é registrada no log de auditoria com referência ao afastamento do titular e à designação do substituto.

**RN05 — Escalonamento por Inércia:** O escalonamento hierárquico ocorre exclusivamente por omissão do aprovador efetivo (titular ou seu substituto ativo):
- (a) Se a solicitação não for analisada pelo aprovador efetivo em 2 dias úteis, o sistema escalona ao nível hierárquico superior.
- (b) Notificação ao aprovador omisso e ao solicitante.
- (c) O aprovador escalonado DEVE confirmar ciência do contexto da solicitação e da excepcionalidade do escalonamento antes de aprovar (checkbox obrigatório: "Declaro ciência de que esta aprovação foi escalonada por inércia do aprovador [Nome/Matrícula] e que analisei o mérito da solicitação").

**RN06 — Conflito de Agendamento:** O sistema DEVE impedir a alocação de um mesmo veículo ou condutor para viagens com sobreposição de datas e horários, considerando margem de segurança configurável (padrão: 1 hora entre viagens).

**RN07 — Indisponibilidade por Status:** Veículos com status "Em Manutenção", "Inativo", "Em Processo de Baixa", "Baixado" ou "Sinistrado" não podem ser listados como disponíveis para reservas.

**RN08 — Suspensão Preventiva de Reservas:** Quando um veículo transicionar para "Em Manutenção" ou "Sinistrado", o sistema DEVE: (a) sinalizar em destaque todas as reservas futuras (até 30 dias) deste veículo no dashboard do Gestor de Frota, (b) enviar notificação ao Gestor de Frota e aos solicitantes afetados, (c) exigir que o Gestor de Frota confirme a realocação ou cancele cada viagem afetada em até 1 dia útil.

**RN09 — Antecedência Mínima:** Solicitações de viagem DEVEM ser registradas com antecedência mínima configurável (padrão: 48 horas). Solicitações urgentes fora do prazo exigem justificativa obrigatória e aprovação direta do Gestor de Frota, com registro da excepcionalidade.

**RN10 — Prestação de Contas Obrigatória:** A conclusão de uma viagem e liberação do veículo para novas alocações requer: (a) preenchimento do checklist de retorno, (b) registro da quilometragem final, (c) anexo de comprovantes de despesas (quando houver). Viagens sem prestação de contas em até 3 dias úteis após o retorno geram alerta ao Gestor de Frota e notificação ao condutor. Após 5 dias úteis sem prestação de contas, o sistema DEVE gerar registro de pendência no prontuário do condutor.

**RN11 — Limite de Jornada de Direção:** O sistema DEVE impedir a alocação de condutor que ultrapasse 8 horas de direção contínua ou 10 horas de direção diária. Para viagens que excedam esses limites, DEVE ser obrigatória a alocação de condutor adicional para revezamento.

**RN12 — Finalidade Institucional:** Toda viagem DEVE ter finalidade institucional documentada. O sistema DEVE sinalizar automaticamente para revisão do Gestor de Frota destinos que não correspondam a unidades acadêmicas, órgãos públicos ou locais previamente cadastrados como pontos de interesse institucional.

**RN13 — Inviolabilidade do Log de Auditoria:** Registros de auditoria não podem ser alterados, excluídos ou truncados por nenhum perfil de usuário, incluindo Administrador do Sistema. A retenção mínima é de 5 anos, conforme tabela de temporalidade da UFMT. Nenhuma operação de manutenção de banco de dados pode afetar a integridade dos logs.

**RN14 — Desfazimento Patrimonial:** A baixa de veículos DEVE seguir o rito do Decreto nº 9.373/2018, exigindo: (a) laudo técnico de avaliação, (b) parecer da comissão de desfazimento, (c) autorização da autoridade competente, (d) publicação em boletim. Cada etapa DEVE ser registrada e vinculada ao processo no SEI.

**RN15 — Ressarcimento de Multas:** A cobrança de ressarcimento por multas de trânsito a servidor DEVE seguir processo administrativo com: (a) notificação formal ao condutor identificado ou ao responsável patrimonial (quando condutor não identificável), (b) prazo mínimo de 10 dias úteis para defesa, (c) decisão fundamentada da autoridade competente. O sistema DEVE registrar todas as etapas e prazos.

**RN16 — Responsável Patrimonial:** Todo veículo ativo na frota DEVE ter um responsável patrimonial vinculado (CPF de servidor). Ao transferir veículo entre departamentos, o responsável patrimonial DEVE ser atualizado. Infrações e ocorrências em que o condutor não é identificável são direcionadas ao responsável patrimonial para providências.

**RN17 — Dupla Custódia para Dados Críticos:** Alterações em campos identificadores patrimoniais (Chassi, Renavam) e correções de odômetro exigem ação de dois usuários distintos: um propositor e um aprovador com papel diferente, conforme tabela de SoD (seção 5.1). Alterações unilaterais são bloqueadas pelo sistema.

### Regras de Cálculo

**RN18 — Cálculo de TCO:** O TCO de cada veículo DEVE ser recalculado automaticamente a cada novo registro de custo (abastecimento, manutenção, seguro, pedágio) e mensalmente para incorporar a depreciação. A fórmula está definida na seção 3.3. Veículos com dados de aquisição ausentes utilizam valor FIPE como referência.

**RN19 — Alertas de Consumo:** O sistema DEVE emitir alerta quando o consumo médio de um veículo desviar negativamente em mais de 15% (configurável) da média histórica dos últimos 6 meses da mesma categoria de veículo. A média de referência é recalculada mensalmente.

**RN20 — Projeção de Manutenção Preventiva:** Gatilhos de manutenção preventiva são calculados com base no menor entre: (a) intervalo por quilometragem, (b) intervalo por tempo, (c) intervalo por horas de uso (quando aplicável). A projeção utiliza a taxa média de utilização dos últimos 3 meses para estimar a data provável do próximo serviço.

**RN21 — Depreciação Patrimonial:** A depreciação é calculada pelo método linear: `Depreciação_Anual = (Valor_Aquisição - Valor_Residual_Mínimo) / Vida_Útil_Anos`. A taxa anual e vida útil são configuráveis por categoria de veículo (RF-AST-11). A depreciação cessa quando o valor contábil atinge o valor residual mínimo configurado (padrão: R$ 0,00) ou quando o veículo é baixado.

### Regras de Sincronização Offline

**RN22 — Concorrência Otimista (OCC):** Toda operação realizada offline no PWA DEVE utilizar controle de concorrência otimista:
- (a) Cada registro sincronizável carrega um `version_token` (hash ou timestamp de versão).
- (b) Ao sincronizar, o sistema compara o `version_token` local com o do servidor.
- (c) Se os tokens coincidem: escrita aceita, token incrementado.
- (d) Se os tokens divergem: registro entra no estado "Conflito de Sincronização Pendente" e é adicionado à fila de resolução do Gestor de Frota.
- (e) O Gestor de Frota DEVE resolver o conflito escolhendo a versão correta ou mesclando dados manualmente.
- (f) Até a resolução, ambas as versões são preservadas e nenhuma é descartada.
- (g) O log de auditoria registra o conflito, as versões envolvidas e a decisão de resolução.

**RN23 — Bloqueio de Alocação Duplicada Offline:** O PWA DEVE armazenar localmente a lista de veículos e condutores já alocados (sincronizada na última conexão). Se o condutor tentar iniciar uma viagem com um veículo que consta como alocado a outro condutor no cache local, o sistema DEVE bloquear a operação e exibir alerta, mesmo offline. Quando a sincronização ocorrer, se duas viagens forem detectadas para o mesmo veículo no mesmo período, ambas entram em conflito obrigatório (RN22).

---

## 7. Requisitos Funcionais (RF)

### Módulo 1: Gestão de Ativos (AST)

**RF-AST-01 — Cadastrar Veículo** `[Must]`
O sistema DEVE permitir o cadastro de um novo veículo na frota.
- **Entradas obrigatórias:** Placa, Chassi, Renavam, Marca, Modelo, Ano de Fabricação, Ano do Modelo, Cor, Categoria (conforme CTB), Tipo de Combustível, Departamento responsável, Número de Patrimônio, Responsável Patrimonial (CPF de servidor — conforme RN16).
- **Entradas opcionais:** Capacidade de passageiros, Capacidade de carga (kg), Potência (cv), Dados de aquisição (NF, valor, data, processo licitatório), Especificações adicionais.
- **Validações:** Chassi, Renavam e Placa DEVEM ser únicos no sistema. Formato de placa conforme padrão Mercosul ou antigo. Responsável patrimonial DEVE ser servidor ativo na UFMT.
- **Saída:** Registro do veículo com status inicial "Ativo", taxa de depreciação aplicada conforme categoria (RF-AST-11), e entrada no log de auditoria.
- **RNs relacionadas:** RN16, RN21

**RF-AST-02 — Editar Dados de Veículo** `[Must]`
O sistema DEVE permitir a edição de dados cadastrais de veículos existentes.
- **Dados editáveis livremente** (por FLEET_MGR ou ASSET_MGR): Cor, Departamento, Capacidade, Especificações, Responsável Patrimonial (com aprovação do DEPT_HEAD destino — RN17 SoD).
- **Dados editáveis com dupla custódia** (conforme RN17): Chassi e Renavam. Placa pode ser alterada em caso de mudança para padrão Mercosul, com registro do histórico e dupla custódia.
- **Saída:** Registro atualizado com log de auditoria contendo valores anteriores e posteriores. Para operações de dupla custódia, log registra propositor, aprovador e justificativa.
- **RNs relacionadas:** RN17

**RF-AST-03 — Consultar Veículos** `[Must]`
O sistema DEVE permitir consulta de veículos com filtros combinados: placa, marca, modelo, status, departamento, categoria, ano, faixa de quilometragem, responsável patrimonial.
- **Saída:** Lista paginada com dados resumidos e opção de visualizar ficha completa do veículo, incluindo TCO atual e status de depreciação.

**RF-AST-04 — Gerenciar Anexos de Veículo** `[Must]`
O sistema DEVE permitir o upload, visualização e substituição de documentos vinculados ao veículo: CRLV, Nota Fiscal de aquisição, Apólices de seguro, Laudos técnicos, Fotos.
- **Validações:** Formatos aceitos: PDF, JPG, PNG. Tamanho máximo por arquivo: 10MB. Ao substituir, o anexo anterior DEVE ser mantido como versão histórica.

**RF-AST-05 — Gerenciar Status de Veículo** `[Must]`
O sistema DEVE registrar e controlar os seguintes estados de veículo: Ativo, Em Manutenção, Reservado, Inativo, Em Processo de Baixa, Baixado, Sinistrado.
- **Transições permitidas:**
  - Ativo → Em Manutenção, Reservado, Inativo, Em Processo de Baixa, Sinistrado
  - Em Manutenção → Ativo
  - Reservado → Ativo (ao fim da viagem), Em Manutenção (se falha detectada no retorno)
  - Inativo → Ativo, Em Processo de Baixa
  - Em Processo de Baixa → Baixado, Ativo (se processo cancelado)
  - Sinistrado → Em Processo de Baixa (para baixa definitiva), Ativo (se recuperado)
  - Baixado → *(terminal, sem transição de saída)*
- **Comportamento automático:** Ao transicionar para "Em Manutenção" ou "Sinistrado", o sistema DEVE executar a verificação de reservas futuras conforme RN08.
- **Saída:** Transição registrada com timestamp, responsável e justificativa no log inalterável.
- **RNs relacionadas:** RN07, RN08

**RF-AST-06 — Registrar Transferência de Departamento** `[Must]`
O sistema DEVE registrar a transferência de um veículo entre departamentos.
- **Entradas:** Departamento de origem, Departamento de destino, Novo responsável patrimonial (CPF), Data efetiva, Motivo, Documento autorizativo (número SEI).
- **Validações:** Novo responsável patrimonial DEVE ser servidor ativo lotado no departamento destino. Transferência de responsável patrimonial sujeita a SoD (seção 5.1).
- **Saída:** Atualização do departamento responsável e do responsável patrimonial. Registro histórico da transferência.
- **RNs relacionadas:** RN16, RN17

**RF-AST-07 — Registrar Identificação Visual** `[Should]`
O sistema DEVE registrar o padrão de adesivação institucional do veículo: tipo de adesivação, data de aplicação, data de validade/renovação, fornecedor responsável, fotos do veículo adesivado.
- **Saída:** Alerta automático configurável (padrão: 30 dias) antes do vencimento da adesivação.

**RF-AST-08 — Calcular Projeção de Substituição** `[Should]`
O sistema DEVE calcular a idade e quilometragem atuais de cada veículo e projetar a data estimada de substituição com base nos parâmetros configuráveis de vida útil (anos e/ou km).
- **Saída:** Ranking de prioridade de substituição ordenado por: idade, quilometragem, TCO, valor contábil residual (depreciação) e número de manutenções corretivas no último ano.
- **RNs relacionadas:** RN18, RN21

**RF-AST-09 — Iniciar Processo de Baixa** `[Must]`
O sistema DEVE permitir iniciar o processo de desfazimento patrimonial de um veículo.
- **Entradas:** Justificativa (sinistro total, roubo/furto irrecuperável, fim de vida útil, antieconômico, determinação administrativa), Destino pretendido (leilão, sucata, doação, cessão), Laudo técnico de avaliação (anexo obrigatório).
- **Validações:** Se veículo possui reservas futuras, sistema exige confirmação de que todas serão canceladas/realocadas (RN08).
- **Saída:** Status do veículo alterado para "Em Processo de Baixa". Impedimento de novas alocações. Depreciação suspensa.
- **RNs relacionadas:** RN07, RN08, RN14, RN21

**RF-AST-10 — Registrar Etapas do Desfazimento** `[Must]`
O sistema DEVE registrar cada etapa do processo de desfazimento conforme RN14: parecer da comissão, autorização da autoridade, publicação, resultado (arrematação, efetivação da doação, etc.).
- **Entradas por etapa:** Tipo da etapa, Data, Responsável, Documento vinculado (número SEI), Observações.
- **Saída:** Ao registrar a última etapa (conclusão), status do veículo alterado para "Baixado". Dados do veículo movidos para base histórica inativa, preservando todo o histórico.
- **RNs relacionadas:** RN14

**RF-AST-11 — Configurar e Calcular Depreciação** `[Must]`
O sistema DEVE permitir a configuração da taxa de depreciação por categoria de veículo e calcular a depreciação acumulada de cada veículo.
- **Entradas de configuração:** Categoria do veículo, Vida útil em anos, Valor residual mínimo (padrão: R$ 0,00), Método de depreciação (linear — único método nesta versão).
- **Cálculo automático:** Depreciação mensal = (Valor_Aquisição - Valor_Residual_Mínimo) / (Vida_Útil_Anos × 12). Calculada automaticamente no primeiro dia de cada mês. Para veículos legados sem valor de aquisição, utiliza valor FIPE conforme RN18.
- **Saída:** Valor contábil atualizado mensalmente. Depreciação acumulada visível na ficha do veículo e incorporada ao TCO.
- **RNs relacionadas:** RN18, RN21

**RF-AST-12 — Registrar Sinistro (Roubo/Furto/Perda Total)** `[Must]`
O sistema DEVE registrar ocorrências de sinistro que resultem em indisponibilidade total do veículo.
- **Entradas:** Tipo de sinistro (Roubo, Furto, Perda Total por acidente, Perda Total por evento natural), Data/hora da ocorrência, Local, Boletim de Ocorrência (anexo obrigatório), Número do sinistro na seguradora (quando aplicável), Descrição detalhada, Fotos (quando disponíveis).
- **Comportamento automático:** (a) Status do veículo → "Sinistrado", (b) Suspensão de reservas futuras (RN08), (c) Suspensão da depreciação, (d) Notificação imediata ao Gestor de Frota e Gestor de Patrimônio, (e) Se veículo possui seguro ativo (RF-INS-09), alerta para acionamento da seguradora.
- **Saída:** Registro do sinistro. Veículo indisponível para alocação. Processo de baixa pode ser iniciado (RF-AST-09) quando confirmada a irrecuperabilidade.
- **RNs relacionadas:** RN07, RN08

### Módulo 2: Gestão de Insumos (INS)

**RF-INS-01 — Registrar Abastecimento Manual** `[Must]`
O sistema DEVE permitir o registro manual de abastecimento.
- **Entradas:** Veículo (placa), Data, Hora, Local (posto/cidade), Tipo de Combustível, Quantidade (litros), Valor Unitário, Valor Total, Leitura do Odômetro, Condutor responsável, NF (anexo opcional).
- **Validações:** (a) Odômetro informado DEVE ser maior que a leitura do último abastecimento registrado (para registros cronologicamente ordenados). (b) Valor Total DEVE ser igual a Quantidade × Valor Unitário (tolerância: R$ 0,05). (c) Veículo DEVE estar com status Ativo ou Reservado.
- **Saída:** Registro criado, consumo médio recalculado automaticamente, TCO atualizado.
- **Sincronização offline:** Registro feito offline utiliza OCC (RN22). Version token do veículo é verificado na sincronização.
- **RNs relacionadas:** RN18, RN19, RN22

**RF-INS-02 — Registrar Abastecimento Retroativo** `[Must]`
O sistema DEVE permitir o registro de abastecimentos fora de ordem cronológica (cenário: condutor em área rural sem conectividade).
- **Entradas:** Mesmas de RF-INS-01, acrescidas de: Justificativa obrigatória para registro retroativo.
- **Validações:** Registro retroativo requer aprovação do Gestor de Frota. O sistema DEVE recalcular o consumo médio considerando a ordem cronológica correta dos registros após inserção.
- **Saída:** Registro criado com flag "retroativo" e aprovação vinculada.

**RF-INS-03 — Corrigir Leitura de Odômetro** `[Must]`
O sistema DEVE permitir a correção de leitura de odômetro em casos de: troca de painel/hodômetro, erro de digitação comprovado, reset por manutenção.
- **Entradas:** Veículo, Nova leitura, Motivo da correção, Documento comprobatório (OS de manutenção, declaração).
- **Validações:** Operação sujeita a dupla custódia conforme RN17 (seção 5.1): propositor FLEET_MGR, aprovador SYS_ADMIN ou AUDITOR.
- **Saída:** Leitura corrigida, histórico de consumo recalculado a partir do ponto de correção, entrada no log de auditoria com registro de ambos os participantes da dupla custódia.
- **RNs relacionadas:** RN17

**RF-INS-04 — Importar Abastecimentos em Lote** `[Should]`
O sistema DEVE permitir a importação de registros de abastecimento via arquivo.
- **Formatos aceitos:** CSV, XLSX (conforme template disponibilizado pelo sistema).
- **Validações:** Validação linha a linha conforme regras de RF-INS-01. Encoding aceito: UTF-8 (CSV) ou padrão Excel. Tamanho máximo: 5MB / 5.000 registros por importação.
- **Saída:** Relatório de importação detalhando: registros importados com sucesso, registros rejeitados com motivo específico por linha, registros pendentes de revisão. O Gestor de Frota DEVE confirmar a importação após revisar o relatório.

**RF-INS-05 — Emitir Alertas de Consumo** `[Must]`
O sistema DEVE emitir notificação ao Gestor de Frota quando: (a) consumo médio de um veículo desviar negativamente conforme RN19, (b) gasto mensal de combustível do departamento ultrapassar limite configurado.
- **Saída:** Notificação in-app e e-mail ao gestor, com dados do veículo, consumo atual vs. esperado, e histórico recente.
- **RNs relacionadas:** RN19

**RF-INS-06 — Registrar Passagem em Pedágio** `[Must]`
O sistema DEVE registrar passagens em pedágios.
- **Entradas:** Data, Hora, Praça de pedágio (localização), Valor, Veículo, Viagem vinculada (quando aplicável).
- **Vinculação automática:** O sistema DEVE sugerir vinculação à viagem ativa do veículo na data/hora da passagem.
- **Saída:** Registro criado, custo computado na viagem e no TCO do veículo.

**RF-INS-07 — Importar Extratos de Pedágio** `[Should]`
O sistema DEVE permitir importação de extratos de operadoras de pedágio (CSV/XLSX conforme template).
- **Validações:** Vinculação automática por placa + data/hora. Registros sem viagem correspondente ficam pendentes para vinculação manual.
- **Saída:** Relatório de conciliação com registros vinculados e pendentes.

**RF-INS-08 — Registrar Aplicação de Insumos** `[Must]`
O sistema DEVE registrar a aplicação de insumos consumíveis: óleos, filtros, baterias, pneus, fluidos.
- **Entradas:** Veículo, Tipo de insumo, Marca/especificação, Quantidade, Quilometragem na troca, Data, NF (opcional), Próxima troca projetada (km e/ou data).
- **Saída:** Registro criado, projeção de próxima substituição calculada e agendada como alerta.
- **RNs relacionadas:** RN20

**RF-INS-09 — Registrar Apólice de Seguro** `[Must]`
O sistema DEVE registrar apólices de seguro vinculadas ao veículo.
- **Entradas:** Seguradora, Número da apólice, Tipo de cobertura, Valor do prêmio, Vigência (início/fim), Franquia, Documento (anexo).
- **Saída:** Alerta automático configurável (padrão: 30 dias) antes do vencimento. Custo computado no TCO. Em caso de sinistro (RF-AST-12), alerta automático para acionamento se apólice vigente.

**RF-INS-10 — Registrar Licenciamento e Taxas** `[Must]`
O sistema DEVE registrar IPVA, DPVAT/SPVAT, taxas de licenciamento e vistoria.
- **Entradas:** Veículo, Tipo de taxa, Exercício/ano, Valor, Data de pagamento, Comprovante (anexo).
- **Saída:** Alerta para taxas com vencimento próximo. Registro de pendências por veículo.

### Módulo 3: Gestão de Manutenção (MAN)

**RF-MAN-01 — Configurar Plano de Manutenção Preventiva** `[Must]`
O sistema DEVE permitir a configuração de planos de manutenção preventiva por categoria de veículo.
- **Entradas:** Categoria do veículo, Tipo de serviço (ex: troca de óleo, revisão geral), Intervalo por quilometragem, Intervalo por tempo (dias), Intervalo por horas de uso (quando aplicável), Checklist de itens a verificar.
- **Saída:** Plano configurado e aplicável a todos os veículos da categoria.
- **RNs relacionadas:** RN20

**RF-MAN-02 — Gerar OS Preventiva Automaticamente** `[Must]`
O sistema DEVE gerar Ordens de Serviço preventivas automaticamente quando qualquer gatilho do plano for atingido (o menor entre km, tempo ou horas conforme RN20).
- **Saída:** OS criada com status "Programada", vinculada ao plano e ao veículo, com notificação ao Gestor de Manutenção.
- **RNs relacionadas:** RN20

**RF-MAN-03 — Solicitar Manutenção Corretiva** `[Must]`
O sistema DEVE permitir que condutores e gestores reportem falhas ou problemas.
- **Entradas:** Veículo, Descrição do problema, Nível de urgência (Baixa, Média, Alta, Crítica), Imagens da avaria (até 5 fotos), Localização do veículo (quando fora da sede).
- **Saída:** OS criada com status "Pendente de Avaliação" e notificação ao Gestor de Manutenção.
- **Comportamento adicional:** Se urgência "Crítica", notificação imediata ao Gestor de Frota e alteração automática do status do veículo para "Em Manutenção" (disparando RN08 para verificação de reservas futuras).
- **Sincronização offline:** Solicitação pode ser criada offline e sincronizada via OCC (RN22).

**RF-MAN-04 — Gerenciar Ciclo de Vida da OS** `[Must]`
O sistema DEVE controlar a Ordem de Serviço conforme a seguinte máquina de estados:
- **Estados:** Programada, Pendente de Avaliação, Aprovada, Em Execução, Aguardando Peça, Concluída, Cancelada.
- **Transições permitidas:**
  - Programada → Pendente de Avaliação (ao atingir gatilho), Cancelada
  - Pendente de Avaliação → Aprovada, Cancelada (com justificativa)
  - Aprovada → Em Execução, Cancelada (com justificativa)
  - Em Execução → Aguardando Peça, Concluída
  - Aguardando Peça → Em Execução (peça recebida)
  - Concluída → *(terminal)*
  - Cancelada → *(terminal)*
- **Restrição:** Cada transição DEVE registrar responsável, timestamp e observação opcional.

**RF-MAN-05 — Registrar Execução de OS** `[Must]`
O sistema DEVE registrar os detalhes da execução de manutenção.
- **Entradas:** Tipo de execução (interna/externa), Oficina/fornecedor, Orçamento(s) apresentado(s) (com anexo), Peças utilizadas (descrição, quantidade, valor), Mão de obra (horas, valor), Valor total, Data de início, Data de conclusão, NF (anexo obrigatório para serviço externo), Garantia (prazo e condições).
- **Validações:** Para manutenções externas, exige-se justificativa de escolha do fornecedor quando houver mais de um orçamento.
- **Saída:** OS transicionada para "Concluída". Cálculo automático do tempo de indisponibilidade (data conclusão - data início). Recálculo do TCO do veículo. Status do veículo retornado para "Ativo" (se estava "Em Manutenção").

**RF-MAN-06 — Acionar Garantia de Serviço** `[Should]`
O sistema DEVE permitir registrar acionamento de garantia de manutenção previamente executada.
- **Entradas:** OS original (referência), Descrição do problema recorrente, Oficina/fornecedor original.
- **Saída:** Nova OS vinculada à original, com flag "Garantia" e prazo de validade da garantia verificado automaticamente.

**RF-MAN-07 — Realizar Inspeção com Checklist** `[Must]`
O sistema DEVE fornecer formulários de checklist configuráveis para vistorias periódicas.
- **Entradas:** Veículo, Modelo de checklist (configurável por categoria), Resultado de cada item (Conforme / Não Conforme / Não Aplicável), Observações por item, Fotos (opcionais).
- **Saída:** Registro da inspeção. Para cada item "Não Conforme", opção de gerar OS corretiva diretamente vinculada ao item reprovado.

### Módulo 4: Gestão de Condutores (CND)

**RF-CND-01 — Cadastrar Condutor Servidor** `[Must]`
O sistema DEVE registrar condutores que são servidores públicos, vinculando-os ao SouGov.
- **Entradas via SouGov (autopreenchimento):** Matrícula, Nome completo, CPF, Departamento de lotação, Cargo/função.
- **Entradas manuais:** CNH (Número, Categoria, Data de emissão, Data de validade, UF de emissão), Anexo da CNH (foto/scan).
- **Saída:** Perfil do condutor criado com status "Ativo" e tipo "Servidor".
- **Integração:** Se SouGov estiver indisponível, permitir preenchimento manual com flag "Pendente de Validação SouGov", gerando alerta para conciliação posterior.

**RF-CND-02 — Cadastrar Condutor Terceirizado** `[Must]`
O sistema DEVE permitir o cadastro de motoristas terceirizados (CLT) sem vínculo SouGov.
- **Entradas obrigatórias:** Nome completo, CPF, CNH (Número, Categoria, Data de emissão, Data de validade, UF de emissão), Empresa contratada, Número do contrato de prestação de serviço, Vigência do contrato (início/fim), Anexo do contrato (PDF), Anexo da CNH (foto/scan).
- **Validações:** CPF DEVE ser único. Contrato DEVE estar vigente. Aprovação manual obrigatória do Gestor de Frota.
- **Saída:** Perfil do condutor criado com status "Ativo" e tipo "Terceirizado".
- **Comportamento automático:** Alerta de vencimento de contrato (padrão: 30 dias antes). Suspensão automática do credenciamento quando contrato vencer.

**RF-CND-03 — Gerenciar Status de Condutor** `[Must]`
O sistema DEVE controlar os estados: Ativo, Suspenso, Revogado, Pendente de Validação.
- **Transições:**
  - Ativo → Suspenso (CNH vencida, curso obrigatório vencido, contrato vencido para terceirizados, decisão administrativa), Revogado
  - Suspenso → Ativo (regularização comprovada), Revogado
  - Revogado → *(terminal para o credenciamento vigente; novo credenciamento possível)*
  - Pendente de Validação → Ativo (dados confirmados via SouGov), Revogado
- **Automação:** O sistema DEVE suspender preventivamente o credenciamento quando: CNH vencer, curso obrigatório vencer, ou contrato de terceirizado vencer. Notificação ao condutor e ao Gestor de Frota 30 dias antes do vencimento.

**RF-CND-04 — Registrar Treinamento** `[Must]`
O sistema DEVE registrar cursos e treinamentos de condutores.
- **Entradas:** Condutor, Nome do curso (ex: Direção Defensiva, Primeiros Socorros, Transporte de Cargas), Carga horária, Data de conclusão, Validade (se aplicável), Certificado (anexo).
- **Saída:** Registro no prontuário. Alerta de reciclagem configurável (padrão: 60 dias antes do vencimento).
- **RNs relacionadas:** RN02 (cursos obrigatórios vencidos impedem alocação)

**RF-CND-05 — Registrar Avaliação de Desempenho** `[Should]`
O sistema DEVE permitir o registro de avaliações periódicas do condutor.
- **Entradas:** Condutor, Período avaliado, Critérios configuráveis (ex: conservação do veículo, pontualidade, relacionamento), Nota por critério, Observações.
- **Saída:** Média ponderada calculada. Histórico de avaliações no prontuário.

**RF-CND-06 — Manter Prontuário de Infrações e Ocorrências** `[Must]`
O sistema DEVE manter prontuário individual do condutor com: infrações de trânsito vinculadas, acidentes (com apuração de responsabilidade), advertências administrativas, pontuação estimada da CNH.
- **Saída:** Prontuário consultável com timeline. Flag de alerta quando pontuação da CNH se aproximar do limite suspensivo (≥ 15 pontos).

**RF-CND-07 — Controlar Jornada de Direção** `[Must]`
O sistema DEVE calcular o tempo de direção com base nos registros de início e fim de viagens.
- **Entradas:** Dados derivados automaticamente dos checklists de saída/retorno e do registro de início/fim de viagem.
- **Saída:** Alerta ao Gestor de Frota quando o condutor atingir 7 horas de direção contínua (alerta preventivo) ou 9 horas diárias (alerta preventivo de limite).
- **RNs relacionadas:** RN11

### Módulo 5: Gestão de Rotas (ROT)

**RF-ROT-01 — Cadastrar Rota Padrão** `[Must]`
O sistema DEVE armazenar rotas frequentes.
- **Entradas:** Nome/identificador da rota, Origem (endereço ou coordenadas), Destino (endereço ou coordenadas), Pontos intermediários (opcionais), Distância estimada (km), Tempo estimado de percurso, Orientações e observações (ex: "estrada sem pavimentação nos últimos 30km"), Pontos de referência para locais não mapeados (aldeias, fazendas, áreas remotas).
- **Saída:** Rota cadastrada disponível para seleção em solicitações de viagem.

**RF-ROT-02 — Sugerir Roteirização** `[Could]`
O sistema DEVE sugerir ordenação otimizada de múltiplas paradas visando menor distância total.
- **Implementação V1:** Cálculo baseado em distância euclidiana entre pontos cadastrados, com opção de reordenação manual pelo gestor.
- **Implementação futura (quando disponível integração com API cartográfica):** Roteirização com dados reais de vias.
- **Nota:** O sistema DEVE sempre permitir inserção manual de coordenadas ou pontos de referência descritivos para locais não mapeados.

**RF-ROT-03 — Auditar Trajeto Realizado** `[Could]`
Quando integrado a sistema de rastreamento GPS (conforme RNF-02), o sistema DEVE comparar a rota planejada com o trajeto real, sinalizando desvios superiores a limite configurável (padrão: 5km).
- **Entradas:** Dados de GPS (via API de telemetria), Rota planejada da viagem.
- **Saída:** Relatório de aderência ao trajeto. Desvios sinalizados ao Gestor de Frota para análise. Condutor pode registrar justificativa para desvios (desvio por obra, acidente, condição da via).

### Módulo 6: Gestão de Viagens (VIG)

**RF-VIG-01 — Solicitar Viagem** `[Must]`
O sistema DEVE prover interface para o servidor requisitar veículos.
- **Entradas:** Data e Hora de saída (previstas), Data e Hora de retorno (previstas), Origem, Destino(s), Rota padrão (se aplicável — seleção da lista de RF-ROT-01), Número de passageiros, Lista de passageiros (nomes; servidores vinculados ao SouGov quando possível), Necessidade de carga (sim/não, descrição e peso estimado), Finalidade institucional (seleção de lista configurável), Justificativa textual, Documentos de apoio (anexos opcionais).
- **Autopreenchimento:** Solicitante e departamento preenchidos automaticamente via dados do login/SouGov.
- **Saída:** Solicitação criada com status "Aguardando Aprovação da Chefia". O sistema verifica se o aprovador titular está ativo ou afastado (RN04): se afastado com substituto ativo, roteia ao substituto; caso contrário, roteia ao titular. Notificação enviada ao aprovador efetivo.
- **RNs relacionadas:** RN01, RN03, RN04, RN09, RN12

**RF-VIG-02 — Aprovar/Rejeitar Viagem (Chefia)** `[Must]`
O sistema DEVE apresentar a solicitação ao aprovador efetivo (titular ou substituto conforme RN04).
- **Ações:** Aprovar (com observações opcionais) ou Rejeitar (com justificativa obrigatória).
- **Comportamento em escalonamento (RN05):** Se a aprovação foi escalonada por inércia, o aprovador escalonado DEVE marcar checkbox obrigatório de ciência antes de aprovar: "Declaro ciência de que esta aprovação foi escalonada por inércia do aprovador [Nome/Matrícula] e que analisei o mérito da solicitação."
- **Saída:** Se aprovada, status alterado para "Aguardando Alocação", notificação ao Gestor de Frota. Se rejeitada, notificação ao solicitante com motivo.
- **RNs relacionadas:** RN03, RN04, RN05

**RF-VIG-03 — Analisar Disponibilidade** `[Must]`
O sistema DEVE apresentar ao Gestor de Frota a lista de veículos disponíveis para a viagem aprovada, filtrando automaticamente por: (a) status Ativo, (b) categoria/capacidade compatível com passageiros e carga, (c) ausência de conflito de agendamento no período (considerando margem de RN06), (d) quilometragem projetada dentro dos limites de manutenção.
- **Saída:** Lista de veículos compatíveis com indicação de adequação (ideal, aceitável, necessita atenção).
- **RNs relacionadas:** RN06, RN07

**RF-VIG-04 — Alocar Veículo e Condutor** `[Must]`
O sistema DEVE permitir ao Gestor de Frota selecionar veículo e condutor(es) para a viagem.
- **Validações:** Condutor DEVE atender RN02 integralmente (aplicável a servidores e terceirizados). Para viagens com duração estimada superior ao limite de RN11, exigir alocação de segundo condutor.
- **Saída:** Status da viagem alterado para "Alocada/Confirmada". Notificação ao solicitante, condutor(es) e passageiros. Veículo marcado como "Reservado" no período.
- **RNs relacionadas:** RN02, RN06, RN11

**RF-VIG-05 — Cancelar Viagem** `[Must]`
O sistema DEVE permitir o cancelamento de viagens nos estados "Aguardando Aprovação", "Aguardando Alocação" e "Alocada/Confirmada".
- **Permissões:** Solicitante pode cancelar em qualquer estado pré-execução. Chefia e Gestor de Frota podem cancelar em qualquer estado pré-execução.
- **Entradas:** Motivo do cancelamento (obrigatório).
- **Saída:** Status alterado para "Cancelada". Recursos liberados (veículo retorna a "Ativo", condutor liberado). Notificação a todos os envolvidos. Registro no log de auditoria.

**RF-VIG-06 — Alterar Viagem Aprovada** `[Must]`
O sistema DEVE permitir a alteração de dados de viagem aprovada ou alocada: datas, destino, passageiros.
- **Regras:** Alteração de data ou destino DEVE submeter a viagem a re-aprovação da chefia (retorna para "Aguardando Aprovação"). Alteração de passageiros não exige re-aprovação, apenas registro.
- **Saída:** Viagem atualizada, re-validação de disponibilidade de recursos quando aplicável.

**RF-VIG-07 — Estender Duração de Viagem** `[Must]`
O sistema DEVE permitir que o condutor ou solicitante solicite extensão de viagem em andamento.
- **Entradas:** Nova data/hora de retorno prevista, Justificativa.
- **Validações:** Sistema verifica conflito com próxima reserva do mesmo veículo. Se houver conflito, notifica Gestor de Frota para resolução (realocação ou cancelamento da próxima viagem).
- **Saída:** Extensão registrada. Gestor de Frota notificado para aprovação. Se offline, extensão registrada localmente e sincronizada via OCC (RN22).

**RF-VIG-08 — Substituir Veículo ou Condutor** `[Must]`
O sistema DEVE permitir a troca de veículo ou condutor em viagem alocada ou em andamento, mantendo vínculo com a solicitação original.
- **Entradas:** Recurso substituído, Novo recurso, Justificativa.
- **Validações:** Novo recurso DEVE atender mesmos requisitos da alocação original (RN02 para condutores, RN06/RN07 para veículos).
- **Saída:** Histórico da substituição preservado na viagem. Recursos originais liberados.

**RF-VIG-09 — Preencher Checklist de Saída** `[Must]`
O sistema DEVE exigir preenchimento de checklist obrigatório antes do início da viagem.
- **Entradas:** Quilometragem inicial, Nível de combustível, Estado geral do veículo (itens do checklist configurável: pneus, luzes, documentos, equipamentos obrigatórios, etc.), Fotos (opcional), Confirmação de recebimento de chave/documentos.
- **Saída:** Viagem transicionada para "Em Andamento". Horário de saída registrado.
- **Sincronização offline:** Checklist pode ser preenchido offline com timestamp original preservado (RN22).
- **RNs relacionadas:** RN10, RN22

**RF-VIG-10 — Registrar Intercorrência durante Viagem** `[Must]`
O sistema DEVE permitir que o condutor registre ocorrências durante a viagem.
- **Entradas:** Tipo (Acidente, Pane mecânica, Apreensão do veículo, Problema de saúde, Condição climática adversa, Outro), Descrição, Localização (coordenadas ou descrição textual), Fotos, Boletim de ocorrência (quando aplicável).
- **Saída:** Registro vinculado à viagem. Notificação imediata ao Gestor de Frota para acidente, pane ou apreensão. Para acidente: registro também vinculado ao prontuário do condutor (RF-CND-06). Para pane mecânica: geração automática de OS corretiva (RF-MAN-03). Para sinistro (roubo/perda total): dispara RF-AST-12.
- **Sincronização offline:** Registro offline com sincronização via OCC (RN22). Fotos armazenadas localmente até conectividade.

**RF-VIG-11 — Preencher Checklist de Retorno** `[Must]`
O sistema DEVE exigir preenchimento de checklist obrigatório no retorno.
- **Entradas:** Quilometragem final, Nível de combustível, Estado geral (itens do checklist), Avarias identificadas (descrição + fotos), Confirmação de devolução de chave/documentos.
- **Validações:** Quilometragem final DEVE ser maior que quilometragem de saída.
- **Saída:** Viagem transicionada para "Aguardando Prestação de Contas". Horário de retorno registrado. Se avarias reportadas, OS corretiva gerada automaticamente (RF-MAN-03).
- **Sincronização offline:** Mesma política de RF-VIG-09 (RN22).
- **RNs relacionadas:** RN10, RN22

**RF-VIG-12 — Realizar Prestação de Contas** `[Must]`
O sistema DEVE permitir o upload de comprovantes e a conciliação financeira da viagem.
- **Entradas:** Comprovantes de abastecimento fora da rede, Comprovantes de pedágio, Outros comprovantes de despesa, Observações finais.
- **Saída:** Cálculo automático: custo real da viagem vs. estimado. Viagem transicionada para "Concluída". Veículo retornado ao status "Ativo".
- **Comportamento de prazo:** Se prestação de contas não realizada em 3 dias úteis após retorno, alerta ao Gestor de Frota e notificação ao condutor. Após 5 dias úteis, registro de pendência no prontuário do condutor conforme RN10.
- **RNs relacionadas:** RN10

**RF-VIG-13 — Suspender Reservas por Indisponibilidade** `[Must]`
Quando um veículo transicionar para "Em Manutenção" ou "Sinistrado" (RF-AST-05, RF-AST-12), o sistema DEVE:
- (a) Listar todas as reservas futuras (até 30 dias) do veículo em dashboard destacado do Gestor de Frota.
- (b) Notificar o Gestor de Frota e os solicitantes das viagens afetadas.
- (c) Exigir que o Gestor de Frota realize, para cada reserva afetada, uma das ações: realocar para outro veículo (RF-VIG-08) ou cancelar a viagem (RF-VIG-05).
- (d) Prazo: 1 dia útil para resolução. Após o prazo, alerta recorrente.
- **RNs relacionadas:** RN08

### Módulo 7: Gestão de Multas (MLT)

**RF-MLT-01 — Registrar Auto de Infração** `[Must]`
O sistema DEVE registrar multas de trânsito.
- **Entradas:** Veículo, Número do auto, Data/hora da infração, Local, Tipo de infração, Gravidade (leve, média, grave, gravíssima), Pontuação, Valor, Prazo para recurso, Documento (anexo do auto).
- **Vinculação automática:** O sistema DEVE identificar automaticamente o condutor responsável cruzando data/hora da infração com as viagens registradas do veículo.
- **Quando condutor não identificável:** Se não houver viagem ativa na data/hora da infração (veículo estacionado, uso não registrado), a multa é direcionada ao responsável patrimonial do veículo (RN16) para providências de apuração.
- **Saída:** Multa registrada, condutor identificado (ou responsável patrimonial) notificado, entrada no prontuário.
- **RNs relacionadas:** RN15, RN16

**RF-MLT-02 — Notificar Condutor ou Responsável Patrimonial** `[Must]`
O sistema DEVE notificar formalmente o condutor identificado ou o responsável patrimonial, registrando data de ciência e iniciando contagem de prazos conforme RN15.
- **Saída:** Notificação enviada (in-app e e-mail), prazo de defesa iniciado.
- **RNs relacionadas:** RN15

**RF-MLT-03 — Registrar Defesa/Recurso** `[Must]`
O sistema DEVE permitir o registro de defesa prévia ou recurso pelo condutor.
- **Entradas:** Texto da defesa, Documentos comprobatórios (anexos), Data de protocolo.
- **Saída:** Prazo de análise registrado. Status da multa atualizado para "Em Recurso".

**RF-MLT-04 — Registrar Decisão e Ressarcimento** `[Must]`
O sistema DEVE registrar a decisão administrativa sobre responsabilidade.
- **Entradas:** Decisão (Condutor responsável / Condutor isento / Recurso deferido pelo DETRAN), Fundamentação, Documento da decisão.
- **Saída:** Se condutor responsável: geração de processo de ressarcimento com valor, forma de pagamento e acompanhamento. Se isento/deferido: multa encerrada com registro do resultado.
- **RNs relacionadas:** RN15

**RF-MLT-05 — Controlar Prazos de Multas** `[Must]`
O sistema DEVE controlar e alertar sobre todos os prazos relevantes: prazo para indicação de condutor, prazo para defesa prévia, prazo para recurso, prazo para pagamento com desconto, vencimento.
- **Saída:** Alertas configuráveis (padrão: 5 dias antes de cada prazo) ao Gestor de Frota e ao condutor/responsável patrimonial.

### Módulo 8: Indicadores e Relatórios (IND)

**RF-IND-01 — Exibir Dashboard de Frota** `[Must]`
O sistema DEVE exibir painel com indicadores da frota, atualizados com dados cacheados (refresh a cada 5 minutos):
- Taxa de disponibilidade (% veículos ativos vs. total)
- Idade média da frota
- TCO médio por veículo e por km
- Valor contábil total da frota (considerando depreciação — RN21)
- Distribuição de veículos por status
- Veículos com manutenção ou documentação pendente
- Reservas com conflito pendente de resolução (RN08)
- Conflitos de sincronização pendentes (RN22)
- **Filtros:** Departamento, Período, Categoria de veículo.
- **Permissões:** FLEET_MGR, DEPT_HEAD (apenas seu departamento), AUDITOR.

**RF-IND-02 — Exibir Dashboard de Manutenção** `[Must]`
O sistema DEVE exibir indicadores de manutenção:
- MTBF e MTTR por veículo e por categoria (calculados conforme seção 3.3)
- Custo total de manutenção no período
- Adesão ao plano preventivo (% de OS preventivas executadas no prazo vs. programadas)
- OS abertas por status e antiguidade
- Top 10 veículos com maior custo de manutenção

**RF-IND-03 — Exibir Dashboard de Consumo** `[Must]`
O sistema DEVE exibir indicadores de consumo:
- Consumo médio (km/l) por veículo e por categoria
- Custo de combustível por km
- Ranking de veículos por eficiência
- Veículos com alerta de consumo ativo (conforme RN19)

**RF-IND-04 — Exibir Dashboard de Viagens** `[Must]`
O sistema DEVE exibir indicadores de viagens:
- Nível de serviço (demandas atendidas vs. não atendidas vs. canceladas)
- Destinos mais frequentes
- Taxa de ocupação da frota (veículos em viagem vs. disponíveis ao longo do tempo)
- Tempo médio entre solicitação e execução
- Viagens com prestação de contas pendente (com flag de atraso conforme RN10)
- Aprovações escalonadas no período (para monitorar efetividade do fluxo de aprovação)

**RF-IND-05 — Gerar Relatórios Síncronos** `[Must]`
O sistema DEVE gerar relatórios de período curto (até 30 dias) de forma síncrona, com tempo de resposta conforme RNF-07. Formatos: PDF, XLSX, CSV.
- Relatórios disponíveis: mensal de frota, veículos subutilizados, infrações, condutores, manutenção.
- **Permissões:** Relatórios filtrados por departamento para DEPT_HEAD. Relatórios completos para FLEET_MGR e AUDITOR.

**RF-IND-06 — Gerar Relatórios Assíncronos** `[Must]`
O sistema DEVE gerar relatórios de período longo (acima de 30 dias) ou consolidados de forma assíncrona:
- (a) Usuário solicita geração do relatório.
- (b) Sistema registra solicitação e inicia processamento em background (worker).
- (c) Usuário recebe notificação (in-app e e-mail) quando o relatório estiver pronto.
- (d) Relatório fica disponível para download por 7 dias.
- Relatórios assíncronos: TCO comparativo anual, relatório anual consolidado, relatório de depreciação da frota, relatório de custos por departamento (anual).
- **Permissões:** FLEET_MGR e AUDITOR.

**RF-IND-07 — Agendar Relatórios Recorrentes** `[Could]`
O sistema DEVE permitir agendamento de geração automática de relatórios (diário, semanal, mensal) com envio por e-mail aos destinatários configurados. Relatórios agendados seguem a mesma lógica síncrona/assíncrona conforme o período.

### Módulo 9: Administração e Importação (ADM)

**RF-ADM-01 — Parametrizar Sistema** `[Must]`
O sistema DEVE permitir a configuração de parâmetros operacionais sem alteração de código:
- Limites de consumo médio por categoria de veículo (para RN19)
- Intervalos de manutenção preventiva por categoria
- Vida útil e taxa de depreciação por categoria de veículo (RN21)
- Prazos de aprovação e escalonamento (RN05)
- Antecedência mínima para solicitação de viagem (RN09)
- Margem de segurança entre viagens (RN06)
- Limites de jornada de direção (RN11)
- Prazo para prestação de contas (RN10)
- Antecedência de alertas (vencimentos de CNH, seguros, licenciamentos, cursos, contratos de terceirizados)
- Período de retenção de relatórios assíncronos (padrão: 7 dias)
- **Permissões:** SYS_ADMIN e FLEET_MGR.

**RF-ADM-02 — Gerenciar Templates de Checklist** `[Must]`
O sistema DEVE permitir criar, editar e versionar templates de checklist utilizados em: inspeções (RF-MAN-07), saída de viagem (RF-VIG-09), retorno de viagem (RF-VIG-11).
- **Entradas:** Nome do template, Categoria de veículo aplicável, Lista de itens (texto, tipo de resposta: conforme/não conforme, texto, numérico, foto), Obrigatoriedade de cada item.

**RF-ADM-03 — Importar Dados Legados** `[Must]`
O sistema DEVE fornecer interface para importação em lote de dados históricos.
- **Dados importáveis:** Veículos (incluindo responsável patrimonial), Condutores (servidores e terceirizados), Histórico de manutenções, Histórico de abastecimentos.
- **Processo:** (a) Upload de arquivo (XLSX conforme template), (b) Validação automática com relatório de inconsistências, (c) Revisão manual dos registros com erro, (d) Confirmação e efetivação pelo Gestor de Frota, (e) Capacidade de rollback completo da importação em até 48h.
- **Nota:** Dados importados de Chassi e Renavam durante a carga inicial não exigem dupla custódia (RN17), pois são operação de primeira carga supervisionada. Alterações posteriores a esses campos seguem RN17 normalmente.
- **Saída:** Relatório detalhado: registros importados, rejeitados (com motivo), e ajustados manualmente.

**RF-ADM-04 — Gerenciar Notificações** `[Must]`
O sistema DEVE possuir motor de notificações centralizado com:
- **Canais:** In-app (obrigatório), E-mail (configurável por tipo de evento).
- **Tipos de evento configuráveis:** Vencimento de CNH, Vencimento de contrato de terceirizado, Vencimento de seguro/licenciamento, Manutenção preventiva próxima, Solicitação de viagem pendente de aprovação, Escalonamento de aprovação, Prestação de contas pendente, Alerta de consumo, Prazo de multa, Viagem em atraso (não retornou na data prevista), Conflito de sincronização pendente (RN22), Reserva afetada por indisponibilidade (RN08), Relatório assíncrono pronto para download.
- **Configuração:** Cada tipo de evento DEVE ter: antecedência configurável, destinatários (por papel ou servidor específico), template de mensagem editável.

**RF-ADM-05 — Gerenciar Acessórios de Veículo** `[Should]`
O sistema DEVE permitir vincular equipamentos ao veículo: extintor, triângulo, rádio, GPS, macaco, estepe, kit primeiros socorros.
- **Entradas por acessório:** Tipo, Número de série (quando aplicável), Data de validade/calibração, Status (presente, ausente, vencido).
- **Saída:** Alertas de validade/calibração. Verificação no checklist de saída.

**RF-ADM-06 — Resolver Conflitos de Sincronização** `[Must]`
O sistema DEVE fornecer interface dedicada para o Gestor de Frota resolver conflitos de sincronização offline (RN22).
- **Interface:** Lista de conflitos pendentes com: registro em conflito, versão do servidor, versão do dispositivo, condutor/dispositivo de origem, timestamps de ambas as versões.
- **Ações:** (a) Aceitar versão do servidor, (b) Aceitar versão do dispositivo, (c) Mesclar manualmente (editar e salvar versão final).
- **Saída:** Conflito resolvido, ambas as versões originais preservadas no log de auditoria, versão final registrada como "Resolução de Conflito" com responsável.
- **RNs relacionadas:** RN22

### Módulo Transversal: Segurança, Auditoria e LGPD (SEC)

**RF-SEC-01 — Autenticar Usuário** `[Must]`
O sistema DEVE autenticar usuários via integração com o sistema de autenticação única da UFMT/SouGov.
- **Fallback:** Se autenticação central indisponível, modo degradado com autenticação local (credenciais sincronizadas previamente) e registro da operação em modo contingência.
- **Sessão:** Timeout por inatividade configurável (padrão: 30 minutos). Sessão única por usuário (novo login invalida sessão anterior).

**RF-SEC-02 — Autorizar por Papel (RBAC)** `[Must]`
O sistema DEVE implementar controle de acesso conforme papéis definidos na seção 5.
- Cada recurso/operação do sistema DEVE ter permissões declaradas por papel.
- Um usuário PODE acumular múltiplos papéis, respeitando restrições de SoD (seção 5.1).
- Alterações de papéis DEVEM ser registradas no log de auditoria.

**RF-SEC-03 — Implementar Dupla Custódia (SoD)** `[Must]`
O sistema DEVE implementar o mecanismo de dupla custódia para operações críticas conforme tabela de SoD (seção 5.1):
- (a) Propositor submete a alteração com justificativa obrigatória.
- (b) Sistema bloqueia a efetivação e notifica o(s) aprovador(es) elegível(is).
- (c) Aprovador (papel diferente do propositor) analisa e aprova ou rejeita.
- (d) Apenas após aprovação a alteração é efetivada.
- (e) Log de auditoria registra: propositor, aprovador, justificativa, valores anterior/posterior, timestamps de cada etapa.
- **RNs relacionadas:** RN17

**RF-SEC-04 — Registrar Trilha de Auditoria** `[Must]`
O sistema DEVE registrar ininterruptamente, para todas as operações de escrita (criação, edição, exclusão, transição de estado, aprovação, login/logout, resolução de conflito de sincronização):
- Identificação do usuário
- Data e hora (timestamp UTC com precisão de milissegundos)
- Endereço IP de origem (ou identificador do dispositivo para operações offline, com IP registrado no momento da sincronização)
- Ação realizada (operação e recurso)
- Dados alterados: valores anteriores e posteriores (diff)
- Para operações de dupla custódia: identificação do propositor e do aprovador
- Para resoluções de conflito: ambas as versões em conflito e a versão final
- **Inviolabilidade:** Conforme RN13, logs não podem ser alterados ou excluídos por nenhum perfil. Retenção mínima: 5 anos.
- **Saída:** Interface de consulta de logs com filtros (usuário, período, tipo de operação, recurso) acessível ao papel AUDITOR e FLEET_MGR.
- **RNs relacionadas:** RN13

**RF-SEC-05 — Gerenciar Consentimento LGPD** `[Must]`
O sistema DEVE apresentar Termo de Uso e Política de Privacidade no primeiro acesso de cada usuário, registrando aceite com timestamp.
- **Base legal:** O sistema trata dados pessoais (CNH, CPF, matrícula, localização, jornada) com base legal de "execução de políticas públicas" (Art. 7º, III e Art. 11, II, b da LGPD) e "cumprimento de obrigação legal" (Art. 7º, II).
- **Portal do titular:** O sistema DEVE disponibilizar ao usuário a capacidade de: consultar quais dados pessoais estão armazenados, solicitar retificação de dados incorretos, exportar seus dados pessoais em formato legível.

**RF-SEC-06 — Anonimizar Dados de Condutores Desligados** `[Must]`
O sistema DEVE anonimizar dados pessoais de condutores cujo vínculo com a UFMT foi encerrado (servidores desligados e terceirizados com contrato encerrado), após período definido na tabela de temporalidade (padrão: 2 anos após desligamento/encerramento).
- **Escopo:** CPF, endereço, telefone, foto da CNH são anonimizados. Matrícula (hash), registro de viagens e dados agregados de desempenho são preservados para fins estatísticos e de auditoria.
- **Processo:** Anonimização é irreversível e registrada no log de auditoria.

---

## 8. Requisitos Não Funcionais (RNF)

### Integração e Interoperabilidade

**RNF-01 — APIs de Integração** `[Must]`
O sistema DEVE possuir APIs RESTful documentadas (OpenAPI 3.0) para integração bidirecional com:
- **SouGov:** Autenticação SSO, consulta de dados de servidores (matrícula, lotação, cargo, situação funcional), e sincronização proativa de afastamentos (férias, licenças) para alimentar a lógica de delegação de aprovação (RN04).
- **SEI:** Geração e vinculação de documentos de autorização, processos de baixa, notificações formais.
- **Sistema de Patrimônio UFMT:** Consulta e conciliação de dados patrimoniais de veículos.
- **Comportamento em falha:** Cada integração DEVE ter estratégia de fallback documentada conforme seção 2.3. O sistema DEVE operar em modo degradado quando dependências externas estiverem indisponíveis, nunca bloqueando operações críticas indefinidamente.
- **Paginação:** Todas as APIs de listagem DEVEM suportar paginação (cursor-based ou offset-based, máximo 100 registros por página).

**RNF-02 — Prontidão para Telemetria** `[Should]`
O sistema DEVE possuir arquitetura preparada para receber e processar dados de APIs de rastreadores GPS (localização, velocidade, status de ignição).
- **Requisitos arquiteturais:** Endpoint de ingestão para dados de telemetria, modelo de dados para armazenamento de posições, abstração de provedor (interface genérica para múltiplos fabricantes de rastreador).
- **Nota:** A integração efetiva com rastreadores específicos é escopo de fase futura.

### Segurança

**RNF-03 — Criptografia em Trânsito** `[Must]`
Toda comunicação entre cliente e servidor DEVE utilizar TLS 1.2 ou superior. Certificados DEVEM ser válidos e emitidos por autoridade certificadora reconhecida.

**RNF-04 — Criptografia em Repouso** `[Must]`
Dados classificados como sensíveis (CNH, CPF, dados de localização GPS, dados de jornada) DEVEM ser armazenados com criptografia AES-256 ou equivalente.

**RNF-05 — Proteção contra Vulnerabilidades** `[Must]`
O sistema DEVE implementar proteções contra as vulnerabilidades OWASP Top 10, incluindo mas não limitado a: injeção (SQL, XSS), autenticação quebrada, exposição de dados sensíveis, controle de acesso quebrado.
- Headers de segurança obrigatórios: Content-Security-Policy, Strict-Transport-Security, X-Content-Type-Options, X-Frame-Options.
- Rate limiting em endpoints de autenticação e APIs públicas.

**RNF-06 — Gestão de Sessão** `[Must]`
- Timeout por inatividade: 30 minutos (configurável).
- Sessão única por usuário (concurrent session control).
- Token de sessão com entropia mínima de 128 bits.
- Invalidação de sessão no logout e na troca de senha.

### Performance

**RNF-07 — Tempo de Resposta** `[Must]`
- Consultas simples (listagens, detalhes): ≤ 2 segundos para p95.
- Operações de escrita (cadastros, aprovações): ≤ 3 segundos para p95.
- Geração de relatórios síncronos (período ≤ 30 dias): ≤ 30 segundos.
- Geração de relatórios assíncronos (período > 30 dias): processados em background worker, sem bloqueio de interface. Tempo de processamento máximo: 10 minutos. Usuário notificado ao concluir.
- Atualização de dashboards: dados cacheados atualizados a cada 5 minutos.

**RNF-08 — Capacidade** `[Must]`
O sistema DEVE suportar:
- 200 usuários simultâneos sem degradação de performance.
- Base de dados de até 500 veículos, 2.000 condutores (servidores + terceirizados) e 50.000 viagens/ano.
- Armazenamento de até 500GB de anexos (documentos, fotos).
- Fila de até 100 relatórios assíncronos simultâneos.

### Disponibilidade e Recuperação

**RNF-09 — Disponibilidade** `[Must]`
O sistema DEVE garantir 99,5% de uptime mensal, excluindo janelas de manutenção programada.
- Janela de manutenção programada: domingos das 02:00 às 06:00 (horário de Cuiabá — UTC-4).
- Métrica: (Tempo total - Tempo indisponível não programado) / Tempo total × 100.

**RNF-10 — Backup e Recuperação** `[Must]`
- **RPO (Recovery Point Objective):** Máximo 1 hora de perda de dados.
- **RTO (Recovery Time Objective):** Máximo 4 horas para restauração completa.
- Backup completo diário, backup incremental a cada hora.
- Testes de restore DEVEM ser executados mensalmente e documentados.
- Logs de auditoria DEVEM ser incluídos no backup com integridade verificável.

### Usabilidade

**RNF-11 — Usabilidade do Portal** `[Must]`
- Solicitação de viagem padrão (rota conhecida, dados do login): completável em no máximo 3 passos de interface (selecionar rota → preencher dados → confirmar envio).
- Interface responsiva para uso em desktop (1024px+) e dispositivos móveis (360px+).

**RNF-12 — PWA com Capacidade Offline e Concorrência Otimista** `[Must]`
A interface mobile DEVE ser implementada como Progressive Web App (PWA) com:
- Capacidade de instalação no dispositivo.
- Funcionalidades offline: visualizar agenda de viagens, preencher checklists, registrar abastecimentos, reportar avarias com fotos, registrar intercorrências.
- **Resolução de conflitos:** Concorrência Otimista (OCC) conforme RN22. Registros com version token divergente entram em fila de conflito para resolução manual pelo Gestor de Frota (RF-ADM-06). Nenhum dado é sobrescrito silenciosamente.
- **Bloqueio local:** Cache local de alocações para bloqueio preventivo de operações duplicadas conforme RN23.
- Sincronização automática quando conectividade for restabelecida, com timestamp original preservado.
- **RNs relacionadas:** RN22, RN23

### Acessibilidade

**RNF-13 — Conformidade e-MAG** `[Must]`
O sistema DEVE estar em conformidade com o Modelo de Acessibilidade em Governo Eletrônico (e-MAG) versão 3.1 e WCAG 2.1 nível AA, conforme obrigatoriedade para órgãos do Governo Federal (Decreto nº 5.296/2004, IN nº 1/2014).

### Observabilidade

**RNF-14 — Monitoramento e Alertas** `[Should]`
O sistema DEVE fornecer:
- Health check endpoint (/health) para monitoramento externo.
- Logging estruturado (formato JSON) de todas as operações de aplicação.
- Métricas de aplicação exportáveis (tempo de resposta, taxa de erro, uso de recursos, tamanho da fila de relatórios assíncronos, número de conflitos de sincronização pendentes).
- Alertas operacionais configuráveis para: indisponibilidade de serviço, taxa de erro acima de 1%, tempo de resposta acima de 5s, fila de relatórios acima de 50 itens.

### Compatibilidade

**RNF-15 — Navegadores e Dispositivos** `[Must]`
O sistema DEVE funcionar corretamente nos seguintes navegadores (últimas 2 versões estáveis): Google Chrome, Mozilla Firefox, Microsoft Edge.
- Resolução mínima desktop: 1024×768.
- Resolução mínima mobile: 360×640.

---

## 9. Casos de Uso Expandidos

### UC01 — Solicitar e Aprovar Viagem

| Campo | Descrição |
|-------|-----------|
| **Atores** | Solicitante, Chefia de Departamento (ou Substituto Legal), Gestor de Frota, Condutor |
| **Pré-condições** | Solicitante autenticado. Existe pelo menos 1 veículo ativo na frota. |
| **Pós-condições (sucesso)** | Viagem alocada com veículo e condutor confirmados. Todos os envolvidos notificados. |

**Fluxo Principal:**
1. Solicitante acessa módulo de viagens e seleciona "Nova Solicitação".
2. Sistema preenche automaticamente dados do solicitante e departamento (via login/SouGov).
3. Solicitante preenche: datas, destino, passageiros (identificados), finalidade, justificativa.
4. Sistema valida antecedência mínima (RN09) e completude dos campos obrigatórios.
5. Solicitante confirma envio.
6. Sistema identifica o aprovador efetivo: verifica se a chefia titular está ativa ou afastada (via dados sincronizados do SouGov — RN04). Se afastada com substituto ativo, roteia diretamente ao substituto.
7. Sistema cria solicitação com status "Aguardando Aprovação" e notifica o aprovador efetivo.
8. Aprovador analisa solicitação e seleciona "Aprovar".
9. Sistema altera status para "Aguardando Alocação" e notifica o Gestor de Frota.
10. Gestor de Frota visualiza demanda e aciona RF-VIG-03 (Análise de Disponibilidade).
11. Gestor seleciona veículo e condutor compatíveis.
12. Sistema valida condutor (RN02 — aplicável a servidores e terceirizados), verifica conflitos (RN06) e confirma alocação.
13. Sistema altera status para "Alocada/Confirmada" e notifica solicitante, condutor(es) e passageiros.

**Fluxos Alternativos:**

- **FA01 — Chefia rejeita (passo 8):** Chefia seleciona "Rejeitar" com justificativa obrigatória. Sistema notifica solicitante com motivo. Status → "Rejeitada". Solicitante pode criar nova solicitação revisada.
- **FA02 — Sem veículo disponível (passo 10):** Gestor de Frota não encontra veículo compatível. Pode: (a) colocar solicitação em fila de espera, (b) sugerir datas alternativas, (c) rejeitar por indisponibilidade.
- **FA03 — Solicitação urgente fora do prazo (passo 4):** Antecedência inferior ao mínimo. Solicita justificativa de urgência. Roteada diretamente ao Gestor de Frota conforme RN09.
- **FA04 — Escalonamento por inércia (passo 8):** Aprovação não ocorre em 2 dias úteis pelo aprovador efetivo (titular ou substituto). Sistema escalona ao superior hierárquico conforme RN05. Superior DEVE marcar checkbox de ciência antes de aprovar.
- **FA05 — Viagem com múltiplos condutores (passo 11):** Duração estimada excede limite de RN11. Sistema exige alocação de segundo condutor.
- **FA06 — Cancelamento pelo solicitante (qualquer passo pré-execução):** Solicitante aciona RF-VIG-05. Recursos liberados, envolvidos notificados.
- **FA07 — Condutor terceirizado (passo 11):** Gestor seleciona condutor terceirizado. Sistema verifica RN02 e adicionalmente: contrato vigente, aprovação de cadastro concluída (RF-CND-02).

**Fluxos de Exceção:**

- **EX01 — SouGov indisponível (passos 2, 6):** Passo 2: preenchimento manual com flag "Pendente de Validação". Passo 6: sistema utiliza último estado de afastamento sincronizado; se cache vazio, roteia ao titular como fallback.
- **EX02 — Falha ao notificar (passos 7, 9, 13):** Sistema registra falha, agenda retry (até 3 tentativas com backoff), disponibiliza pendência no painel do destinatário.

---

### UC02 — Executar Viagem

| Campo | Descrição |
|-------|-----------|
| **Atores** | Condutor, Gestor de Frota |
| **Pré-condições** | Viagem no estado "Alocada/Confirmada". Condutor autenticado no PWA. |
| **Pós-condições (sucesso)** | Viagem concluída com checklists de saída/retorno preenchidos e prestação de contas realizada. |

**Fluxo Principal:**
1. Condutor acessa sua agenda no PWA e seleciona a viagem do dia.
2. Condutor preenche checklist de saída (RF-VIG-09): km inicial, combustível, estado do veículo.
3. Sistema registra horário de saída e transiciona viagem para "Em Andamento".
4. Condutor realiza o deslocamento.
5. No retorno, condutor preenche checklist de retorno (RF-VIG-11): km final, combustível, avarias.
6. Sistema registra horário de retorno e transiciona para "Aguardando Prestação de Contas".
7. Condutor/Solicitante faz upload de comprovantes (RF-VIG-12).
8. Sistema calcula custo real vs. estimado e transiciona para "Concluída".
9. Veículo retorna ao status "Ativo".

**Fluxos Alternativos:**

- **FA01 — Intercorrência durante viagem (passo 4):** Condutor registra ocorrência (RF-VIG-10). Para pane: OS corretiva gerada, Gestor notificado. Para acidente: registro com fotos e BO, prontuário do condutor atualizado. Para sinistro: dispara RF-AST-12 (veículo → "Sinistrado", reservas futuras suspensas via RN08).
- **FA02 — Extensão de viagem (passo 4):** Condutor solicita extensão (RF-VIG-07). Gestor avalia conflitos e aprova/rejeita.
- **FA03 — Substituição de condutor (passo 4):** Condutor indisponível (doença, etc.). Gestor aciona RF-VIG-08, registrando justificativa.
- **FA04 — Avaria no retorno (passo 5):** Checklist indica avaria. OS corretiva gerada. Veículo → "Em Manutenção" se avaria impede uso (dispara RN08).

**Fluxos de Exceção:**

- **EX01 — PWA offline (passos 2, 5, FA01):** Condutor preenche dados offline com timestamp original preservado. Dados sincronizados via OCC (RN22) quando conectividade restabelecida. Se conflito detectado na sincronização, entra na fila de resolução (RF-ADM-06).
- **EX02 — Prestação de contas atrasada (passo 7):** Após 3 dias úteis: alerta. Após 5 dias úteis: pendência no prontuário (RN10). Veículo permanece "Ativo" com flag de pendência.

---

### UC03 — Registrar Abastecimento

| Campo | Descrição |
|-------|-----------|
| **Atores** | Condutor, Gestor de Frota |
| **Pré-condições** | Veículo cadastrado com status Ativo ou Reservado. |
| **Pós-condições (sucesso)** | Abastecimento registrado, consumo médio recalculado, TCO atualizado. |

**Fluxo Principal:**
1. Condutor acessa módulo de insumos (via PWA ou desktop).
2. Seleciona veículo (ou sistema pré-seleciona se condutor está em viagem ativa).
3. Preenche dados de abastecimento (RF-INS-01).
4. Sistema valida: odômetro > último registro, valor total = qtd × preço unitário.
5. Sistema registra abastecimento, recalcula consumo médio e TCO.
6. Se consumo médio desviar da referência (RN19), sistema emite alerta ao gestor.

**Fluxos Alternativos:**

- **FA01 — Registro retroativo (passo 3):** Condutor registra abastecimento de data anterior. Sistema aplica RF-INS-02: exige justificativa, submete a aprovação do gestor.
- **FA02 — Odômetro inconsistente (passo 4):** Leitura menor que última registrada. Sistema bloqueia e oferece: (a) corrigir valor digitado, (b) solicitar correção de odômetro (RF-INS-03) com dupla custódia (RN17).

**Fluxos de Exceção:**

- **EX01 — PWA offline (passo 3):** Registro feito offline com version token local. Na sincronização, se version token divergir (outro abastecimento registrado por outro usuário no intervalo), conflito entra na fila de resolução (RN22, RF-ADM-06).

---

### UC04 — Manutenção Corretiva

| Campo | Descrição |
|-------|-----------|
| **Atores** | Condutor, Gestor de Manutenção, Gestor de Frota |
| **Pré-condições** | Veículo cadastrado. Problema identificado. |
| **Pós-condições (sucesso)** | OS concluída, veículo retornado ao status Ativo, TCO recalculado. |

**Fluxo Principal:**
1. Condutor reporta falha (RF-MAN-03): veículo, descrição, urgência, fotos.
2. Sistema cria OS com status "Pendente de Avaliação" e notifica Gestor de Manutenção.
3. Gestor de Manutenção avalia e aprova OS (status → "Aprovada").
4. Gestor define execução: interna ou externa, solicita orçamentos se externa.
5. Serviço executado (OS → "Em Execução").
6. Gestor registra conclusão (RF-MAN-05): peças, mão de obra, valores, NF, garantia.
7. Sistema transiciona OS para "Concluída", recalcula TCO, retorna veículo para "Ativo".

**Fluxos Alternativos:**

- **FA01 — Urgência crítica (passo 2):** Gestor de Frota notificado imediatamente. Veículo → "Em Manutenção" automaticamente (dispara RN08 para verificação de reservas futuras). OS pode pular para "Em Execução" com aprovação simplificada.
- **FA02 — Aguardando peça (passo 5):** Peça indisponível. OS → "Aguardando Peça". Veículo permanece "Em Manutenção".
- **FA03 — Acionamento de garantia (passo 6):** Problema recorrente coberto por garantia. Gestor aciona RF-MAN-06.

---

### UC05 — Processo de Baixa de Veículo

| Campo | Descrição |
|-------|-----------|
| **Atores** | Gestor de Frota, Gestor de Patrimônio, Comissão de Desfazimento |
| **Pré-condições** | Veículo cadastrado com status Ativo, Inativo ou Sinistrado. |
| **Pós-condições (sucesso)** | Veículo com status "Baixado", dados preservados em base histórica, processo documentado no SEI. |

**Fluxo Principal:**
1. Gestor de Frota/Patrimônio inicia processo de baixa (RF-AST-09): justificativa, destino, laudo técnico.
2. Se veículo possui reservas futuras, sistema exige confirmação de cancelamento/realocação (RN08).
3. Sistema altera status para "Em Processo de Baixa", suspende depreciação e bloqueia novas alocações.
4. Gestor registra etapas conforme RN14: parecer da comissão, autorização, publicação.
5. Cada etapa é registrada com documento SEI vinculado (RF-AST-10).
6. Na etapa final (conclusão do desfazimento), sistema altera status para "Baixado".
7. Dados do veículo movidos para base histórica inativa.

**Fluxos Alternativos:**

- **FA01 — Processo cancelado (passo 4):** Autoridade competente cancela o desfazimento. Veículo retorna a status anterior, depreciação retomada.
- **FA02 — Sinistro irrecuperável (passo 1):** Para veículos no estado "Sinistrado" confirmados como irrecuperáveis, laudo técnico pode ser substituído pelo boletim de ocorrência e parecer da seguradora (quando aplicável). Valor residual para fins de TCO é zero.

---

### UC06 — Registrar Sinistro

| Campo | Descrição |
|-------|-----------|
| **Atores** | Condutor, Gestor de Frota, Gestor de Patrimônio |
| **Pré-condições** | Veículo cadastrado com status Ativo ou Reservado. Ocorrência de roubo, furto ou perda total. |
| **Pós-condições (sucesso)** | Veículo no estado "Sinistrado". Reservas futuras sinalizadas. Seguradora notificada (quando aplicável). |

**Fluxo Principal:**
1. Condutor registra intercorrência tipo sinistro durante viagem (RF-VIG-10) ou Gestor de Frota registra diretamente (RF-AST-12).
2. Sistema altera status do veículo para "Sinistrado".
3. Sistema suspende depreciação do veículo.
4. Sistema executa verificação de reservas futuras (RN08, RF-VIG-13): sinaliza no dashboard do Gestor de Frota, notifica solicitantes afetados.
5. Se veículo possui seguro vigente (RF-INS-09), sistema emite alerta para acionamento da seguradora.
6. Gestor de Frota resolve as reservas afetadas (realocação ou cancelamento).
7. Quando situação do veículo for definida: se recuperável → status retorna a "Ativo" (possivelmente via "Em Manutenção"); se irrecuperável → Gestor inicia processo de baixa (UC05, FA02).

**Fluxos de Exceção:**

- **EX01 — Registro offline (passo 1):** Condutor registra ocorrência offline. Na sincronização, se houver operações conflitantes (ex: outro abastecimento registrado para o mesmo veículo no período do sinistro), entram na fila de conflito (RN22).

---

## 10. Matriz de Rastreabilidade

### RN → RF

| RN | Requisitos Funcionais que Implementam |
|----|---------------------------------------|
| RN01 | RF-VIG-01 |
| RN02 | RF-VIG-04, RF-CND-01, RF-CND-02, RF-CND-03 |
| RN03 | RF-VIG-01, RF-VIG-02 |
| RN04 | RF-VIG-01, RF-VIG-02, RF-ADM-04 |
| RN05 | RF-VIG-02, RF-ADM-04, RF-ADM-01 |
| RN06 | RF-VIG-03, RF-VIG-04 |
| RN07 | RF-AST-05, RF-VIG-03 |
| RN08 | RF-AST-05, RF-AST-12, RF-VIG-13 |
| RN09 | RF-VIG-01, RF-ADM-01 |
| RN10 | RF-VIG-11, RF-VIG-12, RF-ADM-04 |
| RN11 | RF-VIG-04, RF-CND-07, RF-ADM-01 |
| RN12 | RF-VIG-01 |
| RN13 | RF-SEC-04 |
| RN14 | RF-AST-09, RF-AST-10 |
| RN15 | RF-MLT-01, RF-MLT-02, RF-MLT-03, RF-MLT-04, RF-MLT-05 |
| RN16 | RF-AST-01, RF-AST-06, RF-MLT-01, RF-MLT-02 |
| RN17 | RF-AST-02, RF-INS-03, RF-SEC-03 |
| RN18 | RF-INS-01, RF-MAN-05, RF-AST-08, RF-AST-11 |
| RN19 | RF-INS-05, RF-ADM-01 |
| RN20 | RF-MAN-01, RF-MAN-02, RF-INS-08 |
| RN21 | RF-AST-01, RF-AST-08, RF-AST-09, RF-AST-11, RF-AST-12 |
| RN22 | RF-INS-01, RF-VIG-09, RF-VIG-10, RF-VIG-11, RF-MAN-03, RF-ADM-06 |
| RN23 | (implementação no PWA — RNF-12) |

### RF → UC

| Caso de Uso | Requisitos Funcionais Envolvidos |
|-------------|----------------------------------|
| UC01 — Solicitar e Aprovar Viagem | RF-VIG-01, RF-VIG-02, RF-VIG-03, RF-VIG-04, RF-VIG-05, RF-VIG-06, RF-SEC-01, RF-ADM-04 |
| UC02 — Executar Viagem | RF-VIG-09, RF-VIG-10, RF-VIG-11, RF-VIG-12, RF-VIG-07, RF-VIG-08, RF-VIG-13, RF-INS-01, RF-ADM-06 |
| UC03 — Registrar Abastecimento | RF-INS-01, RF-INS-02, RF-INS-03, RF-INS-05, RF-ADM-06 |
| UC04 — Manutenção Corretiva | RF-MAN-03, RF-MAN-04, RF-MAN-05, RF-MAN-06, RF-VIG-13 |
| UC05 — Processo de Baixa | RF-AST-09, RF-AST-10, RF-AST-05, RF-VIG-13 |
| UC06 — Registrar Sinistro | RF-AST-12, RF-VIG-10, RF-VIG-13, RF-INS-09, RF-ADM-06 |

---

## 11. Priorização (MoSCoW) e Faseamento

### Fase 1 — MVP (Meses 1–8)

**Must Have — Funcionalidades mínimas para operação:**

- Módulo AST: RF-AST-01 a RF-AST-06, RF-AST-09, RF-AST-10, RF-AST-11, RF-AST-12
- Módulo INS: RF-INS-01, RF-INS-02, RF-INS-03, RF-INS-05, RF-INS-06, RF-INS-08, RF-INS-09, RF-INS-10
- Módulo MAN: RF-MAN-01 a RF-MAN-05, RF-MAN-07
- Módulo CND: RF-CND-01 a RF-CND-04, RF-CND-06, RF-CND-07
- Módulo VIG: RF-VIG-01 a RF-VIG-05, RF-VIG-09, RF-VIG-11, RF-VIG-12, RF-VIG-13
- Módulo MLT: RF-MLT-01, RF-MLT-02, RF-MLT-05
- Módulo IND: RF-IND-01, RF-IND-05
- Módulo ADM: RF-ADM-01, RF-ADM-02, RF-ADM-03, RF-ADM-04, RF-ADM-06
- Módulo SEC: RF-SEC-01 a RF-SEC-05
- Módulo ROT: RF-ROT-01
- RNFs: RNF-01 (SouGov), RNF-03 a RNF-13, RNF-15

### Fase 2 — Consolidação (Meses 9–14)

**Should Have — Funcionalidades de robustez:**

- Módulo AST: RF-AST-07, RF-AST-08
- Módulo INS: RF-INS-04, RF-INS-07
- Módulo MAN: RF-MAN-06
- Módulo CND: RF-CND-05
- Módulo VIG: RF-VIG-06, RF-VIG-07, RF-VIG-08, RF-VIG-10
- Módulo MLT: RF-MLT-03, RF-MLT-04
- Módulo IND: RF-IND-02 a RF-IND-04, RF-IND-06
- Módulo ADM: RF-ADM-05
- Módulo SEC: RF-SEC-06
- RNFs: RNF-01 (SEI + Patrimônio), RNF-02, RNF-14

### Fase 3 — Evolução (Meses 15–18)

**Could Have — Funcionalidades avançadas:**

- Módulo ROT: RF-ROT-02, RF-ROT-03 (dependente de infraestrutura GPS)
- Módulo IND: RF-IND-07
- Integração efetiva com rastreadores GPS
- Integração com DETRAN para consulta automática de multas

---

## 12. Escopo Futuro Condicionado

### Carsharing Interno (CSH) — `[Won't nesta versão]`

**Justificativa da exclusão:** O módulo de Carsharing (reservas de curta duração para uso administrativo local) foi avaliado e considerado inviável para implantação na versão atual do SIGFROTA. A análise identificou que o Carsharing depende de uma cadeia de custódia física (controle de chaves) que a UFMT não possui digitalizada. Sem um sistema de portaria estruturado e integrado, qualquer solução de software para este módulo seria "software morto" — existiria no sistema mas não refletiria a realidade operacional, criando brechas de compliance e antifraude.

**Pré-requisitos para reintrodução:**
1. Implantação de sistema de controle de portaria nos campi com capacidade de integração digital (API ou interface web), **ou** estruturação de processo formal de custódia de chaves com registro digital auditável.
2. Definição por portaria interna dos locais habilitados para carsharing e dos responsáveis pela custódia.
3. Treinamento dos agentes de portaria/custódia no uso do sistema.

**Especificação preservada:** A especificação técnica dos requisitos RF-CSH-01 a RF-CSH-03 e da RN específica de carsharing (validação de condutor sem aprovação hierárquica) está documentada no anexo técnico do projeto para implantação futura sem retrabalho de análise, assim que os pré-requisitos forem atendidos.

**Tratamento transitório:** Até a implantação do módulo CSH, deslocamentos administrativos locais de curta duração DEVEM seguir o fluxo padrão de solicitação de viagem (RF-VIG-01 a RF-VIG-12) com aprovação hierárquica normal, conforme RN01.

---

## 13. Considerações Finais

Este DRS v3.0 consolida três ciclos de análise e revisão, incorporando:

- **Concorrência Otimista (OCC)** para sincronização offline, substituindo a abordagem perigosa de last-write-wins por um mecanismo que preserva a cadeia de custódia e a integridade dos dados em cenários de conectividade intermitente — realidade operacional do Mato Grosso.
- **Segregação de Funções (SoD) e Dupla Custódia** para operações críticas sobre dados patrimoniais, blindando o sistema contra fraudes e atendendo exigências do TCU.
- **Harmonização de delegação/escalonamento** (RN04/RN05) com sincronização proativa de afastamentos do SouGov, eliminando o race condition entre as duas regras.
- **Responsável Patrimonial** como conceito obrigatório, resolvendo a lacuna de multas sem condutor identificável e fortalecendo a accountability patrimonial.
- **Condutores terceirizados** como cidadãos de primeiro nível no sistema, com fluxo de cadastro próprio e validações adequadas.
- **Relatórios assíncronos** para consultas pesadas, garantindo que relatórios anuais de TCO não degradem a performance do sistema.
- **Ciência obrigatória em escalonamento** como mitigação de UX contra "aprovação borrachada".
- **Sinistro (roubo/furto/perda total)** como fluxo completo integrado a seguros, baixa patrimonial e suspensão de reservas.
- **Depreciação patrimonial** incorporada ao TCO, com requisito funcional dedicado (RF-AST-11) e regra de cálculo explícita (RN21).
- **Carsharing removido do escopo ativo** com justificativa técnica e pré-requisitos documentados para reintrodução futura.

### Fatores Críticos de Sucesso

1. **Dados legados:** A importação e padronização da base histórica (RF-ADM-03) é pré-requisito para a credibilidade dos indicadores e do TCO. Nota: a carga inicial de Chassi/Renavam dispensa dupla custódia.
2. **Perfis de acesso e SoD:** O estabelecimento dos papéis e das restrições de segregação de funções em conformidade com as portarias internas da UFMT deve preceder a implantação.
3. **Sincronização SouGov:** A qualidade e frequência da sincronização de afastamentos com o SouGov é determinante para o funcionamento correto da delegação automática de aprovação (RN04).
4. **Capacitação:** Treinamento de gestores de frota (incluindo resolução de conflitos de sincronização), condutores (uso do PWA offline) e aprovadores é essencial para adesão.
5. **Infraestrutura PWA:** A viabilidade do PWA com OCC (RNF-12) é determinante para a adoção por condutores em viagens rurais. Testes de campo em áreas sem cobertura são obrigatórios antes do go-live.
6. **Testes de concorrência:** Cenários de conflito de sincronização DEVEM ser cobertos por testes automatizados e simulações de campo antes da implantação.

---

*Fim do Documento — SIGFROTA DRS v3.0*
