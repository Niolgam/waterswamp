import { Component, OnInit } from '@angular/core';
import { OrganizationalService } from '../../services/organizational.service';
import {
  Organization,
  SyncSummary,
  SiorgHealthResponse
} from '../../models/organizational.models';

interface SyncOperation {
  id: string;
  type: 'organization' | 'unit' | 'bulk';
  target: string;
  status: 'pending' | 'running' | 'completed' | 'error';
  startedAt?: Date;
  completedAt?: Date;
  summary?: SyncSummary;
  error?: string;
}

@Component({
  selector: 'app-siorg-sync',
  templateUrl: './siorg-sync.component.html',
  styleUrls: ['./siorg-sync.component.scss']
})
export class SiorgSyncComponent implements OnInit {
  organizations: Organization[] = [];
  siorgHealthy = false;
  checkingHealth = false;
  healthError: string | null = null;

  // Sync operations
  syncOperations: SyncOperation[] = [];

  // Form models
  orgSiorgCode: number | null = null;
  unitSiorgCode: number | null = null;
  selectedOrgForBulk: string | null = null;

  constructor(private organizationalService: OrganizationalService) {}

  ngOnInit(): void {
    this.checkSiorgHealth();
    this.loadOrganizations();
    this.loadSyncHistory();
  }

  checkSiorgHealth(): void {
    this.checkingHealth = true;
    this.healthError = null;

    this.organizationalService.checkSiorgHealth()
      .subscribe({
        next: (response: SiorgHealthResponse) => {
          this.siorgHealthy = response.status === 'healthy';
          this.checkingHealth = false;
        },
        error: (err) => {
          this.siorgHealthy = false;
          this.healthError = 'API SIORG indisponível';
          this.checkingHealth = false;
          console.error('Error checking SIORG health:', err);
        }
      });
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

  loadSyncHistory(): void {
    // Load from localStorage (in a real app, this would be from the API)
    const history = localStorage.getItem('siorg_sync_history');
    if (history) {
      this.syncOperations = JSON.parse(history).map((op: any) => ({
        ...op,
        startedAt: op.startedAt ? new Date(op.startedAt) : undefined,
        completedAt: op.completedAt ? new Date(op.completedAt) : undefined
      }));
    }
  }

  saveSyncHistory(): void {
    localStorage.setItem('siorg_sync_history', JSON.stringify(this.syncOperations));
  }

  syncOrganization(): void {
    if (!this.orgSiorgCode) {
      alert('Por favor, informe o código SIORG da organização');
      return;
    }

    const operation: SyncOperation = {
      id: this.generateId(),
      type: 'organization',
      target: `Organização ${this.orgSiorgCode}`,
      status: 'running',
      startedAt: new Date()
    };

    this.syncOperations.unshift(operation);

    this.organizationalService.syncOrganization({ siorg_code: this.orgSiorgCode })
      .subscribe({
        next: (org) => {
          operation.status = 'completed';
          operation.completedAt = new Date();
          this.saveSyncHistory();
          alert(`Organização ${org.name} sincronizada com sucesso!`);
          this.orgSiorgCode = null;
        },
        error: (err) => {
          operation.status = 'error';
          operation.completedAt = new Date();
          operation.error = err.message || 'Erro desconhecido';
          this.saveSyncHistory();
          alert('Erro ao sincronizar organização: ' + (err.error?.message || err.message));
        }
      });
  }

  syncUnit(): void {
    if (!this.unitSiorgCode) {
      alert('Por favor, informe o código SIORG da unidade');
      return;
    }

    const operation: SyncOperation = {
      id: this.generateId(),
      type: 'unit',
      target: `Unidade ${this.unitSiorgCode}`,
      status: 'running',
      startedAt: new Date()
    };

    this.syncOperations.unshift(operation);

    this.organizationalService.syncUnit({ siorg_code: this.unitSiorgCode })
      .subscribe({
        next: (unit) => {
          operation.status = 'completed';
          operation.completedAt = new Date();
          this.saveSyncHistory();
          alert(`Unidade ${unit.name} sincronizada com sucesso!`);
          this.unitSiorgCode = null;
        },
        error: (err) => {
          operation.status = 'error';
          operation.completedAt = new Date();
          operation.error = err.message || 'Erro desconhecido';
          this.saveSyncHistory();
          alert('Erro ao sincronizar unidade: ' + (err.error?.message || err.message));
        }
      });
  }

  syncBulk(): void {
    if (!this.selectedOrgForBulk) {
      alert('Por favor, selecione uma organização');
      return;
    }

    const org = this.organizations.find(o => o.id === this.selectedOrgForBulk);
    if (!org || !org.siorg_code) {
      alert('Organização selecionada não possui código SIORG');
      return;
    }

    if (!confirm(`Deseja sincronizar todas as unidades da organização ${org.name}? Esta operação pode demorar vários minutos.`)) {
      return;
    }

    const operation: SyncOperation = {
      id: this.generateId(),
      type: 'bulk',
      target: org.name,
      status: 'running',
      startedAt: new Date()
    };

    this.syncOperations.unshift(operation);

    this.organizationalService.syncOrganizationUnits({ org_siorg_code: org.siorg_code })
      .subscribe({
        next: (summary: SyncSummary) => {
          operation.status = 'completed';
          operation.completedAt = new Date();
          operation.summary = summary;
          this.saveSyncHistory();

          const message = `
            Sincronização concluída!

            Total processado: ${summary.total_processed}
            Criadas: ${summary.created}
            Atualizadas: ${summary.updated}
            Falhas: ${summary.failed}
          `;

          alert(message);
          this.selectedOrgForBulk = null;
        },
        error: (err) => {
          operation.status = 'error';
          operation.completedAt = new Date();
          operation.error = err.message || 'Erro desconhecido';
          this.saveSyncHistory();
          alert('Erro na sincronização em massa: ' + (err.error?.message || err.message));
        }
      });
  }

  clearHistory(): void {
    if (confirm('Deseja limpar o histórico de sincronizações?')) {
      this.syncOperations = [];
      localStorage.removeItem('siorg_sync_history');
    }
  }

  getStatusClass(status: string): string {
    switch (status) {
      case 'running': return 'status-running';
      case 'completed': return 'status-completed';
      case 'error': return 'status-error';
      default: return 'status-pending';
    }
  }

  getStatusLabel(status: string): string {
    switch (status) {
      case 'running': return 'Em execução';
      case 'completed': return 'Concluída';
      case 'error': return 'Erro';
      default: return 'Pendente';
    }
  }

  getDuration(operation: SyncOperation): string {
    if (!operation.startedAt || !operation.completedAt) {
      return '-';
    }

    const duration = operation.completedAt.getTime() - operation.startedAt.getTime();
    const seconds = Math.floor(duration / 1000);

    if (seconds < 60) {
      return `${seconds}s`;
    }

    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}m ${remainingSeconds}s`;
  }

  private generateId(): string {
    return `sync-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}
