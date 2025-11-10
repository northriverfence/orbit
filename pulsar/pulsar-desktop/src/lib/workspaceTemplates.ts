/**
 * Workspace Templates
 *
 * Pre-built workspace layouts for common development scenarios
 */

import type { WorkspaceLayout, PaneConfig } from '../types/workspace';

export interface WorkspaceTemplate {
  id: string;
  name: string;
  description: string;
  icon: string;
  tags: string[];
  layout: WorkspaceLayout;
  preview: string; // ASCII art representation
}

/**
 * Single Pane - Simple, focused workspace
 */
export const singlePaneTemplate: WorkspaceTemplate = {
  id: 'single-pane',
  name: 'Single Pane',
  description: 'One terminal window for focused work',
  icon: '□',
  tags: ['simple', 'minimal'],
  preview: '┌─────────────┐\n│             │\n│   Terminal  │\n│             │\n└─────────────┘',
  layout: {
    version: '1.0.0',
    type: 'single',
    panes: [
      {
        id: crypto.randomUUID(),
        size: 100,
        min_size: 100,
      },
    ],
  },
};

/**
 * Horizontal Split - Top/bottom layout
 */
export const horizontalSplitTemplate: WorkspaceTemplate = {
  id: 'horizontal-split',
  name: 'Horizontal Split',
  description: 'Two terminals stacked vertically',
  icon: '⬒',
  tags: ['split', 'dual'],
  preview: '┌─────────────┐\n│  Terminal 1 │\n├─────────────┤\n│  Terminal 2 │\n└─────────────┘',
  layout: {
    version: '1.0.0',
    type: 'split',
    panes: [
      {
        id: crypto.randomUUID(),
        size: 50,
        direction: 'horizontal',
        min_size: 20,
      },
      {
        id: crypto.randomUUID(),
        size: 50,
        direction: 'horizontal',
        min_size: 20,
      },
    ],
  },
};

/**
 * Vertical Split - Side-by-side layout
 */
export const verticalSplitTemplate: WorkspaceTemplate = {
  id: 'vertical-split',
  name: 'Vertical Split',
  description: 'Two terminals side by side',
  icon: '⬓',
  tags: ['split', 'dual'],
  preview: '┌──────┬──────┐\n│      │      │\n│  T1  │  T2  │\n│      │      │\n└──────┴──────┘',
  layout: {
    version: '1.0.0',
    type: 'split',
    panes: [
      {
        id: crypto.randomUUID(),
        size: 50,
        direction: 'vertical',
        min_size: 20,
      },
      {
        id: crypto.randomUUID(),
        size: 50,
        direction: 'vertical',
        min_size: 20,
      },
    ],
  },
};

/**
 * DevOps - Monitoring and deployment
 */
export const devOpsTemplate: WorkspaceTemplate = {
  id: 'devops',
  name: 'DevOps Dashboard',
  description: 'Perfect for monitoring logs, deployments, and system metrics',
  icon: '🚀',
  tags: ['devops', 'monitoring', 'deployment'],
  preview: '┌──────────────┬────┐\n│              │ T2 │\n│   Main (T1)  ├────┤\n│              │ T3 │\n└──────────────┴────┘',
  layout: {
    version: '1.0.0',
    type: 'split',
    panes: [
      {
        id: crypto.randomUUID(),
        size: 70,
        direction: 'vertical',
        min_size: 50,
      },
      {
        id: crypto.randomUUID(),
        size: 30,
        direction: 'vertical',
        min_size: 20,
        children: [
          {
            id: crypto.randomUUID(),
            size: 50,
            direction: 'horizontal',
            min_size: 20,
          },
          {
            id: crypto.randomUUID(),
            size: 50,
            direction: 'horizontal',
            min_size: 20,
          },
        ],
      },
    ],
  },
};

/**
 * Full-Stack - Frontend, backend, and database
 */
export const fullStackTemplate: WorkspaceTemplate = {
  id: 'fullstack',
  name: 'Full-Stack Development',
  description: 'Three-way split for frontend, backend, and database work',
  icon: '💻',
  tags: ['fullstack', 'development', 'web'],
  preview: '┌─────┬─────┬─────┐\n│     │     │     │\n│ FE  │ BE  │ DB  │\n│     │     │     │\n└─────┴─────┴─────┘',
  layout: {
    version: '1.0.0',
    type: 'split',
    panes: [
      {
        id: crypto.randomUUID(),
        size: 33.33,
        direction: 'vertical',
        min_size: 20,
      },
      {
        id: crypto.randomUUID(),
        size: 33.33,
        direction: 'vertical',
        min_size: 20,
      },
      {
        id: crypto.randomUUID(),
        size: 33.34,
        direction: 'vertical',
        min_size: 20,
      },
    ],
  },
};

/**
 * Code Review - Main editor with context panes
 */
