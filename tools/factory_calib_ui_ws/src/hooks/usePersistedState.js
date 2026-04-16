import { useEffect, useState } from 'react';

function resolveDefault(defaultValue) {
  return typeof defaultValue === 'function' ? defaultValue() : defaultValue;
}

function readPersistedValue(key, defaultValue, normalize) {
  const fallback = resolveDefault(defaultValue);
  try {
    if (typeof window === 'undefined') return fallback;
    const raw = window.localStorage.getItem(key);
    if (!raw) return fallback;
    const parsed = JSON.parse(raw);
    if (typeof normalize === 'function') return normalize(parsed, fallback);
    return parsed ?? fallback;
  } catch {
    return fallback;
  }
}

export function usePersistedState(key, defaultValue, normalize) {
  const [value, setValue] = useState(() => readPersistedValue(key, defaultValue, normalize));

  useEffect(() => {
    try {
      if (typeof window === 'undefined') return;
      window.localStorage.setItem(key, JSON.stringify(value));
    } catch {
      // ignore localStorage failures
    }
  }, [key, value]);

  return [value, setValue];
}
