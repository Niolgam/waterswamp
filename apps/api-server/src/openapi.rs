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
