import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { OrganizationalService } from '../../services/organizational.service';
import {
  OrganizationalUnitWithDetails,
  Organization,
  ActivityArea,
  InternalUnitType
} from '../../models/organizational.models';

@Component({
  selector: 'app-units-list',
  templateUrl: './units-list.component.html',
  styleUrls: ['./units-list.component.scss']
})
export class UnitsListComponent implements OnInit {
  units: OrganizationalUnitWithDetails[] = [];
  organizations: Organization[] = [];
  loading = false;
  error: string | null = null;

  // Filters
  selectedOrganization: string | null = null;
  selectedActivityArea: string | null = null;
  selectedInternalType: string | null = null;
  searchTerm = '';
  isActiveFilter: boolean | null = true;
  isSiorgManagedFilter: boolean | null = null;

  // Pagination
  total = 0;
  limit = 50;
  offset = 0;
  currentPage = 1;

  // Enums for dropdowns
  activityAreas = Object.values(ActivityArea);
  internalTypes = Object.values(InternalUnitType);

  constructor(
    private organizationalService: OrganizationalService,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.loadOrganizations();
    this.loadUnits();
  }

  loadOrganizations(): void {
    this.organizationalService.listOrganizations({ is_active: true })
      .subscribe({
        next: (response) => {
          this.organizations = response.organizations;
        },
        error: (err) => {
          console.error('Error loading organizations:', err);
        }
      });
  }

  loadUnits(): void {
    this.loading = true;
    this.error = null;

    const params: any = {
      limit: this.limit,
      offset: this.offset
    };

    if (this.selectedOrganization) params.organization_id = this.selectedOrganization;
    if (this.selectedActivityArea) params.activity_area = this.selectedActivityArea;
    if (this.selectedInternalType) params.internal_type = this.selectedInternalType;
    if (this.searchTerm) params.search = this.searchTerm;
    if (this.isActiveFilter !== null) params.is_active = this.isActiveFilter;
    if (this.isSiorgManagedFilter !== null) params.is_siorg_managed = this.isSiorgManagedFilter;

    this.organizationalService.listOrganizationalUnits(params)
      .subscribe({
        next: (response) => {
          this.units = response.units;
          this.total = response.total;
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Erro ao carregar unidades organizacionais';
          this.loading = false;
          console.error('Error loading units:', err);
        }
      });
  }

  applyFilters(): void {
    this.offset = 0;
    this.currentPage = 1;
    this.loadUnits();
  }

  clearFilters(): void {
    this.selectedOrganization = null;
    this.selectedActivityArea = null;
    this.selectedInternalType = null;
    this.searchTerm = '';
    this.isActiveFilter = true;
    this.isSiorgManagedFilter = null;
    this.applyFilters();
  }

  onPageChange(page: number): void {
    this.currentPage = page;
    this.offset = (page - 1) * this.limit;
    this.loadUnits();
  }

  viewDetails(unitId: string): void {
    this.router.navigate(['/organizational/units', unitId]);
  }

  viewTree(): void {
    this.router.navigate(['/organizational/tree']);
  }

  syncWithSiorg(): void {
    this.router.navigate(['/organizational/sync']);
  }

  deactivateUnit(unitId: string): void {
    if (confirm('Tem certeza que deseja desativar esta unidade?')) {
      const reason = prompt('Motivo da desativação (opcional):');
      this.organizationalService.deactivateOrganizationalUnit(unitId, reason || undefined)
        .subscribe({
          next: () => {
            this.loadUnits();
          },
          error: (err) => {
            alert('Erro ao desativar unidade: ' + err.message);
          }
        });
    }
  }

  activateUnit(unitId: string): void {
    if (confirm('Tem certeza que deseja ativar esta unidade?')) {
      this.organizationalService.activateOrganizationalUnit(unitId)
        .subscribe({
          next: () => {
            this.loadUnits();
          },
          error: (err) => {
            alert('Erro ao ativar unidade: ' + err.message);
          }
        });
    }
  }

  getTotalPages(): number {
    return Math.ceil(this.total / this.limit);
  }

  getActivityAreaLabel(area: ActivityArea): string {
    return area === ActivityArea.Support ? 'Área Meio' : 'Área Fim';
  }

  getInternalTypeLabel(type: InternalUnitType): string {
    const labels: { [key in InternalUnitType]: string } = {
      [InternalUnitType.Administration]: 'Administração',
      [InternalUnitType.Department]: 'Departamento',
      [InternalUnitType.Laboratory]: 'Laboratório',
      [InternalUnitType.Sector]: 'Setor',
      [InternalUnitType.Council]: 'Conselho',
      [InternalUnitType.Coordination]: 'Coordenação',
      [InternalUnitType.Center]: 'Centro',
      [InternalUnitType.Division]: 'Divisão'
    };
    return labels[type];
  }
}
