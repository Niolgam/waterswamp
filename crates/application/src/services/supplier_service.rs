use crate::errors::ServiceError;
use domain::{
    models::supplier::*,
    ports::supplier::*,
};
use std::sync::Arc;
use uuid::Uuid;

// ============================
// Brazilian Document Validators
// ============================

/// Validates a Brazilian CPF (11 digits with 2 check digits)
pub fn validate_cpf(cpf: &str) -> Result<(), String> {
    let digits_only: String = cpf.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits_only.len() != 11 {
        return Err("CPF deve ter exatamente 11 dígitos".to_string());
    }

    // Reject known invalid sequences (all same digit)
    let first = digits_only.chars().next().unwrap();
    if digits_only.chars().all(|c| c == first) {
        return Err("CPF inválido".to_string());
    }

    let digits: Vec<u32> = digits_only.chars().map(|c| c.to_digit(10).unwrap()).collect();

    // First check digit
    let sum1: u32 = digits.iter().take(9).enumerate().map(|(i, d)| d * (10 - i as u32)).sum();
    let check1 = {
        let rem = (sum1 * 10) % 11;
        if rem >= 10 { 0 } else { rem }
    };
    if check1 != digits[9] {
        return Err("CPF inválido (dígito verificador incorreto)".to_string());
    }

    // Second check digit
    let sum2: u32 = digits.iter().take(10).enumerate().map(|(i, d)| d * (11 - i as u32)).sum();
    let check2 = {
        let rem = (sum2 * 10) % 11;
        if rem >= 10 { 0 } else { rem }
    };
    if check2 != digits[10] {
        return Err("CPF inválido (dígito verificador incorreto)".to_string());
    }

    Ok(())
}

