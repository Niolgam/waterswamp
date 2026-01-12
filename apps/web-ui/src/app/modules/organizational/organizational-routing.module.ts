import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';
import { UnitsListComponent } from './components/units-list/units-list.component';
import { UnitsTreeComponent } from './components/units-tree/units-tree.component';
import { SiorgSyncComponent } from './components/siorg-sync/siorg-sync.component';

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
  }
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule]
})
export class OrganizationalRoutingModule { }
