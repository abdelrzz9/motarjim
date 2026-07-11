# HTML Support Coverage Matrix

**Legend:** ✅ Complete | 🟡 Partial | ❌ Missing | — Not Applicable

## Document Structure

| Feature | Status | Notes |
|---------|--------|-------|
| `<!DOCTYPE html>` | ✅ | Parsed and preserved |
| `<html>` | ✅ | → Root |
| `<head>` | ✅ | Content preserved |
| `<body>` | ✅ | Content preserved |
| Character references (`&amp;`, `&lt;`, etc.) | ❌ | Not decoded by custom parser; html5ever handles |
| Comments (`<!-- -->`) | ✅ | Parsed, mapped to Spacer semantic |
| CDATA sections | 🟡 | html5ever handles; custom parser doesn't |

## Semantic HTML5 Tags

| Feature | Status | IR Mapping |
|---------|--------|------------|
| `<header>` | ✅ | → Header |
| `<nav>` | ✅ | → Navigation |
| `<main>` | ✅ | → Main |
| `<footer>` | ✅ | → Footer |
| `<article>` | ✅ | → Article |
| `<section>` | ✅ | → Section |
| `<aside>` | ✅ | → Aside |
| `<h1>` - `<h6>` | ✅ | → Heading { level } |
| `<p>` | ✅ | → Paragraph |
| `<div>` | ✅ | → Container |
| `<span>` | ✅ | → Text |

## Forms

| Feature | Status | Notes |
|---------|--------|-------|
| `<form>` | ✅ | → Form |
| `<fieldset>` | 🟡 | → Form (generic) |
| `<legend>` | ❌ | Not mapped |
| `<label>` | ❌ | Not mapped; placeholder used as fallback |
| `<input type="text">` | ✅ | → Input |
| `<input type="email">` | ✅ | → Input |
| `<input type="password">` | ✅ | → Input |
| `<input type="number">` | ✅ | → Input |
| `<input type="date">` | ✅ | → Input |
| `<input type="checkbox">` | ✅ | → Checkbox |
| `<input type="radio">` | ✅ | → Radio |
| `<input type="file">` | 🟡 | → Custom("file_picker") |
| `<input type="color">` | 🟡 | → Custom("color_picker") |
| `<input type="range">` | 🟡 | → Custom("slider") |
| `<input type="hidden">` | ✅ | → Spacer (removed) |
| `<input type="button">` | ✅ | → Button |
| `<input type="submit">` | ✅ | → Button |
| `<input type="reset">` | ✅ | → Button |
| `<input type="image">` | ✅ | → Button |
| `<textarea>` | ✅ | → TextArea |
| `<select>` | ✅ | → Select |
| `<option>` | 🟡 | → Custom("option") |
| `<optgroup>` | ❌ | Not mapped |
| `<datalist>` | ❌ | Not mapped |
| `<output>` | ❌ | Not mapped |

## Media

| Feature | Status | Notes |
|---------|--------|-------|
| `<img>` | ✅ | → Image (URL hardcoded in generators, not from IR) |
| `<picture>` | ✅ | → Image |
| `<source>` | ❌ | Not mapped |
| `<video>` | 🟡 | → Custom("video") |
| `<audio>` | 🟡 | → Custom("audio") |
| `<track>` | ❌ | Not mapped |
| `<figure>` | ✅ | → Container |
| `<figcaption>` | ❌ | Not mapped |
| `<map>` | ❌ | Not mapped |
| `<area>` | ❌ | Not mapped |

## SVG

| Feature | Status | Notes |
|---------|--------|-------|
| `<svg>` | 🟡 | → Icon (no SVG path/element mapping) |
| Inline SVG elements (`<path>`, `<circle>`, etc.) | ❌ | Not mapped |
| SVG attributes (viewBox, d, etc.) | ❌ | Not extracted |

## Canvas

| Feature | Status | Notes |
|---------|--------|-------|
| `<canvas>` | 🟡 | → Custom("canvas") |
| Canvas drawing operations | ❌ | Not supported |

## Tables

| Feature | Status | Notes |
|---------|--------|-------|
| `<table>` | ✅ | → Table |
| `<caption>` | ❌ | Not mapped |
| `<colgroup>` | ❌ | Not mapped |
| `<col>` | ❌ | Not mapped |
| `<thead>` | 🟡 | → Section |
| `<tbody>` | 🟡 | → Section |
| `<tfoot>` | 🟡 | → Section |
| `<tr>` | ✅ | → TableRow |
| `<td>` | ✅ | → TableCell |
| `<th>` | ✅ | → TableCell |

## Lists

