import { useEffect, useRef, useState } from 'react';

import './style.less';

const PipWindow = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [opacity, setOpacity] = useState(0.3);
  const [debugInfo, setDebugInfo] = useState('初始化中...');

  useEffect(() => {
    const loadConfig = async () => {
      try {
        const response = await fetch('/pip-config.json');
        const config = await response.json();
        if (config.pipOpacity) {
          setOpacity(config.pipOpacity);
          setDebugInfo(`配置加载成功: opacity=${config.pipOpacity}`);
        }
      } catch (err) {
        console.error('Failed to load config:', err);
        setDebugInfo(`配置加载失败: ${err}`);
      }
    };
    loadConfig();
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');

        unlisten = await listen<string>('pip-canvas-update', (event) => {
          if (canvasRef.current) {
            const img = new Image();
            img.onload = () => {
              const ctx = canvasRef.current?.getContext('2d');
              if (ctx && canvasRef.current) {
                canvasRef.current.width = img.width;
                canvasRef.current.height = img.height;
                ctx.drawImage(img, 0, 0);
              }
            };
            img.src = event.payload;
          }
        });
      } catch (err) {
        console.error('PiP setup error:', err);
      }
    };

    setupListener();

    return () => {
      unlisten?.();
    };
  }, []);

  const handleDragStart = async (e: React.MouseEvent) => {
    e.preventDefault();
    setDebugInfo('开始拖动...');
    try {
      const { getCurrentWindow } = await import('@tauri-apps/api/window');
      const window = getCurrentWindow();
      setDebugInfo('调用 startDragging...');
      await window.startDragging();
      setDebugInfo('拖动完成');
    } catch (err) {
      console.error('Failed to start dragging:', err);
      setDebugInfo(`拖动失败: ${err}`);
    }
  };

  return (
    <div className="pip-window">
      <div className="pip-debug">{debugInfo}</div>
      <div
        className="pip-drag-handle"
        onMouseDown={handleDragStart}
        style={{ opacity }}
      >
        ⋮⋮
      </div>
      <canvas ref={canvasRef} className="pip-canvas" style={{ opacity }} />
    </div>
  );
};

export default PipWindow;
