export const APP_DEFAULTS = {
  wsUrl: 'ws://127.0.0.1:9002',
  channel: 'can0',
  scanTimeoutMs: '500',
};

export const DEV_SERVER = {
  host: '0.0.0.0',
  port: Number.parseInt(import.meta?.env?.VITE_FACTORY_UI_PORT || '18110', 10) || 18110,
};

export const CMD_TIMEOUTS = {
  shortMs: 1500,
  controlMs: 6000,
  stateMs: 1500,
  verifyMs: 8000,
  setIdMs: 12000,
  registerMs: 3000,
  storeMs: 4000,
};

export const DAMIAO_CTRL_PARAM_RID = {
  ctrlMode: 10,
  currentBw: 24,
  velKp: 25,
  velKi: 26,
  posKp: 27,
  posKi: 28,
};
