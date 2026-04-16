use super::siorg_client::{SiorgClient, SiorgUnidade, SiorgUnidadeCompleta};
pub use domain::models::{
    ActivityArea, ContactInfo, CreateOrganizationPayload, CreateOrganizationalUnitCategoryPayload,
    CreateOrganizationalUnitPayload, CreateOrganizationalUnitTypePayload,
    CreateSystemSettingPayload, InternalUnitType, OrganizationDto, OrganizationalUnitCategoryDto,
    OrganizationalUnitDto, OrganizationalUnitTypeDto, SiorgUpsertPayload, SyncSummary,
    UpdateOrganizationPayload, UpdateOrganizationalUnitPayload, UpdateSystemSettingPayload,
};
use domain::ports::*;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

// ============================================================================
// SIORG Sync Service
// ============================================================================

/// Prefixo das chaves usadas em `system_settings` para guardar a versão SIORG de cada org.
/// Formato da chave: `siorg_versao:{org_siorg_code}`
const SIORG_VERSAO_KEY_PREFIX: &str = "siorg_versao";

fn versao_setting_key(org_siorg_code: i32) -> String {
    format!("{}:{}", SIORG_VERSAO_KEY_PREFIX, org_siorg_code)
}

pub struct SiorgSyncService {
    siorg_client: Arc<SiorgClient>,
    organization_repo: Arc<dyn OrganizationRepositoryPort>,
    unit_repo: Arc<dyn OrganizationalUnitRepositoryPort>,
    category_repo: Arc<dyn OrganizationalUnitCategoryRepositoryPort>,
    type_repo: Arc<dyn OrganizationalUnitTypeRepositoryPort>,
    settings_repo: Arc<dyn SystemSettingsRepositoryPort>,
    pool: sqlx::PgPool,
}

impl SiorgSyncService {
    pub fn new(
        siorg_client: Arc<SiorgClient>,
        organization_repo: Arc<dyn OrganizationRepositoryPort>,
        unit_repo: Arc<dyn OrganizationalUnitRepositoryPort>,
        category_repo: Arc<dyn OrganizationalUnitCategoryRepositoryPort>,
        type_repo: Arc<dyn OrganizationalUnitTypeRepositoryPort>,
        settings_repo: Arc<dyn SystemSettingsRepositoryPort>,
        pool: sqlx::PgPool,
    ) -> Self {
        Self {
            siorg_client,
            organization_repo,
            unit_repo,
            category_repo,
            type_repo,
            settings_repo,
            pool,
        }
    }

    async fn prepare_dependencies(
        &self,
        units: &[SiorgUnidadeCompleta],
    ) -> Result<(HashMap<String, Uuid>, HashMap<String, Uuid>), SyncError> {
        let mut types_cache = HashMap::new();
        let mut cats_cache = HashMap::new();

        for unit in units {
            // 1. Resolve o Tipo de Unidade (Tabela Dinâmica)
            // Se o SIORG não informar, usamos uma constante clara em vez de esconder o erro
            let tipo_code = unit
                .base
                .codigo_tipo_unidade
                .clone()
                .unwrap_or_else(|| "NAO_INFORMADO".to_string());

            if !types_cache.contains_key(&tipo_code) {
                // A função find_or_create_type já lida com a verificação no DB
                let u_type = self.find_or_create_type(&tipo_code).await?;
                types_cache.insert(tipo_code, u_type.id);
            }

            // 2. Resolve a Categoria (Tabela Dinâmica)
            let cat_code = unit
                .codigo_categoria_unidade
                .clone()
                .unwrap_or_else(|| "NAO_INFORMADA".to_string());

            if !cats_cache.contains_key(&cat_code) {
                // A função find_or_create_category já lida com a verificação no DB
                let cat = self.find_or_create_category(&cat_code).await?;
                cats_cache.insert(cat_code, cat.id);
            }
        }

        Ok((types_cache, cats_cache))
    }

    // ========================================================================
    // Organization Sync
    // ========================================================================

