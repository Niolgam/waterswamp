//! Constantes da aplicação
//!
//! Centraliza todas as constantes usadas na aplicação para facilitar
//! manutenção e configuração.

// =============================================================================
// ROLES E PERMISSÕES (CASBIN)
// =============================================================================

/// Role de administrador (acesso total)
pub const ROLE_ADMIN: &str = "admin";

/// Role de usuário normal (acesso limitado)
pub const ROLE_USER: &str = "user";

// =============================================================================
// RECURSOS (CASBIN)
// =============================================================================

/// Dashboard administrativo
pub const RESOURCE_ADMIN_DASHBOARD: &str = "/admin/dashboard";

/// Perfil de usuário
pub const RESOURCE_USER_PROFILE: &str = "/users/profile";

/// Gestão de políticas (admin)
pub const RESOURCE_ADMIN_POLICIES: &str = "/api/admin/policies";

// =============================================================================
// AÇÕES (CASBIN)
// =============================================================================

/// Ação HTTP: GET
pub const ACTION_GET: &str = "GET";

/// Ação HTTP: POST
pub const ACTION_POST: &str = "POST";

/// Ação HTTP: PUT
pub const ACTION_PUT: &str = "PUT";

/// Ação HTTP: DELETE
pub const ACTION_DELETE: &str = "DELETE";

/// Ação HTTP: PATCH
pub const ACTION_PATCH: &str = "PATCH";

// =============================================================================
// JWT TOKEN EXPIRY
// =============================================================================

/// Tempo de expiração do Access Token (1 hora = 3600 segundos)
pub const ACCESS_TOKEN_EXPIRY_SECONDS: i64 = 3600;

/// Tempo de expiração do Refresh Token (7 dias = 604800 segundos)
pub const REFRESH_TOKEN_EXPIRY_SECONDS: i64 = 604800;

/// Tempo de expiração do Password Reset Token (15 minutos = 900 segundos)
pub const PASSWORD_RESET_EXPIRY_SECONDS: i64 = 900;

/// Tempo de expiração do MFA Challenge Token (5 minutos = 300 segundos)
pub const MFA_CHALLENGE_EXPIRY_SECONDS: i64 = 300;

/// Tempo de expiração do MFA Setup Token (15 minutos = 900 segundos)
pub const MFA_SETUP_EXPIRY_MINUTES: i64 = 15;

// =============================================================================
// EMAIL VERIFICATION
// =============================================================================

/// Tempo de expiração do Email Verification Token (24 horas = 1440 minutos)
pub const EMAIL_VERIFICATION_EXPIRY_MINUTES: i64 = 1440;

/// Limite de requisições de reenvio de verificação (5 minutos)
pub const EMAIL_VERIFICATION_RATE_LIMIT_MINUTES: i64 = 5;

/// Máximo de requisições de reenvio permitidas
pub const EMAIL_VERIFICATION_MAX_REQUESTS: i64 = 3;

// =============================================================================
// MFA / TOTP
// =============================================================================

/// Número de backup codes gerados para MFA
pub const BACKUP_CODES_COUNT: usize = 10;

/// Comprimento de cada backup code
pub const BACKUP_CODE_LENGTH: usize = 12;

/// Charset para backup codes (sem caracteres ambíguos: 0, O, I, l, 1)
pub const BACKUP_CODE_CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

/// Algoritmo TOTP padrão
pub const TOTP_ALGORITHM: &str = "SHA1";

/// Dígitos do código TOTP
pub const TOTP_DIGITS: usize = 6;

/// Período do TOTP (segundos)
pub const TOTP_PERIOD: u64 = 30;

/// Issuer do TOTP (nome da aplicação)
pub const TOTP_ISSUER: &str = "Waterswamp";

// =============================================================================
// RATE LIMITING
// =============================================================================

/// Taxa limite para login (5 tentativas a cada 10 segundos)
pub const LOGIN_RATE_LIMIT_BURST: u32 = 5;
pub const LOGIN_RATE_LIMIT_PERIOD_SECS: u64 = 10;

