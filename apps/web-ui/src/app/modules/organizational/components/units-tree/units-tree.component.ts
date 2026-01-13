import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { OrganizationalService } from '../../services/organizational.service';
import {
  OrganizationalUnitTreeNode,
  Organization
} from '../../models/organizational.models';

@Component({
  selector: 'app-units-tree',
  templateUrl: './units-tree.component.html',
  styleUrls: ['./units-tree.component.scss']
})
export class UnitsTreeComponent implements OnInit {
  treeData: OrganizationalUnitTreeNode[] = [];
  organizations: Organization[] = [];
  selectedOrganization: string | null = null;
  loading = false;
  error: string | null = null;
  expandedNodes: Set<string> = new Set();

  constructor(
    private organizationalService: OrganizationalService,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.loadOrganizations();
    this.loadTree();
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

  loadTree(): void {
    this.loading = true;
    this.error = null;

    const params: any = {};
    if (this.selectedOrganization) {
      params.organization_id = this.selectedOrganization;
    }

    this.organizationalService.getOrganizationalUnitsTree(params)
      .subscribe({
        next: (tree) => {
          this.treeData = tree;
          this.loading = false;

          // Auto-expand first level
          tree.forEach(node => {
            this.expandedNodes.add(node.unit.id);
          });
        },
        error: (err) => {
          this.error = 'Erro ao carregar árvore organizacional';
          this.loading = false;
          console.error('Error loading tree:', err);
        }
      });
  }

  onOrganizationChange(): void {
    this.expandedNodes.clear();
    this.loadTree();
  }

  toggleNode(nodeId: string): void {
    if (this.expandedNodes.has(nodeId)) {
      this.expandedNodes.delete(nodeId);
    } else {
      this.expandedNodes.add(nodeId);
    }
  }

  isExpanded(nodeId: string): boolean {
    return this.expandedNodes.has(nodeId);
  }

  expandAll(): void {
    this.expandedNodes.clear();
    this.addAllNodesToExpanded(this.treeData);
  }

  collapseAll(): void {
    this.expandedNodes.clear();
  }

  private addAllNodesToExpanded(nodes: OrganizationalUnitTreeNode[]): void {
    nodes.forEach(node => {
      this.expandedNodes.add(node.unit.id);
      if (node.children && node.children.length > 0) {
        this.addAllNodesToExpanded(node.children);
      }
    });
  }

  viewUnitDetails(unitId: string): void {
    this.router.navigate(['/organizational/units', unitId]);
  }

  getNodeClass(node: OrganizationalUnitTreeNode): string {
    const classes = ['tree-node'];
    if (!node.unit.is_active) classes.push('inactive');
    if (node.unit.siorg_code) classes.push('siorg-managed');
    return classes.join(' ');
  }
}