    /// Synchronize a single organization from SIORG
    pub async fn sync_organization(&self, siorg_code: i32) -> Result<OrganizationDto, SyncError> {
        info!("Syncing organization with SIORG code {}", siorg_code);

        let siorg_unit = self
            .siorg_client
            .get_unit_complete(siorg_code)
            .await
            .map_err(|e| SyncError::ApiError(format!("{:#}", e)))?
            .ok_or_else(|| SyncError::NotFoundInSiorg(siorg_code))?;

        if let Some(local_org) = self
            .organization_repo
            .find_by_siorg_code(siorg_code)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
        {
            self.update_organization_from_siorg(local_org.id, &siorg_unit)
                .await
        } else {
            self.create_organization_from_siorg(&siorg_unit).await
        }
    }

    async fn create_organization_from_siorg(
        &self,
        siorg_unit: &SiorgUnidadeCompleta,
    ) -> Result<OrganizationDto, SyncError> {
        let siorg_code = siorg_unit
            .siorg_code()
            .ok_or_else(|| SyncError::MissingRequiredField("codigo_unidade".to_string()))?;

        // CNPJ e código UG não são fornecidos pela API SIORG pública.
        // Criamos o registro com valores vazios; o operador deve preenchê-los manualmente.
        warn!(
            "Criando organização siorg={} sem CNPJ/UG (não disponíveis na API SIORG). \
             Preencha manualmente após a sincronização.",
            siorg_code
        );

        let payload = CreateOrganizationPayload {
            acronym: siorg_unit.base.sigla.clone().unwrap_or_default(),
            name: siorg_unit.base.nome.clone(),
            cnpj: String::new(),
            ug_code: 0,
            siorg_code,
            address: None,
            city: None,
            state: None,
            zip_code: None,
            phone: None,
            email: None,
            website: None,
            logo_url: None,
            is_active: true,
        };

        self.organization_repo
            .create(payload)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))
    }

    async fn update_organization_from_siorg(
        &self,
        org_id: Uuid,
        siorg_unit: &SiorgUnidadeCompleta,
    ) -> Result<OrganizationDto, SyncError> {
        let payload = UpdateOrganizationPayload {
            acronym: siorg_unit.base.sigla.clone(),
            name: Some(siorg_unit.base.nome.clone()),
            address: None,
            city: None,
            state: None,
            zip_code: None,
            phone: None,
            email: None,
            website: None,
            logo_url: None,
            is_active: Some(true),
        };

        self.organization_repo
            .update(org_id, payload)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))
    }

    /// Synchronize an organization using its local database UUID (auto-discovers siorg_code)
    pub async fn sync_organization_by_id(
        &self,
        org_id: Uuid,
    ) -> Result<OrganizationDto, SyncError> {
        let org = self
            .organization_repo
            .find_by_id(org_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                SyncError::DatabaseError(format!("Organization {} not found locally", org_id))
            })?;

        info!(
            "Syncing organization {} (siorg_code={}) from local DB lookup",
            org_id, org.siorg_code
        );

        self.sync_organization(org.siorg_code).await
    }

    /// Synchronize all units of an organization using its local database UUID (auto-discovers siorg_code)
    pub async fn sync_organization_units_by_id(
        &self,
        org_id: Uuid,
    ) -> Result<SyncSummary, SyncError> {
        let org = self
            .organization_repo
            .find_by_id(org_id)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                SyncError::DatabaseError(format!("Organization {} not found locally", org_id))
            })?;

        info!(
            "Bulk syncing units for organization {} (siorg_code={}) from local DB lookup",
            org_id, org.siorg_code
        );

        self.sync_organization_units(org.siorg_code).await
    }

    // ========================================================================
    // Unit Sync
    // ========================================================================

    /// Synchronize a single organizational unit from SIORG by SIORG code.
    ///
    /// Faz uma chamada HTTP e persiste apenas a unidade solicitada.
    /// Para sincronizar todas as unidades de um órgão de forma eficiente,
    /// use `sync_organization_units` que faz uma única chamada para toda a estrutura.
    pub async fn sync_unit(&self, siorg_code: i32) -> Result<OrganizationalUnitDto, SyncError> {
        info!("Syncing unit with SIORG code {}", siorg_code);

        let siorg_unit = self
            .siorg_client
            .get_unit_complete(siorg_code)
            .await
            .map_err(|e| SyncError::ApiError(format!("{:#}", e)))?
            .ok_or_else(|| SyncError::NotFoundInSiorg(siorg_code))?;

        self.upsert_unit(&siorg_unit).await
    }

    /// Persiste uma unidade organizacional (create ou update) sem realizar chamada HTTP.
    /// Usado internamente por `sync_organization_units` e pelo sync incremental.
    async fn upsert_unit(
        &self,
        siorg_unit: &SiorgUnidadeCompleta,
    ) -> Result<OrganizationalUnitDto, SyncError> {
        let siorg_code = siorg_unit
            .siorg_code()
            .ok_or_else(|| SyncError::MissingRequiredField("codigo_unidade".to_string()))?;

        if let Some(local_unit) = self
            .unit_repo
            .find_by_siorg_code(siorg_code)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
        {
            self.update_unit_from_siorg(local_unit.id, siorg_unit).await
        } else {
            self.create_unit_from_siorg(siorg_unit).await
        }
    }

    async fn create_unit_from_siorg(
        &self,
        siorg_unit: &SiorgUnidadeCompleta,
    ) -> Result<OrganizationalUnitDto, SyncError> {
        let organization = self
            .organization_repo
            .find_main()
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
            .ok_or_else(|| SyncError::MissingRequiredField("Main Organization".to_string()))?;

        let tipo_unidade = siorg_unit
            .base
            .codigo_tipo_unidade
            .as_deref()
            .unwrap_or("SECTOR");

        let category = self
            .find_or_create_category("Default SIORG Category")
            .await?;

        let unit_type = self.find_or_create_type(tipo_unidade).await?;

        let parent_id = if let Some(parent_code) = siorg_unit.parent_siorg_code() {
            self.unit_repo
                .find_by_siorg_code(parent_code)
                .await
                .map_err(|e| SyncError::DatabaseError(e.to_string()))?
                .map(|u| u.id)
        } else {
            None
        };

        let area_atuacao = siorg_unit.area_atuacao.as_deref().unwrap_or("MEIO");
        let activity_area = match area_atuacao.to_uppercase().as_str() {
            "CORE" | "FIM" => ActivityArea::Core,
            _ => ActivityArea::Support,
        };

        let internal_type = match tipo_unidade.to_uppercase().as_str() {
            "ADMINISTRATION" => InternalUnitType::Administration,
            "DEPARTMENT" => InternalUnitType::Department,
            "LABORATORY" => InternalUnitType::Laboratory,
            "COUNCIL" => InternalUnitType::Council,
            "COORDINATION" => InternalUnitType::Coordination,
            "CENTER" => InternalUnitType::Center,
            "DIVISION" => InternalUnitType::Division,
            _ => InternalUnitType::Sector,
        };

        let siorg_code = siorg_unit.siorg_code();

        let payload = CreateOrganizationalUnitPayload {
            organization_id: organization.id,
            parent_id,
            category_id: category.id,
            unit_type_id: unit_type.id,
            internal_type,
            name: siorg_unit.base.nome.clone(),
            formal_name: None,
            acronym: siorg_unit.base.sigla.clone(),
            siorg_code,
            activity_area,
            contact_info: ContactInfo::default(),
            is_active: true,
        };

        self.unit_repo
            .create(payload)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))
    }

    async fn update_unit_from_siorg(
        &self,
        unit_id: Uuid,
        siorg_unit: &SiorgUnidadeCompleta,
    ) -> Result<OrganizationalUnitDto, SyncError> {
        let parent_id = if let Some(parent_code) = siorg_unit.parent_siorg_code() {
            self.unit_repo
                .find_by_siorg_code(parent_code)
                .await
                .map_err(|e| SyncError::DatabaseError(e.to_string()))?
                .map(|u| u.id)
        } else {
            None
        };

        let payload = UpdateOrganizationalUnitPayload {
            parent_id,
            category_id: None,
            unit_type_id: None,
            internal_type: None,
            name: Some(siorg_unit.base.nome.clone()),
            formal_name: None,
            acronym: siorg_unit.base.sigla.clone(),
            activity_area: None,
            contact_info: None,
            is_active: Some(true),
            deactivation_reason: None,
        };

        self.unit_repo
            .update(unit_id, payload)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))
    }

    /// Desativa uma unidade localmente quando a API SIORG reporta EXCLUSAO/EXTINCAO.
    async fn deactivate_unit(&self, siorg_base: &SiorgUnidade) -> Result<bool, SyncError> {
        let siorg_code = match siorg_base.siorg_code() {
            Some(c) => c,
            None => return Ok(false),
        };

        let local = self
            .unit_repo
            .find_by_siorg_code(siorg_code)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        if let Some(local_unit) = local {
            let payload = UpdateOrganizationalUnitPayload {
                parent_id: None,
                category_id: None,
                unit_type_id: None,
                internal_type: None,
                name: None,
                formal_name: None,
                acronym: None,
                activity_area: None,
                contact_info: None,
                is_active: Some(false),
                deactivation_reason: Some("Unidade extinta no SIORG".to_string()),
            };

            self.unit_repo
                .update(local_unit.id, payload)
                .await
                .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    // ========================================================================
    // Bulk Sync
    // ========================================================================

    /// Synchronize all registered organizations (and their units) directly from the database.
    ///
    /// Para cada órgão:
    /// - Compara a versão armazenada localmente com a versão atual da API SIORG.
    /// - Se a versão não mudou: nenhuma chamada adicional é feita (zero overhead).
    /// - Se mudou e há versão armazenada: sync incremental via `get_alteradas` (N mudanças).
    /// - Se não há versão armazenada: sync completo via `get_estrutura_completa` (primeira vez).
    pub async fn sync_all_from_db(&self) -> Result<SyncSummary, SyncError> {
        info!("Iniciando sincronização global: buscando organizações ativas com siorg_code");

        // 1. Lista as organizações ativas registradas (limite alto para garantir que pegue todas)
        let (orgs, _) = self
            .organization_repo
            .list(Some(true), 1000, 0)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        let mut summary = SyncSummary::default();

        // Filtra apenas as que possuem código SIORG válido
        let siorg_orgs: Vec<_> = orgs.into_iter().filter(|o| o.siorg_code > 0).collect();

        info!(
            "Encontrada(s) {} organização(ões) para sincronizar.",
            siorg_orgs.len()
        );

        for org in siorg_orgs {
            summary.total_processed += 1;

            // PASSO A: Atualiza os dados do Órgão Raiz na tabela 'organizations'
            if let Err(e) = self.sync_organization(org.siorg_code).await {
                summary.failed += 1;
                summary.errors.push(format!(
                    "Falha na raiz Org {} (siorg={}): {}",
                    org.id, org.siorg_code, e
                ));
                error!(
                    "Erro ao sincronizar raiz da organização {}: {}",
                    org.siorg_code, e
                );
                continue;
            }
            summary.updated += 1;

            // PASSO B: Sincroniza a árvore de unidades (Pró-Reitorias, Departamentos, etc.)
            // Passamos o UUID (org.id) para o novo motor de Bulk Sync
            if let Err(e) = self
                .sync_organization_units_versioned(org.id, org.siorg_code, &mut summary)
                .await
            {
                summary.failed += 1;
                summary.errors.push(format!(
                    "Falha nas unidades da Org {} (siorg={}): {}",
                    org.id, org.siorg_code, e
                ));
                error!(
                    "Erro ao sincronizar unidades para org {}: {}",
                    org.siorg_code, e
                );
            }
        }

        info!("Sincronização global concluída: {:?}", summary);
        Ok(summary)
    }

    /// Sync inteligente de unidades: escolhe entre incremental e completo com base na versão.
    async fn sync_organization_units_versioned(
        &self,
        org_id: Uuid,
        org_siorg_code: i32,
        summary: &mut SyncSummary,
    ) -> Result<(), SyncError> {
        let versao_api = match self.siorg_client.get_versao(org_siorg_code).await {
            Ok(v) => v,
            Err(e) => {
                warn!(
                    "Falha ao obter versão para org {}: {}. Forçando sync completo.",
                    org_siorg_code, e
                );
                return self
                    .run_full_unit_sync(org_id, org_siorg_code, summary)
                    .await;
            }
        };

        let stored_versao = self.get_stored_versao(org_siorg_code).await;

        let (_, total_local) = self
            .unit_repo
            .list(
                Some(org_id),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                1,
                0,
            )
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        let base_vazia = total_local == 0;

        match stored_versao {
            // Cenário A: Mesma versão e banco já populado -> Nada a fazer
            Some(ref from) if *from == versao_api.versao_consulta && !base_vazia => {
                info!(
                    "Org {} já está sincronizado na versão {}.",
                    org_siorg_code, from
                );
            }
            // Cenário B: Existe versão anterior e dados locais -> Executa Sync Incremental
            Some(from) if !base_vazia => {
                info!(
                    "Iniciando sync incremental org {}: {} → {}",
                    org_siorg_code, from, versao_api.versao_consulta
                );
                self.run_incremental_unit_sync(org_id, org_siorg_code, &from, summary)
                    .await?;
                // Sucesso total no lote -> Salva a nova versão
                self.save_versao(org_siorg_code, &versao_api.versao_consulta)
                    .await;
            }
            // Cenário C: Base vazia ou sem versão -> Executa Sync Completo Inicial (Bulk Upsert)
            _ => {
                info!(
                    "Iniciando carga completa para org {} (Base vazia ou sem histórico).",
                    org_siorg_code
                );
                self.run_full_unit_sync(org_id, org_siorg_code, summary)
                    .await?;
                // Sucesso total no lote -> Salva a versão inicial
                self.save_versao(org_siorg_code, &versao_api.versao_consulta)
                    .await;
            }
        }

        Ok(())
    }

    async fn run_full_unit_sync(
        &self,
        org_id: Uuid,
        org_siorg_code: i32,
        summary: &mut SyncSummary,
    ) -> Result<(), SyncError> {
        let units = self
            .siorg_client
            .get_estrutura_completa(org_siorg_code)
            .await
            .map_err(|e| SyncError::ApiError(format!("{:#}", e)))?;

        let result = self
            .execute_bulk_sync(org_id, units, org_siorg_code)
            .await?;

        summary.total_processed += result.total_processed;
        summary.updated += result.updated;
        summary.created += result.created;
        summary.failed += result.failed;

        Ok(())
    }

    /// Sync incremental: processa apenas unidades alteradas desde `from_versao`.
    /// - INCLUSAO / ALTERACAO → upsert
    /// - EXCLUSAO / EXTINCAO → desativa localmente
    async fn run_incremental_unit_sync(
        &self,
        org_id: Uuid,
        org_siorg_code: i32,
        from_versao: &str,
        summary: &mut SyncSummary,
    ) -> Result<(), SyncError> {
        let changed_units = self
            .siorg_client
            .get_alteradas(org_siorg_code, from_versao)
            .await
            .map_err(|e| SyncError::ApiError(format!("{:#}", e)))?;

        // 2. Converte SiorgUnidadeAlterada -> SiorgUnidadeCompleta
        // O seu código já tem um impl From para isso
        let units: Vec<SiorgUnidadeCompleta> = changed_units
            .into_iter()
            .map(SiorgUnidadeCompleta::from)
            .collect();

        // 3. Executa o processamento em lote
        let result = self
            .execute_bulk_sync(org_id, units, org_siorg_code)
            .await?;

        // 4. Atualiza o sumário
        summary.total_processed += result.total_processed;
        summary.updated += result.updated;
        summary.deleted += result.deleted;

        Ok(())
    }

    /// Synchronize all units for an organization (sync completo sem versioning).
    ///
    /// Faz uma única chamada HTTP (`GET /estrutura-organizacional/completa?codigoUnidade=X`)
    /// que retorna todas as unidades da estrutura de uma vez, eliminando o padrão N+1.
    pub async fn sync_organization_units(
        &self,
        org_siorg_code: i32,
    ) -> Result<SyncSummary, SyncError> {
        info!(
            "Starting bulk sync for organization SIORG code {} (single API call)",
            org_siorg_code
        );

        let units = self
            .siorg_client
            .get_estrutura_completa(org_siorg_code)
            .await
            .map_err(|e| SyncError::ApiError(format!("{:#}", e)))?;

        let mut summary = SyncSummary {
            total_processed: 0,
            created: 0,
            updated: 0,
            deleted: 0,
            failed: 0,
            errors: Vec::new(),
        };

        // Exclude the root org entity itself — its record lives in `organizations`, not
        // `organizational_units`. The API sometimes includes it in the full structure response.
        let units: Vec<_> = units
            .into_iter()
            .filter(|u| u.siorg_code() != Some(org_siorg_code))
            .collect();

        info!(
            "Received {} unit(s) from SIORG for org {} (root entity excluded)",
            units.len(),
            org_siorg_code
        );

        for siorg_unit in &units {
            summary.total_processed += 1;

            match self.upsert_unit(siorg_unit).await {
                Ok(unit) => {
                    if unit.created_at == unit.updated_at {
                        summary.created += 1;
                    } else {
                        summary.updated += 1;
                    }
                }
                Err(e) => {
                    let code = siorg_unit
                        .siorg_code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| siorg_unit.base.codigo_unidade.clone());
                    summary.failed += 1;
                    summary.errors.push(format!("Unit {}: {}", code, e));
                    error!("Failed to sync unit {}: {}", code, e);
                }
            }
        }

        info!("Bulk sync completed: {:?}", summary);
        Ok(summary)
    }

    async fn execute_bulk_sync(
        &self,
        org_id: Uuid,
        units: Vec<SiorgUnidadeCompleta>,
        org_siorg_code: i32,
    ) -> Result<SyncSummary, SyncError> {
        let units: Vec<_> = units
            .into_iter()
            .filter(|u| u.siorg_code() != Some(org_siorg_code))
            .collect();

        if units.is_empty() {
            return Ok(SyncSummary::default());
        }

        // ====================================================================
        // 1. PRÉ-PROCESSAMENTO: Garante todas as dependências antes de abrir a transação
        // ====================================================================
        let (types_cache, cats_cache) = self.prepare_dependencies(&units).await?;

        let mut id_lookup: HashMap<i32, Uuid> =
            self.unit_repo
                .get_siorg_map_by_org(org_id)
                .await
                .map_err(|e| SyncError::DatabaseError(e.to_string()))?;

        let mut sorted_units = units;
        sorted_units.sort_by_key(|u| u.base.codigo_unidade.len());

        // ====================================================================
        // 2. TRANSAÇÃO PRINCIPAL
        // ====================================================================
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
        let mut summary = SyncSummary::default();

        for siorg_unit in sorted_units {
            summary.total_processed += 1;
            let siorg_code = siorg_unit.siorg_code().unwrap();

            // Busca os IDs reais nos caches que preparamos acima (Tempo: O(1))
            let tipo_ref = siorg_unit
                .base
                .codigo_tipo_unidade
                .as_deref()
                .unwrap_or("NAO_INFORMADO");
            let cat_ref = siorg_unit
                .codigo_categoria_unidade
                .as_deref()
                .unwrap_or("NAO_INFORMADA");

            let unit_type_id = *types_cache.get(tipo_ref).unwrap();
            let category_id = *cats_cache.get(cat_ref).unwrap();

            let payload = SiorgUpsertPayload {
                organization_id: org_id,
                parent_id: siorg_unit
                    .parent_siorg_code()
                    .and_then(|c| id_lookup.get(&c).cloned()),
                category_id,  // <--- Agora usa a categoria real do SIORG!
                unit_type_id, // <--- Agora usa o tipo real do SIORG!
                internal_type: self.map_internal_type(&siorg_unit.base.codigo_tipo_unidade), // O Enum continua heurístico
                name: siorg_unit.base.nome.clone(),
                acronym: siorg_unit.base.sigla.clone(),
                siorg_code,
                activity_area: match siorg_unit.area_atuacao.as_deref() {
                    Some("FIM") | Some("CORE") => ActivityArea::Core,
                    _ => ActivityArea::Support,
                },
                is_active: !siorg_unit.base.is_exclusao(),
            };

            match self.unit_repo.upsert_in_transaction(&mut tx, payload).await {
                Ok(unit) => {
                    id_lookup.insert(siorg_code, unit.id);
                    if siorg_unit.base.is_exclusao() {
                        summary.deleted += 1;
                    } else {
                        summary.updated += 1;
                    }
                }
                Err(e) => {
                    tx.rollback().await.ok();
                    return Err(SyncError::DatabaseError(e.to_string()));
                }
            }
        }

        tx.commit()
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?;
        Ok(summary)
    }

    fn map_internal_type(&self, code: &Option<String>) -> InternalUnitType {
        match code.as_deref() {
            Some("DEPARTMENT") => InternalUnitType::Department,
            Some("LABORATORY") => InternalUnitType::Laboratory,
            Some("COORDINATION") => InternalUnitType::Coordination,
            _ => InternalUnitType::Sector,
        }
    }
    // ========================================================================
    // Version Storage (via SystemSettings)
    // ========================================================================

    /// Retorna a versão SIORG armazenada localmente para um órgão, ou None se nunca sincronizado.
    async fn get_stored_versao(&self, org_siorg_code: i32) -> Option<String> {
        let key = versao_setting_key(org_siorg_code);
        self.settings_repo
            .get(&key)
            .await
            .ok()
            .flatten()
            .and_then(|s| s.value.as_str().map(String::from))
    }

    /// Persiste a versão SIORG atual para um órgão em `system_settings`.
    /// Tenta update; se a chave não existir, cria.
    async fn save_versao(&self, org_siorg_code: i32, versao: &str) {
        let key = versao_setting_key(org_siorg_code);
        let value = serde_json::json!(versao);

        let update_result = self
            .settings_repo
            .update(
                &key,
                UpdateSystemSettingPayload {
                    value: Some(value.clone()),
                    description: None,
                    category: None,
                    is_sensitive: None,
                },
                None,
            )
            .await;

        if update_result.is_err() {
            if let Err(e) = self
                .settings_repo
                .create(CreateSystemSettingPayload {
                    key: key.clone(),
                    value,
                    value_type: "string".to_string(),
                    description: Some("Versão SIORG — gerenciado automaticamente".to_string()),
                    category: Some("siorg".to_string()),
                    is_sensitive: false,
                })
                .await
            {
                warn!(
                    "Falha ao salvar versão SIORG para org {}: {}",
                    org_siorg_code, e
                );
            }
        }
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    async fn find_or_create_category(
        &self,
        name: &str,
    ) -> Result<OrganizationalUnitCategoryDto, SyncError> {
        // Direct lookup by name — avoids the limit-10 list pagination issue
        if let Some(category) = self
            .category_repo
            .find_by_name(name)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
        {
            return Ok(category);
        }

        let payload = CreateOrganizationalUnitCategoryPayload {
            name: name.to_string(),
            description: Some("Auto-created from SIORG sync".to_string()),
            siorg_code: None,
            display_order: 0,
            is_active: true,
        };

        match self.category_repo.create(payload).await {
            Ok(category) => Ok(category),
            Err(e) => {
                // Unique constraint violation: was created between our find and create.
                // Retry the lookup.
                let err_str = e.to_string();
                if err_str.contains("unicidade")
                    || err_str.contains("unique")
                    || err_str.contains("duplicate")
                {
                    self.category_repo
                        .find_by_name(name)
                        .await
                        .map_err(|e2| SyncError::DatabaseError(e2.to_string()))?
                        .ok_or_else(|| {
                            SyncError::DatabaseError(format!(
                                "Category '{}' unique violation but not found on retry: {}",
                                name, err_str
                            ))
                        })
                } else {
                    Err(SyncError::DatabaseError(err_str))
                }
            }
        }
    }

    async fn find_or_create_type(
        &self,
        code: &str,
    ) -> Result<OrganizationalUnitTypeDto, SyncError> {
        if let Some(unit_type) = self
            .type_repo
            .find_by_code(code)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))?
        {
            return Ok(unit_type);
        }

        let payload = CreateOrganizationalUnitTypePayload {
            code: code.to_string(),
            name: code.to_string(),
            description: Some("Auto-created from SIORG sync".to_string()),
            siorg_code: None,
            is_active: true,
        };

        self.type_repo
            .create(payload)
            .await
            .map_err(|e| SyncError::DatabaseError(e.to_string()))
    }

    /// Check SIORG API health
    pub async fn check_health(&self) -> Result<bool, SyncError> {
        self.siorg_client
            .health_check()
            .await
            .map_err(|e| SyncError::ApiError(format!("{:#}", e)))
    }
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("SIORG API error: {0}")]
    ApiError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Entity not found in SIORG: {0}")]
    NotFoundInSiorg(i32),

    #[error("Missing required field: {0}")]
    MissingRequiredField(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}
