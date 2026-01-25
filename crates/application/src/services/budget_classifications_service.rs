use crate::errors::ServiceError;
use domain::models::{
    BudgetClassificationDto, BudgetClassificationTreeNode, BudgetClassificationWithParentDto,
    CreateBudgetClassificationPayload, UpdateBudgetClassificationPayload,
};
use domain::pagination::Paginated;
use domain::ports::BudgetClassificationRepositoryPort;
use std::sync::Arc;
use uuid::Uuid;

pub struct BudgetClassificationsService {
    repo: Arc<dyn BudgetClassificationRepositoryPort>,
}

impl BudgetClassificationsService {
    pub fn new(repo: Arc<dyn BudgetClassificationRepositoryPort>) -> Self {
        Self { repo }
    }

    pub async fn create(
        &self,
        payload: CreateBudgetClassificationPayload,
    ) -> Result<BudgetClassificationDto, ServiceError> {
        // Validate parent exists if provided
        if let Some(parent_id) = payload.parent_id {
            let parent = self.repo.find_by_id(parent_id).await?;
            if parent.is_none() {
                return Err(ServiceError::NotFound(
                    "Parent classification not found".to_string(),
                ));
            }

            // Check if parent is at max level (5)
            if let Some(p) = parent {
                if p.level >= 5 {
                    return Err(ServiceError::BadRequest(format!(
                        "Cannot add child to level {} classification (max is 5)",
                        p.level
                    )));
                }
            }
        }

        let classification = self
            .repo
            .create(
                payload.parent_id,
                &payload.code_part,
                &payload.name,
                payload.is_active,
            )
            .await?;

        Ok(classification)
    }

    pub async fn get(&self, id: Uuid) -> Result<BudgetClassificationWithParentDto, ServiceError> {
        self.repo
            .find_with_parent_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound(
                "Budget classification not found".to_string(),
            ))
    }

    pub async fn update(
        &self,
        id: Uuid,
        payload: UpdateBudgetClassificationPayload,
    ) -> Result<BudgetClassificationDto, ServiceError> {
        // Check if classification exists
        let _ = self.get(id).await?;

        // Validate parent if being updated
        if let Some(pid) = payload.parent_id {
            // Check parent exists
            let parent = self.repo.find_by_id(pid).await?;
            if parent.is_none() {
                return Err(ServiceError::NotFound(
                    "Parent classification not found".to_string(),
                ));
            }

            // Check parent level
            if let Some(p) = parent {
                if p.level >= 5 {
                    return Err(ServiceError::BadRequest(format!(
                        "Cannot add child to level {} classification (max is 5)",
                        p.level
                    )));
                }
            }

            // Prevent circular reference
            if pid == id {
                return Err(ServiceError::BadRequest(
                    "Cannot set parent to self".to_string(),
                ));
            }
        }

        let classification = self
            .repo
            .update(
                id,
                payload.parent_id.map(Some),
                payload.code_part.as_deref(),
                payload.name.as_deref(),
                payload.is_active,
            )
            .await?;

        Ok(classification)
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), ServiceError> {
        // Check if has children
        let children = self.repo.find_children(Some(id)).await?;
        if !children.is_empty() {
            return Err(ServiceError::Conflict(format!(
                "Cannot delete classification with {} child(ren)",
                children.len()
            )));
        }

        let deleted = self.repo.delete(id).await?;

        if !deleted {
            return Err(ServiceError::NotFound(
                "Budget classification not found".to_string(),
            ));
        }

        Ok(())
    }

    pub async fn list(
        &self,
        limit: Option<i64>,
        offset: Option<i64>,
        search: Option<String>,
        parent_id: Option<Uuid>,
        level: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<Paginated<BudgetClassificationWithParentDto>, ServiceError> {
        let limit = limit.unwrap_or(50).min(100);
        let offset = offset.unwrap_or(0);

        let (items, total) = self
            .repo
            .list(limit, offset, search, parent_id, level, is_active)
            .await?;

        Ok(Paginated::new(items, total, limit, offset))
    }

    pub async fn get_tree(&self) -> Result<Vec<BudgetClassificationTreeNode>, ServiceError> {
        // Get all root items (level 1)
        let roots = self.repo.find_by_level(1).await?;

        let mut tree = Vec::new();
        for root in roots {
            let node = self.build_tree_node(root).await?;
            tree.push(node);
        }

        Ok(tree)
    }

    pub async fn get_children(&self, parent_id: Option<Uuid>) -> Result<Vec<BudgetClassificationDto>, ServiceError> {
        self.repo.find_children(parent_id).await.map_err(|e| e.into())
    }

    pub async fn get_by_level(&self, level: i32) -> Result<Vec<BudgetClassificationDto>, ServiceError> {
        if level < 1 || level > 5 {
            return Err(ServiceError::BadRequest(
                "Level must be between 1 and 5".to_string(),
            ));
        }

        self.repo.find_by_level(level).await.map_err(|e| e.into())
    }

    // Helper function to recursively build tree
    fn build_tree_node<'a>(
        &'a self,
        item: BudgetClassificationDto,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<BudgetClassificationTreeNode, ServiceError>> + 'a + Send>> {
        Box::pin(async move {
            let children_data = self.repo.find_children(Some(item.id)).await?;

            let mut children = Vec::new();
            for child in children_data {
                let child_node = self.build_tree_node(child).await?;
                children.push(child_node);
            }

            Ok(BudgetClassificationTreeNode {
                id: item.id,
                parent_id: item.parent_id,
                code_part: item.code_part,
                full_code: item.full_code,
                name: item.name,
                level: item.level,
                is_active: item.is_active,
                children,
                created_at: item.created_at,
                updated_at: item.updated_at,
            })
        })
    }
}
