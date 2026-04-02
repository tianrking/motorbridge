export const VENDORS = ['damiao', 'robstride', 'myactuator', 'hightorque', 'hexfellow'];

export const SET_ID_VENDORS = new Set(['damiao', 'robstride']);

export const VENDOR_LABELS = {
  damiao: 'Damiao',
  robstride: 'RobStride',
  myactuator: 'MyActuator',
  hightorque: 'HighTorque',
  hexfellow: 'HexFellow',
};

export const DEFAULT_VENDOR_CONFIG = {
  damiao: { enabled: true, model: 'auto', startId: '0x01', endId: '0x10', feedbackBase: '0x10' },
  robstride: { enabled: true, model: 'rs-00', startId: '0x01', endId: '0x10', feedbackId: '0xFF' },
  myactuator: { enabled: false, model: 'X8', startId: '0x01', endId: '0x10' },
  hightorque: { enabled: false, model: 'hightorque', startId: '0x01', endId: '0x10' },
  hexfellow: { enabled: false, model: 'hexfellow', startId: '0x01', endId: '0x10' },
};
