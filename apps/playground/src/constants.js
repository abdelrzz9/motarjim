export const PIPELINE_STAGES = [
  { id: 'parse', label: 'Parse', icon: 'code' },
  { id: 'style', label: 'Style', icon: 'palette' },
  { id: 'analyze', label: 'Analyze', icon: 'search' },
  { id: 'ir', label: 'IR', icon: 'node' },
  { id: 'optimize', label: 'Optimize', icon: 'zap' },
  { id: 'generate', label: 'Generate', icon: 'play' },
];

export const TARGETS = [
  { id: 'flutter', label: 'Flutter', ext: 'dart', color: 'var(--flutter)' },
  { id: 'compose', label: 'Compose', ext: 'kt', color: 'var(--compose)' },
  { id: 'swiftui', label: 'SwiftUI', ext: 'swift', color: 'var(--swiftui)' },
];

export const STATUS_MESSAGES = {
  parse: 'Parsing HTML document…',
  style: 'Building style tree…',
  analyze: 'Analyzing semantics and structure…',
  ir: 'Constructing intermediate representation…',
  optimize: 'Optimizing layout and widget tree…',
  generate: 'Generating native code…',
};

export const SAMPLE_HTML = `<nav class="navbar">
  <h1>My App</h1>
</nav>
<section class="hero">
  <h1>Welcome</h1>
  <p>Build something great</p>
  <button>Get Started</button>
</section>`;

export const SAMPLE_CSS = `.navbar { background: #333; color: white; padding: 1rem; }
.hero { text-align: center; padding: 4rem; background: #1a1a2e; color: white; }
button { background: blue; color: white; border-radius: 8px; padding: 12px 24px; }`;
