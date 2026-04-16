import { usePersistedState } from './usePersistedState';

const LS_UI_PREFS_KEY = 'factory_calib_ui_ws_ui_prefs_v1';

export const DEFAULT_UI_PREFS = {
  sectionConnectionCollapsed: false,
  sectionScanCollapsed: false,
  sectionManualCollapsed: false,
  sectionMotorsCollapsed: false,
  sectionStateCollapsed: true,
  sectionLogsCollapsed: true,
  armSliderLiveMove: false,
};

export function usePreferences() {
  const [uiPrefs, setUiPrefs] = usePersistedState(LS_UI_PREFS_KEY, DEFAULT_UI_PREFS, (cached) => ({
    ...DEFAULT_UI_PREFS,
    ...(cached && typeof cached === 'object' ? cached : {}),
  }));

  const toggleUiPref = (key) => setUiPrefs((prev) => ({ ...prev, [key]: !prev[key] }));
  const setUiPref = (key, value) => setUiPrefs((prev) => ({ ...prev, [key]: value }));

  return { uiPrefs, toggleUiPref, setUiPref };
}
