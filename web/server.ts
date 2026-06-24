import express from 'express';
import path from 'path';
import { fileURLToPath } from 'url';

import { runPipeline } from '@motarjim/pipeline-core';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

const app = express();
app.use(express.json({ limit: '2mb' }));
app.use(express.static(path.join(__dirname, 'public')));

app.post('/api/convert', async (req, res) => {
  const { html, css, target } = req.body ?? {};

  if (typeof html !== 'string' || !html.trim()) {
    return res.status(400).json({ error: 'Missing "html" field.' });
  }
  if (!['flutter', 'compose', 'swiftui'].includes(target)) {
    return res.status(400).json({ error: 'Target must be one of: flutter, compose, swiftui.' });
  }

  try {
    const output = await runPipeline({ html, css: css ?? '', target });
    res.json(output);
  } catch (err: unknown) {
    res.status(500).json({ error: err instanceof Error ? err.message : 'Conversion failed.' });
  }
});

const PORT = Number(process.env.PORT) || 3000;
app.listen(PORT, () => {
  console.log(`motarjim web UI running at http://localhost:${PORT}`);
});