import React from 'react';

import { useRecoilState } from 'recoil';

import useI18N from '@/i18n';
import langState from '@/store/lang';

import './style.less';

export interface SettingProps {
  directoryHandler?: string;
  tarkovGamePathHandler?: FileSystemDirectoryHandle;
  locationScale: boolean;
  pipOpacity: number;
  onClickEftWatcherPath: () => void;
  onClickTarkovGamePathPath: () => void;
  onLocationScaleChange: (b: boolean) => void;
  onPipOpacityChange: (opacity: number) => void;
}

const Index = (props: SettingProps) => {
  const {
    locationScale,
    directoryHandler,
    tarkovGamePathHandler,
    pipOpacity,
    onLocationScaleChange,
    onClickEftWatcherPath,
    onClickTarkovGamePathPath,
    onPipOpacityChange,
  } = props;

  const [lang] = useRecoilState(langState);

  const { t } = useI18N(lang);

  const handleClickEftWatcherPath = () => {
    onClickEftWatcherPath();
  };

  const handleClickTarkovGamePathPath = () => {
    onClickTarkovGamePathPath();
  };

  const handleToggleLocationScale = () => {
    onLocationScaleChange(!locationScale);
  };

  const handleOpacityChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = parseInt(e.target.value) || 100;
    const clamped = Math.max(1, Math.min(100, value));
    onPipOpacityChange(clamped);
  };

  return (
    <div className="im-quicktools-modal-setting" onMouseDown={(e) => e.stopPropagation()}>
      <div className="im-quicktools-modal-setting-title">
        <span>{t('setting.title')}</span>
      </div>
      <div className="im-quicktools-modal-setting-block">
        {self === top && (
          <button
            className="im-quicktools-modal-setting-button"
            style={{ color: !directoryHandler ? '#ffffff' : '#288828' }}
            onClick={handleClickEftWatcherPath}
          >
            {directoryHandler
              ? `${t('setting.realtimeMarker')} ${directoryHandler.split('\\').pop() || directoryHandler}`
              : t('setting.enableMarker')}
          </button>
        )}
        {self === top && (
          <button
            className="im-quicktools-modal-setting-button"
            style={{ color: !tarkovGamePathHandler ? '#ffffff' : '#288828' }}
            onClick={handleClickTarkovGamePathPath}
          >
            {tarkovGamePathHandler
              ? `${t('setting.tarkovGamePath')} ${tarkovGamePathHandler?.name}`
              : t('setting.enableTarkovGamePath')}
          </button>
        )}
        <button
          style={{ color: !locationScale ? '#882828' : '#288828' }}
          className="im-quicktools-modal-setting-button"
          onClick={handleToggleLocationScale}
        >
          {t('setting.markerScale')} ({locationScale ? t('common.enable') : t('common.disable')})
        </button>
        <div className="im-quicktools-modal-setting-input-group">
          <label>{t('setting.windowOpacity')}</label>
          <input
            type="number"
            min="1"
            max="100"
            value={pipOpacity}
            onChange={handleOpacityChange}
            className="im-quicktools-modal-setting-input"
          />
          <span>%</span>
        </div>
      </div>
    </div>
  );
};

Index.displayName = 'Setting';
export default React.memo(Index);
