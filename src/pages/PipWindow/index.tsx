import { useEffect, useRef, useState } from 'react';

import './style.less';

const PipWindow = () => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [debugInfo, setDebugInfo] = useState('初始化中...');

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    const setupListener = async () => {
      try {
        setDebugInfo('正在设置事件监听...');
        const { listen } = await import('@tauri-apps/api/event');

        unlisten = await listen<string>('pip-canvas-update', (event) => {
          setDebugInfo(`收到数据: ${event.payload.substring(0, 50)}...`);

          if (canvasRef.current) {
            const img = new Image();
            img.onload = () => {
              const ctx = canvasRef.current?.getContext('2d');
              if (ctx && canvasRef.current) {
                canvasRef.current.width = img.width;
                canvasRef.current.height = img.height;
                ctx.drawImage(img, 0, 0);
                setDebugInfo(`已渲染: ${img.width}x${img.height}`);
              }
            };
            img.onerror = () => {
              setDebugInfo('图片加载失败');
            };
            img.src = event.payload;
          }
        });

        setDebugInfo('事件监听已设置，等待数据...');
      } catch (err) {
        console.error('PiP setup error:', err);
        setDebugInfo(`错误: ${err}`);
      }
    };

    setupListener();

    return () => {
      unlisten?.();
    };
  }, []);

  return (
    <div className="pip-window">
      <canvas ref={canvasRef} className="pip-canvas" />
      <div className="pip-debug">{debugInfo}</div>
    </div>
  );
};

export default PipWindow;
