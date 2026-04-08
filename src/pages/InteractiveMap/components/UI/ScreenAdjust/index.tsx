import React, { useEffect, useRef, useState } from 'react';

import Slider from 'rc-slider';
import { useRecoilState } from 'recoil';

import useI18N from '@/i18n';
import langState from '@/store/lang';

import './style.less';

interface ScreenParams {
  brightness: number;
  contrast: number;
  gamma: number;
  red: number;
  green: number;
  blue: number;
}

interface MonitorItem {
  index: number;
  name: string;
  device: string;
}

const DEFAULT_PARAMS: ScreenParams = {
  brightness: 100,
  contrast: 100,
  gamma: 1.0,
  red: 128,
  green: 128,
  blue: 128,
};

const PRESETS: Array<{ name: string; params: ScreenParams }> = [
  {
    name: 'screenAdjust.presetDefault',
    params: { ...DEFAULT_PARAMS },
  },
  {
    name: 'screenAdjust.presetBalanced',
    params: { brightness: 105, contrast: 110, gamma: 1.1, red: 128, green: 130, blue: 125 },
  },
  {
    name: 'screenAdjust.presetNight',
    params: { brightness: 218, contrast: 118, gamma: 1.18, red: 128, green: 128, blue: 128 },
  },
];

function storageKey(device: string) {
  return `im-screen-adjust-${device.replace(/[\\./]/g, '_')}`;
}

function loadParams(device: string): ScreenParams {
  try {
    const raw = localStorage.getItem(storageKey(device));
    if (raw) return { ...DEFAULT_PARAMS, ...JSON.parse(raw) };
  } catch {
    // ignore
  }
  return { ...DEFAULT_PARAMS };
}

function saveParams(device: string, params: ScreenParams) {
  localStorage.setItem(storageKey(device), JSON.stringify(params));
}

async function invokeSetParams(device: string, params: ScreenParams) {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('set_monitor_params', {
      device,
      gamma: params.gamma,
      brightness: params.brightness,
      contrast: params.contrast,
      red: params.red,
      green: params.green,
      blue: params.blue,
    });
  } catch (e) {
    console.error('[ScreenAdjust] set_monitor_params failed:', e);
  }
}

async function invokeResetParams(device: string) {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('reset_monitor_params', { device });
  } catch (e) {
    console.error('[ScreenAdjust] reset_monitor_params failed:', e);
  }
}

