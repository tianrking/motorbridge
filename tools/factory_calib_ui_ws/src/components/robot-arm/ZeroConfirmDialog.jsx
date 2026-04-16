import React from 'react';
import { useI18n } from '../../i18n';

export function ZeroConfirmDialog({ open, title, message, danger, onCancel, onConfirm }) {
  const { t } = useI18n();
  if (!open) return null;
  return (
    <div className="armDialogMask" role="dialog" aria-modal="true" aria-live="assertive">
      <div className="armDialogCard">
        <h3>{title || t('arm_zero_all_confirm_title')}</h3>
        <p>{message}</p>
        <div className="row toolbar compactToolbar">
          <button className="ghostBtn" onClick={onCancel}>
            {t('cancel')}
          </button>
          <button className={danger ? 'dangerBtn' : 'primary'} onClick={onConfirm}>
            {t('confirm')}
          </button>
        </div>
      </div>
    </div>
  );
}
