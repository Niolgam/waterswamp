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
    )
)]
pub struct ApiDoc;
