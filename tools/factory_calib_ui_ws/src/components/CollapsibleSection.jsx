import React from 'react';
import { useI18n } from '../i18n';

export function CollapsibleSection({
  title,
  collapsed,
  onToggleCollapsed,
  collapsedHint = '',
  rightHint = '',
  className = 'card glass',
  children,
}) {
  const { t } = useI18n();

  return (
    <section className={className}>
      <div className="sectionTitle">
        <h2>{title}</h2>
        <button className="ghostBtn small" onClick={onToggleCollapsed}>
          {collapsed ? t('expand') : t('collapse')}
        </button>
      </div>

      {!collapsed && children}
      {collapsed && (collapsedHint || rightHint) && <div className="tip">{collapsedHint || rightHint}</div>}
    </section>
  );
}
