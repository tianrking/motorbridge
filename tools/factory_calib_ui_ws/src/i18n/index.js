import React, { createContext, useContext, useEffect, useMemo, useState } from 'react';
import { en } from './messages/en';
import { zh } from './messages/zh';
import { es } from './messages/es';

const LS_LOCALE_KEY = 'factory_calib_ui_ws_locale_v1';
const LOCALES = ['en', 'zh', 'es'];
const messages = { en, zh, es };

function localeFromPath(pathname) {
  const parts = String(pathname || '/').split('/').filter(Boolean);
  const seg = (parts[0] || '').toLowerCase();
  return LOCALES.includes(seg) ? seg : null;
}

function buildPathWithLocale(pathname, locale) {
  const parts = String(pathname || '/').split('/').filter(Boolean);
  if (parts.length > 0 && LOCALES.includes(parts[0].toLowerCase())) {
    parts.shift();
  }
  if (locale !== 'en') {
    parts.unshift(locale);
  }
  return `/${parts.join('/')}`;
}

function fillTemplate(text, params) {
  return String(text).replace(/\{\{\s*(\w+)\s*\}\}/g, (_, k) => {
    const v = params?.[k];
    return v === undefined || v === null ? '' : String(v);
  });
}

function resolveMessage(locale, key) {
  return messages[locale]?.[key] ?? messages.en[key] ?? key;
}

const I18nContext = createContext({
  locale: 'en',
  setLocale: () => {},
  t: (key) => key,
});

export function I18nProvider({ children }) {
  const [locale, setLocaleState] = useState(() => {
    try {
      if (typeof window === 'undefined') return 'en';
      const fromPath = localeFromPath(window.location.pathname);
      if (fromPath) return fromPath;
      const v = window.localStorage.getItem(LS_LOCALE_KEY);
      return v && LOCALES.includes(v) ? v : 'en';
    } catch {
      return 'en';
    }
  });

  const setLocale = (next) => {
    const safe = LOCALES.includes(next) ? next : 'en';
    setLocaleState(safe);
    try {
      if (typeof window !== 'undefined') {
        window.localStorage.setItem(LS_LOCALE_KEY, safe);
        const nextPath = buildPathWithLocale(window.location.pathname, safe);
        const nextUrl = `${nextPath}${window.location.search}${window.location.hash}`;
        if (nextUrl !== `${window.location.pathname}${window.location.search}${window.location.hash}`) {
          window.history.replaceState(null, '', nextUrl);
        }
      }
    } catch {
      // ignore
    }
  };

  useEffect(() => {
    if (typeof window === 'undefined') return undefined;
    const onPopState = () => {
      const fromPath = localeFromPath(window.location.pathname);
      if (fromPath && fromPath !== locale) {
        setLocaleState(fromPath);
      }
      if (!fromPath && locale !== 'en') {
        setLocaleState('en');
      }
    };
    window.addEventListener('popstate', onPopState);
    return () => window.removeEventListener('popstate', onPopState);
  }, [locale]);

  const value = useMemo(
    () => ({
      locale,
      setLocale,
      t: (key, params) => fillTemplate(resolveMessage(locale, key), params),
    }),
    [locale],
  );

  return React.createElement(I18nContext.Provider, { value }, children);
}

export function useI18n() {
  return useContext(I18nContext);
}
