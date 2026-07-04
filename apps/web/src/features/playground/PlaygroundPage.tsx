import { useEffect, useCallback, useMemo } from 'react';
import EditorPanel from './components/EditorPanel';
import OutputPanel from './components/OutputPanel';
import Toolbar from './components/Toolbar';
import StatusBar from './components/StatusBar';
import { usePlaygroundStore } from '../../stores/playgroundStore';
import { useCompiler } from '../../hooks/useCompiler';
import { useKeyboard } from '../../hooks/useKeyboard';
import styles from './PlaygroundPage.module.css';

export default function PlaygroundPage() {
  const { html, css, platform, minify, isCompiling } = usePlaygroundStore();
  const compileMutation = useCompiler();

  const handleCompile = useCallback(() => {
    compileMutation.mutate({ html, css, platform, minify });
  }, [html, css, platform, minify, compileMutation]);

  const shortcuts = useMemo(() => [
    { key: 'Enter', ctrl: true, handler: handleCompile },
    { key: 's', ctrl: true, handler: handleCompile },
  ], [handleCompile]);

  useKeyboard(shortcuts);

  useEffect(() => {
    const timer = setTimeout(handleCompile, 500);
    return () => clearTimeout(timer);
  }, [handleCompile]);

  return (
    <div className={styles.page}>
      <Toolbar onCompile={handleCompile} isCompiling={isCompiling} />
      <div className={styles.content}>
        <EditorPanel />
        <OutputPanel />
      </div>
      <StatusBar />
    </div>
  );
}
