import './styles.css';

document.querySelector('#app').innerHTML = `
  <section class="hero">
    <img src="/motarjim.png" alt="motarjim logo" />
    <p class="eyebrow">HTML/CSS → Native UI Code compiler</p>
    <h1>Build native Flutter, Compose, and SwiftUI screens from web markup.</h1>
    <p class="lede">motarjim is a local-first compiler workspace with Rust crates, TypeScript SDK packages, a web playground, and a VS Code extension surface.</p>
    <div class="actions"><a href="/docs/introduction.md">Read the docs</a><a href="http://localhost:3000">Open playground</a></div>
  </section>
`;
