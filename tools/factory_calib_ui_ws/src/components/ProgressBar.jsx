import React from 'react';
import { useI18n } from '../i18n';

export function ProgressBar({ active, progress, fallbackLabel }) {
  const { t } = useI18n();
  if (!active) return null;

  const done = Number(progress?.done || 0);
  const total = Math.max(1, Number(progress?.total || 1));
  const percent = Math.max(0, Math.min(100, Number(progress?.percent || 0)));
  const label = progress?.label || fallbackLabel || t('scanning');

  return (
    <div className="scanProgressWrap">
      <div className="scanProgressText">
        <span>{label}</span>
        <span>
          {done}/{total} ({percent}%)
        </span>
      </div>
      <div className="scanProgressTrack">
        <div className="scanProgressFill" style={{ width: `${percent}%` }} />
      </div>
    </div>
  );
}
