# CSS Support Coverage Matrix

**Legend:** ✅ Complete | 🟡 Partial | ❌ Missing | — Not Applicable

## Selectors

| Feature | Parser Status | Engine Status | Notes |
|---------|:------------:|:-------------:|-------|
| Type (tag) selector | ✅ | ✅ | |
| Class selector (`.class`) | ✅ | ✅ | |
| ID selector (`#id`) | ✅ | ✅ | |
| Universal selector (`*`) | ✅ | ✅ | |
| Attribute selectors (`[attr]`, `[attr=val]`, `[attr~=val]`, `[attr\|=val]`, `[attr^=val]`, `[attr$=val]`, `[attr*=val]`) | ✅ | ✅ | All 6 operators supported |
| Descendant combinator (` `) | ✅ | ❌ | Parsed correctly, engine ignores it |
| Child combinator (`>`) | ✅ | ❌ | Parsed correctly, engine ignores it |
| Adjacent sibling (`+`) | ✅ | ❌ | Parsed correctly, engine ignores it |
| General sibling (`~`) | ✅ | ❌ | Parsed correctly, engine ignores it |
| Column combinator (`\|\|`) | ✅ | ❌ | |
| Selector list (comma) | ✅ | ✅ | |

## Pseudo-classes

| Feature | Parser Status | Engine Status | Notes |
|---------|:------------:|:-------------:|-------|
| `:hover` | ✅ | ❌ | Always returns true (no state awareness) |
| `:focus`, `:focus-visible`, `:focus-within` | ✅ | ❌ | Always returns true |
| `:active` | ✅ | ❌ | Always returns true |
| `:visited`, `:link` | ✅ | ❌ | Always returns true |
| `:disabled`, `:enabled` | ✅ | ❌ | Always returns true |
| `:checked` | ✅ | ❌ | Always returns true |
| `:required`, `:optional` | ✅ | ❌ | Always returns true |
| `:valid`, `:invalid` | ✅ | ❌ | Always returns true |
| `:read-only`, `:read-write` | ✅ | ❌ | Always returns true |
| `:first-child`, `:last-child`, `:only-child` | ✅ | ❌ | |
| `:first-of-type`, `:last-of-type`, `:only-of-type` | ✅ | ❌ | |
| `:nth-child()` | 🟡 | ❌ | Parsed but function arguments not evaluable in engine |
| `:nth-last-child()` | 🟡 | ❌ | |
| `:nth-of-type()` | 🟡 | ❌ | |
| `:nth-last-of-type()` | 🟡 | ❌ | |
| `:not()` | 🟡 | ❌ | Parsed, not evaluated |
| `:is()` | 🟡 | ❌ | Parsed, not evaluated |
| `:where()` | 🟡 | ❌ | Parsed, not evaluated |
| `:has()` | 🟡 | ❌ | Parsed, not evaluated |
| `:root` | ✅ | ❌ | |
| `:empty` | ✅ | ❌ | |
| `:target` | ✅ | ❌ | |
| `:lang()` | ✅ | ❌ | |
| `:dir()` | ✅ | ❌ | |
| `:scope` | ✅ | ❌ | |

## Pseudo-elements

| Feature | Parser Status | Engine Status | Notes |
|---------|:------------:|:-------------:|-------|
| `::before` | 🟡 | ❌ | Parsed; no content generation |
| `::after` | 🟡 | ❌ | Parsed; no content generation |
| `::first-line` | 🟡 | ❌ | |
| `::first-letter` | 🟡 | ❌ | |
| `::placeholder` | 🟡 | ❌ | |
| `::selection` | 🟡 | ❌ | |
| `::marker` | 🟡 | ❌ | |

## Specificity & Cascade

| Feature | Status | Notes |
|---------|:------:|-------|
| (id, class, type) specificity calculation | ✅ | Correct |
| `!important` priority | ✅ | Important overrides normal |
| Source order tie-breaking | ✅ | Later declarations win |
| Author/user/UA origin cascade | 🟡 | Author only |
| `@layer` cascade | 🟡 | Parsed, layer resolution not implemented |
| `inherit` keyword | ✅ | |
| `initial` keyword | ✅ | |
| `unset` keyword | ✅ | |
| `revert` keyword | ❌ | |
| Parent inheritance | ✅ | Child starts as parent clone |

## At-Rules

