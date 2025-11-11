pub async fn handler_user_profile() -> &'static str {
    // O usuário já foi autenticado e autorizado pelo middleware.
    "Este é o seu perfil de usuário."
}

// Handler de admin
pub async fn handler_admin_dashboard() -> &'static str {
    "Você está no dashboard de admin."
}
