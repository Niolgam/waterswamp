import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { SyncService } from '../../services/sync.service';
import { SiorgSyncQueueItem, QueueStatsResponse } from '../../models/sync.models';

@Component({
  selector: 'app-conflicts-list',
  templateUrl: './conflicts-list.component.html',
  styleUrls: ['./conflicts-list.component.scss']
})
export class ConflictsListComponent implements OnInit {
  conflicts: SiorgSyncQueueItem[] = [];
  stats: QueueStatsResponse | null = null;
  loading = false;
  error: string | null = null;

  // Pagination
  limit = 20;
  offset = 0;
  total = 0;

  constructor(
    private syncService: SyncService,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.loadStats();
    this.loadConflicts();
  }

  loadStats(): void {
    this.syncService.getQueueStats().subscribe({
      next: (stats) => {
        this.stats = stats;
        this.total = stats.conflicts;
      },
      error: (err) => {
        console.error('Error loading stats:', err);
      }
    });
  }

  loadConflicts(): void {
    this.loading = true;
    this.error = null;

    this.syncService.listConflicts({
      limit: this.limit,
      offset: this.offset
    }).subscribe({
      next: (conflicts) => {
        this.conflicts = conflicts;
        this.loading = false;
      },
      error: (err) => {
        this.error = 'Erro ao carregar conflitos: ' + (err.error?.message || err.message);
        this.loading = false;
        console.error('Error loading conflicts:', err);
      }
    });
  }

  viewConflictDetail(conflict: SiorgSyncQueueItem): void {
    this.router.navigate(['/organizational/conflicts', conflict.id]);
  }

  deleteConflict(conflict: SiorgSyncQueueItem, event: Event): void {
    event.stopPropagation();

    if (!confirm(`Tem certeza que deseja remover este conflito da fila?`)) {
      return;
    }

    this.syncService.deleteQueueItem(conflict.id).subscribe({
      next: () => {
        this.loadConflicts();
        this.loadStats();
      },
      error: (err) => {
        alert('Erro ao remover conflito: ' + (err.error?.message || err.message));
        console.error('Error deleting conflict:', err);
      }
    });
  }

  getEntityTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      'ORGANIZATION': 'Organização',
      'UNIT': 'Unidade',
      'CATEGORY': 'Categoria',
      'TYPE': 'Tipo'
    };
    return labels[type] || type;
  }

  getOperationLabel(operation: string): string {
    const labels: Record<string, string> = {
      'CREATION': 'Criação',
      'UPDATE': 'Atualização',
      'EXTINCTION': 'Extinção',
      'HIERARCHY_CHANGE': 'Mudança de Hierarquia',
      'MERGE': 'Fusão',
      'SPLIT': 'Divisão'
    };
    return labels[operation] || operation;
  }

  formatDate(date?: string): string {
    if (!date) return 'N/A';
    return new Date(date).toLocaleString('pt-BR');
  }

  nextPage(): void {
    if (this.offset + this.limit < this.total) {
      this.offset += this.limit;
      this.loadConflicts();
    }
  }

  previousPage(): void {
    if (this.offset > 0) {
      this.offset = Math.max(0, this.offset - this.limit);
      this.loadConflicts();
    }
  }

  get currentPage(): number {
    return Math.floor(this.offset / this.limit) + 1;
  }

  get totalPages(): number {
    return Math.ceil(this.total / this.limit);
  }

  get hasNextPage(): boolean {
    return this.offset + this.limit < this.total;
  }

  get hasPreviousPage(): boolean {
    return this.offset > 0;
  }
}
