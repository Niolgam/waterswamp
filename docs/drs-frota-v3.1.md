# Documento de Requisitos de Software (DRS)

## Módulo Frota — Plataforma de Gestão UFMT
### Universidade Federal de Mato Grosso

---

| Campo | Valor |
|-------|-------|
| **Versão** | 3.1 |
| **Data** | 19/04/2026 |
| **Status** | Em Revisão |
| **Autor original** | Equipe UFMT |
| **Revisado por** | Análise de Engenharia de Requisitos + Auditoria de Conformidade |
| **Aprovado por** | *(Pendente)* |

### Histórico de Revisões

| Versão | Data | Autor | Descrição |
|--------|------|-------|-----------|
| 1.0 | — | Equipe UFMT | Versão inicial do DRS |
| 2.0 | 27/03/2026 | Revisão técnica | Decomposição atômica, LGPD, segurança, casos de uso expandidos, rastreabilidade, MoSCoW |
| 3.0 | 27/03/2026 | Auditoria + Consolidação | OCC, harmonização delegação/escalonamento, SoD, responsável patrimonial, terceirizados, relatórios assíncronos, sinistros, Carsharing → Won't |
| 3.1 | 19/04/2026 | Revisão operacional | Validação de condutor até data de retorno, antecedência por finalidade em horas úteis, abastecimento/manutenção por contrato com importação configurável (CATMAT/CATSER), remoção de IPVA/DPVAT (imunidade/extinção), contingência para perda de dispositivo offline, material informativo em veículos, telemetria/GPS/TLS/responsável patrimonial como referência futura, gestão de pneus em escopo futuro, renomeação para "Módulo Frota" |

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
13. Referências Normativas Futuras
14. Considerações Finais

---

## 1. Introdução

### 1.1. Propósito

Este documento especifica os requisitos de software para o Módulo Frota da Plataforma de Gestão da Universidade Federal de Mato Grosso (UFMT). Constitui a base autoritativa para desenvolvimento, testes, homologação e auditoria da solução.

### 1.2. Base Legal

| Norma | Descrição | Aplicação no Módulo |
|-------|-----------|---------------------|
| **Lei nº 9.327/1996** | Dispõe sobre a condução de veículo oficial | Exige autorização (credenciamento) do dirigente máximo para servidor não-motorista conduzir veículo oficial. Base legal do fluxo de credenciamento. |
| **Decreto nº 9.287/2018** | Utilização de veículos oficiais na administração federal | Classificação de veículos, vedações de uso, regras de guarda e identificação. |
| **IN SLTI/MPOG nº 3/2008** | Classificação, utilização, aquisição e alienação de veículos oficiais | Registro obrigatório de identificação do condutor, origem, destino, finalidade, horários e quilometragens. |
| **Decreto nº 9.373/2018** | Alienação e gestão de bens imóveis e móveis | Rito de desfazimento patrimonial de veículos. |
| **Lei nº 13.709/2018 (LGPD)** | Proteção de dados pessoais | Tratamento de CNH, CPF, dados de jornada e localização. |
| **CF Art. 150, VI, "a"** | Imunidade tributária recíproca | UFMT é imune ao IPVA (autarquia federal). |

### 1.3. Público-Alvo

- Equipe de desenvolvimento
- Gestores de frota e administradores da UFMT
- Auditoria interna e órgãos de controle (CGU/TCU)
- Usuários-chave para validação

### 1.4. Convenções do Documento

