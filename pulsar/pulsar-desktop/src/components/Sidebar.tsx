import { SidebarSection } from '../App'

interface SidebarProps {
  expandedSection: SidebarSection
  onSectionToggle: (section: SidebarSection) => void
}

interface SidebarSectionData {
  id: SidebarSection
  label: string
  icon: string
  badge?: number
  children?: { label: string; onClick?: () => void }[]
}

const sidebarSections: SidebarSectionData[] = [
  {
    id: 'workspaces',
    label: 'Workspaces',
    icon: '📁',
    children: [
      { label: 'Default Workspace' },
      { label: '+ New Workspace' }
    ]
  },
  {
    id: 'servers',
    label: 'Servers',
    icon: '🖥️',
    badge: 2,
    children: [
      { label: 'Production (prod.example.com)' },
      { label: 'AWS Instance (ec2-54-123...)' },
      { label: '+ Add Server' }
    ]
  },
  {
    id: 'file-transfer',
    label: 'File Transfer',
    icon: '📤',
    children: [
      { label: 'Quick Transfer' },
      { label: 'Recent Files' },
      { label: 'Scheduled' }
    ]
  },
  {
    id: 'vaults',
    label: 'Vaults',
    icon: '🔐',
    children: [
      { label: 'Credentials' },
      { label: 'SSH Keys' },
      { label: 'Certificates' }
    ]
  },
  {
    id: 'settings',
    label: 'Settings',
    icon: '⚙️',
    children: [
      { label: 'Appearance' },
      { label: 'Connections' },
      { label: 'Security' }
    ]
  }
]

export default function Sidebar({ expandedSection, onSectionToggle }: SidebarProps) {
  const handleToggle = (section: SidebarSection) => {
    // If clicking the already expanded section, collapse it
    // Otherwise, expand the clicked section (auto-collapse others)
    onSectionToggle(expandedSection === section ? null : section)
  }

  return (
    <div className="w-64 bg-sidebar-bg border-r border-gray-200 flex flex-col">
      {/* Header */}
      <div className="h-14 flex items-center px-4 border-b border-gray-200">
        <h1 className="text-xl font-bold text-gray-800">Pulsar</h1>
      </div>

      {/* Sections */}
      <div className="flex-1 overflow-y-auto">
        {sidebarSections.map((section) => (
          <div key={section.id} className="border-b border-gray-200">
            {/* Section Header */}
            <button
              onClick={() => handleToggle(section.id)}
              className="w-full px-4 py-3 flex items-center justify-between hover:bg-sidebar-hover transition-colors"
            >
              <div className="flex items-center gap-2">
                <span className="text-lg">{section.icon}</span>
                <span className="font-medium text-gray-700">{section.label}</span>
              </div>
              <div className="flex items-center gap-2">
                {section.badge && (
                  <span className="bg-accent-secondary text-white text-xs font-semibold px-2 py-0.5 rounded-full">
                    {section.badge}
                  </span>
                )}
                <span className={`text-gray-500 transition-transform ${expandedSection === section.id ? 'rotate-90' : ''}`}>
                  ›
                </span>
              </div>
            </button>

            {/* Section Content (Collapsible) */}
            {expandedSection === section.id && section.children && (
              <div className="bg-white">
                {section.children.map((child, idx) => (
                  <button
                    key={idx}
                    onClick={child.onClick}
                    className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors flex items-center justify-between group"
                  >
                    <span>{child.label}</span>
                    {child.label.includes('Resume') && (
                      <span className="text-xs text-accent-primary font-medium opacity-0 group-hover:opacity-100 transition-opacity">
                        Resume
                      </span>
                    )}
                  </button>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Footer */}
      <div className="h-12 border-t border-gray-200 flex items-center px-4 text-xs text-gray-500">
        v0.1.0
      </div>
    </div>
  )
}
