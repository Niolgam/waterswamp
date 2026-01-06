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
        (url = "https://staging-api.seudominio.com", description = "Staging"),
        (url = "https://api.seudominio.com", description = "Produção")
    ),
    tags(
        (name = "Public", description = "Rotas públicas sem autenticação"),
        (name = "Auth", description = "Autenticação e gerenciamento de tokens"),
        (name = "Health", description = "Endpoints de saúde e monitoramento"),
        (name = "User", description = "Rotas de usuário autenticado"),
        (name = "Admin", description = "Rotas administrativas (requer permissão admin)"),
        (name = "Material Groups", description = "Gestão de grupos de materiais"),
        (name = "Materials", description = "Gestão de materiais e serviços"),
        (name = "Warehouses", description = "Gestão de almoxarifados"),
        (name = "Stock", description = "Gestão de estoque e movimentações"),
        (name = "Requisitions", description = "Gestão de requisições de materiais"),
        (name = "Reports", description = "Relatórios e análises de almoxarifado"),
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
        crate::api::locations::geo_regions::handlers::list_countries,
        crate::api::locations::geo_regions::handlers::get_country,
        crate::api::locations::geo_regions::handlers::create_country,
        crate::api::locations::geo_regions::handlers::update_country,
        crate::api::locations::geo_regions::handlers::delete_country,

        // Admin - Locations (States)
        crate::api::locations::geo_regions::handlers::list_states,
        crate::api::locations::geo_regions::handlers::get_state,
        crate::api::locations::geo_regions::handlers::create_state,
        crate::api::locations::geo_regions::handlers::update_state,
        crate::api::locations::geo_regions::handlers::delete_state,

        // Admin - Locations (Cities)
        crate::api::locations::geo_regions::handlers::list_cities,
        crate::api::locations::geo_regions::handlers::get_city,
        crate::api::locations::geo_regions::handlers::create_city,
        crate::api::locations::geo_regions::handlers::update_city,
        crate::api::locations::geo_regions::handlers::delete_city,

        // Material Groups
        crate::api::warehouse::handlers::list_material_groups,
        crate::api::warehouse::handlers::get_material_group,
        crate::api::warehouse::handlers::create_material_group,
        crate::api::warehouse::handlers::update_material_group,
        crate::api::warehouse::handlers::delete_material_group,

        // Materials
        crate::api::warehouse::handlers::list_materials,
        crate::api::warehouse::handlers::get_material,
        crate::api::warehouse::handlers::create_material,
        crate::api::warehouse::handlers::update_material,
        crate::api::warehouse::handlers::delete_material,

        // Warehouses
        crate::api::warehouse::handlers::create_warehouse,
        crate::api::warehouse::handlers::get_warehouse,
        crate::api::warehouse::handlers::update_warehouse,

        // Stock Movements
        crate::api::warehouse::handlers::register_stock_entry,
        crate::api::warehouse::handlers::register_stock_exit,
        crate::api::warehouse::handlers::register_stock_adjustment,
        crate::api::warehouse::handlers::get_warehouse_stock,

        // Stock Maintenance
        crate::api::warehouse::handlers::update_stock_maintenance,
        crate::api::warehouse::handlers::block_material,
        crate::api::warehouse::handlers::unblock_material,
        crate::api::warehouse::handlers::transfer_stock,

        // Requisitions
        crate::api::requisitions::handlers::create_requisition,
        crate::api::requisitions::handlers::get_requisition,
        crate::api::requisitions::handlers::list_requisitions,
        crate::api::requisitions::handlers::approve_requisition,
        crate::api::requisitions::handlers::reject_requisition,
        crate::api::requisitions::handlers::fulfill_requisition,
        crate::api::requisitions::handlers::cancel_requisition,

        // Reports
        crate::api::warehouse::reports_handlers::get_stock_value_report,
        crate::api::warehouse::reports_handlers::get_stock_value_detail,
        crate::api::warehouse::reports_handlers::get_consumption_report,
        crate::api::warehouse::reports_handlers::get_most_requested_materials,
        crate::api::warehouse::reports_handlers::get_movement_analysis,
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
            crate::api::locations::geo_regions::contracts::CountryResponse,
            crate::api::locations::geo_regions::contracts::StateResponse,
            crate::api::locations::geo_regions::contracts::StateWithCountryResponse,
            crate::api::locations::geo_regions::contracts::CityResponse,
            crate::api::locations::geo_regions::contracts::CityWithStateResponse,

            // Domain models
            domain::models::CreateMaterialGroupPayload,
            domain::models::UpdateMaterialGroupPayload,
            domain::models::CreateMaterialPayload,
            domain::models::UpdateMaterialPayload,
            domain::models::CreateWarehousePayload,
            domain::models::UpdateWarehousePayload,
            domain::models::UpdateStockMaintenancePayload,
            domain::models::TransferStockPayload,
            domain::models::BlockMaterialPayload,
            domain::models::CreateRequisitionItemPayload,
            domain::models::FulfillRequisitionItemPayload,
            domain::models::MovementType,
            domain::models::RequisitionStatus,

            // Value objects
            domain::value_objects::MaterialCode,
            domain::value_objects::CatmatCode,
            domain::value_objects::UnitOfMeasure,

            // Handler request structs
            crate::api::warehouse::handlers::ListQuery,
            crate::api::warehouse::handlers::StockEntryRequest,
            crate::api::warehouse::handlers::StockExitRequest,
            crate::api::warehouse::handlers::StockAdjustmentRequest,
            crate::api::requisitions::handlers::CreateRequisitionRequest,
            crate::api::requisitions::handlers::ApproveRequisitionRequest,
            crate::api::requisitions::handlers::RejectRequisitionRequest,
            crate::api::requisitions::handlers::FulfillRequisitionRequest,
            crate::api::requisitions::handlers::ListRequisitionsQuery,
            crate::api::warehouse::reports_handlers::StockValueReportQuery,
            crate::api::warehouse::reports_handlers::StockValueDetailQuery,
            crate::api::warehouse::reports_handlers::ConsumptionReportQuery,
            crate::api::warehouse::reports_handlers::MostRequestedQuery,
            crate::api::warehouse::reports_handlers::MovementAnalysisQuery,
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