- **DEVE/OBRIGATÓRIO**: Requisito mandatório para a fase indicada.
- **DEVERIA**: Requisito fortemente recomendado, pode ser negociado com justificativa.
- **PODE**: Requisito desejável, implementado conforme disponibilidade.
- Cada requisito possui identificador único no formato `[MÓDULO]-[SEQ]` (ex: `RF-AST-01`).
- Requisitos são atômicos: cada ID cobre uma única capacidade testável.
- Prioridade segue classificação MoSCoW (Must/Should/Could/Won't nesta versão).

---

## 2. Escopo

### 2.1. Escopo Incluído

O Módulo Frota abrangerá o ciclo de vida completo da frota institucional da UFMT:

- **Gestão patrimonial**: aquisição, registro, transferência, depreciação, desfazimento, baixa, sinistros.
- **Controle operacional**: abastecimentos (via contrato com importação configurável), seguros, licenciamentos, acessórios.
- **Manutenção**: planos preventivos, solicitações corretivas, ordens de serviço, inspeções (via contrato com importação configurável).
- **Condutores**: credenciamento conforme Lei 9.327/96 (servidores e terceirizados), treinamentos, avaliação, jornada.
- **Viagens**: solicitação, aprovação hierárquica, alocação, execução, prestação de contas, cancelamento, extensão, incidentes.
- **Rotas**: cadastro de rotas padrão, roteirização simplificada.
- **Multas**: registro, vinculação ao condutor, recursos, ressarcimento.
- **Indicadores e relatórios**: dashboards operacionais, relatórios gerenciais (síncronos e assíncronos), exportações.
- **Catálogos**: tipos de combustível vinculados ao CATMAT, tipos de serviço de manutenção vinculados ao CATSER.
- **Importação de dados legados**: migração estruturada da base histórica.
- **Integrações**: SouGov (RH/autenticação), SEI (documentos), Sistema de Patrimônio.

### 2.2. Escopo Excluído

- Gestão de combustível em nível de estoque de posto próprio.
- Integração com DETRAN para consulta automática de multas (escopo futuro).
- Aplicativo nativo mobile (será PWA responsivo com capacidade offline).
- Gestão financeira/orçamentária (o sistema fornece dados de custo mas não executa pagamentos).
- Carsharing interno — escopo futuro condicionado (seção 12).
- Telemetria e rastreamento GPS — escopo futuro condicionado (seção 12).
- Auditoria de trajeto realizado (dependente de rastreamento GPS).
- Gestão de pedágios (não há pedágio nas rotas habituais da UFMT no MT; quando necessário, registro manual na prestação de contas da viagem).
- Gestão de pneus — escopo futuro (seção 12).

### 2.3. Dependências Externas

| Dependência | Tipo | Impacto se Indisponível |
|-------------|------|-------------------------|
| SouGov | Autenticação, dados de servidores, afastamentos | Sistema opera em modo degradado com cache local |
| SEI | Geração de documentos formais | Documentos gerados localmente para inserção manual no SEI |
| Sistema de Patrimônio UFMT | Dados patrimoniais de veículos | Cadastro manual com conciliação posterior |
| Catálogo CATMAT/CATSER | Referência para combustíveis e serviços | Catálogo local sincronizado periodicamente |
| Conectividade Internet (áreas rurais) | Operação do PWA em campo | PWA opera offline com OCC e fila de resolução de conflitos |

---

## 3. Glossário, Definições e Siglas

### 3.1. Termos do Domínio

| Termo | Definição |
|-------|-----------|
| **Condutor** | Pessoa devidamente credenciada conforme Lei 9.327/96, autorizada a conduzir veículos da frota. Pode ser: (a) servidor público federal com portaria de autorização do dirigente máximo, ou (b) motorista terceirizado com contrato vigente. Termo oficial em todo o sistema. |
| **Credenciamento** | Ato administrativo formal (portaria publicada) que autoriza o servidor a conduzir veículos oficiais, conforme Lei 9.327/96. Possui validade definida e requer CNH compatível vigente durante todo o período. |
| **Solicitante** | Servidor que requisita uso de veículo para finalidade institucional. Pode ou não ser o condutor. |
| **Gestor de Frota** | Servidor designado por portaria para administrar operacionalmente a frota. |
| **Viagem** | Deslocamento institucional com veículo e condutor alocados, checklists obrigatórios e prestação de contas. |
| **Reserva** | Solicitação aprovada para período futuro, com recursos alocados mas viagem não iniciada. |
| **Ordem de Serviço (OS)** | Documento que autoriza e registra execução de manutenção, preventiva ou corretiva. |
| **TCO** | Total Cost of Ownership — soma de aquisição, operação, manutenção e depreciação, deduzido o valor residual. |
| **Fornecedor de Abastecimento** | Empresa contratada via licitação para fornecimento de combustível. Motoristas utilizam cartão de abastecimento vinculado ao contrato. |
| **Fornecedor de Manutenção** | Empresa contratada via licitação para prestação de serviços de manutenção veicular. |
| **OCC** | Concorrência Otimista — cada registro sincronizável carrega version token; conflitos na sincronização entram em fila de resolução manual. |
| **Dupla Custódia (SoD)** | Segregação de Funções: operações críticas exigem dois usuários distintos (propositor ≠ aprovador). |

### 3.2. Siglas

| Sigla | Significado |
|-------|-------------|
| CATMAT | Catálogo de Materiais do Governo Federal |
| CATSER | Catálogo de Serviços do Governo Federal |
| CRLV | Certificado de Registro e Licenciamento de Veículo |
| CNH | Carteira Nacional de Habilitação |
| CTB | Código de Trânsito Brasileiro |
| e-MAG | Modelo de Acessibilidade em Governo Eletrônico |
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
- Quando dados de aquisição estiverem ausentes (veículos legados), utiliza-se valor FIPE na data de incorporação.
- **Depreciação:** Linear. `Depreciação_Anual = (Valor_Aquisição - Valor_Residual_Mínimo) / Vida_Útil_Anos`.
- **Valor Residual Estimado:** `Valor_Aquisição - Depreciação_Acumulada`, piso R$ 0,00.

**Consumo Médio:**
```
Consumo_Médio (km/l) = (Odômetro_Atual - Odômetro_Abastecimento_Anterior) / Litros_Abastecidos
```

**MTBF / MTTR:** Conforme definições padrão (seção 3.2). Veículos sem falhas no período: MTBF = tempo total do período.

### 3.4. Regra de Contagem de Horas Úteis

Para fins de cálculo de antecedência mínima (RN09) e prazos operacionais:
- **Horário útil**: segunda a sexta, 08:00 às 18:00 (horário de Cuiabá, UTC-4).
- **Excluídos**: sábados, domingos, feriados nacionais e feriados locais cadastrados no sistema.
- **Exemplo**: solicitação feita sexta-feira às 19:00 para segunda-feira às 07:00 = 0 horas úteis (todo o intervalo cai fora do horário útil).

---

## 4. Visão Geral da Arquitetura Funcional

| # | Módulo | Código | Descrição |
|---|--------|--------|-----------|
| 1 | Gestão de Ativos | AST | Ciclo de vida patrimonial, depreciação, sinistros |
| 2 | Gestão de Insumos | INS | Abastecimentos (contrato/importação), seguros, licenciamentos |
| 3 | Gestão de Manutenção | MAN | Preventiva, corretiva, OS, inspeções (contrato/importação) |
| 4 | Gestão de Condutores | CND | Credenciamento (Lei 9.327/96), treinamentos, avaliação, jornada |
| 5 | Gestão de Rotas | ROT | Rotas padrão, roteirização simplificada |
| 6 | Gestão de Viagens | VIG | Solicitação, aprovação, execução, prestação de contas |
| 7 | Gestão de Multas | MLT | Registro, vinculação, recursos, ressarcimento |
| 8 | Indicadores e Relatórios | IND | Dashboards, relatórios síncronos/assíncronos |
| 9 | Administração e Catálogos | ADM | Parametrização, catálogos CATMAT/CATSER, importação legada, notificações |
| T | Transversal: Segurança, Auditoria, LGPD | SEC | Autenticação, RBAC, SoD, auditoria, privacidade |

---

## 5. Perfis de Acesso (Papéis)

| Papel | Código | Descrição |
|-------|--------|-----------|
| Administrador do Sistema | SYS_ADMIN | Configuração técnica, parametrização. Sujeito a SoD. |
| Gestor de Frota | FLEET_MGR | Gestão operacional: alocar veículos, aprovar OS, gerenciar condutores, resolver conflitos. |
| Gestor de Manutenção | MAINT_MGR | Gestão de manutenções: criar/aprovar OS, registrar execuções, solicitar manutenção corretiva. |
| Chefia de Departamento | DEPT_HEAD | Aprovação hierárquica, relatórios departamentais. |
| Condutor | DRIVER | Operação em campo: agenda, checklists, abastecimentos, avarias. Servidores e terceirizados. |
| Solicitante | REQUESTER | Requisição de veículos, acompanhamento, cancelamento. |
| Gestor de Patrimônio | ASSET_MGR | Gestão patrimonial: aquisições, transferências, baixas. |
| Auditor | AUDITOR | Consulta irrestrita somente-leitura. Segundo aprovador em SoD. |

### 5.1. Restrições de Segregação de Funções (SoD)

| Operação Crítica | Propositor | Aprovador | Restrição |
|------------------|-----------|-----------|-----------|
| Alteração de Chassi/Renavam | SYS_ADMIN | AUDITOR ou outro SYS_ADMIN | Propositor ≠ Aprovador |
| Correção de odômetro | FLEET_MGR | SYS_ADMIN ou AUDITOR | Propositor ≠ Aprovador |
| Exclusão de log de auditoria | — | — | **Proibido para todos os papéis** |

---

## 6. Regras de Negócio (RN)

### Regras de Condutores e Credenciamento

**RN01 — Obrigatoriedade de Registro:** Toda utilização de veículo da frota DEVE ser registrada no sistema como viagem, incluindo deslocamentos locais e aulas de campo. Nenhum veículo pode sair sem registro ativo.

**RN02 — Validação de Condutor até Data de Retorno:** Um condutor só pode ser alocado a uma viagem se, **durante todo o período previsto (da data de saída até a data de retorno estimada)**, cumulativamente: (a) credenciamento ativo (portaria vigente conforme Lei 9.327/96), (b) CNH válida com categoria compatível com o veículo, (c) nenhum curso obrigatório vencido, (d) pontuação da CNH abaixo do limite suspensivo. Para terceirizados, adicionalmente: (e) contrato de prestação de serviço vigente. Se qualquer documento vencer durante o período previsto da viagem, a alocação DEVE ser bloqueada e o sistema DEVE informar a data de vencimento e o item impeditivo.

**RN03 — Limite de Jornada de Direção:** O sistema DEVE impedir alocação de condutor que ultrapasse 8 horas de direção contínua ou 10 horas diárias. Viagens que excedam esses limites exigem condutor adicional para revezamento.

### Regras de Aprovação e Fluxo

**RN04 — Aprovação Hierárquica:** Toda solicitação de viagem requer aprovação da chefia imediata do departamento solicitante antes da alocação de recursos.

**RN05 — Delegação de Aprovação:** Delegação segue lógica:
- (a) Sistema sincroniza proativamente afastamentos do SouGov.
- (b) Se aprovador titular afastado com substituto ativo, roteia diretamente ao substituto sem espera.
- (c) Delegação registrada no log de auditoria.

**RN06 — Escalonamento por Inércia:** Exclusivamente por omissão do aprovador efetivo:
- (a) Solicitação não analisada em 2 dias úteis → escalonamento ao nível hierárquico superior.
- (b) Aprovador escalonado DEVE marcar checkbox obrigatório: "Declaro ciência de que esta aprovação foi escalonada por inércia do aprovador [Nome/Matrícula] e que analisei o mérito da solicitação."

**RN07 — Antecedência Mínima por Finalidade:** Solicitações de viagem DEVEM respeitar antecedência mínima **em horas úteis** (conforme regra 3.4), configurável por finalidade institucional:

| Finalidade | Antecedência Mínima (horas úteis) | Justificativa |
|------------|-----------------------------------|---------------|
| Atividade administrativa | 48h úteis | Padrão operacional |
| Aula de campo | 72h úteis | Coordenação de alunos, seguros acadêmicos, autorizações |
| Evento institucional | 72h úteis | Logística de participantes e materiais |
| Viagem interestadual | 96h úteis | Planejamento de rota, hospedagem, diárias |
| Emergência/Urgência | Sem antecedência mínima | Exige justificativa obrigatória e aprovação direta do Gestor de Frota |

Solicitações fora do prazo (exceto emergência) exigem justificativa obrigatória e aprovação direta do Gestor de Frota, com registro da excepcionalidade. Os valores de antecedência e as finalidades são configuráveis pelo SYS_ADMIN (RF-ADM-01).

### Regras de Agendamento e Disponibilidade

**RN08 — Conflito de Agendamento:** O sistema DEVE impedir alocação de mesmo veículo ou condutor para viagens com sobreposição, considerando margem configurável (padrão: 1 hora entre viagens).

**RN09 — Indisponibilidade por Status:** Veículos com status "Em Manutenção", "Inativo", "Em Processo de Baixa", "Baixado" ou "Sinistrado" não podem ser listados como disponíveis.

**RN10 — Suspensão Preventiva de Reservas:** Quando veículo transicionar para "Em Manutenção" ou "Sinistrado": (a) sinalizar reservas futuras (até 30 dias) no dashboard do Gestor, (b) notificar Gestor e solicitantes, (c) exigir resolução (realocação ou cancelamento) em 1 dia útil.

### Regras de Execução e Prestação de Contas

**RN11 — Prestação de Contas Obrigatória:** Conclusão de viagem requer: checklist de retorno, quilometragem final, comprovantes de despesas. Prazo: 3 dias úteis → alerta. 5 dias úteis → pendência no prontuário do condutor.

**RN12 — Finalidade Institucional:** Toda viagem DEVE ter finalidade institucional documentada. Destinos não correspondentes a locais institucionais cadastrados são sinalizados para revisão do Gestor de Frota.

### Regras de Auditoria e Segurança

**RN13 — Inviolabilidade do Log de Auditoria:** Logs não podem ser alterados/excluídos por nenhum perfil. Retenção mínima: 5 anos.

**RN14 — Desfazimento Patrimonial:** Baixa segue Decreto 9.373/2018: laudo, parecer da comissão, autorização, publicação. Cada etapa vinculada ao SEI.

**RN15 — Ressarcimento de Multas:** Processo administrativo com: notificação formal, 10 dias úteis para defesa, decisão fundamentada.

**RN16 — Dupla Custódia para Dados Críticos:** Alterações em Chassi/Renavam e correções de odômetro exigem dois usuários distintos conforme SoD (seção 5.1).

### Regras de Cálculo

**RN17 — Cálculo de TCO:** Recalculado a cada novo custo e mensalmente (depreciação). Fórmula na seção 3.3.

**RN18 — Alertas de Consumo:** Alerta quando consumo médio desviar negativamente >15% (configurável) da média dos últimos 6 meses da categoria. Média recalculada mensalmente.

**RN19 — Projeção de Manutenção Preventiva:** Gatilho = menor entre km, tempo ou horas de uso. Projeção usa taxa média de utilização dos últimos 3 meses.

**RN20 — Depreciação Patrimonial:** Método linear. Taxa e vida útil configuráveis por categoria. Cessa ao atingir valor residual mínimo ou na baixa.

### Regras de Importação de Dados Contratuais

**RN21 — Validação Pré-Commit de Importações:** Toda importação de dados de abastecimento ou manutenção via arquivo DEVE passar por validação completa antes de efetivar os registros:
- (a) Todos os campos são validados conforme regras do requisito funcional correspondente.
- (b) Cupons fiscais/NFs duplicados (mesmo número + mesmo fornecedor) DEVEM ser reportados e impedir a importação do registro duplicado, exigindo verificação manual.
- (c) Registros inválidos são reportados com motivo específico por linha.
- (d) Somente após revisão e confirmação do Gestor de Frota os registros válidos são efetivados.
- (e) Nenhum registro é gravado no banco antes da confirmação (commit explícito).

### Regras de Sincronização Offline

**RN22 — Concorrência Otimista (OCC):** Toda operação offline utiliza version token. Conflitos entram em fila de resolução manual do Gestor de Frota. Ambas versões preservadas até resolução.

**RN23 — Bloqueio de Alocação Duplicada Offline:** PWA armazena localmente alocações vigentes. Bloqueio preventivo se veículo consta alocado a outro condutor no cache local.

**RN24 — Contingência por Perda de Dispositivo:** Se o condutor perder o dispositivo móvel durante viagem (com dados offline não sincronizados), o Gestor de Frota DEVE poder preencher retroativamente todos os dados da viagem (checklists de saída/retorno, abastecimentos, intercorrências) via interface desktop, exigindo: (a) justificativa obrigatória, (b) Boletim de Ocorrência da perda/roubo do dispositivo (anexo obrigatório), (c) registro de que os dados foram inseridos por terceiro. Os registros retroativos recebem flag "Contingência — dados inseridos pelo gestor" no log de auditoria.

---

## 7. Requisitos Funcionais (RF)

### Módulo 1: Gestão de Ativos (AST)

**RF-AST-01 — Cadastrar Veículo** `[Must]`
O sistema DEVE permitir o cadastro de um novo veículo.
- **Entradas obrigatórias:** Placa, Chassi, Renavam, Marca, Modelo, Ano Fab./Mod., Cor, Categoria (CTB), Tipo de Combustível (vinculado ao catálogo CATMAT — RF-ADM-07), Departamento responsável, Número de Patrimônio.
- **Entradas opcionais:** Capacidade passageiros, Capacidade carga (kg), Potência (cv), Dados de aquisição (NF, valor, data, processo licitatório), Especificações adicionais.
- **Validações:** Chassi, Renavam e Placa únicos. Formato de placa Mercosul ou antigo.
- **Saída:** Veículo com status "Ativo", depreciação aplicada conforme categoria (RF-AST-11).
- **RNs:** RN20

**RF-AST-02 — Editar Dados de Veículo** `[Must]`
- **Editáveis livremente** (FLEET_MGR, ASSET_MGR): Cor, Departamento, Capacidade, Especificações.
- **Editáveis com dupla custódia** (RN16): Chassi, Renavam, Placa.
- **Saída:** Log de auditoria com valores anterior/posterior. Dupla custódia: log com propositor + aprovador.

**RF-AST-03 — Consultar Veículos** `[Must]`
Filtros: placa, marca, modelo, status, departamento, categoria, ano, faixa de km. Lista paginada com TCO atual.

**RF-AST-04 — Gerenciar Anexos de Veículo** `[Must]`
Upload/visualização/substituição de: CRLV, NF de aquisição, Apólices, Laudos, Fotos. Formatos: PDF, JPG, PNG (máx. 10MB). Versão histórica preservada.

**RF-AST-05 — Gerenciar Status de Veículo** `[Must]`
Estados: Ativo, Em Manutenção, Reservado, Inativo, Em Processo de Baixa, Baixado, Sinistrado.
- **Transições permitidas:**
  - Ativo → Em Manutenção, Reservado, Inativo, Em Processo de Baixa, Sinistrado
  - Em Manutenção → Ativo
  - Reservado → Ativo, Em Manutenção
  - Inativo → Ativo, Em Processo de Baixa
  - Em Processo de Baixa → Baixado, Ativo (processo cancelado)
  - Sinistrado → Em Processo de Baixa, Ativo (recuperado)
  - Baixado → *(terminal)*
- Ao transicionar para "Em Manutenção" ou "Sinistrado": dispara RN10.
- **RNs:** RN09, RN10

**RF-AST-06 — Registrar Transferência de Departamento** `[Must]`
Entradas: Origem, Destino, Data efetiva, Motivo, Documento SEI. Saída: Histórico de transferência.

**RF-AST-07 — Registrar Identificação Visual** `[Should]`
Adesivação: tipo, data aplicação/validade, fornecedor, fotos. Alerta 30 dias antes do vencimento.

**RF-AST-08 — Calcular Projeção de Substituição** `[Should]`
Ranking de prioridade por: idade, km, TCO, valor contábil residual, manutenções corretivas no último ano.

**RF-AST-09 — Iniciar Processo de Baixa** `[Must]`
Entradas: Justificativa, Destino pretendido, Laudo técnico (obrigatório). Status → "Em Processo de Baixa". Depreciação suspensa. **RNs:** RN09, RN10, RN14, RN20

**RF-AST-10 — Registrar Etapas do Desfazimento** `[Must]`
Cada etapa conforme RN14 com documento SEI. Na conclusão: status → "Baixado", dados para base histórica. **RN:** RN14

**RF-AST-11 — Configurar e Calcular Depreciação** `[Must]`
Configuração por categoria: vida útil, valor residual mínimo, método linear. Cálculo mensal automático. **RN:** RN17, RN20

**RF-AST-12 — Registrar Sinistro** `[Must]`
Entradas: Tipo (Roubo, Furto, Perda Total), Data/hora, Local, BO (obrigatório), Nº sinistro seguradora, Fotos.
Comportamento: Status → "Sinistrado", suspensão de depreciação, dispara RN10, alerta para seguradora se apólice vigente.
**RNs:** RN09, RN10

### Módulo 2: Gestão de Insumos (INS)

**RF-INS-01 — Registrar Abastecimento Manual** `[Must]`
O sistema DEVE permitir registro manual de abastecimento.
- **Entradas:** Veículo (placa), Data, Hora, Local (posto/cidade), Tipo de Combustível (seleção do catálogo CATMAT — RF-ADM-07), Quantidade (litros), Valor Unitário, Valor Total, Leitura do Odômetro, Condutor responsável, Número do cupom fiscal, Centro de custo.
- **Validações:** Odômetro > último registro. Valor Total = Qtd × Preço (tolerância R$ 0,05). Veículo Ativo ou Reservado. Cupom fiscal único (mesmo número + mesmo fornecedor = bloqueio por duplicidade).
- **Saída:** Registro criado, consumo médio recalculado, TCO atualizado.
- **RNs:** RN17, RN18, RN22

**RF-INS-02 — Registrar Abastecimento Retroativo** `[Must]`
Mesmas entradas de RF-INS-01 + justificativa obrigatória. Requer aprovação do Gestor de Frota. Recálculo de consumo na ordem cronológica correta. Flag "retroativo".

**RF-INS-03 — Corrigir Leitura de Odômetro** `[Must]`
Entradas: Veículo, Nova leitura, Motivo, Documento comprobatório. Dupla custódia (RN16). Recálculo de consumo. **RN:** RN16

**RF-INS-04 — Configurar Fornecedor de Abastecimento** `[Must]`
O sistema DEVE permitir o cadastro e configuração de fornecedores de abastecimento (empresas contratadas).
- **Entradas:** Razão social, CNPJ, Número do contrato, Vigência (início/fim), Contato.
- **Configuração de importação:** Para cada fornecedor, o Gestor de Frota DEVE poder configurar o mapeamento dos campos do arquivo de importação:
  - Formato do arquivo (CSV, XLSX, TXT)
  - Encoding (UTF-8, ISO-8859-1, etc.)
  - Separador (para CSV: vírgula, ponto-e-vírgula, tab)
  - Mapeamento de colunas: qual coluna do arquivo corresponde a cada campo do sistema (Placa, Data, Hora, Tipo Combustível, Quantidade, Valor Unitário, Valor Total, Odômetro, Cupom Fiscal, Posto)
  - Formato de data (dd/mm/aaaa, aaaa-mm-dd, etc.)
  - Formato numérico (separador decimal: vírgula ou ponto)
- **Saída:** Fornecedor cadastrado com template de importação configurado.

**RF-INS-05 — Importar Abastecimentos de Fornecedor** `[Must]`
O sistema DEVE permitir importação de registros de abastecimento a partir de arquivos enviados pelo fornecedor contratado.
- **Processo:**
  1. Gestor seleciona o fornecedor (RF-INS-04) e faz upload do arquivo.
  2. Sistema aplica o mapeamento configurado para interpretar os campos.
  3. Sistema executa validação completa pré-commit conforme RN21: todos os campos validados, duplicidade de cupom fiscal detectada, registros inválidos reportados com motivo por linha.
  4. Sistema apresenta relatório de validação: registros válidos, rejeitados (com motivo), duplicados (com referência ao registro existente).
  5. Gestor revisa, pode corrigir registros individuais, e confirma importação.
  6. Sistema efetiva os registros válidos confirmados (commit), recalcula consumo médio e TCO.
- **Tamanho máximo:** 5MB / 5.000 registros por importação.
- **RNs:** RN17, RN18, RN21

**RF-INS-06 — Emitir Alertas de Consumo** `[Must]`
Notificação ao Gestor quando: consumo desvia conforme RN18, ou gasto departamental ultrapassa limite. **RN:** RN18

**RF-INS-07 — Registrar Aplicação de Insumos** `[Must]`
Insumos consumíveis: óleos, filtros, baterias, fluidos (NÃO pneus — ver seção 12). Entradas: Veículo, Tipo, Marca, Quantidade, Km na troca, Data, NF, Próxima troca projetada. **RN:** RN19

**RF-INS-08 — Registrar Apólice de Seguro** `[Must]`
Entradas: Seguradora, Nº apólice, Cobertura, Prêmio, Vigência, Franquia, Documento. Alerta 30 dias antes do vencimento. Custo no TCO. Alerta de acionamento em sinistro.

**RF-INS-09 — Registrar Licenciamento e Taxas** `[Must]`
O sistema DEVE registrar taxas de licenciamento e vistorias obrigatórias.
- **Nota:** A UFMT, como autarquia federal, é imune ao IPVA (CF Art. 150, VI, "a"). O DPVAT/SPVAT foi extinto. Portanto, os registros aplicáveis são: taxa de licenciamento anual, taxa de vistoria (quando exigível), outras taxas obrigatórias que venham a ser criadas.
- **Entradas:** Veículo, Tipo de taxa, Exercício/ano, Valor, Data de pagamento, Comprovante.
- **Saída:** Alerta de vencimento. Registro de pendências por veículo.

### Módulo 3: Gestão de Manutenção (MAN)

**RF-MAN-01 — Configurar Plano de Manutenção Preventiva** `[Must]`
Planos por categoria: tipo de serviço (vinculado ao catálogo de serviços — RF-ADM-08), intervalos (km, tempo, horas), checklist. **RN:** RN19

**RF-MAN-02 — Gerar OS Preventiva Automaticamente** `[Must]`
OS "Programada" quando gatilho atingido. Notificação ao Gestor de Manutenção. **RN:** RN19

**RF-MAN-03 — Solicitar Manutenção Corretiva** `[Must]`
O sistema DEVE permitir que **condutores, Gestores de Frota e Gestores de Manutenção** reportem falhas ou problemas.
- **Entradas:** Veículo, Descrição, Urgência (Baixa/Média/Alta/Crítica), Imagens (até 5), Localização.
- **Saída:** OS "Pendente de Avaliação". Notificação ao Gestor de Manutenção.
- **Urgência Crítica:** Notificação imediata ao Gestor de Frota + veículo → "Em Manutenção" (dispara RN10).
- Pode ser criada offline via PWA (OCC — RN22).

**RF-MAN-04 — Gerenciar Ciclo de Vida da OS** `[Must]`
Estados: Programada → Pendente de Avaliação → Aprovada → Em Execução → Aguardando Peça → Concluída | Cancelada. Cada transição: responsável + timestamp + observação.

**RF-MAN-05 — Registrar Execução de OS** `[Must]`
Entradas: Tipo (interna/externa), Oficina/fornecedor, Orçamentos, Peças (descrição + tipo de serviço do catálogo RF-ADM-08 + quantidade + valor), Mão de obra, Valor total, Datas, NF, Garantia. Para externas: justificativa de escolha do fornecedor. Saída: OS "Concluída", tempo indisponibilidade, TCO recalculado, veículo → "Ativo".

**RF-MAN-06 — Configurar Fornecedor de Manutenção** `[Must]`
O sistema DEVE permitir cadastro e configuração de fornecedores de manutenção (empresas contratadas), com a mesma mecânica de configuração de importação de RF-INS-04:
- **Entradas:** Razão social, CNPJ, Número do contrato, Vigência, Tipos de serviço cobertos, Contato.
- **Configuração de importação:** Formato, encoding, separador, mapeamento de colunas (Placa, Data, Tipo de Serviço, Descrição, Peças, Valores, NF, Garantia), formato de data, formato numérico.
- **Saída:** Fornecedor cadastrado com template de importação configurado.

**RF-MAN-07 — Importar Manutenções de Fornecedor** `[Must]`
O sistema DEVE permitir importação de registros de manutenção executada pelo fornecedor contratado.
- **Processo:** Mesmo padrão de RF-INS-05: upload → mapeamento → validação pré-commit (RN21) → relatório → revisão → confirmação → efetivação.
- **Validações específicas:** Tipo de serviço DEVE corresponder a item do catálogo (RF-ADM-08). NF duplicada bloqueada. Veículo DEVE existir no cadastro.
- **Saída:** OS criadas/concluídas automaticamente para cada registro importado. TCO recalculado.
- **RNs:** RN17, RN21

**RF-MAN-08 — Acionar Garantia de Serviço** `[Should]`
OS vinculada à original, flag "Garantia", prazo verificado automaticamente.

**RF-MAN-09 — Realizar Inspeção com Checklist** `[Must]`
Formulários configuráveis. Cada item: Conforme/Não Conforme/N/A. Item reprovado → opção de gerar OS.

### Módulo 4: Gestão de Condutores (CND)

**RF-CND-01 — Cadastrar Condutor Servidor** `[Must]`
Vinculação ao SouGov (autopreenchimento). CNH (Número, Categoria, Validades). Número da portaria de credenciamento (Lei 9.327/96), data de publicação, validade. Status "Ativo", tipo "Servidor". Fallback se SouGov indisponível: preenchimento manual + flag "Pendente de Validação".

**RF-CND-02 — Cadastrar Condutor Terceirizado** `[Must]`
Entradas: Nome, CPF, CNH, Empresa contratada, Nº contrato, Vigência, Anexo contrato, Anexo CNH. CPF único. Contrato vigente. Aprovação manual do Gestor de Frota. Status "Ativo", tipo "Terceirizado". Alerta 30 dias antes de vencimento do contrato. Suspensão automática quando contrato vencer.

**RF-CND-03 — Gerenciar Status de Condutor** `[Must]`
Estados: Ativo, Suspenso, Revogado, Pendente de Validação. Suspensão automática quando: CNH vencer, curso obrigatório vencer, contrato de terceirizado vencer, credenciamento (portaria) vencer. Notificação 30 dias antes.

**RF-CND-04 — Registrar Treinamento** `[Must]`
Cursos com carga horária, validade, certificado. Alerta de reciclagem 60 dias antes. **RN:** RN02

**RF-CND-05 — Registrar Avaliação de Desempenho** `[Should]`
Critérios configuráveis, notas, média ponderada, histórico.

**RF-CND-06 — Manter Prontuário** `[Must]`
Infrações, acidentes (responsabilidade), advertências, pontuação CNH. Alerta quando ≥ 15 pontos.

**RF-CND-07 — Controlar Jornada de Direção** `[Must]`
Cálculo automático via checklists. Alertas: 7h contínuas (preventivo), 9h diárias (preventivo de limite). **RN:** RN03

### Módulo 5: Gestão de Rotas (ROT)

**RF-ROT-01 — Cadastrar Rota Padrão** `[Must]`
Nome, Origem, Destino, Pontos intermediários, Distância, Tempo estimado, Orientações, Pontos de referência para locais não mapeados (aldeias, fazendas).

**RF-ROT-02 — Sugerir Roteirização Simplificada** `[Could]`
Ordenação de múltiplas paradas por distância euclidiana com reordenação manual. Inserção manual de coordenadas para locais não mapeados.

### Módulo 6: Gestão de Viagens (VIG)

**RF-VIG-01 — Solicitar Viagem** `[Must]`
- **Entradas:** Datas/horas (saída e retorno previstos), Origem, Destino(s), Rota padrão (opcional), Nº passageiros, Lista de passageiros, Carga, **Finalidade institucional** (seleção de lista configurável — determina antecedência mínima conforme RN07), Justificativa, Documentos de apoio.
- **Autopreenchimento:** Solicitante e departamento via login/SouGov.
- **Validações:** Antecedência mínima conforme RN07 (em horas úteis, por finalidade). Se fora do prazo e não for emergência: justificativa obrigatória + roteamento direto ao Gestor de Frota.
- **Saída:** Solicitação criada. Aprovador identificado conforme RN05. Notificação ao aprovador efetivo.
- **RNs:** RN01, RN04, RN05, RN07, RN12

**RF-VIG-02 — Aprovar/Rejeitar Viagem** `[Must]`
Aprovador efetivo (titular ou substituto conforme RN05). Aprovar ou Rejeitar (com justificativa obrigatória). Em escalonamento (RN06): checkbox de ciência obrigatório. **RNs:** RN04, RN05, RN06

**RF-VIG-03 — Analisar Disponibilidade** `[Must]`
Lista de veículos disponíveis: status Ativo, capacidade compatível, sem conflito (RN08), km dentro de limites de manutenção. **RNs:** RN08, RN09

**RF-VIG-04 — Alocar Veículo e Condutor** `[Must]`
- **Validações:** Condutor DEVE atender RN02 **durante todo o período da viagem** (da saída ao retorno estimado). Sistema verifica: credenciamento, CNH, cursos, contrato (terceirizados) — todos válidos até a data de retorno. Se qualquer item vence durante a viagem → alocação bloqueada com indicação do item e data de vencimento. Para viagens com duração > limite RN03: segundo condutor obrigatório.
- **Saída:** Status → "Alocada/Confirmada". Veículo → "Reservado".
- **RNs:** RN02, RN03, RN08

**RF-VIG-05 — Cancelar Viagem** `[Must]`
Cancelável em qualquer estado pré-execução. Motivo obrigatório. Recursos liberados. Notificação a todos.

**RF-VIG-06 — Alterar Viagem Aprovada** `[Must]`
Alteração de data/destino → re-aprovação. Alteração de passageiros → apenas registro. Se nova data de retorno afetar validade de documentos do condutor (RN02): sistema re-valida e alerta.

**RF-VIG-07 — Estender Duração de Viagem** `[Must]`
Nova data/hora retorno + justificativa. Verificação de conflito com próxima reserva do veículo. **Validação adicional:** se extensão fizer com que documentos do condutor vençam durante o novo período, sistema alerta o Gestor de Frota. Sincronização offline via OCC.

**RF-VIG-08 — Substituir Veículo ou Condutor** `[Must]`
Novo recurso DEVE atender mesmos requisitos. Histórico de substituição preservado.

**RF-VIG-09 — Preencher Checklist de Saída** `[Must]`
Km inicial, combustível, estado (checklist configurável), fotos, confirmação de chave/documentos. Viagem → "Em Andamento". Offline com timestamp preservado (RN22). **RN:** RN11, RN22

**RF-VIG-10 — Registrar Intercorrência** `[Must]`
Tipos: Acidente, Pane, Apreensão, Saúde, Clima, Outro. Descrição, Localização, Fotos, BO. Notificação ao Gestor. Pane → OS automática. Sinistro → dispara RF-AST-12. Acidente → prontuário do condutor. Offline com OCC.

**RF-VIG-11 — Preencher Checklist de Retorno** `[Must]`
Km final (> km saída), combustível, estado, avarias (descrição + fotos), devolução chave. Viagem → "Aguardando Prestação de Contas". Avarias → OS automática. Offline com OCC. **RN:** RN11, RN22

**RF-VIG-12 — Realizar Prestação de Contas** `[Must]`
Upload comprovantes, custo real vs. estimado, viagem → "Concluída", veículo → "Ativo". Prazo: RN11. **RN:** RN11

**RF-VIG-13 — Suspender Reservas por Indisponibilidade** `[Must]`
Dashboard destacado do Gestor com reservas afetadas (até 30 dias). Exige realocação ou cancelamento em 1 dia útil. **RN:** RN10

**RF-VIG-14 — Preencher Dados por Contingência (Perda de Dispositivo)** `[Must]`
O sistema DEVE permitir que o Gestor de Frota preencha retroativamente, via interface desktop, todos os dados de uma viagem em andamento ou concluída cujo condutor perdeu o dispositivo móvel:
- **Dados preenchíveis:** Checklists de saída e retorno, abastecimentos, intercorrências.
- **Entradas obrigatórias adicionais:** Justificativa, BO da perda/roubo do dispositivo (anexo obrigatório).
- **Saída:** Registros criados com flag "Contingência — dados inseridos pelo gestor [Nome/Matrícula]" em todos os campos. Log de auditoria registra a operação de contingência.
- **RNs:** RN24

### Módulo 7: Gestão de Multas (MLT)

**RF-MLT-01 — Registrar Auto de Infração** `[Must]`
Entradas: Veículo, Nº auto, Data/hora, Local, Tipo, Gravidade, Pontuação, Valor, Prazo recurso, Documento. Vinculação automática: cruzamento data/hora com viagens. Se não identificável: multa direcionada ao Gestor de Frota para apuração manual (identificação do condutor por investigação administrativa).

**RF-MLT-02 — Notificar Condutor** `[Must]`
Notificação formal, data de ciência, prazo de defesa iniciado (RN15). **RN:** RN15

**RF-MLT-03 — Registrar Defesa/Recurso** `[Must]`
Texto da defesa, documentos, data protocolo. Status → "Em Recurso".

**RF-MLT-04 — Registrar Decisão e Ressarcimento** `[Must]`
Decisão (responsável/isento/deferido), fundamentação, documento. Se responsável: processo de ressarcimento. **RN:** RN15

**RF-MLT-05 — Controlar Prazos** `[Must]`
Alertas 5 dias antes de cada prazo (indicação condutor, defesa, recurso, pagamento com desconto, vencimento).

### Módulo 8: Indicadores e Relatórios (IND)

**RF-IND-01 — Dashboard de Frota** `[Must]`
Dados cacheados (5 min): taxa disponibilidade, idade média, TCO médio, valor contábil (depreciação), distribuição por status, pendências, conflitos de sincronização. Filtros: Departamento, Período, Categoria.

**RF-IND-02 — Dashboard de Manutenção** `[Must]`
MTBF/MTTR, custo total, adesão preventiva, OS por status/antiguidade, Top 10 custo.

**RF-IND-03 — Dashboard de Consumo** `[Must]`
Km/l por veículo/categoria, custo por km, ranking eficiência, alertas ativos.

**RF-IND-04 — Dashboard de Viagens** `[Must]`
Nível de serviço, destinos frequentes, taxa de ocupação, tempo médio solicitação→execução, pendências prestação de contas, aprovações escalonadas.

**RF-IND-05 — Relatórios Síncronos** `[Must]`
Período ≤ 30 dias. PDF/XLSX/CSV. Mensal de frota, subutilizados, infrações, condutores, manutenção. DEPT_HEAD: filtrado. FLEET_MGR/AUDITOR: completo.

**RF-IND-06 — Relatórios Assíncronos** `[Must]`
Período > 30 dias ou consolidados. Background worker → notificação → download (7 dias). TCO anual, consolidado anual, depreciação, custos por departamento.

**RF-IND-07 — Agendar Relatórios Recorrentes** `[Could]`
Agendamento automático (diário/semanal/mensal) com envio por e-mail.

### Módulo 9: Administração e Catálogos (ADM)

**RF-ADM-01 — Parametrizar Sistema** `[Must]`
Parâmetros configuráveis sem alteração de código:
- Limites de consumo por categoria (RN18)
- Intervalos de manutenção preventiva por categoria
- Vida útil e depreciação por categoria (RN20)
- **Antecedência mínima por finalidade institucional** (tabela configurável — RN07)
- **Finalidades institucionais** (lista configurável: Administrativa, Aula de Campo, Evento, Interestadual, Emergência, etc.)
- **Calendário de feriados** (nacionais e locais, para cálculo de horas úteis — seção 3.4)
- Prazos de aprovação e escalonamento (RN06)
- Margem entre viagens (RN08)
- Limites de jornada (RN03)
- Prazo para prestação de contas (RN11)
- Antecedência de alertas (CNH, seguros, licenciamentos, cursos, contratos)
- Retenção de relatórios assíncronos

**RF-ADM-02 — Gerenciar Templates de Checklist** `[Must]`
Criar, editar, versionar. Por categoria: inspeções, saída, retorno. Itens: texto, tipo resposta, obrigatoriedade.

**RF-ADM-03 — Importar Dados Legados** `[Must]`
Dados importáveis: Veículos, Condutores, Histórico manutenções/abastecimentos. Processo: upload XLSX → validação → revisão → confirmação → rollback em 48h. Carga inicial de Chassi/Renavam dispensa SoD.

**RF-ADM-04 — Gerenciar Notificações** `[Must]`
Motor centralizado. Canais: in-app (obrigatório), e-mail (configurável). Tipos: vencimento CNH, contrato terceirizado, seguro/licenciamento, manutenção, aprovação pendente, escalonamento, prestação de contas, consumo, multa, viagem atrasada, conflito de sincronização, reserva afetada, relatório pronto. Template editável por tipo.

**RF-ADM-05 — Gerenciar Acessórios** `[Should]`
Vincular equipamentos ao veículo (extintor, triângulo, etc.), validades, verificação no checklist.

**RF-ADM-06 — Resolver Conflitos de Sincronização** `[Must]`
Interface dedicada para Gestor de Frota. Lista de conflitos: registro, versão servidor, versão dispositivo, timestamps. Ações: aceitar servidor, aceitar dispositivo, mesclar manualmente. Ambas versões preservadas no log. **RN:** RN22

**RF-ADM-07 — Gerenciar Catálogo de Combustíveis** `[Must]`
O sistema DEVE permitir o cadastro de tipos de combustível com:
- **Entradas:** Descrição resumida (ex: "Gasolina Comum", "Diesel S-10", "Etanol"), Código CATMAT de referência, Centro de custo padrão, Status (ativo/inativo).
- **Saída:** Lista disponível para seleção em abastecimentos (RF-INS-01) e configuração de veículos (RF-AST-01).

**RF-ADM-08 — Gerenciar Catálogo de Serviços de Manutenção** `[Must]`
O sistema DEVE permitir o cadastro de tipos de serviço de manutenção com granularidade operacional:
- **Entradas:** Descrição específica (ex: "Troca pastilha de freio 2º eixo L.D.", "Alinhamento e balanceamento eixo dianteiro"), Código CATSER de referência (genérico), Categoria do serviço (Freios, Suspensão, Motor, Elétrica, Funilaria, etc.), Centro de custo padrão, Status (ativo/inativo).
- **Nota:** O CATSER é genérico (ex: "Serviço de manutenção de freios"). O catálogo local permite desdobramento em serviços específicos, todos vinculados ao mesmo código CATSER para fins de prestação de contas e auditoria.
- **Saída:** Lista disponível para seleção em OS (RF-MAN-05), planos preventivos (RF-MAN-01) e importação de manutenções (RF-MAN-07).

**RF-ADM-09 — Gerar Material Informativo para Veículos** `[Should]`
O sistema DEVE permitir a geração e impressão de material informativo padronizado para afixação nos veículos da frota, contendo:
- QR Code de acesso rápido ao PWA
- Instruções resumidas dos procedimentos obrigatórios: como acessar o sistema, como registrar saída/retorno, como registrar abastecimento, como reportar avarias/intercorrências
- Telefone e e-mail de contato do setor de frota
- **Formato:** PDF para impressão em formato A5 (para porta-luvas) ou adesivo (para painel).
- **Nota:** Essencial para condutores eventuais (professores, técnicos) que não utilizam o sistema com frequência.

### Módulo Transversal: Segurança, Auditoria e LGPD (SEC)

**RF-SEC-01 — Autenticar Usuário** `[Must]`
Via autenticação única UFMT/SouGov. Fallback: autenticação local com credenciais sincronizadas + modo contingência. Timeout: 30 min. Sessão única por usuário.

**RF-SEC-02 — Autorizar por Papel (RBAC)** `[Must]`
Permissões por papel (seção 5). Múltiplos papéis com restrições SoD. Alterações de papéis no log.

**RF-SEC-03 — Implementar Dupla Custódia (SoD)** `[Must]`
Conforme tabela SoD (seção 5.1). Propositor → justificativa → bloqueio → notificação → aprovador analisa → efetivação somente após aprovação. Log completo. **RN:** RN16

**RF-SEC-04 — Registrar Trilha de Auditoria** `[Must]`
Para todas operações de escrita: usuário, timestamp UTC (ms), IP (ou dispositivo para offline), ação, diff, dupla custódia (propositor + aprovador), resoluções de conflito (versões + versão final), contingências (flag + justificativa). Inviolável. Retenção: 5 anos. Interface de consulta com filtros para AUDITOR e FLEET_MGR. **RN:** RN13

**RF-SEC-05 — Gerenciar Consentimento LGPD** `[Must]`
Termo de Uso + Política de Privacidade no primeiro acesso. Base legal: execução de políticas públicas + obrigação legal. Portal do titular: consulta, retificação, exportação.

**RF-SEC-06 — Anonimizar Dados de Desligados** `[Must]`
Após 2 anos do desligamento/encerramento de contrato. CPF, endereço, telefone, foto CNH anonimizados. Matrícula (hash), viagens e dados agregados preservados. Irreversível + log.

---

## 8. Requisitos Não Funcionais (RNF)

### Integração e Interoperabilidade

**RNF-01 — APIs de Integração** `[Must]`
APIs RESTful documentadas (OpenAPI 3.0) para:
- **SouGov:** SSO, dados de servidores, sincronização de afastamentos (RN05).
- **SEI:** Geração/vinculação de documentos.
- **Sistema de Patrimônio UFMT:** Consulta/conciliação patrimonial.
- **CATMAT/CATSER:** Sincronização periódica de catálogos.
- Fallback documentado para cada integração (seção 2.3). Paginação em todas as listagens (cursor-based, máx. 100/página).

### Segurança

**RNF-02 — Proteção contra Vulnerabilidades** `[Must]`
OWASP Top 10: injeção, autenticação quebrada, exposição de dados, controle de acesso. Headers: CSP, HSTS, X-Content-Type-Options, X-Frame-Options. Rate limiting em autenticação e APIs.

**RNF-03 — Criptografia em Repouso** `[Must]`
Dados sensíveis (CNH, CPF, dados de jornada) com criptografia AES-256 ou equivalente.

**RNF-04 — Gestão de Sessão** `[Must]`
Timeout 30 min (configurável). Sessão única. Token com entropia ≥ 128 bits. Invalidação no logout/troca de senha.

### Performance

**RNF-05 — Tempo de Resposta** `[Must]`
- Consultas simples: ≤ 2s (p95)
- Escrita: ≤ 3s (p95)
- Relatórios síncronos (≤ 30 dias): ≤ 30s
- Relatórios assíncronos: background worker, máx. 10 min, notificação ao concluir
- Dashboards: cache 5 min

**RNF-06 — Capacidade** `[Must]`
200 usuários simultâneos, 500 veículos, 2.000 condutores, 50.000 viagens/ano, 500GB anexos, fila de 100 relatórios assíncronos.

### Disponibilidade e Recuperação

**RNF-07 — Disponibilidade** `[Must]`
99,5% uptime mensal (excluindo manutenção programada: domingos 02:00–06:00 UTC-4).

**RNF-08 — Backup e Recuperação** `[Must]`
RPO: 1h. RTO: 4h. Backup completo diário + incremental horário. Teste de restore mensal documentado. Logs de auditoria incluídos com integridade verificável.

### Usabilidade

**RNF-09 — Usabilidade do Portal** `[Must]`
Solicitação padrão: 3 passos (rota → dados → confirmar). Responsivo: desktop 1024px+, mobile 360px+.

**RNF-10 — PWA com OCC** `[Must]`
PWA instalável. Offline: agenda, checklists, abastecimentos, avarias, intercorrências. OCC (RN22) para conflitos. Bloqueio local (RN23). Sincronização com timestamp original. **RNs:** RN22, RN23

### Acessibilidade

**RNF-11 — Conformidade e-MAG** `[Must]`
e-MAG 3.1 + WCAG 2.1 nível AA (Decreto 5.296/2004, IN 1/2014).

### Observabilidade

**RNF-12 — Monitoramento e Alertas** `[Should]`
Health check (/health). Logging JSON. Métricas exportáveis (tempo resposta, taxa erro, fila relatórios, conflitos pendentes). Alertas: indisponibilidade, erro > 1%, resposta > 5s, fila > 50.

### Compatibilidade

**RNF-13 — Navegadores e Dispositivos** `[Must]`
Chrome, Firefox, Edge (últimas 2 versões). Desktop: 1024×768 mín. Mobile: 360×640 mín.

---

## 9. Casos de Uso Expandidos

### UC01 — Solicitar e Aprovar Viagem

| Campo | Descrição |
|-------|-----------|
| **Atores** | Solicitante, Chefia (ou Substituto), Gestor de Frota, Condutor |
| **Pré-condições** | Solicitante autenticado. ≥ 1 veículo ativo. |
| **Pós-condições** | Viagem alocada, todos notificados. |

**Fluxo Principal:**
1. Solicitante seleciona "Nova Solicitação".
2. Autopreenchimento via SouGov.
3. Preenche: datas, destino, passageiros, **finalidade institucional** (seleciona da lista configurável).
4. Sistema calcula antecedência em **horas úteis** conforme RN07 e valida.
5. Confirma envio.
6. Sistema identifica aprovador efetivo (RN05): titular ou substituto.
7. Solicitação criada → notificação ao aprovador.
8. Aprovador aprova.
9. Status → "Aguardando Alocação" → notificação Gestor de Frota.
10. Gestor analisa disponibilidade (RF-VIG-03).
11. Seleciona veículo e condutor.
12. Sistema valida condutor **até a data de retorno** (RN02): credenciamento, CNH, cursos, contrato. Verifica conflitos (RN08).
13. Status → "Alocada/Confirmada". Notificação a todos.

**Fluxos Alternativos:**
- **FA01 — Rejeição:** Justificativa obrigatória → notificação ao solicitante.
- **FA02 — Sem veículo:** Fila de espera, datas alternativas, ou rejeição por indisponibilidade.
- **FA03 — Fora do prazo:** Justificativa + roteamento direto ao Gestor de Frota (RN07). Emergência: sem antecedência mínima.
- **FA04 — Escalonamento:** 2 dias úteis sem resposta → superior hierárquico com checkbox de ciência (RN06).
- **FA05 — Condutor com documento vencendo durante viagem:** Sistema bloqueia alocação, informa: "CNH do condutor [Nome] vence em [Data], anterior ao retorno previsto [Data]. Selecione outro condutor."
- **FA06 — Múltiplos condutores:** Duração > limite RN03 → segundo condutor obrigatório.
- **FA07 — Terceirizado:** Verificação adicional de contrato vigente até data de retorno.

**Exceções:**
- **EX01 — SouGov offline:** Preenchimento manual + flag. Aprovador: último estado de afastamento sincronizado; se cache vazio, roteia ao titular.
- **EX02 — Falha de notificação:** Retry (3×, backoff) + pendência no painel.

---

### UC02 — Executar Viagem

| Campo | Descrição |
|-------|-----------|
| **Atores** | Condutor, Gestor de Frota |
| **Pré-condições** | Viagem "Alocada/Confirmada". Condutor autenticado no PWA. |
| **Pós-condições** | Viagem concluída, prestação de contas realizada. |

**Fluxo Principal:**
1. Condutor acessa agenda no PWA → seleciona viagem.
2. Preenche checklist de saída (RF-VIG-09).
3. Viagem → "Em Andamento".
4. Realiza deslocamento.
5. Preenche checklist de retorno (RF-VIG-11).
6. Viagem → "Aguardando Prestação de Contas".
7. Upload comprovantes (RF-VIG-12).
8. Viagem → "Concluída". Veículo → "Ativo".

**Fluxos Alternativos:**
- **FA01 — Intercorrência:** RF-VIG-10. Pane → OS. Acidente → prontuário. Sinistro → RF-AST-12 + RN10.
- **FA02 — Extensão:** RF-VIG-07. Gestor avalia conflitos. **Se extensão faz CNH/credenciamento vencer:** alerta ao Gestor.
- **FA03 — Substituição:** RF-VIG-08.
- **FA04 — Avaria no retorno:** OS automática. Se impede uso: veículo → "Em Manutenção" (RN10).

**Exceções:**
- **EX01 — PWA offline:** Dados offline com timestamp original. Sincronização via OCC (RN22). Conflitos → fila (RF-ADM-06).
- **EX02 — Perda do dispositivo:** Condutor perdeu celular com dados não sincronizados. **Procedimento:** (a) Condutor informa o Gestor de Frota presencialmente ou por telefone. (b) Condutor registra BO da perda/roubo do dispositivo. (c) Gestor de Frota preenche todos os dados retroativamente via RF-VIG-14: checklists, abastecimentos, intercorrências. (d) BO do dispositivo anexado como documento obrigatório. (e) Todos os registros recebem flag "Contingência". (f) Se havia dados no dispositivo que posteriormente for recuperado: dados do dispositivo são tratados como conflito (RN22) e resolvidos via RF-ADM-06.
- **EX03 — Prestação de contas atrasada:** 3 dias úteis → alerta. 5 dias úteis → pendência no prontuário (RN11).

---

### UC03 — Registrar Abastecimento

| Campo | Descrição |
|-------|-----------|
| **Atores** | Condutor, Gestor de Frota |
| **Pré-condições** | Veículo Ativo ou Reservado. Condutor com cartão de abastecimento do contrato. |
| **Pós-condições** | Abastecimento registrado, consumo recalculado, TCO atualizado. |

**Fluxo Principal (importação — cenário mais comum):**
1. Fornecedor contratado envia planilha periódica de abastecimentos realizados.
2. Gestor de Frota acessa RF-INS-05, seleciona fornecedor, faz upload.
3. Sistema aplica mapeamento configurado (RF-INS-04).
4. Validação pré-commit (RN21): campos, odômetro, cupom fiscal duplicado.
5. Relatório de validação apresentado.
6. Gestor revisa, confirma.
7. Registros efetivados, consumo e TCO recalculados.

**Fluxo Alternativo (registro manual):**
- **FA01 — Manual pelo condutor:** Condutor registra via PWA (RF-INS-01) quando abastece. Mesmo fluxo de validação, mas individual.
- **FA02 — Retroativo:** Justificativa + aprovação do Gestor (RF-INS-02).
- **FA03 — Odômetro inconsistente:** Bloqueio → corrigir digitação ou solicitar correção com SoD (RF-INS-03).
- **FA04 — Cupom fiscal duplicado na importação:** Sistema reporta duplicidade e impede importação do registro. Gestor verifica se é duplicidade real ou erro.

**Exceções:**
- **EX01 — Offline:** Registro com OCC. Conflito → fila de resolução.

**Nota operacional:** Muitos condutores eventuais (professores, técnicos) não registrarão abastecimentos manualmente no PWA. O fluxo principal de registro é a importação da planilha do fornecedor contratado. O registro manual é complementar para situações fora da rede contratada (abastecimento de emergência em viagem, por exemplo).

---

### UC04 — Manutenção Corretiva

| Campo | Descrição |
|-------|-----------|
| **Atores** | Condutor, Gestor de Manutenção, Gestor de Frota |
| **Pré-condições** | Veículo cadastrado. Problema identificado. |
| **Pós-condições** | OS concluída, veículo → Ativo, TCO recalculado. |

**Fluxo Principal:**
1. **Condutor, Gestor de Manutenção ou Gestor de Frota** reporta falha (RF-MAN-03).
2. OS "Pendente de Avaliação" → notificação Gestor de Manutenção.
3. Gestor avalia e aprova → "Aprovada".
4. Define execução (interna/externa), solicita orçamentos.
5. Execução → "Em Execução".
6. Conclusão registrada (RF-MAN-05) → "Concluída". TCO recalculado. Veículo → "Ativo".

**Fluxo Alternativo (importação via contrato):**
- **FA01 — Manutenção por fornecedor contratado:** Fornecedor envia planilha de serviços executados. Gestor importa via RF-MAN-07 com validação pré-commit (RN21). OS criadas/concluídas automaticamente.

**Outros Alternativos:**
- **FA02 — Urgência crítica:** Veículo → "Em Manutenção" (RN10). OS pode pular para "Em Execução".
- **FA03 — Aguardando peça:** OS → "Aguardando Peça".
- **FA04 — Garantia:** RF-MAN-08.

---

### UC05 — Processo de Baixa

*(Mantido conforme v3.0 — sem alterações)*

| Campo | Descrição |
|-------|-----------|
| **Atores** | Gestor de Frota, Gestor de Patrimônio, Comissão |
| **Pré-condições** | Veículo Ativo, Inativo ou Sinistrado. |
| **Pós-condições** | "Baixado", histórico preservado, SEI documentado. |

Fluxo principal e alternativas conforme v3.0 (RF-AST-09, RF-AST-10, RN14).

---

### UC06 — Registrar Sinistro

*(Mantido conforme v3.0 — sem alterações)*

Fluxo principal e alternativas conforme v3.0 (RF-AST-12, RF-VIG-10, RN10).

---

## 10. Matriz de Rastreabilidade

### RN → RF

| RN | Requisitos Funcionais |
|----|----------------------|
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
| RN16 | RF-AST-02, RF-INS-03, RF-SEC-03 |
| RN17 | RF-INS-01, RF-MAN-05, RF-AST-08, RF-AST-11 |
| RN18 | RF-INS-06, RF-ADM-01 |
| RN19 | RF-MAN-01, RF-MAN-02, RF-INS-07 |
| RN20 | RF-AST-01, RF-AST-08, RF-AST-09, RF-AST-11, RF-AST-12 |
| RN21 | RF-INS-05, RF-MAN-07 |
| RN22 | RF-INS-01, RF-VIG-09, RF-VIG-10, RF-VIG-11, RF-MAN-03, RF-ADM-06 |
| RN23 | (RNF-10 — implementação PWA) |
| RN24 | RF-VIG-14 |

### RF → UC

| Caso de Uso | Requisitos Funcionais |
|-------------|----------------------|
| UC01 — Solicitar e Aprovar | RF-VIG-01 a 06, RF-SEC-01, RF-ADM-04 |
| UC02 — Executar Viagem | RF-VIG-07 a 14, RF-INS-01, RF-ADM-06 |
| UC03 — Registrar Abastecimento | RF-INS-01 a 06, RF-ADM-06 |
| UC04 — Manutenção Corretiva | RF-MAN-03 a 09, RF-VIG-13 |
| UC05 — Processo de Baixa | RF-AST-09, RF-AST-10, RF-AST-05, RF-VIG-13 |
| UC06 — Registrar Sinistro | RF-AST-12, RF-VIG-10, RF-VIG-13, RF-INS-08, RF-ADM-06 |

---

## 11. Priorização (MoSCoW) e Faseamento

### Fase 1 — MVP (Meses 1–8)

**Must Have:**
- AST: RF-AST-01 a 06, 09, 10, 11, 12
- INS: RF-INS-01, 02, 03, 04, 05, 06, 07, 08, 09
- MAN: RF-MAN-01 a 07, 09
- CND: RF-CND-01 a 04, 06, 07
- ROT: RF-ROT-01
- VIG: RF-VIG-01 a 05, 09, 11, 12, 13, 14
- MLT: RF-MLT-01, 02, 05
- IND: RF-IND-01, 05
- ADM: RF-ADM-01, 02, 03, 04, 06, 07, 08
- SEC: RF-SEC-01 a 05
- RNFs: RNF-01 a 11, 13

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
- RNFs: RNF-01 (SEI + Patrimônio), RNF-12

### Fase 3 — Evolução (Meses 15–18)

**Could Have:**
- ROT: RF-ROT-02
- IND: RF-IND-07
- Integração com DETRAN

---

## 12. Escopo Futuro Condicionado

### Carsharing Interno `[Won't nesta versão]`

**Justificativa:** Depende de cadeia de custódia física (controle de chaves) que a UFMT não possui digitalizada. Sem sistema de portaria integrado, qualquer solução de software seria inefetiva.

**Pré-requisitos:** Sistema de portaria com integração digital, ou processo formal de custódia de chaves com registro auditável. Definição por portaria dos locais habilitados.

**Tratamento transitório:** Deslocamentos locais seguem fluxo padrão de viagem (RF-VIG-01 a 12).

### Telemetria e Rastreamento GPS `[Won't nesta versão]`

**Justificativa:** Infraestrutura de rastreadores GPS não disponível nos veículos da frota atual.

**Pré-requisitos:** Aquisição e instalação de rastreadores GPS, contrato com provedor de telemetria, infraestrutura de rede para recepção de dados.

**Módulos dependentes:** Auditoria de trajeto realizado (comparação rota planejada vs. real), monitoramento de velocidade, geofencing, controle de ignição.

### Gestão de Pneus `[Won't nesta versão]`

**Justificativa:** A gestão de pneus possui complexidade própria que justifica módulo dedicado: controle de vida útil por posição (dianteiro/traseiro/estepe), rodízio entre posições, recapagem (até 2 recapagens por pneu), rastreabilidade individual por DOT/número de série, e controle de estoque.

**Escopo futuro sugerido:**
- Cadastro individual de pneus com número de série/DOT, fabricante, modelo, data de compra, CATMAT.
- Vinculação pneu ↔ veículo ↔ posição (DE, DD, TE, TD, estepe, etc.).
- Registro de rodízio (troca de posição) com km.
- Registro de recapagem (fornecedor, tipo, km na recapagem, km projetado).
- Alertas de vida útil por km e por tempo.
- Histórico completo: km rodados por pneu em cada posição.
- Integração com manutenção: item de checklist específico para pneus.

### Outros Módulos Futuros Sugeridos

| Módulo | Descrição | Dependência |
|--------|-----------|-------------|
| **Gestão de Pedágios** | Importação de extratos de operadoras (TAG), conciliação com viagens. Relevante se rotas regulares passarem a incluir trechos pedagiados. | Contrato com operadora de TAG |
| **Gestão de Sinistros (expandida)** | Workflow completo de sinistro com acionamento de seguradora, acompanhamento de perícia, indenização, reposição. | Integração com seguradoras |
| **Gestão de Contratos de Frota** | Controle de contratos de locação de veículos, vigência, quilometragem contratada vs. utilizada, multas contratuais. | Frota locada |
| **Gestão de Diárias e Passagens** | Integração com SCDP para vincular diárias/passagens às viagens da frota. | Integração SCDP |
| **Painel de Compliance** | Dashboard específico para auditores com indicadores de conformidade: % de viagens com checklist completo, % de manutenções preventivas no prazo, multas não tratadas, credenciamentos vencidos. | Dados dos módulos existentes |

---

## 13. Referências Normativas Futuras

Os seguintes requisitos foram identificados como necessários para conformidade plena, mas sua implementação depende de infraestrutura ou maturidade organizacional ainda não disponíveis. Estão documentados como referência para inclusão em versões futuras.

### Criptografia em Trânsito (TLS)

**Descrição:** Toda comunicação entre cliente e servidor DEVERIA utilizar TLS 1.2 ou superior com certificados válidos emitidos por autoridade certificadora reconhecida.

**Justificativa do adiamento:** A infraestrutura atual da UFMT pode não suportar certificados gerenciados em todos os ambientes. Será implementado quando a infraestrutura de rede e servidores suportar adequadamente.

**Risco aceito:** Comunicações podem trafegar sem criptografia de transporte no ambiente interno da rede da UFMT. Dados sensíveis em repouso já estão protegidos (RNF-03).

### Responsável Patrimonial Obrigatório

**Descrição:** Todo veículo ativo DEVERIA ter um responsável patrimonial vinculado (CPF de servidor — "fiel depositário"), designado formalmente. Infrações sem condutor identificável seriam direcionadas ao responsável patrimonial. Transferências de veículo exigiriam atualização do responsável.

**Justificativa do adiamento:** Conceito necessário mas requer discussão institucional sobre designação formal, responsabilidades e implicações jurídicas para os servidores designados. A implementação prematura sem respaldo administrativo geraria resistência.

**Tratamento atual:** Multas sem condutor identificável são direcionadas ao Gestor de Frota para apuração manual (RF-MLT-01).

**Recomendação:** Incluir na próxima versão do DRS após publicação de portaria interna da UFMT regulamentando a figura do responsável patrimonial de veículos.

---

## 14. Considerações Finais

Este DRS v3.1 consolida quatro ciclos de análise e revisão, incorporando ajustes operacionais derivados do conhecimento direto da realidade da UFMT:

- **Validação de condutor até a data de retorno** (RN02) — elimina o risco de condutor dirigindo veículo oficial com documentos vencidos durante a viagem.
- **Antecedência mínima por finalidade em horas úteis** (RN07) — diferencia prazos por complexidade da viagem e elimina a brecha de solicitações feitas em horários não-comerciais.
- **Abastecimento e manutenção por contrato** com importação configurável por fornecedor (RF-INS-04/05, RF-MAN-06/07) e validação pré-commit (RN21) — reflete o modelo operacional real com cartão de abastecimento e planilhas de fornecedores.
- **Catálogos CATMAT/CATSER** com desdobramento local (RF-ADM-07/08) — permite granularidade operacional ("troca pastilha freio 2º eixo L.D.") vinculada ao código genérico para prestação de contas.
- **Contingência por perda de dispositivo** (RN24, RF-VIG-14) — procedimento formal para o cenário real de perda de celular em viagem com dados offline.
- **Material informativo nos veículos** (RF-ADM-09) — solução pragmática para condutores eventuais (professores/técnicos) que não usam o sistema com frequência.
- **Correção de IPVA/DPVAT** — removidos por imunidade tributária e extinção legal.
- **Gestão de pneus, pedágios, telemetria, TLS e responsável patrimonial** documentados como referência futura com pré-requisitos claros.

### Fatores Críticos de Sucesso

1. **Dados legados e configuração de fornecedores:** A importação da base histórica (RF-ADM-03) e a configuração dos templates de importação dos fornecedores contratados (RF-INS-04, RF-MAN-06) são pré-requisitos para operação.
2. **Catálogos CATMAT/CATSER:** O cadastro dos tipos de combustível e serviços de manutenção com códigos de referência deve ser realizado antes do go-live.
3. **Calendário de feriados:** Cadastro de feriados nacionais e locais (RF-ADM-01) é necessário para cálculo correto de horas úteis (antecedência, prazos).
4. **Material nos veículos:** A impressão e distribuição do material informativo (RF-ADM-09) nos veículos é essencial para adoção por condutores eventuais.
5. **Treinamento:** Gestores de Frota (importação, conflitos, contingências), condutores habituais (PWA), e aprovadores (fluxo de aprovação/escalonamento).
6. **Testes de campo:** PWA offline em áreas sem cobertura, cenários de conflito de sincronização, e contingência de perda de dispositivo devem ser testados antes do go-live.

---

*Fim do Documento — Módulo Frota, Plataforma de Gestão UFMT — DRS v3.1*
