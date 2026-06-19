#!/usr/bin/env tsx
// Generates reusable 1000-element HTML/CSS fixture files for benchmarking.
//
// Target: 1000 raw HTML elements (non-text tags). The repeating card
// pattern produces 9 tags per item × 106 items = 954, plus ~47 static
// structural tags (nav, hero, feature wrapper, contact form, footer) = ~1001.
//
// Usage: npx tsx benchmarks/fixtures/generate.ts

import { writeFileSync } from 'fs';
import { join } from 'path';

const FIXTURE_DIR = join(import.meta.dirname, '.');

function generateHtml(): string {
  const lines: string[] = [];
  lines.push('<!DOCTYPE html>');
  lines.push('<html><head><title>Benchmark</title><link rel="stylesheet" href="1000-nodes.css"></head><body>');
  lines.push('<div class="page-wrapper">');

  // Static structural elements (~30 tags)
  lines.push('  <nav class="navbar"><div class="nav-inner"><h1>Benchmark App</h1><ul class="nav-links"><li><a href="/">Home</a></li><li><a href="/features">Features</a></li><li><a href="/pricing">Pricing</a></li><li><a href="/contact">Contact</a></li></ul></div></nav>');
  lines.push('  <section class="hero-section"><div class="hero-content"><h1>Welcome to the Benchmark</h1><p>This page tests compiler performance with realistic nested content.</p><button class="hero-btn">Get Started</button></div></section>');
  lines.push('  <section class="features-section"><h2 class="section-title">Features</h2><div class="feature-grid">');

  // Repeating items: 97 items × 10 tags each = 970
  for (let i = 0; i < 106; i++) {
    const modClass = i % 3 === 0 ? 'featured' : i % 3 === 1 ? 'standard' : 'compact';
    lines.push(`    <article class="card card-${modClass}">`);
    lines.push(`      <h3>Feature ${i + 1}</h3>`);
    lines.push(`      <p>Description for feature item number ${i + 1} with enough text to make it realistic.</p>`);
    lines.push(`      <div class="card-footer">`);
    lines.push(`        <button class="btn btn-${modClass}">Learn ${i + 1}</button>`);
    lines.push(`        <a href="/detail/${i + 1}" class="card-link">Details →</a>`);
    lines.push(`      </div>`);
    lines.push(`      <ul class="tag-list">`);
    lines.push(`        <li class="tag tag-${i % 5}">category-${i % 5}</li>`);
    lines.push(`        <li class="tag tag-${(i + 1) % 5}">category-${(i + 1) % 5}</li>`);
    lines.push(`      </ul>`);
    lines.push(`    </article>`);
  }

  lines.push('  </div></section>');

  // Contact form section (~12 tags)
  lines.push('  <section class="contact-section"><h2 class="section-title">Contact Us</h2><form class="contact-form"><div class="form-group"><label for="name">Name</label><input type="text" id="name" placeholder="Your name" /></div><div class="form-group"><label for="email">Email</label><input type="email" id="email" placeholder="your@email.com" /></div><div class="form-group"><label for="message">Message</label><textarea id="message" placeholder="Your message"></textarea></div><button type="submit" class="btn btn-primary">Send Message</button></form></section>');

  // Footer (~5 tags)
  lines.push('  <footer class="site-footer"><div class="footer-inner"><p>&copy; 2026 Benchmark Inc.</p><ul class="footer-links"><li><a href="/privacy">Privacy</a></li><li><a href="/terms">Terms</a></li></ul></div></footer>');

  lines.push('</div></body></html>');
  return lines.join('\n') + '\n';
}

