import { BrowserRouter, Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import PlaygroundPage from './features/playground/PlaygroundPage';
import SettingsPage from './features/settings/SettingsPage';
import ErrorBoundary from './components/ErrorBoundary';

export default function App() {
  return (
    <ErrorBoundary>
      <BrowserRouter>
        <Layout>
          <Routes>
            <Route path="/" element={<PlaygroundPage />} />
            <Route path="/settings" element={<SettingsPage />} />
          </Routes>
        </Layout>
      </BrowserRouter>
    </ErrorBoundary>
  );
}
