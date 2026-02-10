-- Seed: Categorias
INSERT INTO vehicle_categories (name, description) VALUES
    ('Passeio', 'Veículo de passeio'),
    ('Utilitário', 'Veículo utilitário'),
    ('Caminhão', 'Caminhão de carga'),
    ('Ônibus', 'Ônibus de transporte'),
    ('Van', 'Van de transporte'),
    ('Motocicleta', 'Motocicleta'),
    ('Ambulância', 'Veículo de emergência médica');

-- Seed: Tipos de combustível
INSERT INTO vehicle_fuel_types (name) VALUES
    ('Gasolina'),
    ('Etanol'),
    ('Diesel'),
    ('Flex'),
    ('GNV'),
    ('Elétrico'),
    ('Híbrido');

-- Seed: Tipos de transmissão
INSERT INTO vehicle_transmission_types (name) VALUES
    ('Manual'),
    ('Automático'),
    ('CVT'),
    ('Automatizado');

-- Seed: Cores
INSERT INTO vehicle_colors (name) VALUES
    ('Branco'),
    ('Preto'),
    ('Prata'),
    ('Cinza'),
    ('Vermelho'),
    ('Azul'),
    ('Verde'),
    ('Amarelo');