| Feature | Parser Status | Engine Status | Notes |
|---------|:------------:|:-------------:|-------|
| `@media` | ✅ | ❌ | Parsed fully; engine always includes nested rules |
| `@supports` | ✅ | ❌ | Parsed; engine always includes nested rules |
| `@keyframes` | ✅ | ❌ | Parsed; not mapped to platform animations |
| `@font-face` | ✅ | ❌ | Parsed; not resolved |
| `@import` | ✅ | ❌ | Parsed; not followed |
| `@charset` | ✅ | ❌ | |
| `@namespace` | ✅ | ❌ | |
| `@page` | ✅ | ❌ | |
| `@container` | 🟡 | ❌ | Via LightningCSS |
| `@layer` | 🟡 | ❌ | Via LightningCSS; cascade not implemented |
| `@scope` | 🟡 | ❌ | Via LightningCSS |
| `@starting-style` | 🟡 | ❌ | Via LightningCSS |
| `@view-transition` | 🟡 | ❌ | Via LightningCSS |
| `@counter-style` | 🟡 | ❌ | |
| `@property` | ❌ | ❌ | |

## Values & Units

| Feature | Parser Status | Conversion Engine | Notes |
|---------|:------------:|:-----------------:|-------|
| `px` | ✅ | ✅ | Converted to f64 |
| `em`, `rem` | ✅ | 🟡 | Parsed but relative conversion requires context |
| `%` | ✅ | 🟡 | Parsed but relative conversion requires context |
| `vw`, `vh` | ✅ | 🟡 | Parsed but needs viewport |
| `vmin`, `vmax` | ✅ | ❌ | |
| `fr` | 🟡 | ❌ | Parsed in grid, not converted |
| `deg`, `rad`, `grad`, `turn` | ✅ | ❌ | |
| `s`, `ms` | ✅ | ❌ | |
| `ch`, `ex` | ✅ | ❌ | |
| `cm`, `mm`, `in`, `pt`, `pc` | ✅ | ❌ | |
| `calc()` | 🟡 | ❌ | Parsed by LightningCSS, not evaluated |
| `min()`, `max()`, `clamp()` | 🟡 | ❌ | Parsed, not evaluated |
| `var()` | 🟡 | ❌ | **Not resolved** — critical gap |
| `attr()` | ❌ | ❌ | |

## Property Support

### Display & Positioning

| Property | Parsing | ComputedStyle | Generator Mapping | Notes |
|----------|:-------:|:-------------:|:-----------------:|-------|
| `display` | ✅ | ✅ (enum) | ❌ | Not used by generators |
| `position` | ✅ | ✅ (enum) | ❌ | Not used by generators |
| `top` | ✅ | ❌ | ❌ | **Not in ComputedStyle** |
| `right` | ✅ | ❌ | ❌ | **Not in ComputedStyle** |
| `bottom` | ✅ | ❌ | ❌ | **Not in ComputedStyle** |
| `left` | ✅ | ❌ | ❌ | **Not in ComputedStyle** |
| `z-index` | ✅ | ✅ (i32) | ❌ | Not used by generators |

### Sizing

| Property | Parsing | ComputedStyle | Generator Mapping | Notes |
|----------|:-------:|:-------------:|:-----------------:|-------|
| `width` | ✅ | ✅ (String) | 🟡 | Compose only; Flutter/SwiftUI ignore |
| `height` | ✅ | ✅ (String) | 🟡 | Compose only |
| `min-width` | ✅ | ✅ (String) | ❌ | |
| `max-width` | ✅ | ✅ (String) | ❌ | |
| `min-height` | ✅ | ✅ (String) | ❌ | |
| `max-height` | ✅ | ✅ (String) | ❌ | |

### Box Model

| Property | Parsing | ComputedStyle | Generator Mapping | Notes |
|----------|:-------:|:-------------:|:-----------------:|-------|
| `margin` | ✅ | ✅ (EdgeValues) | ✅ | Flutter: EdgeInsets; Compose: Modifier.padding; SwiftUI: .padding |
| `margin-top/right/bottom/left` | ✅ | ✅ (EdgeValues) | ✅ | |
| `padding` | ✅ | ✅ (EdgeValues) | ✅ | All three platforms |
| `padding-top/right/bottom/left` | ✅ | ✅ (EdgeValues) | ✅ | |
| `border` | ✅ | ✅ (Border) | ❌ | Not used by generators |
| `border-width` | ✅ | ✅ | ❌ | |
| `border-color` | ✅ | ✅ | ❌ | |
| `border-style` | ✅ | ✅ | ❌ | |
| `border-radius` | ✅ | ✅ (EdgeValues) | ❌ | Not used by generators |
| `box-sizing` | ✅ | ❌ | ❌ | |

