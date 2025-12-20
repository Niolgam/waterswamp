# Backend API Guide - Public Map & Locations

Guia completo de endpoints necessários para o Mapa Público e CRUD de Localizações.

---

## 📋 Índice

1. [Endpoints do Mapa Público](#endpoints-do-mapa-público)
2. [Endpoints CRUD de Locations](#endpoints-crud-de-locations)
3. [Estrutura de Dados](#estrutura-de-dados)
4. [Coordenadas Geográficas](#coordenadas-geográficas)
5. [Tipos e Catálogos](#tipos-e-catálogos)
6. [Autenticação e Permissões](#autenticação-e-permissões)

---

## Endpoints do Mapa Público

Endpoints públicos (sem autenticação) para visualização do mapa.

### 1. Listar Sites Disponíveis

**Endpoint:** `GET /api/locations/public/sites`

**Autenticação:** Não requerida

**Resposta Esperada:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid-site-1",
      "name": "Campus Downtown",
      "description": "Main downtown campus with administrative buildings",
      "address": "123 Main Street, Downtown",
      "code": "DOWNTOWN",
      "buildingCount": 15,
      "spaceCount": 450,
      "bounds": {
        "minLng": -122.4194,
        "minLat": 37.7749,
        "maxLng": -122.4000,
        "maxLat": 37.7900
      },
      "center": {
        "lng": -122.4097,
        "lat": 37.7825
      },
      "defaultZoom": 15
    },
    {
      "id": "uuid-site-2",
      "name": "Medical District",
      "description": "Medical facilities and research centers",
      "address": "456 Health Ave, Medical District",
      "code": "MEDICAL",
      "buildingCount": 8,
      "spaceCount": 320,
      "bounds": {
        "minLng": -122.4300,
        "minLat": 37.7650,
        "maxLng": -122.4100,
        "maxLat": 37.7800
      },
      "center": {
        "lng": -122.4200,
        "lat": 37.7725
      },
      "defaultZoom": 16
    }
  ],
  "timestamp": "2024-12-20T12:00:00Z"
}
```

---

### 2. Buscar Site por ID

**Endpoint:** `GET /api/locations/public/sites/:id`

**Parâmetros:**
- `id` (path): ID do site

**Resposta Esperada:**
```json
{
  "success": true,
  "data": {
    "id": "uuid-site-1",
    "name": "Campus Downtown",
    "description": "Main downtown campus",
    "address": "123 Main Street",
    "code": "DOWNTOWN",
    "cityId": "uuid-city-1",
    "siteTypeId": "uuid-type-campus",
    "buildingCount": 15,
    "spaceCount": 450,
    "bounds": {
      "minLng": -122.4194,
      "minLat": 37.7749,
      "maxLng": -122.4000,
      "maxLat": 37.7900
    },
    "center": {
      "lng": -122.4097,
      "lat": 37.7825
    },
    "defaultZoom": 15,
    "createdAt": "2024-01-15T10:00:00Z",
    "updatedAt": "2024-12-20T10:00:00Z"
  }
}
```

---

### 3. Listar Buildings de um Site

**Endpoint:** `GET /api/locations/public/sites/:siteId/buildings`

**Parâmetros:**
- `siteId` (path): ID do site

**Resposta Esperada:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid-building-1",
      "name": "Administration Building",
      "code": "ADMIN-001",
      "siteId": "uuid-site-1",
      "buildingTypeId": "uuid-type-admin",
      "buildingType": {
        "id": "uuid-type-admin",
        "name": "Administrative",
        "icon": "ki-outline ki-building",
        "color": "#FF9955",
        "description": "Administrative buildings"
      },
      "totalFloors": 5,
      "floorCount": 5,
      "spaceCount": 120,
      "address": "Building A, 123 Main St",
      "coordinates": [
        [-122.4100, 37.7800],
        [-122.4090, 37.7800],
        [-122.4090, 37.7790],
        [-122.4100, 37.7790],
        [-122.4100, 37.7800]
      ],
      "createdAt": "2024-01-15T10:00:00Z",
      "updatedAt": "2024-12-20T10:00:00Z"
    },
    {
      "id": "uuid-building-2",
      "name": "Library Building",
      "code": "LIB-001",
      "siteId": "uuid-site-1",
      "buildingTypeId": "uuid-type-library",
      "buildingType": {
        "id": "uuid-type-library",
        "name": "Library",
        "icon": "ki-outline ki-book",
        "color": "#88CC88",
        "description": "Library and study buildings"
      },
      "totalFloors": 3,
      "floorCount": 3,
      "spaceCount": 45,
      "address": "Building B, 123 Main St",
      "coordinates": [
        [-122.4120, 37.7810],
        [-122.4110, 37.7810],
        [-122.4110, 37.7800],
        [-122.4120, 37.7800],
        [-122.4120, 37.7810]
      ],
      "createdAt": "2024-02-01T10:00:00Z",
      "updatedAt": "2024-12-15T10:00:00Z"
    }
  ],
  "meta": {
    "total": 15,
    "page": 1,
    "limit": 50
  }
}
```

**IMPORTANTE - Formato das Coordenadas:**
```typescript
// Polígono para edifício (array de [lng, lat])
coordinates: [
  [longitude, latitude],  // Ponto 1
  [longitude, latitude],  // Ponto 2
  [longitude, latitude],  // Ponto 3
  [longitude, latitude],  // Ponto 4
  [longitude, latitude]   // Ponto 1 (fecha o polígono)
]

// Exemplo real:
coordinates: [
  [-122.4100, 37.7800],  // Canto superior esquerdo
  [-122.4090, 37.7800],  // Canto superior direito
  [-122.4090, 37.7790],  // Canto inferior direito
  [-122.4100, 37.7790],  // Canto inferior esquerdo
  [-122.4100, 37.7800]   // Volta ao início (fecha)
]
```

---

### 4. Listar Spaces de um Site

**Endpoint:** `GET /api/locations/public/sites/:siteId/spaces`

**Parâmetros:**
- `siteId` (path): ID do site

**Resposta Esperada:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid-space-1",
      "name": "Room 101",
      "code": "A-101",
      "floorId": "uuid-floor-1",
      "spaceTypeId": "uuid-type-office",
      "spaceType": {
        "id": "uuid-type-office",
        "name": "Office",
        "icon": "ki-outline ki-briefcase",
        "color": "#3B82F6",
        "description": "Office spaces"
      },
      "locationType": "point",
      "coordinates": [-122.4095, 37.7795],
      "capacity": 4,
      "area": 25.5,
      "description": "Small office with 4 workstations",
      "floor": {
        "id": "uuid-floor-1",
        "name": "Ground Floor",
        "floorNumber": 0,
        "buildingId": "uuid-building-1",
        "building": {
          "id": "uuid-building-1",
          "name": "Administration Building",
          "siteId": "uuid-site-1"
        }
      },
      "createdAt": "2024-01-20T10:00:00Z",
      "updatedAt": "2024-12-10T10:00:00Z"
    },
    {
      "id": "uuid-space-2",
      "name": "Conference Room A",
      "code": "A-CONF-A",
      "floorId": "uuid-floor-1",
      "spaceTypeId": "uuid-type-meeting",
      "spaceType": {
        "id": "uuid-type-meeting",
        "name": "Meeting Room",
        "icon": "ki-outline ki-people",
        "color": "#F59E0B",
        "description": "Meeting and conference rooms"
      },
      "locationType": "polygon",
      "coordinates": [
        [-122.4098, 37.7796],
        [-122.4096, 37.7796],
        [-122.4096, 37.7794],
        [-122.4098, 37.7794],
        [-122.4098, 37.7796]
      ],
      "capacity": 20,
      "area": 45.0,
      "description": "Large conference room with AV equipment",
      "floor": {
        "id": "uuid-floor-1",
        "name": "Ground Floor",
        "floorNumber": 0,
        "buildingId": "uuid-building-1"
      },
      "createdAt": "2024-01-20T10:00:00Z",
      "updatedAt": "2024-12-10T10:00:00Z"
    }
  ],
  "meta": {
    "total": 450,
    "page": 1,
    "limit": 100
  }
}
```

**Tipos de Coordenadas para Spaces:**
```typescript
// Ponto (para espaços pequenos)
{
  "locationType": "point",
  "coordinates": [longitude, latitude]  // Apenas um ponto
}

