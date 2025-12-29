#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::ServiceError;
    use async_trait::async_trait;
    use domain::errors::RepositoryError;
    use domain::models::*;
    use domain::ports::*;
    use domain::value_objects::*;
    use rust_decimal::Decimal;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    // ============================
    // Mock Repositories
    // ============================

    #[derive(Clone)]
    struct MockMaterialGroupRepository;

    #[async_trait]
    impl MaterialGroupRepositoryPort for MockMaterialGroupRepository {
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<MaterialGroupDto>, RepositoryError> {
            Ok(Some(MaterialGroupDto {
                id: Uuid::new_v4(),
                code: MaterialCode::try_from("01".to_string()).unwrap(),
                name: "Test Group".to_string(),
                description: None,
                expense_element: None,
                is_personnel_exclusive: false,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        }

        async fn find_by_code(
            &self,
            _code: &MaterialCode,
        ) -> Result<Option<MaterialGroupDto>, RepositoryError> {
            Ok(None)
        }

        async fn exists_by_code(&self, _code: &MaterialCode) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        async fn exists_by_code_excluding(
            &self,
            _code: &MaterialCode,
            _exclude_id: Uuid,
        ) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        async fn create(
            &self,
            code: &MaterialCode,
            name: &str,
            description: Option<&str>,
            expense_element: Option<&str>,
            is_personnel_exclusive: bool,
        ) -> Result<MaterialGroupDto, RepositoryError> {
            Ok(MaterialGroupDto {
                id: Uuid::new_v4(),
                code: code.clone(),
                name: name.to_string(),
                description: description.map(|s| s.to_string()),
                expense_element: expense_element.map(|s| s.to_string()),
                is_personnel_exclusive,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            })
        }

        async fn update(
            &self,
            _id: Uuid,
            _code: Option<&MaterialCode>,
            _name: Option<&str>,
            _description: Option<&str>,
            _expense_element: Option<&str>,
            _is_personnel_exclusive: Option<bool>,
        ) -> Result<MaterialGroupDto, RepositoryError> {
            unimplemented!()
        }

        async fn delete(&self, _id: Uuid) -> Result<bool, RepositoryError> {
            unimplemented!()
        }

        async fn list(
            &self,
            _limit: i64,
            _offset: i64,
            _search: Option<String>,
        ) -> Result<(Vec<MaterialGroupDto>, i64), RepositoryError> {
            Ok((vec![], 0))
        }
    }

    #[derive(Clone)]
    struct MockMaterialRepository;

    #[async_trait]
    impl MaterialRepositoryPort for MockMaterialRepository {
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<MaterialDto>, RepositoryError> {
            Ok(Some(MaterialDto {
                id: Uuid::new_v4(),
                material_group_id: Uuid::new_v4(),
                code: MaterialCode::try_from("MAT001".to_string()).unwrap(),
                name: "Test Material".to_string(),
                description: None,
                catmat_code: Some(CatmatCode::try_from("123456".to_string()).unwrap()),
                unit_of_measure: UnitOfMeasure::Unidade,
                estimated_value: Decimal::from_str("10.00").unwrap(),
                minimum_quantity: None,
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        }

        async fn find_by_code(
            &self,
            _code: &MaterialCode,
        ) -> Result<Option<MaterialDto>, RepositoryError> {
            Ok(None)
        }

        async fn find_with_group_by_id(
            &self,
            _id: Uuid,
        ) -> Result<Option<MaterialWithGroupDto>, RepositoryError> {
            unimplemented!()
        }

        async fn exists_by_code(&self, _code: &MaterialCode) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        async fn exists_by_code_excluding(
            &self,
            _code: &MaterialCode,
            _exclude_id: Uuid,
        ) -> Result<bool, RepositoryError> {
            Ok(false)
        }

        async fn create(
            &self,
            _material_group_id: Uuid,
            _code: &MaterialCode,
            _name: &str,
            _description: Option<&str>,
            _catmat_code: Option<&CatmatCode>,
            _unit_of_measure: UnitOfMeasure,
            _estimated_value: Decimal,
            _minimum_quantity: Option<Decimal>,
            _is_active: bool,
        ) -> Result<MaterialDto, RepositoryError> {
            unimplemented!()
        }

        async fn update(
            &self,
            _id: Uuid,
            _material_group_id: Option<Uuid>,
            _code: Option<&MaterialCode>,
            _name: Option<&str>,
            _description: Option<&str>,
            _catmat_code: Option<&CatmatCode>,
            _unit_of_measure: Option<UnitOfMeasure>,
            _estimated_value: Option<Decimal>,
            _minimum_quantity: Option<Decimal>,
            _is_active: Option<bool>,
        ) -> Result<MaterialDto, RepositoryError> {
            unimplemented!()
        }

        async fn delete(&self, _id: Uuid) -> Result<bool, RepositoryError> {
            unimplemented!()
        }

        async fn list(
            &self,
            _limit: i64,
            _offset: i64,
            _material_group_id: Option<Uuid>,
            _search: Option<String>,
            _is_active: Option<bool>,
        ) -> Result<(Vec<MaterialWithGroupDto>, i64), RepositoryError> {
            Ok((vec![], 0))
        }
    }

    #[derive(Clone)]
    struct MockWarehouseRepository;

    #[async_trait]
    impl WarehouseRepositoryPort for MockWarehouseRepository {
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<WarehouseDto>, RepositoryError> {
            Ok(Some(WarehouseDto {
                id: Uuid::new_v4(),
                city_id: Uuid::new_v4(),
                name: "Test Warehouse".to_string(),
                code: Some("WH001".to_string()),
                address: Some("Test Address".to_string()),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            }))
        }

        async fn find_with_city_by_id(
            &self,
            _id: Uuid,
        ) -> Result<Option<WarehouseWithCityDto>, RepositoryError> {
            unimplemented!()
        }

        async fn create(
            &self,
            _city_id: Uuid,
            _name: &str,
            _code: Option<&str>,
            _address: Option<&str>,
        ) -> Result<WarehouseDto, RepositoryError> {
            unimplemented!()
        }

        async fn update(
            &self,
            _id: Uuid,
            _city_id: Option<Uuid>,
            _name: Option<&str>,
            _code: Option<&str>,
            _address: Option<&str>,
        ) -> Result<WarehouseDto, RepositoryError> {
            unimplemented!()
        }

        async fn list(
            &self,
            _limit: i64,
            _offset: i64,
            _city_id: Option<Uuid>,
            _search: Option<String>,
        ) -> Result<(Vec<WarehouseWithCityDto>, i64), RepositoryError> {
            Ok((vec![], 0))
        }
    }

    // Mock for WarehouseStockRepository with state tracking
    #[derive(Clone)]
    struct MockWarehouseStockRepository {
        stocks: Arc<Mutex<Vec<WarehouseStockDto>>>,
    }

    impl MockWarehouseStockRepository {
        fn new() -> Self {
            Self {
                stocks: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    #[async_trait]
    impl WarehouseStockRepositoryPort for MockWarehouseStockRepository {
        async fn find_by_id(&self, id: Uuid) -> Result<Option<WarehouseStockDto>, RepositoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks.iter().find(|s| s.id == id).cloned())
        }

        async fn find_by_warehouse_and_material(
            &self,
            warehouse_id: Uuid,
            material_id: Uuid,
        ) -> Result<Option<WarehouseStockDto>, RepositoryError> {
            let stocks = self.stocks.lock().unwrap();
            Ok(stocks
                .iter()
                .find(|s| s.warehouse_id == warehouse_id && s.material_id == material_id)
                .cloned())
        }

        async fn find_with_details_by_id(
            &self,
            _id: Uuid,
        ) -> Result<Option<WarehouseStockWithDetailsDto>, RepositoryError> {
            unimplemented!()
        }

        async fn create(
            &self,
            warehouse_id: Uuid,
            material_id: Uuid,
            quantity: Decimal,
            average_unit_value: Decimal,
            minimum_quantity: Option<Decimal>,
        ) -> Result<WarehouseStockDto, RepositoryError> {
            let stock = WarehouseStockDto {
                id: Uuid::new_v4(),
                warehouse_id,
                material_id,
                quantity,
                average_unit_value,
                minimum_quantity,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            self.stocks.lock().unwrap().push(stock.clone());
            Ok(stock)
        }

        async fn update_stock_and_average(
            &self,
            id: Uuid,
            new_quantity: Decimal,
            new_average: Decimal,
        ) -> Result<WarehouseStockDto, RepositoryError> {
            let mut stocks = self.stocks.lock().unwrap();
            if let Some(stock) = stocks.iter_mut().find(|s| s.id == id) {
                stock.quantity = new_quantity;
                stock.average_unit_value = new_average;
                stock.updated_at = chrono::Utc::now();
                Ok(stock.clone())
            } else {
                Err(RepositoryError::NotFound)
            }
        }

        async fn delete(&self, _id: Uuid) -> Result<bool, RepositoryError> {
            unimplemented!()
        }

        async fn list(
            &self,
            _limit: i64,
            _offset: i64,
            _warehouse_id: Option<Uuid>,
            _material_id: Option<Uuid>,
            _search: Option<String>,
            _low_stock: Option<bool>,
        ) -> Result<(Vec<WarehouseStockWithDetailsDto>, i64), RepositoryError> {
            Ok((vec![], 0))
        }
    }

    #[derive(Clone)]
    struct MockStockMovementRepository;

    #[async_trait]
    impl StockMovementRepositoryPort for MockStockMovementRepository {
        async fn create(
            &self,
            warehouse_stock_id: Uuid,
            movement_type: MovementType,
            quantity: Decimal,
            unit_value: Decimal,
            total_value: Decimal,
            balance_before: Decimal,
            balance_after: Decimal,
            average_before: Decimal,
            average_after: Decimal,
            user_id: Uuid,
            document_number: Option<&str>,
            requisition_id: Option<Uuid>,
            notes: Option<&str>,
        ) -> Result<StockMovementDto, RepositoryError> {
            Ok(StockMovementDto {
                id: Uuid::new_v4(),
                warehouse_stock_id,
                movement_type,
                quantity,
                unit_value,
                total_value,
                balance_before,
                balance_after,
                average_before,
                average_after,
                user_id,
                document_number: document_number.map(|s| s.to_string()),
                requisition_id,
                notes: notes.map(|s| s.to_string()),
                movement_date: chrono::Utc::now(),
                created_at: chrono::Utc::now(),
            })
        }

        async fn find_by_id(&self, _id: Uuid) -> Result<Option<StockMovementDto>, RepositoryError> {
            unimplemented!()
        }

        async fn find_with_details_by_id(
            &self,
            _id: Uuid,
        ) -> Result<Option<StockMovementWithDetailsDto>, RepositoryError> {
            unimplemented!()
        }

        async fn list(
            &self,
            _limit: i64,
            _offset: i64,
            _warehouse_id: Option<Uuid>,
            _material_id: Option<Uuid>,
            _movement_type: Option<MovementType>,
            _start_date: Option<chrono::DateTime<chrono::Utc>>,
            _end_date: Option<chrono::DateTime<chrono::Utc>>,
        ) -> Result<(Vec<StockMovementWithDetailsDto>, i64), RepositoryError> {
            Ok((vec![], 0))
        }
    }

    // ============================
    // Tests
    // ============================

    use crate::services::warehouse_service::WarehouseService;

    #[tokio::test]
    async fn test_weighted_average_calculation() {
        let material_group_repo = Arc::new(MockMaterialGroupRepository);
        let material_repo = Arc::new(MockMaterialRepository);
        let warehouse_repo = Arc::new(MockWarehouseRepository);
        let stock_repo = Arc::new(MockWarehouseStockRepository::new());
        let movement_repo = Arc::new(MockStockMovementRepository);

        let service = WarehouseService::new(
            material_group_repo as Arc<dyn MaterialGroupRepositoryPort>,
            material_repo as Arc<dyn MaterialRepositoryPort>,
            warehouse_repo as Arc<dyn WarehouseRepositoryPort>,
            stock_repo.clone() as Arc<dyn WarehouseStockRepositoryPort>,
            movement_repo as Arc<dyn StockMovementRepositoryPort>,
        );

        let warehouse_id = Uuid::new_v4();
        let material_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // First entry: 100 units @ R$ 7.00
        let (stock1, _) = service
            .register_stock_entry(
                warehouse_id,
                material_id,
                Decimal::from_str("100").unwrap(),
                Decimal::from_str("7.00").unwrap(),
                user_id,
                Some("NF001"),
                Some("First entry"),
            )
            .await
            .unwrap();

        assert_eq!(stock1.quantity, Decimal::from_str("100").unwrap());
        assert_eq!(stock1.average_unit_value, Decimal::from_str("7.00").unwrap());

        // Second entry: 50 units @ R$ 8.00
        // Expected average: (100*7 + 50*8) / 150 = 1100/150 = 7.333...
        let (stock2, _) = service
            .register_stock_entry(
                warehouse_id,
                material_id,
                Decimal::from_str("50").unwrap(),
                Decimal::from_str("8.00").unwrap(),
                user_id,
                Some("NF002"),
                None,
            )
            .await
            .unwrap();

        assert_eq!(stock2.quantity, Decimal::from_str("150").unwrap());

        let expected_avg = (Decimal::from_str("100").unwrap() * Decimal::from_str("7.00").unwrap()
            + Decimal::from_str("50").unwrap() * Decimal::from_str("8.00").unwrap())
            / Decimal::from_str("150").unwrap();

        assert_eq!(stock2.average_unit_value, expected_avg);
    }

    #[tokio::test]
    async fn test_stock_exit_maintains_average() {
        let material_group_repo = Arc::new(MockMaterialGroupRepository);
        let material_repo = Arc::new(MockMaterialRepository);
        let warehouse_repo = Arc::new(MockWarehouseRepository);
        let stock_repo = Arc::new(MockWarehouseStockRepository::new());
        let movement_repo = Arc::new(MockStockMovementRepository);

        let service = WarehouseService::new(
            material_group_repo as Arc<dyn MaterialGroupRepositoryPort>,
            material_repo as Arc<dyn MaterialRepositoryPort>,
            warehouse_repo as Arc<dyn WarehouseRepositoryPort>,
            stock_repo.clone() as Arc<dyn WarehouseStockRepositoryPort>,
            movement_repo as Arc<dyn StockMovementRepositoryPort>,
        );

        let warehouse_id = Uuid::new_v4();
        let material_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Entry: 200 units @ R$ 10.00
        service
            .register_stock_entry(
                warehouse_id,
                material_id,
                Decimal::from_str("200").unwrap(),
                Decimal::from_str("10.00").unwrap(),
                user_id,
                None,
                None,
            )
            .await
            .unwrap();

        // Exit: 50 units (average should remain 10.00)
        let (stock, _) = service
            .register_stock_exit(
                warehouse_id,
                material_id,
                Decimal::from_str("50").unwrap(),
                user_id,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(stock.quantity, Decimal::from_str("150").unwrap());
        assert_eq!(stock.average_unit_value, Decimal::from_str("10.00").unwrap());
    }

    #[tokio::test]
    async fn test_stock_exit_insufficient_quantity() {
        let material_group_repo = Arc::new(MockMaterialGroupRepository);
        let material_repo = Arc::new(MockMaterialRepository);
        let warehouse_repo = Arc::new(MockWarehouseRepository);
        let stock_repo = Arc::new(MockWarehouseStockRepository::new());
        let movement_repo = Arc::new(MockStockMovementRepository);

        let service = WarehouseService::new(
            material_group_repo as Arc<dyn MaterialGroupRepositoryPort>,
            material_repo as Arc<dyn MaterialRepositoryPort>,
            warehouse_repo as Arc<dyn WarehouseRepositoryPort>,
            stock_repo.clone() as Arc<dyn WarehouseStockRepositoryPort>,
            movement_repo as Arc<dyn StockMovementRepositoryPort>,
        );

        let warehouse_id = Uuid::new_v4();
        let material_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Entry: only 10 units
        service
            .register_stock_entry(
                warehouse_id,
                material_id,
                Decimal::from_str("10").unwrap(),
                Decimal::from_str("5.00").unwrap(),
                user_id,
                None,
                None,
            )
            .await
            .unwrap();

        // Try to exit 20 units (more than available)
        let result = service
            .register_stock_exit(
                warehouse_id,
                material_id,
                Decimal::from_str("20").unwrap(),
                user_id,
                None,
                None,
                None,
            )
            .await;

        assert!(matches!(result, Err(ServiceError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_negative_quantity_validation() {
        let material_group_repo = Arc::new(MockMaterialGroupRepository);
        let material_repo = Arc::new(MockMaterialRepository);
        let warehouse_repo = Arc::new(MockWarehouseRepository);
        let stock_repo = Arc::new(MockWarehouseStockRepository::new());
        let movement_repo = Arc::new(MockStockMovementRepository);

        let service = WarehouseService::new(
            material_group_repo as Arc<dyn MaterialGroupRepositoryPort>,
            material_repo as Arc<dyn MaterialRepositoryPort>,
            warehouse_repo as Arc<dyn WarehouseRepositoryPort>,
            stock_repo as Arc<dyn WarehouseStockRepositoryPort>,
            movement_repo as Arc<dyn StockMovementRepositoryPort>,
        );

        let warehouse_id = Uuid::new_v4();
        let material_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Try to register entry with negative quantity
        let result = service
            .register_stock_entry(
                warehouse_id,
                material_id,
                Decimal::from_str("-10").unwrap(),
                Decimal::from_str("5.00").unwrap(),
                user_id,
                None,
                None,
            )
            .await;

        assert!(matches!(result, Err(ServiceError::BadRequest(_))));
    }
}
