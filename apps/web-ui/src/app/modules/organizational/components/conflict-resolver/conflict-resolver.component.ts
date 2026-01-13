import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { SyncService } from '../../services/sync.service';
import {
  ConflictDetail,
  ConflictDiff,
  ResolutionAction,
  FieldResolution,
  ResolveConflictPayload
} from '../../models/sync.models';

@Component({
  selector: 'app-conflict-resolver',
  templateUrl: './conflict-resolver.component.html',
  styleUrls: ['./conflict-resolver.component.scss']
})
export class ConflictResolverComponent implements OnInit {
  conflictId: string | null = null;
  conflict: ConflictDetail | null = null;
  loading = false;
  resolving = false;
  error: string | null = null;

  // Resolution state
  selectedAction: ResolutionAction | null = null;
  fieldChoices: Map<string, FieldResolution> = new Map();
  resolutionNotes = '';

  // UI state
  expandedFields: Set<string> = new Set();

  constructor(
    private route: ActivatedRoute,
    private router: Router,
    private syncService: SyncService
  ) {}

  ngOnInit(): void {
    this.conflictId = this.route.snapshot.paramMap.get('id');
    if (this.conflictId) {
      this.loadConflictDetail();
    }
  }

  loadConflictDetail(): void {
    if (!this.conflictId) return;

    this.loading = true;
    this.error = null;

    this.syncService.getConflictDetail(this.conflictId).subscribe({
      next: (detail) => {
        this.conflict = detail;
        this.loading = false;
        // Auto-expand all conflicting fields
        detail.fields.filter(f => f.has_conflict).forEach(f => {
          this.expandedFields.add(f.field);
        });
      },
      error: (err) => {
        this.error = 'Erro ao carregar conflito: ' + (err.error?.message || err.message);
        this.loading = false;
        console.error('Error loading conflict detail:', err);
      }
    });
  }

  selectAction(action: ResolutionAction): void {
    this.selectedAction = action;

    // Clear field choices if not using MERGE
    if (action !== 'MERGE') {
      this.fieldChoices.clear();
    }
  }

  chooseField(fieldName: string, choice: FieldResolution): void {
    this.fieldChoices.set(fieldName, choice);
  }

  getFieldChoice(fieldName: string): FieldResolution | null {
    return this.fieldChoices.get(fieldName) || null;
  }

  toggleField(fieldName: string): void {
    if (this.expandedFields.has(fieldName)) {
      this.expandedFields.delete(fieldName);
    } else {
      this.expandedFields.add(fieldName);
    }
  }

  isFieldExpanded(fieldName: string): boolean {
    return this.expandedFields.has(fieldName);
  }

  canResolve(): boolean {
    if (!this.selectedAction) return false;

    // For MERGE, all conflicting fields must have a choice
    if (this.selectedAction === 'MERGE') {
      const conflictingFields = this.conflict?.fields.filter(f => f.has_conflict) || [];
      return conflictingFields.every(f => this.fieldChoices.has(f.field));
    }

    return true;
  }

  resolve(): void {
    if (!this.canResolve() || !this.conflictId || !this.selectedAction) {
      return;
    }

    const payload: ResolveConflictPayload = {
      action: this.selectedAction,
      notes: this.resolutionNotes || undefined
    };

    // Add field resolutions for MERGE action
    if (this.selectedAction === 'MERGE') {
      payload.field_resolutions = Object.fromEntries(this.fieldChoices.entries());
    }

    this.resolving = true;
    this.error = null;

    this.syncService.resolveConflict(this.conflictId, payload).subscribe({
      next: () => {
        alert('Conflito resolvido com sucesso!');
        this.router.navigate(['/organizational/conflicts']);
      },
      error: (err) => {
        this.error = 'Erro ao resolver conflito: ' + (err.error?.message || err.message);
        this.resolving = false;
        console.error('Error resolving conflict:', err);
      }
    });
  }

  cancel(): void {
    if (confirm('Deseja cancelar sem resolver o conflito?')) {
      this.router.navigate(['/organizational/conflicts']);
    }
  }

  formatValue(value: any): string {
    if (value === null || value === undefined) {
      return 'N/A';
    }
    if (typeof value === 'boolean') {
      return value ? 'Sim' : 'Não';
    }
    if (typeof value === 'object') {
      return JSON.stringify(value, null, 2);
    }
    return String(value);
  }

  getFieldLabel(fieldName: string): string {
    const labels: Record<string, string> = {
      'name': 'Nome',
      'formal_name': 'Nome Formal',
      'acronym': 'Sigla',
      'cnpj': 'CNPJ',
      'ug_code': 'Código UG',
      'is_active': 'Ativo',
      'parent': 'Unidade Pai'
    };
    return labels[fieldName] || fieldName;
  }

  getActionLabel(action: ResolutionAction): string {
    const labels: Record<ResolutionAction, string> = {
      'ACCEPT_SIORG': 'Aceitar SIORG',
      'KEEP_LOCAL': 'Manter Local',
      'MERGE': 'Mesclar Campos',
      'SKIP': 'Ignorar'
    };
    return labels[action];
  }

  getActionDescription(action: ResolutionAction): string {
    const descriptions: Record<ResolutionAction, string> = {
      'ACCEPT_SIORG': 'Sobrescrever dados locais com os dados do SIORG',
      'KEEP_LOCAL': 'Manter dados locais e ignorar mudanças do SIORG',
      'MERGE': 'Escolher campo por campo qual valor manter',
      'SKIP': 'Adiar decisão para depois'
    };
    return descriptions[action];
  }

  get conflictingFieldsCount(): number {
    return this.conflict?.fields.filter(f => f.has_conflict).length || 0;
  }

  get nonConflictingFieldsCount(): number {
    return this.conflict?.fields.filter(f => !f.has_conflict).length || 0;
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
}
