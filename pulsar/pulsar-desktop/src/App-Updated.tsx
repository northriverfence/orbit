import { useState } from 'react';
import { WorkspaceProvider } from './lib/WorkspaceManager';
import SidebarNew from './components/SidebarNew';
import MainContentMultiSession from './components/MainContentMultiSession';
import './App.css';

export type SidebarSection = 'workspaces' | 'servers' | 'file-transfer' | 'vaults' | 'settings' | null;

function App() {
  const [expandedSection, setExpandedSection] = useState<SidebarSection>('workspaces');

  return (
    <WorkspaceProvider>
      <div className="flex h-screen w-screen bg-white">
        <SidebarNew
          expandedSection={expandedSection}
          onSectionToggle={setExpandedSection}
        />
        <MainContentMultiSession />
      </div>
    </WorkspaceProvider>
  );
}

export default App;