### Flexbox

| Property | Parsing | ComputedStyle | Generator Mapping | Notes |
|----------|:-------:|:-------------:|:-----------------:|-------|
| `display: flex` | ✅ | ✅ | ✅ | LayoutIr::FlexRow/FlexColumn |
| `flex-direction` | ✅ | ✅ (enum) | ✅ | IR distinction |
| `flex-wrap` | ✅ | ✅ (enum) | ❌ | Not used by generators |
| `flex-grow` | ✅ | ✅ (f64) | ❌ | Not used by generators |
| `flex-shrink` | ✅ | ✅ (f64) | ❌ | |
| `flex-basis` | ✅ | ✅ (String) | ❌ | |
| `flex` (shorthand) | ✅ | ✅ (decomposed) | ❌ | |
| `justify-content` | ✅ | ✅ (enum) | 🟡 | Flutter/Compose only; SwiftUI missing |
| `align-items` | ✅ | ✅ (enum) | 🟡 | Flutter/Compose only; SwiftUI bugged |
| `align-self` | ✅ | ✅ (enum) | ❌ | |
| `align-content` | ✅ | ✅ (enum) | ❌ | |
| `gap` | ✅ | ✅ (String) | ❌ | |
| `row-gap` | ✅ | ✅ (String) | ❌ | |
| `column-gap` | ✅ | ✅ (String) | ❌ | |
| `order` | ✅ | ❌ | ❌ | |

### Grid

| Property | Parsing | ComputedStyle | Generator Mapping | Notes |
|----------|:-------:|:-------------:|:-----------------:|-------|
| `display: grid` | ✅ | ✅ | ✅ | LayoutIr::Grid |
| `grid-template-columns` | ✅ | 🟡 | ❌ | Stored as raw string |
| `grid-template-rows` | ✅ | 🟡 | ❌ | Stored as raw string |
| `grid-template-areas` | ✅ | ❌ | ❌ | |
| `grid-template` (shorthand) | 🟡 | ❌ | ❌ | |
| `grid-column` | ✅ | 🟡 | ❌ | Stored as raw string |
| `grid-row` | ✅ | 🟡 | ❌ | Stored as raw string |
| `grid-area` | ✅ | ❌ | ❌ | |
| `grid-auto-flow` | ✅ | ❌ | ❌ | |
| `grid-auto-rows` | ✅ | ❌ | ❌ | |
| `grid-auto-columns` | ✅ | ❌ | ❌ | |
| `gap` | ✅ | ✅ | ❌ | |
| `place-items` | 🟡 | ❌ | ❌ | |

### Colors & Backgrounds

| Property | Parsing | ComputedStyle | Generator Mapping | Notes |
|----------|:-------:|:-------------:|:-----------------:|-------|
| `color` | ✅ | ✅ (String) | ✅ | CSS hex → Color() on all platforms |
| `background-color` | ✅ | ✅ (Background) | 🟡 | Parsed but only color part used |
| `background-image` | ✅ | 🟡 | ❌ | Stored as raw string |
| `background` (shorthand) | 🟡 | 🟡 | ❌ | Partial |
| `linear-gradient()` | 🟡 | ❌ | ❌ | Parsed via LightningCSS, stored as raw |
| `radial-gradient()` | 🟡 | ❌ | ❌ | |

### Typography

| Property | Parsing | ComputedStyle | Generator Mapping | Notes |
|----------|:-------:|:-------------:|:-----------------:|-------|
| `font-family` | ✅ | ✅ (String) | ❌ | Not used by generators |
| `font-size` | ✅ | ✅ (String) | 🟡 | Hardcoded for headings only |
| `font-weight` | ✅ | ✅ (enum) | 🟡 | Named + numeric; used for heading bold only |
| `font-style` | ✅ | ✅ (String) | ❌ | |
| `font` (shorthand) | ❌ | ❌ | ❌ | |
| `line-height` | ✅ | ✅ (String) | ❌ | |
| `text-align` | ✅ | ✅ (enum) | ❌ | Not used by generators |
| `text-decoration` | ✅ | ✅ (String) | ❌ | |
| `text-transform` | ✅ | ❌ | ❌ | |
| `letter-spacing` | ✅ | ❌ | ❌ | |
| `word-spacing` | ✅ | ❌ | ❌ | |
| `white-space` | ✅ | ❌ | ❌ | |
| `word-break` | ✅ | ❌ | ❌ | |
| `overflow-wrap` | ✅ | ❌ | ❌ | |
| `direction` | ✅ | ❌ | ❌ | |
| `text-overflow` | ✅ | ❌ | ❌ | |

