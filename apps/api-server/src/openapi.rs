use utoipa::OpenApi;

/// Definição básica da OpenAPI (Swagger)
/// Esta versão não depende de handlers específicos - apenas fornece a estrutura
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Waterswamp API",
        version = "1.0.0",
        description = "API de autenticação e autorização baseada em RBAC com Casbin",
        contact(
            name = "Equipe Waterswamp",
            email = "suporte@waterswamp.com",
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Desenvolvimento Local"),
    ),
    tags(
        (name = "Public", description = "Rotas públicas sem autenticação"),
        (name = "Auth", description = "Autenticação e gerenciamento de tokens"),
        (name = "Health", description = "Endpoints de saúde e monitoramento"),
        (name = "User", description = "Rotas de usuário autenticado"),
        (name = "Admin", description = "Rotas administrativas (requer permissão admin)"),
        (name = "Geo Regions - Countries", description = "Gerenciamento de países"),
        (name = "Geo Regions - States", description = "Gerenciamento de estados/províncias"),
        (name = "Geo Regions - Cities", description = "Gerenciamento de cidades"),
        (name = "Budget Classifications", description = "Classificações orçamentárias hierárquicas (c.g.mm.ee.dd)"),
        (name = "Catalog - Units", description = "Unidades de medida para catálogo"),
        (name = "Catalog - Conversions", description = "Conversões entre unidades de medida"),
        (name = "CATMAT - Groups", description = "Grupos do Catálogo de Materiais (CATMAT)"),
        (name = "CATMAT - Classes", description = "Classes do Catálogo de Materiais (CATMAT)"),
        (name = "CATMAT - Items", description = "Itens PDM do Catálogo de Materiais (CATMAT)"),
        (name = "CATSER - Groups", description = "Grupos do Catálogo de Serviços (CATSER)"),
        (name = "CATSER - Classes", description = "Classes do Catálogo de Serviços (CATSER)"),
        (name = "CATSER - Items", description = "Itens do Catálogo de Serviços (CATSER)"),
        (name = "Organization - System Settings", description = "Configurações globais do sistema"),
        (name = "Organization - Organizations", description = "Gerenciamento de organizações (CNPJ, SIORG)"),
        (name = "Organization - Unit Categories", description = "Categorias de unidades organizacionais"),
        (name = "Organization - Unit Types", description = "Tipos de unidades organizacionais"),
        (name = "Organization - Organizational Units", description = "Unidades organizacionais hierárquicas com sincronização SIORG"),
        (name = "Organization - SIORG Sync", description = "Sincronização bidirecional com API do SIORG"),
    ),
    paths(
        // Health
        crate::handlers::health_handler::handler_health,
        crate::handlers::health_handler::handler_ready,
        crate::handlers::health_handler::handler_liveness,

        // Auth
        crate::api::auth::handlers::login,
        crate::api::auth::handlers::register,
        crate::api::auth::handlers::refresh_token,
        crate::api::auth::handlers::logout,
        crate::api::auth::handlers::forgot_password,
        crate::api::auth::handlers::reset_password,

        // User
        crate::api::users::handlers::get_profile,
        crate::api::users::handlers::update_profile,
        crate::api::users::handlers::change_password,

        // Admin - Users
        crate::api::admin::users::handlers::list_users,
        crate::api::admin::users::handlers::create_user,
        crate::api::admin::users::handlers::get_user,
        crate::api::admin::users::handlers::update_user,
        crate::api::admin::users::handlers::delete_user,
        crate::api::admin::users::handlers::ban_user,
        crate::api::admin::users::handlers::unban_user,

        // Admin - Locations (Countries)
        crate::api::geo_regions::handlers::list_countries,
        crate::api::geo_regions::handlers::get_country,
        crate::api::geo_regions::handlers::create_country,
        crate::api::geo_regions::handlers::update_country,
        crate::api::geo_regions::handlers::delete_country,

        // Admin - Locations (States)
        crate::api::geo_regions::handlers::list_states,
        crate::api::geo_regions::handlers::get_state,
        crate::api::geo_regions::handlers::create_state,
        crate::api::geo_regions::handlers::update_state,
        crate::api::geo_regions::handlers::delete_state,

        // Admin - Locations (Cities)
        crate::api::geo_regions::handlers::list_cities,
        crate::api::geo_regions::handlers::get_city,
        crate::api::geo_regions::handlers::create_city,
        crate::api::geo_regions::handlers::update_city,
        crate::api::geo_regions::handlers::delete_city,

        // Budget Classifications
        crate::api::budget_classifications::handlers::list_budget_classifications,
        crate::api::budget_classifications::handlers::get_tree,
        crate::api::budget_classifications::handlers::get_budget_classification,
        crate::api::budget_classifications::handlers::create_budget_classification,
        crate::api::budget_classifications::handlers::update_budget_classification,
        crate::api::budget_classifications::handlers::delete_budget_classification,

        // Catalog - Units of Measure
        crate::api::catalog::handlers::create_unit_of_measure,
        crate::api::catalog::handlers::get_unit_of_measure,
        crate::api::catalog::handlers::list_units_of_measure,
        crate::api::catalog::handlers::update_unit_of_measure,
        crate::api::catalog::handlers::delete_unit_of_measure,

        // Catalog - Unit Conversions
        crate::api::catalog::handlers::create_unit_conversion,
        crate::api::catalog::handlers::get_unit_conversion,
        crate::api::catalog::handlers::list_unit_conversions,
        crate::api::catalog::handlers::update_unit_conversion,
        crate::api::catalog::handlers::delete_unit_conversion,

        // CATMAT - Groups
        crate::api::catalog::handlers::create_catmat_group,
        crate::api::catalog::handlers::get_catmat_group,
        crate::api::catalog::handlers::list_catmat_groups,
        crate::api::catalog::handlers::get_catmat_tree,
        crate::api::catalog::handlers::update_catmat_group,
        crate::api::catalog::handlers::delete_catmat_group,

        // CATMAT - Classes
        crate::api::catalog::handlers::create_catmat_class,
        crate::api::catalog::handlers::get_catmat_class,
        crate::api::catalog::handlers::list_catmat_classes,
        crate::api::catalog::handlers::update_catmat_class,
        crate::api::catalog::handlers::delete_catmat_class,

        // CATMAT - Items (PDM)
        crate::api::catalog::handlers::create_catmat_item,
        crate::api::catalog::handlers::get_catmat_item,
        crate::api::catalog::handlers::list_catmat_items,
        crate::api::catalog::handlers::update_catmat_item,
        crate::api::catalog::handlers::delete_catmat_item,

        // CATSER - Groups
        crate::api::catalog::handlers::create_catser_group,
        crate::api::catalog::handlers::get_catser_group,
        crate::api::catalog::handlers::list_catser_groups,
        crate::api::catalog::handlers::get_catser_tree,
        crate::api::catalog::handlers::update_catser_group,
        crate::api::catalog::handlers::delete_catser_group,

        // CATSER - Classes
        crate::api::catalog::handlers::create_catser_class,
        crate::api::catalog::handlers::get_catser_class,
        crate::api::catalog::handlers::list_catser_classes,
        crate::api::catalog::handlers::update_catser_class,
        crate::api::catalog::handlers::delete_catser_class,

        // CATSER - Items (Serviço)
        crate::api::catalog::handlers::create_catser_item,
        crate::api::catalog::handlers::get_catser_item,
        crate::api::catalog::handlers::list_catser_items,
        crate::api::catalog::handlers::update_catser_item,
        crate::api::catalog::handlers::delete_catser_item,

        // Organization - System Settings
        crate::api::organizational::handlers::create_system_setting,
        crate::api::organizational::handlers::list_system_settings,
        crate::api::organizational::handlers::get_system_setting,
        crate::api::organizational::handlers::update_system_setting,
        crate::api::organizational::handlers::delete_system_setting,

        // Organization - Organizations
        crate::api::organizational::handlers::create_organization,
        crate::api::organizational::handlers::list_organizations,
        crate::api::organizational::handlers::get_organization,
        crate::api::organizational::handlers::update_organization,
        crate::api::organizational::handlers::delete_organization,

        // Organization - Unit Categories
        crate::api::organizational::handlers::create_unit_category,
        crate::api::organizational::handlers::list_unit_categories,
        crate::api::organizational::handlers::get_unit_category,
        crate::api::organizational::handlers::update_unit_category,
        crate::api::organizational::handlers::delete_unit_category,

        // Organization - Unit Types
        crate::api::organizational::handlers::create_unit_type,
        crate::api::organizational::handlers::list_unit_types,
        crate::api::organizational::handlers::get_unit_type,
        crate::api::organizational::handlers::update_unit_type,
        crate::api::organizational::handlers::delete_unit_type,

        // Organization - Organizational Units
        crate::api::organizational::handlers::create_organizational_unit,
        crate::api::organizational::handlers::list_organizational_units,
        crate::api::organizational::handlers::get_organizational_units_tree,
        crate::api::organizational::handlers::get_organizational_unit,
        crate::api::organizational::handlers::get_organizational_unit_children,
        crate::api::organizational::handlers::get_organizational_unit_path,
        crate::api::organizational::handlers::update_organizational_unit,
        crate::api::organizational::handlers::delete_organizational_unit,
        crate::api::organizational::handlers::deactivate_organizational_unit,
        crate::api::organizational::handlers::activate_organizational_unit,

        // Organization - SIORG Sync
        crate::api::organizational::handlers::sync_all_from_db,
        crate::api::organizational::handlers::sync_organization,
        crate::api::organizational::handlers::sync_organization_by_id,
        crate::api::organizational::handlers::sync_organization_units_by_id,
        crate::api::organizational::handlers::sync_unit,
        crate::api::organizational::handlers::sync_organization_units,
        crate::api::organizational::handlers::check_siorg_health,
    ),
    components(
        schemas(
            // Health
            crate::handlers::health_handler::HealthResponse,
            crate::handlers::health_handler::DatabaseHealth,
            crate::handlers::health_handler::CasbinHealth,

            // Auth
            crate::api::auth::contracts::LoginRequest,
            crate::api::auth::contracts::LoginResponse,
            crate::api::auth::contracts::RegisterRequest,
            crate::api::auth::contracts::RegisterResponse,
            crate::api::auth::contracts::RefreshTokenRequest,
            crate::api::auth::contracts::RefreshTokenResponse,
            crate::api::auth::contracts::LogoutRequest,
            crate::api::auth::contracts::LogoutResponse,
            crate::api::auth::contracts::ForgotPasswordRequest,
            crate::api::auth::contracts::ForgotPasswordResponse,
            crate::api::auth::contracts::ResetPasswordRequest,
            crate::api::auth::contracts::ResetPasswordResponse,
            crate::api::auth::contracts::MfaRequiredResponse,

            // User
            crate::api::users::contracts::ProfileResponse,
            crate::api::users::contracts::UpdateProfileRequest,
            crate::api::users::contracts::ChangePasswordRequest,
            crate::api::users::contracts::ChangePasswordResponse,

            // Admin - Users
            crate::api::admin::users::contracts::AdminUserListResponse,
            crate::api::admin::users::contracts::BanUserRequest,
            crate::api::admin::users::contracts::UserActionResponse,
            crate::api::admin::users::contracts::PaginationParams,

            // Locations
            crate::api::geo_regions::contracts::CountryResponse,
            crate::api::geo_regions::contracts::StateResponse,
            crate::api::geo_regions::contracts::StateWithCountryResponse,
            crate::api::geo_regions::contracts::CityResponse,
            crate::api::geo_regions::contracts::CityWithStateResponse,

            // Budget Classifications
            crate::api::budget_classifications::contracts::BudgetClassificationResponse,
            crate::api::budget_classifications::contracts::BudgetClassificationWithParentResponse,
            domain::models::budget_classifications::CreateBudgetClassificationPayload,
            domain::models::budget_classifications::UpdateBudgetClassificationPayload,
            domain::models::budget_classifications::BudgetClassificationTreeNode,

            // Catalog - Domain Models (Units & Conversions)
            domain::models::catalog::UnitOfMeasureDto,
            domain::models::catalog::CreateUnitOfMeasurePayload,
            domain::models::catalog::UpdateUnitOfMeasurePayload,
            domain::models::catalog::UnitConversionDto,
            domain::models::catalog::UnitConversionWithDetailsDto,
            domain::models::catalog::CreateUnitConversionPayload,
            domain::models::catalog::UpdateUnitConversionPayload,

            // CATMAT - Domain Models
            domain::models::catalog::CatmatGroupDto,
            domain::models::catalog::CreateCatmatGroupPayload,
            domain::models::catalog::UpdateCatmatGroupPayload,
            domain::models::catalog::CatmatGroupTreeNode,
            domain::models::catalog::CatmatClassTreeNode,
            domain::models::catalog::CatmatClassDto,
            domain::models::catalog::CatmatClassWithDetailsDto,
            domain::models::catalog::CreateCatmatClassPayload,
            domain::models::catalog::UpdateCatmatClassPayload,
            domain::models::catalog::CatmatItemDto,
            domain::models::catalog::CatmatItemWithDetailsDto,
            domain::models::catalog::CreateCatmatItemPayload,
            domain::models::catalog::UpdateCatmatItemPayload,

            // CATSER - Domain Models
            domain::models::catalog::CatserGroupDto,
            domain::models::catalog::CreateCatserGroupPayload,
            domain::models::catalog::UpdateCatserGroupPayload,
            domain::models::catalog::CatserGroupTreeNode,
            domain::models::catalog::CatserClassTreeNode,
            domain::models::catalog::CatserClassDto,
            domain::models::catalog::CatserClassWithDetailsDto,
            domain::models::catalog::CreateCatserClassPayload,
            domain::models::catalog::UpdateCatserClassPayload,
            domain::models::catalog::CatserItemDto,
            domain::models::catalog::CatserItemWithDetailsDto,
            domain::models::catalog::CreateCatserItemPayload,
            domain::models::catalog::UpdateCatserItemPayload,

            // Catalog - API Contracts
            crate::api::catalog::contracts::UnitsOfMeasureListResponse,
            crate::api::catalog::contracts::UnitConversionsListResponse,
            crate::api::catalog::contracts::CatmatGroupsListResponse,
            crate::api::catalog::contracts::CatmatClassesListResponse,
            crate::api::catalog::contracts::CatmatItemsListResponse,
            crate::api::catalog::contracts::CatserGroupsListResponse,
            crate::api::catalog::contracts::CatserClassesListResponse,
            crate::api::catalog::contracts::CatserItemsListResponse,

            // Organization - Domain Models
            domain::models::organizational::ActivityArea,
            domain::models::organizational::InternalUnitType,
            domain::models::organizational::ContactInfo,
            domain::models::organizational::SystemSettingDto,
            domain::models::organizational::CreateSystemSettingPayload,
            domain::models::organizational::UpdateSystemSettingPayload,
            domain::models::organizational::OrganizationDto,
            domain::models::organizational::CreateOrganizationPayload,
            domain::models::organizational::UpdateOrganizationPayload,
            domain::models::organizational::OrganizationalUnitCategoryDto,
            domain::models::organizational::CreateOrganizationalUnitCategoryPayload,
            domain::models::organizational::UpdateOrganizationalUnitCategoryPayload,
            domain::models::organizational::OrganizationalUnitTypeDto,
            domain::models::organizational::CreateOrganizationalUnitTypePayload,
            domain::models::organizational::UpdateOrganizationalUnitTypePayload,
            domain::models::organizational::OrganizationalUnitDto,
            domain::models::organizational::OrganizationalUnitWithDetailsDto,
            domain::models::organizational::OrganizationalUnitTreeNode,
            domain::models::organizational::CreateOrganizationalUnitPayload,
            domain::models::organizational::UpdateOrganizationalUnitPayload,

            // Organization - API Contracts
            crate::api::organizational::contracts::SystemSettingsListResponse,
            crate::api::organizational::contracts::OrganizationsListResponse,
            crate::api::organizational::contracts::OrganizationalUnitCategoriesListResponse,
            crate::api::organizational::contracts::OrganizationalUnitTypesListResponse,
            crate::api::organizational::contracts::OrganizationalUnitsListResponse,

        )
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

use utoipa::Modify;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = &mut openapi.components {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }
}