function generateCss(): string {
  return `/* Benchmark fixture CSS — exercises selector matching, cascade, media queries */

/* ---- Reset & base ---- */
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: system-ui, sans-serif; color: #333; background: #f5f5f5; }
h1, h2, h3 { font-weight: 600; line-height: 1.3; }
p { line-height: 1.6; color: #555; }
a { color: #1a73e8; text-decoration: none; }

/* ---- Page wrapper ---- */
.page-wrapper { max-width: 1200px; margin: 0 auto; padding: 16px; }

/* ---- Navbar ---- */
.navbar { background: #1a1a2e; color: white; padding: 16px 24px; border-radius: 8px; margin-bottom: 24px; }
.nav-inner { display: flex; align-items: center; justify-content: space-between; }
.nav-links { display: flex; gap: 24px; list-style: none; }
.nav-links a { color: white; font-weight: 500; }
.nav-links a:hover { color: #90caf9; }

/* ---- Hero section ---- */
.hero-section { padding: 64px 32px; text-align: center; background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); color: white; border-radius: 12px; margin-bottom: 32px; }
.hero-content h1 { font-size: 2.5rem; margin-bottom: 16px; }
.hero-content p { font-size: 1.125rem; color: #b0b0b0; margin-bottom: 24px; }
.hero-btn { background: #e94560; color: white; border: none; padding: 12px 32px; border-radius: 6px; font-size: 1rem; cursor: pointer; }

/* ---- Section titles ---- */
.section-title { font-size: 1.75rem; margin-bottom: 24px; color: #1a1a2e; }

/* ---- Feature grid ---- */
.features-section { margin-bottom: 32px; }
.feature-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 20px; }

/* ---- Cards ---- */
.card { background: white; border-radius: 10px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); display: flex; flex-direction: column; gap: 12px; }
.card h3 { font-size: 1.25rem; color: #1a1a2e; }
.card p { font-size: 0.9rem; }
.card.featured { border-left: 4px solid #e94560; }
.card.standard { border-left: 4px solid #1a73e8; }
.card.compact { padding: 14px; }
.card-footer { display: flex; align-items: center; gap: 12px; margin-top: 4px; }

/* ---- Buttons ---- */
.btn { padding: 8px 20px; border: none; border-radius: 6px; font-size: 0.875rem; font-weight: 500; cursor: pointer; }
.btn-featured { background: #e94560; color: white; }
.btn-standard { background: #1a73e8; color: white; }
.btn-compact { background: #34a853; color: white; }
.btn-primary { background: #1a73e8; color: white; padding: 12px 28px; font-size: 1rem; }

/* ---- Card links ---- */
.card-link { font-size: 0.875rem; font-weight: 500; }
.card-link:hover { text-decoration: underline; }

/* ---- Tag lists ---- */
.tag-list { display: flex; gap: 8px; list-style: none; flex-wrap: wrap; margin-top: 4px; }
.tag { background: #e8f0fe; color: #1a73e8; padding: 4px 10px; border-radius: 12px; font-size: 0.75rem; font-weight: 500; }
.tag-0 { background: #fce8e6; color: #d93025; }
.tag-1 { background: #e6f4ea; color: #188038; }
.tag-2 { background: #fef7e0; color: #ea8600; }
.tag-3 { background: #e8f0fe; color: #1a73e8; }
.tag-4 { background: #f3e8fd; color: #9334e6; }

/* ---- Contact form ---- */
.contact-section { background: white; padding: 32px; border-radius: 12px; margin-bottom: 32px; box-shadow: 0 2px 8px rgba(0,0,0,0.08); }
.contact-form { display: flex; flex-direction: column; gap: 16px; max-width: 480px; }
.form-group { display: flex; flex-direction: column; gap: 4px; }
.form-group label { font-size: 0.875rem; font-weight: 500; color: #333; }
.form-group input,
.form-group textarea { padding: 10px 12px; border: 1px solid #ccc; border-radius: 6px; font-size: 0.9rem; }
.form-group textarea { min-height: 100px; resize: vertical; }

/* ---- Footer ---- */
.site-footer { background: #1a1a2e; color: #b0b0b0; padding: 24px; border-radius: 8px; }
.footer-inner { display: flex; align-items: center; justify-content: space-between; }
.footer-links { display: flex; gap: 16px; list-style: none; }
.footer-links a { color: #90caf9; font-size: 0.875rem; }

/* ---- Media queries ---- */
@media (min-width: 768px) {
  .feature-grid { grid-template-columns: repeat(3, 1fr); }
  .hero-section { padding: 80px 48px; }
  .hero-content h1 { font-size: 3rem; }
}
@media (max-width: 600px) {
  .nav-inner { flex-direction: column; gap: 12px; }
  .feature-grid { grid-template-columns: 1fr; }
  .hero-section { padding: 32px 16px; }
  .footer-inner { flex-direction: column; gap: 12px; text-align: center; }
}
@media (min-width: 1024px) {
  .page-wrapper { padding: 24px 32px; }
  .card { padding: 24px; }
}
@media (min-width: 1400px) {
  .page-wrapper { max-width: 1360px; }
  .feature-grid { grid-template-columns: repeat(4, 1fr); }
}
`;
}

function countHtmlTags(html: string): number {
  return (html.match(/<(\w+)[\s>]/g) || []).length;
}

const html = generateHtml();
const css = generateCss();

writeFileSync(join(FIXTURE_DIR, '1000-nodes.html'), html, 'utf-8');
writeFileSync(join(FIXTURE_DIR, '1000-nodes.css'), css, 'utf-8');

const tagCount = countHtmlTags(html);
console.log(`Generated fixture: ${tagCount} HTML tags (target: ~1000)`);
console.log(`  HTML: ${join(FIXTURE_DIR, '1000-nodes.html')}`);
console.log(`  CSS:  ${join(FIXTURE_DIR, '1000-nodes.css')}`);
console.log(`  CSS size: ${css.length} bytes`);
