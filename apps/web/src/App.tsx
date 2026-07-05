import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import PlaygroundPage from './features/playground/PlaygroundPage';
import JavaScriptPlayground from './features/playground/JavaScriptPlayground';
import SettingsPage from './features/settings/SettingsPage';
import ErrorBoundary from './components/ErrorBoundary';
import Notifications from './components/Notifications';

export default function App() {
  return (
    <ErrorBoundary>
      <BrowserRouter>
        <Layout>
          <Routes>
            <Route path="/" element={<PlaygroundPage />} />
            <Route path="/playground" element={<JavaScriptPlayground />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </Layout>
        <Notifications />
      </BrowserRouter>
    </ErrorBoundary>
  );
}