/// Validates a Brazilian CNPJ (14 digits with 2 check digits)
pub fn validate_cnpj(cnpj: &str) -> Result<(), String> {
    let digits_only: String = cnpj.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits_only.len() != 14 {
        return Err("CNPJ deve ter exatamente 14 dígitos".to_string());
    }

    // Reject known invalid sequences
    let first = digits_only.chars().next().unwrap();
    if digits_only.chars().all(|c| c == first) {
        return Err("CNPJ inválido".to_string());
    }

    let digits: Vec<u32> = digits_only.chars().map(|c| c.to_digit(10).unwrap()).collect();

    // First check digit - weights: 5,4,3,2,9,8,7,6,5,4,3,2
    let weights1: Vec<u32> = vec![5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum1: u32 = digits.iter().take(12).zip(weights1.iter()).map(|(d, w)| d * w).sum();
    let check1 = {
        let rem = sum1 % 11;
        if rem < 2 { 0 } else { 11 - rem }
    };
    if check1 != digits[12] {
        return Err("CNPJ inválido (dígito verificador incorreto)".to_string());
    }

    // Second check digit - weights: 6,5,4,3,2,9,8,7,6,5,4,3,2
    let weights2: Vec<u32> = vec![6, 5, 4, 3, 2, 9, 8, 7, 6, 5, 4, 3, 2];
    let sum2: u32 = digits.iter().take(13).zip(weights2.iter()).map(|(d, w)| d * w).sum();
    let check2 = {
        let rem = sum2 % 11;
        if rem < 2 { 0 } else { 11 - rem }
    };
    if check2 != digits[13] {
        return Err("CNPJ inválido (dígito verificador incorreto)".to_string());
    }

    Ok(())
}

/// Normalizes a document number by stripping non-digit characters
pub fn normalize_document(doc: &str) -> String {
    doc.chars().filter(|c| c.is_ascii_digit()).collect()
}

// ============================
// Supplier Service
// ============================

pub struct SupplierService {
    supplier_repo: Arc<dyn SupplierRepositoryPort>,
}

impl SupplierService {
    pub fn new(supplier_repo: Arc<dyn SupplierRepositoryPort>) -> Self {
        Self { supplier_repo }
    }

    pub async fn create_supplier(
        &self,
        payload: CreateSupplierPayload,
        created_by: Option<Uuid>,
    ) -> Result<SupplierWithDetailsDto, ServiceError> {
        // Normalize document
        let doc = normalize_document(&payload.document_number);

        // Validate document based on supplier type
        match payload.supplier_type {
            SupplierType::Individual => {
                validate_cpf(&doc).map_err(ServiceError::BadRequest)?;
            }
            SupplierType::LegalEntity => {
                validate_cnpj(&doc).map_err(ServiceError::BadRequest)?;
            }
            SupplierType::GovernmentUnit => {
                // UG/GESTÃO code - just ensure it's not empty
                if doc.is_empty() && payload.document_number.trim().is_empty() {
                    return Err(ServiceError::BadRequest("Código UG/Gestão é obrigatório".to_string()));
                }
            }
        }

        // Use normalized digits for CPF/CNPJ, original for UG
        let document_to_store = match payload.supplier_type {
            SupplierType::GovernmentUnit => payload.document_number.trim().to_string(),
            _ => doc,
        };

        // Check uniqueness
        if self.supplier_repo.exists_by_document_number(&document_to_store).await.map_err(ServiceError::from)? {
            return Err(ServiceError::Conflict(format!(
                "Fornecedor com documento '{}' já existe",
                document_to_store
            )));
        }

        let is_intl = payload.is_international_neighborhood.unwrap_or(false);

        let supplier = self.supplier_repo
            .create(
                &payload.supplier_type,
                &payload.legal_name,
                payload.trade_name.as_deref(),
                &document_to_store,
                payload.representative_name.as_deref(),
                payload.address.as_deref(),
                payload.neighborhood.as_deref(),
                is_intl,
                payload.city_id,
                payload.zip_code.as_deref(),
                payload.email.as_deref(),
                payload.phone.as_deref(),
                created_by,
            )
            .await
            .map_err(ServiceError::from)?;

        // Return with details
        self.supplier_repo
            .find_with_details_by_id(supplier.id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar fornecedor criado".to_string()))
    }

    pub async fn get_supplier(&self, id: Uuid) -> Result<SupplierWithDetailsDto, ServiceError> {
        self.supplier_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Fornecedor não encontrado".to_string()))
    }

    pub async fn update_supplier(
        &self,
        id: Uuid,
        payload: UpdateSupplierPayload,
        updated_by: Option<Uuid>,
    ) -> Result<SupplierWithDetailsDto, ServiceError> {
        let current = self.supplier_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Fornecedor não encontrado".to_string()))?;

        // Determine effective supplier type for validation
        let effective_type = payload.supplier_type.as_ref().unwrap_or(&current.supplier_type);

        // Validate and normalize document if being changed
        let normalized_doc = if let Some(ref doc) = payload.document_number {
            let d = normalize_document(doc);
            match effective_type {
                SupplierType::Individual => {
                    validate_cpf(&d).map_err(ServiceError::BadRequest)?;
                }
                SupplierType::LegalEntity => {
                    validate_cnpj(&d).map_err(ServiceError::BadRequest)?;
                }
                SupplierType::GovernmentUnit => {
                    if d.is_empty() && doc.trim().is_empty() {
                        return Err(ServiceError::BadRequest("Código UG/Gestão é obrigatório".to_string()));
                    }
                }
            }
            let document_to_store = match effective_type {
                SupplierType::GovernmentUnit => doc.trim().to_string(),
                _ => d,
            };
            // Check uniqueness
            if self.supplier_repo.exists_by_document_number_excluding(&document_to_store, id).await.map_err(ServiceError::from)? {
                return Err(ServiceError::Conflict(format!(
                    "Fornecedor com documento '{}' já existe",
                    document_to_store
                )));
            }
            Some(document_to_store)
        } else {
            None
        };

        let _ = self.supplier_repo
            .update(
                id,
                payload.supplier_type.as_ref(),
                payload.legal_name.as_deref(),
                payload.trade_name.as_deref(),
                normalized_doc.as_deref(),
                payload.representative_name.as_deref(),
                payload.address.as_deref(),
                payload.neighborhood.as_deref(),
                payload.is_international_neighborhood,
                payload.city_id,
                payload.zip_code.as_deref(),
                payload.email.as_deref(),
                payload.phone.as_deref(),
                payload.is_active,
                updated_by,
            )
            .await
            .map_err(ServiceError::from)?;

        self.supplier_repo
            .find_with_details_by_id(id)
            .await
            .map_err(ServiceError::from)?
            .ok_or(ServiceError::Internal("Falha ao buscar fornecedor atualizado".to_string()))
    }

    pub async fn delete_supplier(&self, id: Uuid) -> Result<bool, ServiceError> {
        let _ = self.supplier_repo.find_by_id(id).await.map_err(ServiceError::from)?
            .ok_or(ServiceError::NotFound("Fornecedor não encontrado".to_string()))?;
        self.supplier_repo.delete(id).await.map_err(ServiceError::from)
    }

    pub async fn list_suppliers(
        &self,
        limit: i64,
        offset: i64,
        search: Option<String>,
        supplier_type: Option<SupplierType>,
        is_active: Option<bool>,
    ) -> Result<(Vec<SupplierWithDetailsDto>, i64), ServiceError> {
        self.supplier_repo
            .list(limit, offset, search, supplier_type, is_active)
            .await
            .map_err(ServiceError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_cpf_valid() {
        assert!(validate_cpf("52998224725").is_ok());
        assert!(validate_cpf("529.982.247-25").is_ok());
    }

    #[test]
    fn test_validate_cpf_invalid() {
        assert!(validate_cpf("11111111111").is_err()); // All same digits
        assert!(validate_cpf("12345678901").is_err()); // Wrong check digit
        assert!(validate_cpf("123").is_err()); // Too short
    }

    #[test]
    fn test_validate_cnpj_valid() {
        assert!(validate_cnpj("11222333000181").is_ok());
        assert!(validate_cnpj("11.222.333/0001-81").is_ok());
    }

    #[test]
    fn test_validate_cnpj_invalid() {
        assert!(validate_cnpj("11111111111111").is_err()); // All same digits
        assert!(validate_cnpj("11222333000182").is_err()); // Wrong check digit
        assert!(validate_cnpj("123").is_err()); // Too short
    }

    #[test]
    fn test_normalize_document() {
        assert_eq!(normalize_document("529.982.247-25"), "52998224725");
        assert_eq!(normalize_document("11.222.333/0001-81"), "11222333000181");
    }
}