// Polígono (para espaços grandes)
{
  "locationType": "polygon",
  "coordinates": [
    [lng, lat],
    [lng, lat],
    [lng, lat],
    [lng, lat],
    [lng, lat]  // Fecha o polígono
  ]
}
```

---

### 5. Buscar Building por ID

**Endpoint:** `GET /api/locations/public/buildings/:id`

**Parâmetros:**
- `id` (path): ID do building

**Resposta Esperada:**
```json
{
  "success": true,
  "data": {
    "id": "uuid-building-1",
    "name": "Administration Building",
    "code": "ADMIN-001",
    "siteId": "uuid-site-1",
    "buildingTypeId": "uuid-type-admin",
    "buildingType": {
      "id": "uuid-type-admin",
      "name": "Administrative",
      "icon": "ki-outline ki-building",
      "color": "#FF9955"
    },
    "totalFloors": 5,
    "floorCount": 5,
    "spaceCount": 120,
    "address": "Building A, 123 Main St",
    "coordinates": [
      [-122.4100, 37.7800],
      [-122.4090, 37.7800],
      [-122.4090, 37.7790],
      [-122.4100, 37.7790],
      [-122.4100, 37.7800]
    ],
    "floors": [
      {
        "id": "uuid-floor-1",
        "name": "Ground Floor",
        "floorNumber": 0,
        "spaceCount": 25
      },
      {
        "id": "uuid-floor-2",
        "name": "First Floor",
        "floorNumber": 1,
        "spaceCount": 24
      }
    ],
    "createdAt": "2024-01-15T10:00:00Z",
    "updatedAt": "2024-12-20T10:00:00Z"
  }
}
```

---

### 6. Buscar Space por ID

**Endpoint:** `GET /api/locations/public/spaces/:id`

**Parâmetros:**
- `id` (path): ID do space

**Resposta Esperada:**
```json
{
  "success": true,
  "data": {
    "id": "uuid-space-1",
    "name": "Room 101",
    "code": "A-101",
    "floorId": "uuid-floor-1",
    "spaceTypeId": "uuid-type-office",
    "spaceType": {
      "id": "uuid-type-office",
      "name": "Office",
      "icon": "ki-outline ki-briefcase",
      "color": "#3B82F6"
    },
    "locationType": "point",
    "coordinates": [-122.4095, 37.7795],
    "capacity": 4,
    "area": 25.5,
    "description": "Small office with 4 workstations",
    "floor": {
      "id": "uuid-floor-1",
      "name": "Ground Floor",
      "floorNumber": 0,
      "buildingId": "uuid-building-1",
      "building": {
        "id": "uuid-building-1",
        "name": "Administration Building",
        "code": "ADMIN-001",
        "siteId": "uuid-site-1"
      }
    },
    "createdAt": "2024-01-20T10:00:00Z",
    "updatedAt": "2024-12-10T10:00:00Z"
  }
}
```

---

### 7. Buscar Locations (Search)

**Endpoint:** `GET /api/locations/public/search`

**Query Parameters:**
- `q` (required): Termo de busca
- `siteId` (optional): Filtrar por site específico

**Exemplo:** `GET /api/locations/public/search?q=admin&siteId=uuid-site-1`

**Resposta Esperada:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid-building-1",
      "type": "building",
      "name": "Administration Building",
      "code": "ADMIN-001",
      "siteId": "uuid-site-1",
      "buildingTypeId": "uuid-type-admin",
      "buildingType": {
        "id": "uuid-type-admin",
        "name": "Administrative",
        "icon": "ki-outline ki-building",
        "color": "#FF9955"
      },
      "coordinates": [[-122.4100, 37.7800], ...],
      "matchType": "name"
    },
    {
      "id": "uuid-space-5",
      "type": "space",
      "name": "Admin Office",
      "code": "B-ADMIN",
      "floorId": "uuid-floor-2",
      "spaceTypeId": "uuid-type-office",
      "spaceType": {
        "id": "uuid-type-office",
        "name": "Office",
        "icon": "ki-outline ki-briefcase",
        "color": "#3B82F6"
      },
      "locationType": "point",
      "coordinates": [-122.4115, 37.7805],
      "floor": {
        "name": "First Floor",
        "floorNumber": 1,
        "buildingId": "uuid-building-2"
      },
      "matchType": "name"
    }
  ],
  "meta": {
    "query": "admin",
    "total": 2,
    "limit": 50
  }
}
```