export const codeReviewTemplate: WorkspaceTemplate = {
  id: 'code-review',
  name: 'Code Review',
  description: 'Main editor with side panels for tests and documentation',
  icon: '🔍',
  tags: ['review', 'development'],
  preview: '┌─────────────┬────┐\n│             │ T2 │\n│   Main (T1) │────│\n│             │ T3 │\n└─────────────┴────┘',
  layout: {
    version: '1.0.0',
    type: 'split',
    panes: [
      {
        id: crypto.randomUUID(),
        size: 65,
        direction: 'vertical',
        min_size: 50,
      },
      {
        id: crypto.randomUUID(),
        size: 35,
        direction: 'vertical',
        min_size: 20,
        children: [
          {
            id: crypto.randomUUID(),
            size: 50,
            direction: 'horizontal',
            min_size: 20,
          },
          {
            id: crypto.randomUUID(),
            size: 50,
            direction: 'horizontal',
            min_size: 20,
          },
        ],
      },
    ],
  },
};

/**
 * Data Science - Jupyter, data exploration, and visualization
 */
export const dataScienceTemplate: WorkspaceTemplate = {
  id: 'data-science',
  name: 'Data Science',
  description: 'Layout for data analysis, notebooks, and visualization',
  icon: '📊',
  tags: ['data', 'science', 'analytics'],
  preview: '┌──────────────┐\n│   Notebook   │\n├──────┬───────┤\n│ Data │ Viz   │\n└──────┴───────┘',
  layout: {
    version: '1.0.0',
    type: 'split',
    panes: [
      {
        id: crypto.randomUUID(),
        size: 60,
        direction: 'horizontal',
        min_size: 40,
      },
      {
        id: crypto.randomUUID(),
        size: 40,
        direction: 'horizontal',
        min_size: 20,
        children: [
          {
            id: crypto.randomUUID(),
            size: 50,
            direction: 'vertical',
            min_size: 20,
          },
          {
            id: crypto.randomUUID(),
            size: 50,
            direction: 'vertical',
            min_size: 20,
          },
        ],
      },
    ],
  },
};

/**
 * Microservices - Multiple service terminals
 */
export const microservicesTemplate: WorkspaceTemplate = {
  id: 'microservices',
  name: 'Microservices',
  description: 'Grid layout for managing multiple services simultaneously',
  icon: '🔧',
  tags: ['microservices', 'backend', 'services'],
  preview: '┌─────┬─────┐\n│ S1  │ S2  │\n├─────┼─────┤\n│ S3  │ S4  │\n└─────┴─────┘',
  layout: {
    version: '1.0.0',
    type: 'split',
    panes: [
      {
        id: crypto.randomUUID(),
        size: 50,
        direction: 'horizontal',
        min_size: 25,
        children: [
          {
            id: crypto.randomUUID(),
            size: 50,
            direction: 'vertical',
            min_size: 25,
          },
          {
            id: crypto.randomUUID(),
            size: 50,
            direction: 'vertical',
            min_size: 25,
          },
        ],
      },
      {
        id: crypto.randomUUID(),
        size: 50,
        direction: 'horizontal',
        min_size: 25,
        children: [
          {
            id: crypto.randomUUID(),
            size: 50,
            direction: 'vertical',
            min_size: 25,
          },
          {
            id: crypto.randomUUID(),
            size: 50,
            direction: 'vertical',
            min_size: 25,
          },
        ],
      },
    ],
  },
};

/**
 * System Admin - Server monitoring and maintenance
 */
export const sysAdminTemplate: WorkspaceTemplate = {
  id: 'sysadmin',
  name: 'System Administration',
  description: 'Monitor multiple servers and run administrative tasks',
  icon: '⚙️',
  tags: ['admin', 'servers', 'monitoring'],
  preview: '┌─────┬─────┬─────┐\n│ Srv1│ Srv2│Logs │\n└─────┴─────┴─────┘',
  layout: {
    version: '1.0.0',
    type: 'split',
    panes: [
      {
        id: crypto.randomUUID(),
        size: 33.33,
        direction: 'vertical',
        min_size: 20,
      },
      {
        id: crypto.randomUUID(),
        size: 33.33,
        direction: 'vertical',
        min_size: 20,
      },
      {
        id: crypto.randomUUID(),
        size: 33.34,
        direction: 'vertical',
        min_size: 20,
      },
    ],
  },
};

/**
 * All available workspace templates
 */
export const workspaceTemplates: WorkspaceTemplate[] = [
  singlePaneTemplate,
  horizontalSplitTemplate,
  verticalSplitTemplate,
  devOpsTemplate,
  fullStackTemplate,
  codeReviewTemplate,
  dataScienceTemplate,
  microservicesTemplate,
  sysAdminTemplate,
];

/**
 * Get template by ID
 */
export function getTemplateById(id: string): WorkspaceTemplate | undefined {
  return workspaceTemplates.find((t) => t.id === id);
}

/**
 * Get templates by tag
 */
export function getTemplatesByTag(tag: string): WorkspaceTemplate[] {
  return workspaceTemplates.filter((t) => t.tags.includes(tag));
}

/**
 * Search templates by name or description
 */
export function searchTemplates(query: string): WorkspaceTemplate[] {
  const lowerQuery = query.toLowerCase();
  return workspaceTemplates.filter(
    (t) =>
      t.name.toLowerCase().includes(lowerQuery) ||
      t.description.toLowerCase().includes(lowerQuery) ||
      t.tags.some((tag) => tag.toLowerCase().includes(lowerQuery))
  );
}
