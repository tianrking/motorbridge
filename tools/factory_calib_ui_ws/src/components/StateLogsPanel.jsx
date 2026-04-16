import React, { useMemo, useState } from 'react';
import { useI18n } from '../i18n';
import { useMotorStudioContext } from '../hooks/useMotorStudioContext';

function downloadText(filename, text) {
  const blob = new Blob([text], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

export function StateLogsPanel() {
  const { t } = useI18n();
  const { stateSnapshot, logs, clearLogs, uiPrefs, toggleUiPref } = useMotorStudioContext();
  const stateCollapsed = uiPrefs.sectionStateCollapsed;
  const logsCollapsed = uiPrefs.sectionLogsCollapsed;
  const [levelFilter, setLevelFilter] = useState('all');

  const filteredLogs = useMemo(() => {
    if (levelFilter === 'all') return logs;
    return logs.filter((line) => (line.level || 'info') === levelFilter);
  }, [logs, levelFilter]);

  const logsText = filteredLogs.map((line) => `[${line.t}] [${line.level || 'info'}] ${line.msg}`).join('\n');

  const copyLogs = async () => {
    try {
      await navigator.clipboard.writeText(logsText || '(empty logs)');
    } catch {
      // clipboard might be unavailable
    }
  };

  return (
    <section className="card glass grid2">
      <div>
        <div className="sectionTitle">
          <h2>{t('section_state')}</h2>
          <button className="ghostBtn small" onClick={() => toggleUiPref('sectionStateCollapsed')}>
            {stateCollapsed ? t('expand') : t('collapse')}
          </button>
        </div>
        {!stateCollapsed && <pre className="box">{stateSnapshot}</pre>}
      </div>

      <div>
        <div className="sectionTitle">
          <h2>{t('section_logs')}</h2>
          <button className="ghostBtn small" onClick={() => toggleUiPref('sectionLogsCollapsed')}>
            {logsCollapsed ? t('expand') : t('collapse')}
          </button>
        </div>

        {!logsCollapsed && (
          <>
            <div className="row toolbar compactToolbar">
              <select className="inlineSelect" value={levelFilter} onChange={(e) => setLevelFilter(e.target.value)}>
                <option value="all">{t('all')}</option>
                <option value="ok">ok</option>
                <option value="err">err</option>
                <option value="info">info</option>
              </select>
              <button onClick={copyLogs}>{t('copy_logs')}</button>
              <button onClick={() => downloadText(`factory-logs-${Date.now()}.txt`, logsText)}>{t('download_logs')}</button>
              <button onClick={clearLogs}>{t('clear_logs')}</button>
            </div>
            <pre className="box logs">{logsText}</pre>
          </>
        )}
      </div>
    </section>
  );
}