### Borders & Effects

| Property | Parsing | ComputedStyle | Generator Mapping | Notes |
|----------|:-------:|:-------------:|:-----------------:|-------|
| `opacity` | ✅ | ✅ (f64) | ❌ | Not used by generators |
| `box-shadow` | ✅ | 🟡 | ❌ | Stored as raw string |
| `text-shadow` | ✅ | ❌ | ❌ | |
| `transform` | ✅ | 🟡 | ❌ | Stored as raw string |
| `transform-origin` | ✅ | ❌ | ❌ | |
| `transition` | ✅ | 🟡 | ❌ | Stored as raw string |
| `animation` | ✅ | 🟡 | ❌ | Stored as raw string |
| `overflow` | ✅ | ✅ (enum) | ❌ | Not used by generators |
| `overflow-x` | ✅ | ❌ | ❌ | |
| `overflow-y` | ✅ | ❌ | ❌ | |
| `clip-path` | ✅ | ❌ | ❌ | |
| `filter` | ✅ | ❌ | ❌ | |
| `backdrop-filter` | ✅ | ❌ | ❌ | |
| `cursor` | ✅ | ✅ (String) | ❌ | |
| `pointer-events` | ✅ | ✅ (String) | ❌ | |
| `resize` | ✅ | ✅ (String) | ❌ | |
| `user-select` | ✅ | ✅ (String) | ❌ | |
| `appearance` | ✅ | ✅ (String) | ❌ | |
| `visibility` | ✅ | ✅ (bool) | ❌ | |
| `content` | ✅ | ❌ | ❌ | For pseudo-elements |

### CSS Variables (Custom Properties)

| Feature | Status | Notes |
|---------|:------:|-------|
| `--custom-property` declaration | 🟡 | Parsed, stored in AST, lost in engine |
| `var(--name)` reference | 🟡 | Parsed into `CssValue::Variable`, passed through as raw string |
| `var(--name, fallback)` | 🟡 | Parsed, fallback not resolved |
| Custom property inheritance | ❌ | Not tracked |
| `@property` registration | ❌ | Not supported |

## Media Query Features

| Feature | Parser Status | Engine Status | Notes |
|---------|:------------:|:-------------:|-------|
| `@media (min-width: ...)` | ✅ | ❌ | Parsed, not evaluated |
| `@media (max-width: ...)` | ✅ | ❌ | Parsed, not evaluated |
| `@media (min-height: ...)` | ✅ | ❌ | |
| `@media (max-height: ...)` | ✅ | ❌ | |
| `@media (orientation: portrait/landscape)` | ✅ | ❌ | |
| `@media (prefers-color-scheme: dark/light)` | ✅ | ❌ | |
| `@media (prefers-reduced-motion)` | ✅ | ❌ | |
| `@media (hover)` | ✅ | ❌ | |
| `@media (pointer)` | ✅ | ❌ | |
| `@media (any-hover)` | ✅ | ❌ | |
| `@media (any-pointer)` | ✅ | ❌ | |
| `@media (resolution)` | ✅ | ❌ | |
| `@media (color-gamut)` | ✅ | ❌ | |
| `@media screen` | ✅ | ❌ | |
| `@media print` | ✅ | ❌ | |
| `@media all` | ✅ | ❌ | |
| `@media only` | ✅ | ❌ | |
| `@media not` | ✅ | ❌ | |
| Logical combinations (`and`, `or`, `not`, `,`) | ✅ | ❌ | |

## Overall CSS Support: ~40% Complete

| Category | Approximate Coverage |
|----------|:-------------------:|
| **Parsing** (via LightningCSS) | 95% |
| **Cascade resolution** | 60% (correct basics, no combinator traversal) |
| **Selector matching** | 30% (type/class/id/attr only; no combinators, no pseudo evaluation) |
| **ComputedStyle construction** | 60% (50+ properties, but missing top/right/bottom/left) |
| **Value resolution** | 20% (no calc, no var, no relative conversion) |
| **Media query evaluation** | 0% |
| **Generator CSS mapping** | 10% (~5 of 50+ properties mapped) |