**IMPORTANTE:** O campo `type` deve ser "building" ou "space" para o frontend diferenciar.

---

### 8. Listar Building Types (para filtros)

**Endpoint:** `GET /api/locations/public/building-types`

**Resposta Esperada:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid-type-admin",
      "name": "Administrative",
      "icon": "ki-outline ki-building",
      "color": "#FF9955",
      "description": "Administrative buildings",
      "count": 5
    },
    {
      "id": "uuid-type-library",
      "name": "Library",
      "icon": "ki-outline ki-book",
      "color": "#88CC88",
      "description": "Library buildings",
      "count": 2
    },
    {
      "id": "uuid-type-lab",
      "name": "Laboratory",
      "icon": "ki-outline ki-flask",
      "color": "#AA88CC",
      "description": "Laboratory buildings",
      "count": 8
    }
  ]
}
```

---

### 9. Listar Space Types (para filtros)

**Endpoint:** `GET /api/locations/public/space-types`

**Resposta Esperada:**
```json
{
  "success": true,
  "data": [
    {
      "id": "uuid-type-office",
      "name": "Office",
      "icon": "ki-outline ki-briefcase",
      "color": "#3B82F6",
      "description": "Office spaces",
      "count": 120
    },
    {
      "id": "uuid-type-meeting",
      "name": "Meeting Room",
      "icon": "ki-outline ki-people",
      "color": "#F59E0B",
      "description": "Meeting rooms",
      "count": 45
    },
    {
      "id": "uuid-type-lab",
      "name": "Laboratory",
      "icon": "ki-outline ki-flask",
      "color": "#8B5CF6",
      "description": "Laboratory spaces",
      "count": 80
    }
  ]
}
```

---

## Endpoints CRUD de Locations

Endpoints administrativos (requerem autenticação e permissões).

### Autenticação

Todos os endpoints abaixo requerem:
- Header: `Authorization: Bearer <token>`
- Permissões: `locations:view`, `locations:create`, `locations:update`, `locations:delete`

---

### Countries

#### Listar Countries
```
GET /api/locations/countries
Query params: page, limit, search
```

#### Criar Country
```
POST /api/locations/countries
Body: {
  "name": "Brazil",
  "code": "BRA"
}
```

#### Atualizar Country
```
PUT /api/locations/countries/:id
Body: {
  "name": "Brazil",
  "code": "BRA"
}
```

#### Deletar Country
```
DELETE /api/locations/countries/:id
```

---

### States

#### Listar States
```
GET /api/locations/states
Query params: page, limit, search, countryId
```

#### Criar State
```
POST /api/locations/states
Body: {
  "countryId": "uuid",
  "name": "São Paulo",
  "code": "SP"
}
```

---

### Cities

#### Listar Cities
```
GET /api/locations/cities
Query params: page, limit, search, stateId
```

#### Criar City
```
POST /api/locations/cities
Body: {
  "stateId": "uuid",
  "name": "São Paulo"
}
```

---

### Sites

#### Listar Sites
```
GET /api/locations/sites
Query params: page, limit, search, cityId
```

#### Criar Site
```
POST /api/locations/sites
Body: {
  "cityId": "uuid",
  "siteTypeId": "uuid",
  "name": "Campus Downtown",
  "code": "DOWNTOWN",
  "address": "123 Main St",
  "bounds": {
    "minLng": -122.4194,
    "minLat": 37.7749,
    "maxLng": -122.4000,
    "maxLat": 37.7900
  },
  "center": {
    "lng": -122.4097,
    "lat": 37.7825
  },
  "defaultZoom": 15
}
```

---

### Buildings

#### Listar Buildings
```
GET /api/locations/buildings
Query params: page, limit, search, siteId
```

#### Criar Building
```
POST /api/locations/buildings
Body: {
  "siteId": "uuid",
  "buildingTypeId": "uuid",
  "name": "Admin Building",
  "code": "ADMIN-001",
  "totalFloors": 5,
  "coordinates": [
    [-122.4100, 37.7800],
    [-122.4090, 37.7800],
    [-122.4090, 37.7790],
    [-122.4100, 37.7790],
    [-122.4100, 37.7800]
  ]
}
```

---

### Floors

#### Listar Floors
```
GET /api/locations/floors
Query params: page, limit, search, buildingId
```

#### Criar Floor
```
POST /api/locations/floors
Body: {
  "buildingId": "uuid",
  "name": "Ground Floor",
  "floorNumber": 0
}
```

---

### Spaces

#### Listar Spaces
```
GET /api/locations/spaces
Query params: page, limit, search, floorId
```

#### Criar Space
```
POST /api/locations/spaces
Body: {
  "floorId": "uuid",
  "spaceTypeId": "uuid",
  "name": "Room 101",
  "code": "A-101",
  "locationType": "point",
  "coordinates": [-122.4095, 37.7795],
  "capacity": 4,
  "area": 25.5,
  "description": "Small office"
}
```

---

## Estrutura de Dados

### Hierarquia

```
Country
  └─ State
      └─ City
          └─ Site
              └─ Building
                  └─ Floor
                      └─ Space
