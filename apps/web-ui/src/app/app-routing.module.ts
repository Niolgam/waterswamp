import { NgModule } from '@angular/core';
import { RouterModule, Routes } from '@angular/router';

const routes: Routes = [
  {
    path: '',
    redirectTo: '/organizational/units',
    pathMatch: 'full'
  },
  {
    path: 'organizational',
    loadChildren: () => import('./modules/organizational/organizational.module')
      .then(m => m.OrganizationalModule)
  },
  {
    path: '**',
    redirectTo: '/organizational/units'
  }
];

@NgModule({
  imports: [RouterModule.forRoot(routes)],
  exports: [RouterModule]
})
export class AppRoutingModule { }