| Feature | Status | Notes |
|---------|--------|-------|
| `<ul>` | ✅ | → List |
| `<ol>` | ✅ | → List |
| `<li>` | ✅ | → ListItem |
| `<dl>` | ✅ | → List |
| `<dt>` | ✅ | → ListItem |
| `<dd>` | ✅ | → ListItem |
| `<menu>` | ❌ | Not mapped |

## Navigation & Links

| Feature | Status | Notes |
|---------|--------|-------|
| `<a href="...">` | 🟡 | → Custom("link") |
| `<a role="button">` | ✅ | → Button |
| `<link>` (stylesheet) | 🟡 | CSS extracted via regex in core |

## Interactive Elements

| Feature | Status | Notes |
|---------|--------|-------|
| `<details>` | 🟡 | → Container |
| `<summary>` | ❌ | Not mapped |
| `<dialog>` | ✅ | → Dialog |
| `<template>` | 🟡 | → Container |
| `<slot>` | ❌ | Not mapped (Web Components) |

## Scripting & Metadata

| Feature | Status | Notes |
|---------|--------|-------|
| `<script>` | ❌ | Not parsed in HTML flow; separate JS pipeline |
| `<noscript>` | 🟡 | → Container |
| `<style>` | ✅ | CSS extracted and parsed |
| `<title>` | ❌ | Not extracted |
| `<meta>` | ❌ | Not extracted |
| `<base>` | ❌ | Not handled |

## Embedded Content

| Feature | Status | Notes |
|---------|--------|-------|
| `<iframe>` | 🟡 | → Container |
| `<embed>` | 🟡 | → Container |
| `<object>` | 🟡 | → Container |
| `<param>` | ❌ | Not mapped |

## Inline Text Semantics

| Feature | Status | Notes |
|---------|--------|-------|
| `<strong>`, `<b>` | ✅ | → Text |
| `<em>`, `<i>` | ✅ | → Text |
| `<u>`, `<ins>` | ✅ | → Text |
| `<s>`, `<del>` | ✅ | → Text |
| `<mark>` | ✅ | → Text |
| `<small>` | ✅ | → Text |
| `<sub>`, `<sup>` | ✅ | → Text |
| `<code>`, `<kbd>`, `<samp>`, `<tt>` | ✅ | → Text |
| `<abbr>`, `<dfn>` | ✅ | → Text |
| `<cite>`, `<q>` | ✅ | → Text |
| `<time>` | ✅ | → Text |
| `<br>` | ✅ | → Spacer |
| `<wbr>` | ❌ | Not mapped |
| `<bdi>`, `<bdo>` | ❌ | Not mapped |
| `<ruby>`, `<rt>`, `<rp>` | ❌ | Not mapped |
| `<data>` | ❌ | Not mapped |

## ARIA & Accessibility

| Feature | Status | Notes |
|---------|--------|-------|
| `role` attribute | ✅ | All major roles mapped (button, navigation, banner, main, etc.) |
| `aria-label` | ✅ | Stored in AccessibilityInfo |
| `aria-labelledby` | 🟡 | Stored as string, NOT resolved to referenced element |
| `aria-describedby` | 🟡 | Stored as string, NOT resolved |
| `aria-hidden` | ✅ | Parsed |
| `aria-expanded` | ✅ | Parsed |
| `aria-controls` | ✅ | Stored |
| `aria-live` | ✅ | Stored |
| `aria-busy` | ✅ | Parsed |
| `aria-level` | ✅ | Parsed (used for heading level) |
| `aria-required` | ❌ | Not mapped |
| `aria-invalid` | ❌ | Not mapped |
| `aria-current` | ❌ | Not mapped |
| `aria-disabled` | ❌ | Not mapped |
| `tabindex` | ✅ | Stored |
| `autofocus` | ✅ | Stored |
| `alt` text (on `<img>`) | ✅ | Used as fallback for label |
| Implicit role resolution | ✅ | e.g., `<nav>` → "navigation", `<header>` → "banner" |

## Void Elements

| Feature | Status |
|---------|--------|
| `<area>`, `<base>`, `<br>`, `<col>`, `<embed>`, `<hr>`, `<img>`, `<input>`, `<link>`, `<meta>`, `<param>`, `<source>`, `<track>`, `<wbr>` | ✅ |

## Overall HTML Support

| Path | Coverage | Notes |
|------|----------|-------|
| Custom parser (default) | ~40% | No character references, no CDATA, attributes via string re-scanning |
| html5ever parser (separate) | ~85% | Full spec compliance, not wired into pipeline |
| Semantic inference | ~60% | Most tags mapped, but not wired through to generators |
| Accessibility | ~50% | ARIA attributes extracted but not used by generators |

**Recommendation:** Migrate to html5ever as the sole HTML parser and wire it into the main pipeline.
