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

const PIPELINE_TOTAL = 5;
const PIPELINE_INTERVAL = 250;

export default function PlaygroundPage() {
  const { html, css, platform, minify, panelRatio, setPanelRatio, setIsCompiling, setPipelineStage, setOutput, setDiagnostics, setStats, setAst, setIr, isCompiling } = usePlaygroundStore();
  const compileMutation = useCompiler();
  const [isResizing, setIsResizing] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const handleCompile = useCallback(() => {
    if (!html.trim()) return;

    let cancelled = false;
    let stage = 0;

    const advancePipeline = () => {
      if (cancelled) return;
      if (stage <= PIPELINE_TOTAL) {
        setPipelineStage(stage);
        stage++;
        setTimeout(advancePipeline, PIPELINE_INTERVAL);
      }
    };
    advancePipeline();

    compileMutation.mutate(
      { html, css, platform, minify },
      {
        onSuccess: (result) => {
          if (cancelled) return;
          setOutput(result.code);
          setDiagnostics(result.diagnostics || []);
          setStats(result.stats || null);
          setAst(result.ast || null);
          setIr(result.ir || null);
          setPipelineStage(PIPELINE_TOTAL + 1);
          setTimeout(() => {
            if (!cancelled) setPipelineStage(-1);
          }, 1200);
        },
        onError: (error: Error) => {
          if (cancelled) return;
          setDiagnostics([{
            severity: 'error',
            code: 'E9999',
            message: error.message,
            suggestions: [],
            notes: [],
          }]);
          setPipelineStage(-1);
        },
        onSettled: () => {
          if (!cancelled) setIsCompiling(false);
        },
      }
    );

    return () => { cancelled = true; };
  }, [html, css, platform, minify, compileMutation, setPipelineStage, setOutput, setDiagnostics, setStats, setAst, setIr, setIsCompiling]);

  const shortcuts = useMemo(() => [
    { key: 'Enter', ctrl: true, handler: handleCompile },
    { key: 's', ctrl: true, handler: handleCompile },
  ], [handleCompile]);

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
    <div className={styles.page}>
      <div className={styles.topBar}>
        <PipelineVisualizer />
        <button
          className={styles.compileBtn}
          onClick={handleCompile}
          disabled={isCompiling || !html.trim()}
          title="Compile (Ctrl+Enter)"
        >
          <Icon.Compile size={12} />
          <span>Compile</span>
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
