import { useEffect, useMemo, useState } from 'react';
import { defaultControlsForHit, motorKey, ts } from '../lib/utils';
import { usePersistedState } from './usePersistedState';
import { useI18n } from '../i18n';
import { useConnectionState } from './useConnectionState';
import { usePreferences } from './usePreferences';
import { useScanState } from './useScanState';
import { useMotorControl } from './useMotorControl';
import { useRobotArmOps } from './useRobotArmOps';

const LS_HITS_KEY = 'factory_calib_ui_ws_hits_v1';
const LS_CONTROLS_KEY = 'factory_calib_ui_ws_controls_v1';
const LS_ACTIVE_MOTOR_KEY = 'factory_calib_ui_ws_active_motor_v1';

export function useMotorStudio() {
  const { t } = useI18n();

  const [hits, setHits] = usePersistedState(LS_HITS_KEY, [], (cached) =>
    Array.isArray(cached) ? cached : []
  );
  const [controls, setControls] = usePersistedState(LS_CONTROLS_KEY, {}, (cached) =>
    cached && typeof cached === 'object' ? cached : {}
  );
  const [activeMotorKey, setActiveMotorKey] = usePersistedState(
    LS_ACTIVE_MOTOR_KEY,
    '',
    (cached) => (typeof cached === 'string' ? cached : '')
  );

  const [selected, setSelected] = useState(new Set());
  const [menuOpen, setMenuOpen] = useState(false);
  const [stateSnapshot, setStateSnapshot] = useState('(no state yet)');
  const [logs, setLogs] = useState([]);

  const pushLog = (msg, level = 'info') => {
    setLogs((prev) => [...prev, { t: ts(), msg, level }].slice(-500));
  };

  const connection = useConnectionState({ pushLog, setStateSnapshot });
  const preferences = usePreferences();

  const scan = useScanState({
    t,
    connected: connection.connected,
    scanTimeoutMs: connection.scanTimeoutMs,
    activeMotorKey,
    setActiveMotorKey,
    setHits,
    setControls,
    pushLog,
    closeBusQuietly: connection.closeBusQuietly,
    setTargetFor: connection.setTargetFor,
    sendCmd: connection.sendCmd,
  });

  const [armBulkBusyForControl, setArmBulkBusyForControl] = useState(false);
  const motorControl = useMotorControl({
    t,
    vendors: scan.vendors,
    controls,
    setHits,
    setControls,
    pushLog,
    setTargetFor: connection.setTargetFor,
    sendCmd: connection.sendCmd,
    closeBusQuietly: connection.closeBusQuietly,
    armBulkBusy: armBulkBusyForControl,
  });

  const robotArm = useRobotArmOps({
    connected: connection.connected,
    vendors: scan.vendors,
    hits,
    setHits,
    controls,
    setControls,
    activeMotorKey,
    setActiveMotorKey,
    pushLog,
    controlMotor: motorControl.controlMotor,
    zeroMotor: motorControl.zeroMotor,
    probeMotor: motorControl.probeMotor,
    setTargetFor: connection.setTargetFor,
    sendCmd: connection.sendCmd,
    closeBusQuietly: connection.closeBusQuietly,
  });

  useEffect(() => {
    setArmBulkBusyForControl(robotArm.armBulkBusy);
  }, [robotArm.armBulkBusy]);

  useEffect(() => {
    if (!activeMotorKey) return;
    const exists = hits.some((h) => motorKey(h) === activeMotorKey);
    if (!exists) setActiveMotorKey('');
  }, [hits, activeMotorKey, setActiveMotorKey]);

  const selectedHits = useMemo(
    () => hits.filter((h) => selected.has(motorKey(h))),
    [hits, selected]
  );
  const activeMotor = useMemo(
    () => hits.find((h) => motorKey(h) === activeMotorKey) || null,
    [hits, activeMotorKey]
  );
  const activeControl = activeMotor
    ? controls[motorKey(activeMotor)] || defaultControlsForHit(activeMotor)
    : null;

  const clearLogs = () => setLogs([]);
  const clearOfflineMotors = () => scan.clearOfflineMotors(hits, setSelected);
  const removeMotorCard = (hit) => scan.removeMotorCard(hit, setSelected);
  const clearDevices = () => {
    scan.clearDevices();
    setSelected(new Set());
  };

  return {
    wsUrl: connection.wsUrl,
    setWsUrl: connection.setWsUrl,
    channel: connection.channel,
    setChannel: connection.setChannel,
    scanTimeoutMs: connection.scanTimeoutMs,
    setScanTimeoutMs: connection.setScanTimeoutMs,
    connText: connection.connText,
    connected: connection.connected,
    targetTransport: connection.targetTransport,
    targetSerialPort: connection.targetSerialPort,
    connectWs: connection.connectWs,
    disconnectWs: connection.disconnectWs,
    scanBusy: scan.scanBusy,
    scanProgress: scan.scanProgress,
    scanFoundFx: scan.scanFoundFx,
    canAction: connection.connected && !scan.scanBusy && !robotArm.armBulkBusy,
    vendors: scan.vendors,
    setVendors: scan.setVendors,
    hits,
    controls,
    selectedHits,
    activeMotor,
    activeControl,
    activeMotorKey,
    setActiveMotorKey,
    newCardKeys: scan.newCardKeys,
    menuOpen,
    setMenuOpen,
    stateSnapshot,
    logs,
    clearLogs,
    uiPrefs: preferences.uiPrefs,
    toggleUiPref: preferences.toggleUiPref,
    setUiPref: preferences.setUiPref,
    manualDraft: scan.manualDraft,
    setManualDraft: scan.setManualDraft,
    robotArmModel: robotArm.robotArmModel,
    armScanBusy: robotArm.armScanBusy,
    armScanProgress: robotArm.armScanProgress,
    armBulkBusy: robotArm.armBulkBusy,
    armSelfCheckBusy: robotArm.armSelfCheckBusy,
    armSelfCheckProgress: robotArm.armSelfCheckProgress,
    armSelfCheckReport: robotArm.armSelfCheckReport,
    setRobotArmModel: robotArm.setRobotArmModel,
    robotArmJointRows: robotArm.robotArmJointRows,
    cardRefs: scan.cardRefs,
    runScan: scan.runScan,
    removeMotorCard,
    moveMotorCard: scan.moveMotorCard,
    addManualCard: scan.addManualCard,
    probeMotor: motorControl.probeMotor,
    clearDevices,
    clearOfflineMotors,
    patchControl: motorControl.patchControl,
    controlMotor: motorControl.controlMotor,
    zeroMotor: motorControl.zeroMotor,
    verifyHit: motorControl.verifyHit,
    setIdFor: motorControl.setIdFor,
    refreshMotorState: motorControl.refreshMotorState,
    ensureRobotArmCards: robotArm.ensureRobotArmCards,
    scanRobotArmJoint: robotArm.scanRobotArmJoint,
    scanRobotArmAll: robotArm.scanRobotArmAll,
    runRobotArmSelfCheck: robotArm.runRobotArmSelfCheck,
    enableAllRobotArm: robotArm.enableAllRobotArm,
    disableAllRobotArm: robotArm.disableAllRobotArm,
    zeroAllRobotArm: robotArm.zeroAllRobotArm,
    resetPoseRobotArm: robotArm.resetPoseRobotArm,
    readRobotArmControlParams: robotArm.readRobotArmControlParams,
    writeRobotArmControlParams: robotArm.writeRobotArmControlParams,
  };
}