/// Taxa limite para rotas admin (10 requisições a cada 2 segundos)
pub const ADMIN_RATE_LIMIT_BURST: u32 = 10;
pub const ADMIN_RATE_LIMIT_PERIOD_SECS: u64 = 2;

/// Taxa limite geral para API (50 requisições a cada 200ms)
pub const API_RATE_LIMIT_BURST: u32 = 50;
pub const API_RATE_LIMIT_PERIOD_MS: u64 = 200;

// =============================================================================
// PAGINATION
// =============================================================================

/// Limite padrão de resultados por página
pub const DEFAULT_PAGE_LIMIT: i64 = 20;

/// Limite máximo de resultados por página
pub const MAX_PAGE_LIMIT: i64 = 100;

// =============================================================================
// AUDIT LOGS
// =============================================================================

/// Período de retenção padrão de audit logs (90 dias)
pub const AUDIT_LOG_RETENTION_DAYS: i64 = 90;

/// Limite de logs retornados por consulta
pub const AUDIT_LOG_QUERY_LIMIT: i64 = 1000;

// =============================================================================
// PASSWORD STRENGTH
// =============================================================================

/// Score mínimo de força de senha (zxcvbn) - 3 = Strong
pub const MIN_PASSWORD_SCORE: u8 = 3;

/// Comprimento mínimo da senha
pub const MIN_PASSWORD_LENGTH: usize = 8;

/// Comprimento máximo da senha
pub const MAX_PASSWORD_LENGTH: usize = 128;

// =============================================================================
// USERNAME VALIDATION
// =============================================================================

/// Comprimento mínimo do username
pub const MIN_USERNAME_LENGTH: usize = 3;

/// Comprimento máximo do username
pub const MAX_USERNAME_LENGTH: usize = 50;

// =============================================================================
// ARGON2 PARAMETERS (Password Hashing)
// =============================================================================

/// Argon2 Memory Cost (64 MiB)
pub const ARGON2_M_COST: u32 = 65536;

/// Argon2 Time Cost (3 iterations)
pub const ARGON2_T_COST: u32 = 3;

/// Argon2 Parallelism (4 threads)
pub const ARGON2_P_COST: u32 = 4;

// =============================================================================
// TESTES
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_expiry_constants() {
        assert_eq!(ACCESS_TOKEN_EXPIRY_SECONDS, 3600);
        assert_eq!(REFRESH_TOKEN_EXPIRY_SECONDS, 604800);
        assert_eq!(PASSWORD_RESET_EXPIRY_SECONDS, 900);
        assert_eq!(MFA_CHALLENGE_EXPIRY_SECONDS, 300);
    }

    #[test]
    fn test_backup_code_charset() {
        // Não deve conter caracteres ambíguos
        let charset_str = std::str::from_utf8(BACKUP_CODE_CHARSET).unwrap();
        assert!(!charset_str.contains('0'));
        assert!(!charset_str.contains('O'));
        assert!(!charset_str.contains('I'));
        assert!(!charset_str.contains('l'));
        assert!(!charset_str.contains('1'));
    }

    #[test]
    fn test_backup_code_config() {
        assert_eq!(BACKUP_CODES_COUNT, 10);
        assert_eq!(BACKUP_CODE_LENGTH, 12);
    }

    #[test]
    fn test_rate_limit_config() {
        assert!(LOGIN_RATE_LIMIT_BURST > 0);
        assert!(ADMIN_RATE_LIMIT_BURST > 0);
        assert!(API_RATE_LIMIT_BURST > 0);
    }

    #[test]
    fn test_pagination_config() {
        assert!(DEFAULT_PAGE_LIMIT <= MAX_PAGE_LIMIT);
        assert!(DEFAULT_PAGE_LIMIT > 0);
        assert!(MAX_PAGE_LIMIT > 0);
    }
}
