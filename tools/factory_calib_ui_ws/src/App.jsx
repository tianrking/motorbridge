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
import { useI18n } from './i18n';

export default function App() {
  const { t } = useI18n();
  const studio = useMotorStudio();
  const [page, setPage] = React.useState('general');
  const [helpOpen, setHelpOpen] = React.useState(false);

  return (
    <div className="app shell">
      <HeaderBar
        connected={studio.connected}
        connText={studio.connText}
        menuOpen={studio.menuOpen}
        setMenuOpen={studio.setMenuOpen}
      />

      {studio.menuOpen && (
        <QuickMenu
          canAction={studio.canAction}
          runScan={studio.runScan}
          clearDevices={studio.clearDevices}
          connectWs={studio.connectWs}
          disconnectWs={studio.disconnectWs}
        />
      )}

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

      <ConnectionPanel
        wsUrl={studio.wsUrl}
        setWsUrl={studio.setWsUrl}
        channel={studio.channel}
        setChannel={studio.setChannel}
        targetTransport={studio.targetTransport}
        targetSerialPort={studio.targetSerialPort}
        scanTimeoutMs={studio.scanTimeoutMs}
        setScanTimeoutMs={studio.setScanTimeoutMs}
        connectWs={studio.connectWs}
        disconnectWs={studio.disconnectWs}
        collapsed={studio.uiPrefs.sectionConnectionCollapsed}
        onToggleCollapsed={() => studio.toggleUiPref('sectionConnectionCollapsed')}
      />

      {page === 'general' ? (
        <>
          <ScanWorkspace
            vendors={studio.vendors}
            setVendors={studio.setVendors}
            connected={studio.connected}
            canAction={studio.canAction}
            scanBusy={studio.scanBusy}
            scanProgress={studio.scanProgress}
            scanFoundFx={studio.scanFoundFx}
            runScan={studio.runScan}
            clearDevices={studio.clearDevices}
            manualDraft={studio.manualDraft}
            setManualDraft={studio.setManualDraft}
            addManualCard={studio.addManualCard}
            scanCollapsed={studio.uiPrefs.sectionScanCollapsed}
            onToggleScanCollapsed={() => studio.toggleUiPref('sectionScanCollapsed')}
            manualCollapsed={studio.uiPrefs.sectionManualCollapsed}
            onToggleManualCollapsed={() => studio.toggleUiPref('sectionManualCollapsed')}
          />

          <MotorSection
            hits={studio.hits}
            selectedHits={studio.selectedHits}
            connected={studio.connected}
            activeMotorKey={studio.activeMotorKey}
            setActiveMotorKey={studio.setActiveMotorKey}
            newCardKeys={studio.newCardKeys}
            cardRefs={studio.cardRefs}
            removeMotorCard={studio.removeMotorCard}
            clearAllMotors={studio.clearDevices}
            clearOfflineMotors={studio.clearOfflineMotors}
            moveMotorCard={studio.moveMotorCard}
            activeMotor={studio.activeMotor}
            activeControl={studio.activeControl}
            patchControl={studio.patchControl}
            controlMotor={studio.controlMotor}
            zeroMotor={studio.zeroMotor}
            probeMotor={studio.probeMotor}
            setIdFor={studio.setIdFor}
            verifyHit={studio.verifyHit}
            refreshMotorState={studio.refreshMotorState}
            collapsed={studio.uiPrefs.sectionMotorsCollapsed}
            onToggleCollapsed={() => studio.toggleUiPref('sectionMotorsCollapsed')}
          />
        </>
      ) : (
        <RobotArmPage
          connected={studio.connected}
          canAction={studio.canAction}
          robotArmModel={studio.robotArmModel}
          armScanBusy={studio.armScanBusy}
          armScanProgress={studio.armScanProgress}
          armBulkBusy={studio.armBulkBusy}
          armSelfCheckBusy={studio.armSelfCheckBusy}
          armSelfCheckProgress={studio.armSelfCheckProgress}
          armSelfCheckReport={studio.armSelfCheckReport}
          setRobotArmModel={studio.setRobotArmModel}
          robotArmJointRows={studio.robotArmJointRows}
          ensureRobotArmCards={studio.ensureRobotArmCards}
          scanRobotArmJoint={studio.scanRobotArmJoint}
          scanRobotArmAll={studio.scanRobotArmAll}
          runRobotArmSelfCheck={studio.runRobotArmSelfCheck}
          enableAllRobotArm={studio.enableAllRobotArm}
          disableAllRobotArm={studio.disableAllRobotArm}
          zeroAllRobotArm={studio.zeroAllRobotArm}
          resetPoseRobotArm={studio.resetPoseRobotArm}
          readRobotArmControlParams={studio.readRobotArmControlParams}
          writeRobotArmControlParams={studio.writeRobotArmControlParams}
          patchControl={studio.patchControl}
          controlMotor={studio.controlMotor}
          refreshMotorState={studio.refreshMotorState}
          uiPrefs={studio.uiPrefs}
          setUiPref={studio.setUiPref}
        />
      )}

      <StateLogsPanel
        stateSnapshot={studio.stateSnapshot}
        logs={studio.logs}
        clearLogs={studio.clearLogs}
        stateCollapsed={studio.uiPrefs.sectionStateCollapsed}
        onToggleStateCollapsed={() => studio.toggleUiPref('sectionStateCollapsed')}
        logsCollapsed={studio.uiPrefs.sectionLogsCollapsed}
        onToggleLogsCollapsed={() => studio.toggleUiPref('sectionLogsCollapsed')}
      />
    </div>
  );
}
