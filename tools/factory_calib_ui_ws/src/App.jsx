import React from 'react';
import { HeaderBar } from './components/HeaderBar';
import { QuickMenu } from './components/QuickMenu';
import { ConnectionPanel } from './components/ConnectionPanel';
import { ScanWorkspace } from './components/ScanWorkspace';
import { MotorSection } from './components/MotorSection';
import { StateLogsPanel } from './components/StateLogsPanel';
import { useMotorStudio } from './hooks/useMotorStudio';

export default function App() {
  const studio = useMotorStudio();

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

      <ConnectionPanel
        wsUrl={studio.wsUrl}
        setWsUrl={studio.setWsUrl}
        channel={studio.channel}
        setChannel={studio.setChannel}
        scanTimeoutMs={studio.scanTimeoutMs}
        setScanTimeoutMs={studio.setScanTimeoutMs}
        connectWs={studio.connectWs}
        disconnectWs={studio.disconnectWs}
        collapsed={studio.uiPrefs.sectionConnectionCollapsed}
        onToggleCollapsed={() => studio.toggleUiPref('sectionConnectionCollapsed')}
      />

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
        moveMotorCard={studio.moveMotorCard}
        activeMotor={studio.activeMotor}
        activeControl={studio.activeControl}
        patchControl={studio.patchControl}
        controlMotor={studio.controlMotor}
        probeMotor={studio.probeMotor}
        setIdFor={studio.setIdFor}
        verifyHit={studio.verifyHit}
        refreshMotorState={studio.refreshMotorState}
        collapsed={studio.uiPrefs.sectionMotorsCollapsed}
        onToggleCollapsed={() => studio.toggleUiPref('sectionMotorsCollapsed')}
      />

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
