import { useEffect } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import PlaygroundPage from './features/playground/PlaygroundPage';
import JavaScriptPlayground from './features/playground/JavaScriptPlayground';
import SettingsPage from './features/settings/SettingsPage';
import ErrorBoundary from './components/ErrorBoundary';
import Notifications from './components/Notifications';
import { useCompiler } from './hooks/useCompiler';

function AppContent() {
  const { compile, cancel, isCompiling } = useCompiler();

  useEffect(() => {
    const handler = () => compile();
    window.addEventListener('compile-trigger', handler);
    return () => window.removeEventListener('compile-trigger', handler);
  }, [compile]);

  return (
    <Layout onCompile={compile} onCancel={cancel} isCompiling={isCompiling}>
      <Routes>
        <Route path="/" element={<PlaygroundPage />} />
        <Route path="/playground" element={<JavaScriptPlayground />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Routes>
    </Layout>
  );
}

export default function App() {
  return (
    <ErrorBoundary>
      <BrowserRouter>
        <AppContent />
        <Notifications />
      </BrowserRouter>
    </ErrorBoundary>
  );
}
