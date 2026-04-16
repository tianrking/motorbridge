import React from 'react';
import { HeaderBar } from './components/HeaderBar';
import { QuickMenu } from './components/QuickMenu';
import { ConnectionPanel } from './components/ConnectionPanel';
import { ScanWorkspace } from './components/ScanWorkspace';
import { MotorSection } from './components/MotorSection';
import { StateLogsPanel } from './components/StateLogsPanel';
import { RobotArmPage } from './components/RobotArmPage';
import { HelpCenterModal } from './components/HelpCenterModal';
import { useMotorStudio } from './hooks/useMotorStudio';
import { MotorStudioProvider } from './hooks/useMotorStudioContext';
import { useI18n } from './i18n';

export default function App() {
  const { t } = useI18n();
  const studio = useMotorStudio();
  const [page, setPage] = React.useState('general');
  const [helpOpen, setHelpOpen] = React.useState(false);

  return (
    <MotorStudioProvider value={studio}>
      <div className="app shell">
        <HeaderBar />

        {studio.menuOpen && <QuickMenu />}

        <section className="card glass">
          <div className="row toolbar compactToolbar">
            <button className={page === 'general' ? 'primary' : ''} onClick={() => setPage('general')}>
              {t('page_general')}
            </button>
            <button className={page === 'robot_arm' ? 'primary' : ''} onClick={() => setPage('robot_arm')}>
              {t('page_robot_arm')}
            </button>
            <button className="ghostBtn" onClick={() => setHelpOpen(true)}>
              {t('help_show')}
            </button>
          </div>
        </section>

        <HelpCenterModal open={helpOpen} page={page} onClose={() => setHelpOpen(false)} />

        <ConnectionPanel />

        {page === 'general' ? (
          <>
            <ScanWorkspace />
            <MotorSection />
          </>
        ) : (
          <RobotArmPage />
        )}

        <StateLogsPanel />
      </div>
    </MotorStudioProvider>
  );
}
