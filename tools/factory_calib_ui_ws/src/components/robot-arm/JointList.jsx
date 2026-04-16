import React from 'react';
import { useI18n } from '../../i18n';
import { toHex } from '../../lib/utils';

function numText(v, digits = 3) {
  return Number.isFinite(v) ? Number(v).toFixed(digits) : '-';
}

export function JointList({
  robotArmJointRows,
  activeRowKey,
  onSelect,
  connected,
  scanRobotArmJoint,
  refreshMotorState,
}) {
  const { t } = useI18n();
  return (
    <div className="armLeftPane">
      <div className="sectionTitle armPaneTitle">
        <h2>{t('arm_left_joints')}</h2>
        <span className="tip">{robotArmJointRows.length}/7</span>
      </div>
      <div className="armJointList">
        {robotArmJointRows.map((row) => (
          <div
            key={row.key}
            className={`armJointCard ${activeRowKey === row.key ? 'active' : ''}`}
            onClick={() => onSelect(row.key)}
          >
            <div className="armCardHead">
              <strong>
                {t('joint')} {row.joint}
              </strong>
              <span className={`chip ${row.hit.online === false ? '' : 'chipOk'}`}>
                {row.hit.online === false ? t('offline') : t('online_unknown')}
              </span>
            </div>
            <div className="armMeta">
              <span>ESC {toHex(row.hit.esc_id)}</span>
              <span>MST {toHex(row.hit.mst_id)}</span>
            </div>
            <div className="armMeta">
              <span>
                {t('pos')} {numText(row.hit.pos)}
              </span>
              <span>
                {t('vel')} {numText(row.hit.vel)}
              </span>
              <span>
                {t('torq')} {numText(row.hit.torq)}
              </span>
            </div>
            <div className="row compactToolbar">
              <button
                className="small ghostBtn"
                disabled={!connected}
                onClick={(e) => {
                  e.stopPropagation();
                  scanRobotArmJoint(row.joint);
                }}
              >
                {t('arm_scan_joint')}
              </button>
              <button
                className="small ghostBtn"
                disabled={!connected}
                onClick={(e) => {
                  e.stopPropagation();
                  refreshMotorState(row.hit);
                }}
              >
                {t('refresh_state')}
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
