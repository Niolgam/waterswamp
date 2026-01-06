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
    )
)]
pub struct ApiDoc;
