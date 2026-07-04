import express, { type Request, type Response, type NextFunction } from 'express';
import path from 'path';
import { fileURLToPath } from 'url';
import { runPipeline, type Target } from '@motarjim/pipeline-core';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const VALID_TARGETS: Target[] = ['flutter', 'compose', 'swiftui'];

const app = express();

app.use(express.json({ limit: '2mb' }));
app.use(express.static(path.join(__dirname, 'public')));

app.use((_req, res, next) => {
  res.setHeader('Access-Control-Allow-Origin', '*');
  res.setHeader('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
  res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
  next();
});

app.get('/api/health', (_req: Request, res: Response) => {
  res.json({ status: 'ok', service: 'motarjim' });
});

app.post('/api/convert', async (req: Request, res: Response) => {
  const { html, css, target } = req.body ?? {};

  if (typeof html !== 'string' || !html.trim()) {
    return res.status(400).json({ error: 'Missing "html" field.' });
  }
  if (!VALID_TARGETS.includes(target)) {
    return res.status(400).json({ error: `Target must be one of: ${VALID_TARGETS.join(', ')}.` });
  }

  try {
    const output = await runPipeline({ html, css: css ?? '', target });
    res.json(output);
  } catch (err: unknown) {
    res.status(500).json({ error: err instanceof Error ? err.message : 'Conversion failed.' });
  }
});

app.use((_req: Request, res: Response) => {
  res.status(404).json({ error: 'Not found.' });
});

const PORT = Math.max(1024, Number(process.env.PORT) || 3000);
const server = app.listen(PORT, () => {
  console.log(`motarjim web UI running at http://localhost:${PORT}`);
});

function shutdown() {
  server.close(() => process.exit(0));
}
process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);
