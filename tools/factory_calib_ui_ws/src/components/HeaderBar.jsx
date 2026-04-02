import React from 'react';
import { useI18n } from '../i18n';

export function HeaderBar({ connected, connText, menuOpen, setMenuOpen }) {
  const { t, locale, setLocale } = useI18n();

  return (
    <header className="hero">
      <div>
        <h1>{t('app_title')}</h1>
        <p>{t('app_subtitle')}</p>
      </div>
      <div className="heroActions">
        <label className="langBox">
          <span>{t('language')}</span>
          <select value={locale} onChange={(e) => setLocale(e.target.value)}>
            <option value="en">{t('lang_en')}</option>
            <option value="zh">{t('lang_zh')}</option>
            <option value="es">{t('lang_es')}</option>
          </select>
        </label>
        <button className="menuToggle" onClick={() => setMenuOpen((v) => !v)}>
          {t('quick_menu')}
        </button>
        <div className={`badge ${connected ? 'ok' : 'err'}`}>{connText}</div>
      </div>
    </header>
  );
}
