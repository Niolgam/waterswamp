import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { UnitsListComponent } from './components/units-list/units-list.component';
import { UnitsTreeComponent } from './components/units-tree/units-tree.component';
import { SiorgSyncComponent } from './components/siorg-sync/siorg-sync.component';
import { ConflictsListComponent } from './components/conflicts-list/conflicts-list.component';
import { ConflictResolverComponent } from './components/conflict-resolver/conflict-resolver.component';
import { StatsDashboardComponent } from './components/stats-dashboard/stats-dashboard.component';

const routes: Routes = [
  {
    path: '',
    redirectTo: 'units',
    pathMatch: 'full'
  },
  {
    path: 'units',
    component: UnitsListComponent,
    data: { title: 'Unidades Organizacionais' }
  },
  {
    path: 'tree',
    component: UnitsTreeComponent,
    data: { title: 'Árvore Organizacional' }
  },
  {
    path: 'sync',
    component: SiorgSyncComponent,
    data: { title: 'Sincronização SIORG' }
  },
  {
    path: 'conflicts',
    component: ConflictsListComponent,
    data: { title: 'Conflitos de Sincronização' }
  },
  {
    path: 'conflicts/:id',
    component: ConflictResolverComponent,
    data: { title: 'Resolver Conflito' }
  },
  {
    path: 'stats',
    component: StatsDashboardComponent,
    data: { title: 'Estatísticas de Sincronização' }
  }
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule]
})
export class OrganizationalRoutingModule { }
