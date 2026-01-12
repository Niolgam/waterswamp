import { Component, OnInit, OnDestroy } from '@angular/core';
import { SyncService } from '../../services/sync.service';
import { DetailedStats, HealthStatus } from '../../models/sync.models';
import { interval, Subscription } from 'rxjs';
import { switchMap, startWith } from 'rxjs/operators';

@Component({
  selector: 'app-stats-dashboard',
  templateUrl: './stats-dashboard.component.html',
  styleUrls: ['./stats-dashboard.component.scss']
})
export class StatsDashboardComponent implements OnInit, OnDestroy {
  stats: DetailedStats | null = null;
  health: HealthStatus | null = null;
  loading = false;
  error: string | null = null;
  autoRefresh = true;
  refreshInterval = 30000; // 30 seconds

  private refreshSubscription?: Subscription;

  constructor(private syncService: SyncService) {}

  ngOnInit(): void {
    this.loadData();
    this.startAutoRefresh();
  }

  ngOnDestroy(): void {
    this.stopAutoRefresh();
  }

  loadData(): void {
    this.loading = true;
    this.error = null;

    // Load both stats and health in parallel
    Promise.all([
      this.syncService.getDetailedStats().toPromise(),
      this.syncService.getHealthStatus().toPromise()
    ])
      .then(([stats, health]) => {
        this.stats = stats || null;
        this.health = health || null;
        this.loading = false;
      })
      .catch(err => {
        this.error = 'Erro ao carregar estatísticas: ' + (err.error?.message || err.message);
        this.loading = false;
        console.error('Error loading stats:', err);
      });
  }

  startAutoRefresh(): void {
    if (this.autoRefresh && !this.refreshSubscription) {
      this.refreshSubscription = interval(this.refreshInterval)
        .pipe(
          startWith(0),
          switchMap(() => this.syncService.getDetailedStats())
        )
        .subscribe({
          next: (stats) => {
            this.stats = stats;
          },
          error: (err) => {
            console.error('Auto-refresh error:', err);
          }
        });
    }
  }

  stopAutoRefresh(): void {
    if (this.refreshSubscription) {
      this.refreshSubscription.unsubscribe();
      this.refreshSubscription = undefined;
    }
  }

  toggleAutoRefresh(): void {
    this.autoRefresh = !this.autoRefresh;
    if (this.autoRefresh) {
      this.startAutoRefresh();
    } else {
      this.stopAutoRefresh();
    }
  }

  refresh(): void {
    this.loadData();
  }

  getHealthStatusClass(): string {
    if (!this.health) return '';

    switch (this.health.status) {
      case 'healthy':
        return 'status-healthy';
      case 'processing':
        return 'status-processing';
      case 'warning':
        return 'status-warning';
      case 'critical':
        return 'status-critical';
      default:
        return '';
    }
  }

  getHealthStatusIcon(): string {
    if (!this.health) return '❓';

    switch (this.health.status) {
      case 'healthy':
        return '✅';
      case 'processing':
        return '⚙️';
      case 'warning':
        return '⚠️';
      case 'critical':
        return '🚨';
      default:
        return '❓';
    }
  }

  formatPercentage(value: number): string {
    return value.toFixed(1) + '%';
  }

  formatDuration(ms: number): string {
    if (ms < 1000) {
      return ms.toFixed(0) + 'ms';
    }
    return (ms / 1000).toFixed(2) + 's';
  }

  formatHours(hours: number): string {
    if (hours < 1) {
      return (hours * 60).toFixed(0) + 'min';
    }
    if (hours < 24) {
      return hours.toFixed(1) + 'h';
    }
    return (hours / 24).toFixed(1) + 'd';
  }

  get queueStats() {
    return this.stats?.queue;
  }

  get processingStats() {
    return this.stats?.processing;
  }

  get conflictStats() {
    return this.stats?.conflicts;
  }

  get historyStats() {
    return this.stats?.history;
  }

  get entityTypeEntries() {
    return this.conflictStats?.by_entity_type ?
      Object.entries(this.conflictStats.by_entity_type) : [];
  }

  get changeTypeEntries() {
    return this.historyStats?.by_change_type ?
      Object.entries(this.historyStats.by_change_type) : [];
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

  getChangeTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      'CREATE': 'Criação',
      'UPDATE': 'Atualização',
      'DELETE': 'Exclusão',
      'CONFLICT': 'Conflito'
    };
    return labels[type] || type;
  }
}
