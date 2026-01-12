# Waterswamp Web UI - Módulo Organizacional

Interface Angular para gestão de unidades organizacionais e sincronização com SIORG.

## 📋 Visão Geral

Este módulo fornece uma interface completa para gerenciar estruturas organizacionais hierárquicas com sincronização bidirecional com o sistema SIORG do governo brasileiro.

## 🎯 Funcionalidades

### 1. **Lista de Unidades Organizacionais** (`/organizational/units`)
- Listagem paginada com filtros avançados
- Filtros por:
  - Organização
  - Área de atuação (Meio/Fim)
  - Tipo interno
  - Status (Ativa/Inativa)
  - Gerenciamento SIORG
  - Busca por nome
- Ações rápidas:
  - Visualizar detalhes
  - Ativar/Desativar unidades
  - Navegação para árvore e sincronização

### 2. **Árvore Organizacional** (`/organizational/tree`)
- Visualização hierárquica interativa
- Expandir/Recolher nós
- Filtro por organização
- Indicadores visuais:
  - Unidades gerenciadas pelo SIORG (verde)
  - Unidades locais (cinza)
  - Unidades inativas (vermelho)
- Navegação para detalhes de cada unidade

### 3. **Sincronização SIORG** (`/organizational/sync`)
- Verificação de saúde da API SIORG
- Três tipos de sincronização:
  - **Organização Individual**: Sincroniza uma organização por código SIORG
  - **Unidade Individual**: Sincroniza uma unidade por código SIORG
  - **Sincronização em Massa**: Sincroniza todas as unidades de uma organização
- Histórico de sincronizações com:
  - Status (Em execução/Concluída/Erro)
  - Duração da operação
  - Estatísticas detalhadas (criadas, atualizadas, falhas)
  - Mensagens de erro quando aplicável

## 🏗️ Estrutura do Código

```
apps/web-ui/src/app/modules/organizational/
├── models/
│   └── organizational.models.ts      # Interfaces e tipos TypeScript
├── services/
│   └── organizational.service.ts      # Serviço de comunicação com API
├── components/
│   ├── units-list/                    # Componente de listagem
│   │   ├── units-list.component.ts
│   │   ├── units-list.component.html
│   │   └── units-list.component.scss
│   ├── units-tree/                    # Componente de árvore
│   │   ├── units-tree.component.ts
│   │   ├── units-tree.component.html
│   │   └── units-tree.component.scss
│   └── siorg-sync/                    # Componente de sincronização
│       ├── siorg-sync.component.ts
│       ├── siorg-sync.component.html
│       └── siorg-sync.component.scss
├── organizational-routing.module.ts   # Configuração de rotas
└── organizational.module.ts           # Módulo Angular
```

## 🔌 Integração com API

### Endpoints Utilizados

**System Settings**
- `GET /api/admin/organizational/settings` - Listar configurações
- `GET /api/admin/organizational/settings/{key}` - Obter configuração
- `POST /api/admin/organizational/settings` - Criar configuração
- `PUT /api/admin/organizational/settings/{key}` - Atualizar configuração
- `DELETE /api/admin/organizational/settings/{key}` - Deletar configuração

**Organizations**
- `GET /api/admin/organizational/organizations` - Listar organizações
- `GET /api/admin/organizational/organizations/{id}` - Obter organização
- `POST /api/admin/organizational/organizations` - Criar organização
- `PUT /api/admin/organizational/organizations/{id}` - Atualizar organização
- `DELETE /api/admin/organizational/organizations/{id}` - Deletar organização

**Organizational Units**
- `GET /api/admin/organizational/units` - Listar unidades (com filtros)
- `GET /api/admin/organizational/units/tree` - Obter árvore hierárquica
- `GET /api/admin/organizational/units/{id}` - Obter unidade com detalhes
- `GET /api/admin/organizational/units/{id}/children` - Obter filhos diretos
- `GET /api/admin/organizational/units/{id}/path` - Obter caminho até raiz
- `POST /api/admin/organizational/units` - Criar unidade
- `PUT /api/admin/organizational/units/{id}` - Atualizar unidade
- `DELETE /api/admin/organizational/units/{id}` - Deletar unidade
- `POST /api/admin/organizational/units/{id}/deactivate` - Desativar unidade
- `POST /api/admin/organizational/units/{id}/activate` - Ativar unidade

**SIORG Sync**
- `POST /api/admin/organizational/sync/organization` - Sincronizar organização
- `POST /api/admin/organizational/sync/unit` - Sincronizar unidade
- `POST /api/admin/organizational/sync/org-units` - Sincronização em massa
- `GET /api/admin/organizational/sync/health` - Verificar saúde da API