const Index = () => {
  const [lang] = useRecoilState(langState);
  const { t } = useI18N(lang);

  const [monitors, setMonitors] = useState<MonitorItem[]>([]);
  const [activeIndex, setActiveIndex] = useState(0);
  const [params, setParams] = useState<ScreenParams>({ ...DEFAULT_PARAMS });
  const activeDevice = useRef('');

  // Fetch real monitor list on mount, then restore saved params for each
  useEffect(() => {
    (async () => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        const list = await invoke<MonitorItem[]>('get_monitors');
        const items = list.length > 0 ? list : [{ index: 0, name: '屏幕 1', device: '' }];
        setMonitors(items);

        await Promise.all(
          items.map(async (m) => {
            const saved = loadParams(m.device);
            const isDefault =
              saved.brightness === DEFAULT_PARAMS.brightness &&
              saved.contrast === DEFAULT_PARAMS.contrast &&
              saved.gamma === DEFAULT_PARAMS.gamma &&
              saved.red === DEFAULT_PARAMS.red &&
              saved.green === DEFAULT_PARAMS.green &&
              saved.blue === DEFAULT_PARAMS.blue;
            if (!isDefault && m.device) {
              await invokeSetParams(m.device, saved);
            }
          }),
        );

        // Load first monitor's params into UI
        const first = items[0];
        activeDevice.current = first.device;
        setParams(loadParams(first.device));
      } catch {
        const fallback = [{ index: 0, name: '屏幕 1', device: '' }];
        setMonitors(fallback);
        activeDevice.current = '';
        setParams({ ...DEFAULT_PARAMS });
      }
    })();
  }, []);

  // Switch monitor tab
  const handleSelectMonitor = (m: MonitorItem) => {
    setActiveIndex(m.index);
    activeDevice.current = m.device;
    setParams(loadParams(m.device));
  };

  const handleChange = (key: keyof ScreenParams, value: number) => {
    setParams((prev) => {
      const next = { ...prev, [key]: value };
      saveParams(activeDevice.current, next);
      invokeSetParams(activeDevice.current, next);
      return next;
    });
  };

  const handlePreset = (preset: ScreenParams) => {
    setParams({ ...preset });
    saveParams(activeDevice.current, preset);
    invokeSetParams(activeDevice.current, preset);
  };

  const handleReset = () => {
    const defaults = { ...DEFAULT_PARAMS };
    setParams(defaults);
    localStorage.removeItem(storageKey(activeDevice.current));
    invokeResetParams(activeDevice.current);
  };

  return (
    <div className="im-screen-adjust" onMouseDown={(e) => e.stopPropagation()}>
      {/* Screen tabs */}
      <div className="im-screen-adjust-tabs">
        {monitors.map((m) => (
          <div
            key={m.device || m.index}
            className={`im-screen-adjust-tab${activeIndex === m.index ? ' active' : ''}`}
            onClick={() => handleSelectMonitor(m)}
          >
            {m.name}
          </div>
        ))}
      </div>

      {/* Global attributes */}
      <div className="im-screen-adjust-section-title">{t('screenAdjust.globalAttrs')}</div>

      <div className="im-screen-adjust-row">
        <label>{t('screenAdjust.gamma')}</label>
        <Slider
          className="im-screen-adjust-slider"
          min={0.1}
          max={3.0}
          step={0.05}
          value={params.gamma}
          onChange={(v) => handleChange('gamma', v as number)}
        />
        <span className="im-screen-adjust-value">{params.gamma.toFixed(2)}</span>
      </div>

      <div className="im-screen-adjust-row">
        <label>{t('screenAdjust.brightness')}</label>
        <Slider
          className="im-screen-adjust-slider"
          min={0}
          max={200}
          step={1}
          value={params.brightness}
          onChange={(v) => handleChange('brightness', v as number)}
        />
        <span className="im-screen-adjust-value">{params.brightness}</span>
      </div>

      <div className="im-screen-adjust-row">
        <label>{t('screenAdjust.contrast')}</label>
        <Slider
          className="im-screen-adjust-slider"
          min={0}
          max={200}
          step={1}
          value={params.contrast}
          onChange={(v) => handleChange('contrast', v as number)}
        />
        <span className="im-screen-adjust-value">{params.contrast}</span>
      </div>

      {/* Color channels */}
      <div className="im-screen-adjust-section-title im-screen-adjust-section-title--channels">
        <span className="im-screen-adjust-dot im-screen-adjust-dot--red" />
        {t('screenAdjust.colorChannels')}
        <span className="im-screen-adjust-hint">({t('screenAdjust.defaultHint')})</span>
      </div>

      <div className="im-screen-adjust-row">
        <label className="im-screen-adjust-label--red">{t('screenAdjust.red')}</label>
        <Slider
          className="im-screen-adjust-slider im-screen-adjust-slider--red"
          min={0}
          max={255}
          step={1}
          value={params.red}
          onChange={(v) => handleChange('red', v as number)}
        />
        <span className="im-screen-adjust-value">{params.red}</span>
      </div>

      <div className="im-screen-adjust-row">
        <label className="im-screen-adjust-label--green">{t('screenAdjust.green')}</label>
        <Slider
          className="im-screen-adjust-slider im-screen-adjust-slider--green"
          min={0}
          max={255}
          step={1}
          value={params.green}
          onChange={(v) => handleChange('green', v as number)}
        />
        <span className="im-screen-adjust-value">{params.green}</span>
      </div>

      <div className="im-screen-adjust-row">
        <label className="im-screen-adjust-label--blue">{t('screenAdjust.blue')}</label>
        <Slider
          className="im-screen-adjust-slider im-screen-adjust-slider--blue"
          min={0}
          max={255}
          step={1}
          value={params.blue}
          onChange={(v) => handleChange('blue', v as number)}
        />
        <span className="im-screen-adjust-value">{params.blue}</span>
      </div>

      {/* Presets */}
      <div className="im-screen-adjust-section-title">{t('screenAdjust.presets')}</div>
      <div className="im-screen-adjust-presets">
        {PRESETS.map((preset) => (
          <div
            key={preset.name}
            className="im-screen-adjust-preset-item"
            onClick={() => handlePreset(preset.params)}
          >
            {t(preset.name as any)}
          </div>
        ))}
      </div>

      {/* Reset button */}
      <button className="im-screen-adjust-reset" onClick={handleReset}>
        {t('screenAdjust.reset')}
      </button>
    </div>
  );
};

Index.displayName = 'ScreenAdjust';
export default React.memo(Index);
