-- Configurações de modo degradado e throttling (RF-035/RF-036)
INSERT INTO system_settings (key, value, value_type, category, description)
VALUES
    -- Circuit breaker: Comprasnet
    ('circuit.comprasnet.failure_threshold', '5', 'number', 'circuit_breaker',
     'Número de falhas consecutivas para abrir o circuit breaker da API Comprasnet'),
    ('circuit.comprasnet.recovery_seconds', '60', 'number', 'circuit_breaker',
     'Segundos para tentar HALF_OPEN após circuit breaker aberto (Comprasnet)'),

    -- Circuit breaker: SIORG
    ('circuit.siorg.failure_threshold', '3', 'number', 'circuit_breaker',
     'Número de falhas consecutivas para abrir o circuit breaker da API SIORG'),
    ('circuit.siorg.recovery_seconds', '120', 'number', 'circuit_breaker',
     'Segundos para tentar HALF_OPEN após circuit breaker aberto (SIORG)'),

    -- Throttle: chamadas por minuto por serviço externo
    ('throttle.comprasnet.max_calls_per_minute', '30', 'number', 'throttle',
     'Máximo de chamadas por minuto para a API Comprasnet (RF-036)'),
    ('throttle.siorg.max_calls_per_minute', '20', 'number', 'throttle',
     'Máximo de chamadas por minuto para a API SIORG (RF-036)'),

    -- Degraded mode
    ('degraded.notify_admin_email', '"admin@ufmt.br"', 'string', 'degraded',
     'Email para notificar quando o sistema entrar em modo degradado (RF-035)')
ON CONFLICT (key) DO NOTHING;
