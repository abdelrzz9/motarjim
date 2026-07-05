import { useCallback, useMemo, useRef, useState } from 'react';
import EditorPanel from './components/EditorPanel';
import OutputPanel from './components/OutputPanel';
import PipelineVisualizer from './components/PipelineVisualizer';
import StatusBar from './components/StatusBar';
import { usePlaygroundStore } from '../../stores/playgroundStore';
import { useCompiler } from '../../hooks/useCompiler';
import { useKeyboard } from '../../hooks/useKeyboard';
import { Icon } from '../../components/Icons';
import styles from './PlaygroundPage.module.css';

export default function PlaygroundPage() {
  const { panelRatio, setPanelRatio } = usePlaygroundStore();
  const { compile, cancel, isCompiling } = useCompiler();
  const [isResizing, setIsResizing] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const handleCompile = useCallback(() => {
    if (isCompiling) {
      cancel();
    } else {
      compile();
    }
  }, [isCompiling, compile, cancel]);

  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      handleCompile();
    }
  }, [handleCompile]);

  const shortcuts = useMemo(() => [
    { key: 'Enter', ctrl: true, handler: handleCompile },
    { key: 's', ctrl: true, handler: handleCompile },
    { key: 'Escape', handler: () => {
      if (isCompiling) cancel();
    }},
  ], [handleCompile, isCompiling, cancel]);

  useKeyboard(shortcuts);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    setIsResizing(true);
    const startX = e.clientX;
    const startRatio = panelRatio;
    const containerWidth = containerRef.current?.offsetWidth || 1;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const dx = moveEvent.clientX - startX;
      const newRatio = Math.max(0.3, Math.min(0.7, startRatio + dx / containerWidth));
      setPanelRatio(newRatio);
    };

    const handleMouseUp = () => {
      setIsResizing(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  }, [panelRatio, setPanelRatio]);

  return (
    <div className={styles.page} onKeyDown={handleKeyDown}>
      <div className={styles.topBar}>
        <PipelineVisualizer />
        <button
          className={`${styles.compileBtn} ${isCompiling ? styles.compileBtnActive : ''}`}
          onClick={handleCompile}
          disabled={false}
          title={isCompiling ? 'Cancel (Escape)' : 'Compile (Ctrl+Enter)'}
        >
          <Icon.Compile size={12} />
          <span>{isCompiling ? 'Cancel' : 'Compile'}</span>
        </button>
      </div>
      <div className={styles.content} ref={containerRef}>
        <div className={styles.panel} style={{ flex: panelRatio }}>
          <EditorPanel />
        </div>
        <div
          className={`${styles.resizeHandle} ${isResizing ? styles.resizing : ''}`}
          onMouseDown={handleMouseDown}
        />
        <div className={styles.panel} style={{ flex: 1 - panelRatio }}>
          <OutputPanel />
        </div>
      </div>
      <StatusBar />
    </div>
  );
}
