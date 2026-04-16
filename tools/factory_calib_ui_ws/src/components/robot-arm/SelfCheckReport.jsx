import React from 'react';
import { useI18n } from '../../i18n';

export function SelfCheckReport({ report }) {
  const { t } = useI18n();
  if (!report) return null;
  return (
    <div className={`armSelfCheckCard ${report.ok ? 'ok' : 'err'}`}>
      <div className="sectionTitle">
        <h2>{t('arm_self_check_result')}</h2>
        <span className="chip">
          {report.ok ? t('arm_self_check_pass') : t('arm_self_check_fail')}
        </span>
      </div>
      <div className="armMeta">
        <span>
          {t('arm_self_check_online')}: {report.onlineCount}/{report.total}
        </span>
        <span>
          {t('arm_self_check_param')}: ok={report.paramOkCount}, fail={report.paramFailCount}
        </span>
      </div>
      <div className="tip">
        {t('arm_self_check_reason')}: {report.reason}
      </div>
    </div>
  );
}