```

---

## Coordenadas Geográficas

### Formato GeoJSON

O frontend usa **GeoJSON** com coordenadas no formato `[longitude, latitude]`.

### Sistema de Coordenadas

- **EPSG:4326** (WGS 84) - Latitude/Longitude
- Longitude: -180 a 180 (Oeste negativo, Leste positivo)
- Latitude: -90 a 90 (Sul negativo, Norte positivo)

### Exemplos de Coordenadas

#### São Paulo, Brasil
```json
{
  "center": {
    "lng": -46.6333,
    "lat": -23.5505
  }
}
```

#### San Francisco, USA
```json
{
  "center": {
    "lng": -122.4194,
    "lat": 37.7749
  }
}
```

#### Lisboa, Portugal
```json
{
  "center": {
    "lng": -9.1393,
    "lat": 38.7223
  }
}
```

---

## Tipos e Catálogos

### Building Types (Sugestões)

```json
[
  {
    "name": "Administrative",
    "icon": "ki-outline ki-building",
    "color": "#FF9955"
  },
  {
    "name": "Library",
    "icon": "ki-outline ki-book",
    "color": "#88CC88"
  },
  {
    "name": "Laboratory",
    "icon": "ki-outline ki-flask",
    "color": "#AA88CC"
  },
  {
    "name": "Residential",
    "icon": "ki-outline ki-home",
    "color": "#3B82F6"
  },
  {
    "name": "Sports Facility",
    "icon": "ki-outline ki-basketball",
    "color": "#F59E0B"
  }
]
```

### Space Types (Sugestões)

```json
[
  {
    "name": "Office",
    "icon": "ki-outline ki-briefcase",
    "color": "#3B82F6"
  },
  {
    "name": "Meeting Room",
    "icon": "ki-outline ki-people",
    "color": "#F59E0B"
  },
  {
    "name": "Laboratory",
    "icon": "ki-outline ki-flask",
    "color": "#8B5CF6"
  },
  {
    "name": "Classroom",
    "icon": "ki-outline ki-book-open",
    "color": "#10B981"
  },
  {
    "name": "Storage",
    "icon": "ki-outline ki-package",
    "color": "#6B7280"
  },
  {
    "name": "Cafeteria",
    "icon": "ki-outline ki-coffee",
    "color": "#EF4444"
  }
]
```

---

## Autenticação e Permissões

### Permissions Necessárias

```typescript
{
  module: 'locations',
  permissions: [
    'locations:view',    // Visualizar
    'locations:create',  // Criar
    'locations:update',  // Editar
    'locations:delete'   // Deletar
  ]
}
```

### Headers de Autenticação

```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
```

---

## Respostas de Erro

### Formato Padrão

```json
{
  "success": false,
  "error": {
    "code": "NOT_FOUND",
    "message": "Building not found",
    "details": {
      "buildingId": "uuid-building-123"
    }
  },
  "timestamp": "2024-12-20T12:00:00Z"
}
```

### Códigos de Erro

- `400 BAD_REQUEST` - Dados inválidos
- `401 UNAUTHORIZED` - Não autenticado
- `403 FORBIDDEN` - Sem permissão
- `404 NOT_FOUND` - Recurso não encontrado
- `409 CONFLICT` - Conflito (ex: código duplicado)
- `500 INTERNAL_ERROR` - Erro do servidor

---

## Notas Importantes

### 1. Performance

- Implementar paginação em todos os endpoints de listagem
- Usar cache para tipos/catálogos (raramente mudam)
- Considerar índices geoespaciais no banco de dados

### 2. Coordenadas

- **SEMPRE** usar formato `[longitude, latitude]` (não latitude, longitude!)
- Validar que coordenadas estejam dentro dos limites válidos
- Para buildings: polígonos devem fechar (primeiro ponto = último ponto)

### 3. Dados Opcionais

Campos que podem ser `null`:
- `code` (em qualquer entidade)
- `description`
- `capacity`, `area` (em spaces)
- `totalFloors` (em buildings)
- `address`

### 4. Contadores

Incluir contadores quando apropriado:
- `buildingCount`, `spaceCount` em Sites
- `floorCount`, `spaceCount` em Buildings
- `spaceCount` em Floors

---

## Checklist de Implementação

### Backend

- [ ] Criar modelo de dados com suporte a coordenadas geográficas
- [ ] Implementar endpoints públicos do mapa
- [ ] Implementar endpoints CRUD de locations
- [ ] Adicionar validação de coordenadas
- [ ] Configurar permissões e guards
- [ ] Adicionar índices geoespaciais
- [ ] Implementar busca (search)
- [ ] Criar seed data com coordenadas de exemplo
- [ ] Documentar API (Swagger/OpenAPI)
- [ ] Testes unitários e de integração

### Dados de Teste

- [ ] Criar pelo menos 2 sites com coordenadas reais
- [ ] Criar pelo menos 5 buildings por site com polígonos
- [ ] Criar pelo menos 20 spaces por building
- [ ] Popular building types e space types
- [ ] Garantir que coordenadas formem polígonos válidos

---

## Ferramentas Úteis

### Para Obter Coordenadas

1. **Google Maps:** Clique direito → "O que há aqui?"
2. **OpenStreetMap:** https://www.openstreetmap.org/
3. **GeoJSON.io:** https://geojson.io/ (desenhar polígonos)
4. **LatLong.net:** https://www.latlong.net/

### Para Validar GeoJSON

- https://geojsonlint.com/
- https://geojson.io/

---

## Exemplo Completo de Site com Buildings

```json
{
  "site": {
    "id": "site-1",
    "name": "Tech Campus",
    "center": { "lng": -46.6333, "lat": -23.5505 },
    "bounds": {
      "minLng": -46.6400,
      "minLat": -23.5550,
      "maxLng": -46.6250,
      "maxLat": -23.5450
    }
  },
  "buildings": [
    {
      "id": "building-1",
      "name": "Main Building",
      "coordinates": [
        [-46.6333, -23.5505],
        [-46.6323, -23.5505],
        [-46.6323, -23.5515],
        [-46.6333, -23.5515],
        [-46.6333, -23.5505]
      ]
    }
  ]
}
```

---

**Versão:** 1.0
**Data:** 2024-12-20
**Autor:** Claude
**Status:** Ready for Implementation