## ⚙️ Configuração

### Variáveis de Ambiente

Edite `src/environments/environment.ts`:

```typescript
export const environment = {
  production: false,
  apiUrl: 'http://localhost:3000',  // URL da API backend
  siorgApiUrl: 'https://api.siorg.gov.br'  // URL da API SIORG
};
```

### Autenticação

O serviço utiliza `HttpClient` do Angular que deve ser configurado com interceptors para adicionar o token JWT:

```typescript
// app.module.ts
import { HTTP_INTERCEPTORS } from '@angular/common/http';
import { AuthInterceptor } from './interceptors/auth.interceptor';

providers: [
  {
    provide: HTTP_INTERCEPTORS,
    useClass: AuthInterceptor,
    multi: true
  }
]
```

## 🚀 Como Usar

### 1. Instalação

```bash
cd apps/web-ui
npm install
```

### 2. Desenvolvimento

```bash
npm start
# ou
ng serve
```

Acesse: `http://localhost:4200`

### 3. Build de Produção

```bash
npm run build
# ou
ng build --configuration production
```

### 4. Integração no App Principal

No módulo raiz da aplicação:

```typescript
// app-routing.module.ts
const routes: Routes = [
  {
    path: 'organizational',
    loadChildren: () => import('./modules/organizational/organizational.module')
      .then(m => m.OrganizationalModule),
    canActivate: [AuthGuard]  // Proteção de rota
  }
];
```

## 📱 Responsividade

A interface é totalmente responsiva e otimizada para:
- Desktop (1920px+)
- Tablet (768px - 1920px)
- Mobile (< 768px)

## 🎨 Personalização

### Temas

Os componentes usam variáveis CSS que podem ser customizadas:

```scss
// styles.scss
:root {
  --primary-color: #4CAF50;
  --secondary-color: #f5f5f5;
  --error-color: #f44336;
  --success-color: #2e7d32;
  --warning-color: #f57c00;
}
```

### Ícones

O projeto utiliza ícones genéricos. Integre com sua biblioteca de ícones preferida (Font Awesome, Material Icons, etc.):

```html
<!-- Substitua classes como 'icon-sync' por: -->
<i class="fas fa-sync-alt"></i>  <!-- Font Awesome -->
<mat-icon>sync</mat-icon>         <!-- Material Icons -->
```

## 🧪 Testes

### Testes Unitários

```bash
npm test
# ou
ng test
```

### Testes E2E

```bash
npm run e2e
# ou
ng e2e
```

## 📊 Métricas e Performance

- **Lazy Loading**: Módulo carregado sob demanda
- **Change Detection**: OnPush onde aplicável
- **Virtual Scrolling**: Para listas grandes (implementar conforme necessário)
- **Service Workers**: PWA support (opcional)

## 🔒 Segurança

- Todas as rotas protegidas por `AuthGuard`
- Tokens JWT em todas as requisições
- Sanitização de inputs
- CORS configurado no backend

## 📝 TODO / Melhorias Futuras

- [ ] Componente de detalhes de unidade
- [ ] Editor de contatos inline
- [ ] Drag & drop para reorganizar hierarquia
- [ ] Exportação para Excel/PDF
- [ ] Notificações em tempo real (WebSocket)
- [ ] Modo escuro
- [ ] Internacionalização (i18n)
- [ ] Undo/Redo para operações
- [ ] Comparação de versões (antes/depois da sincronização)

## 🐛 Troubleshooting

### Erro CORS
```
Configurar proxy no angular.json:
{
  "/api": {
    "target": "http://localhost:3000",
    "secure": false
  }
}
```

### Erro de Autenticação
```
Verificar se o token JWT está sendo enviado no header:
Authorization: Bearer <token>
```

### Sincronização Falha
```
1. Verificar saúde da API SIORG
2. Checar logs do backend
3. Validar códigos SIORG
```

## 📄 Licença

Este projeto faz parte do Waterswamp e segue a mesma licença do projeto principal.

## 👥 Contribuindo

1. Fork o repositório
2. Crie uma branch para sua feature (`git checkout -b feature/MinhaFeature`)
3. Commit suas mudanças (`git commit -m 'Add: Minha feature'`)
4. Push para a branch (`git push origin feature/MinhaFeature`)
5. Abra um Pull Request

## 📞 Suporte

Para problemas ou dúvidas:
- Abra uma issue no GitHub
- Contate a equipe de desenvolvimento
- Consulte a documentação da API em `/swagger`
