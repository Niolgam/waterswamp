import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule } from '@angular/forms';
import { HttpClientModule } from '@angular/common/http';

import { OrganizationalRoutingModule } from './organizational-routing.module';

// Components
import { UnitsListComponent } from './components/units-list/units-list.component';
import { UnitsTreeComponent } from './components/units-tree/units-tree.component';
import { SiorgSyncComponent } from './components/siorg-sync/siorg-sync.component';
import { ConflictsListComponent } from './components/conflicts-list/conflicts-list.component';
import { ConflictResolverComponent } from './components/conflict-resolver/conflict-resolver.component';

// Services
import { OrganizationalService } from './services/organizational.service';
import { SyncService } from './services/sync.service';

@NgModule({
  declarations: [
    UnitsListComponent,
    UnitsTreeComponent,
    SiorgSyncComponent,
    ConflictsListComponent,
    ConflictResolverComponent
  ],
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    HttpClientModule,
    OrganizationalRoutingModule
  ],
  providers: [
    OrganizationalService,
    SyncService
  ]
})
export class OrganizationalModule { }
