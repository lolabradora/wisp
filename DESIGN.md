---
name: wisp
description: Grey ghost-text suggestions for zsh, from a small model on your own machine. HEYLOLA paper system, light only.
colors:
  paper: "#f5f1ea"
  paper-2: "#ece6dc"
  paper-3: "#e2dbce"
  line: "#d9d1c3"
  graphite: "#1c1a17"
  graphite-2: "#5a554f"
  graphite-3: "#6b655d"
  amber: "#e2a254"
typography:
  display:
    fontFamily: "Hanken Grotesk, ui-sans-serif, system-ui, sans-serif"
    fontSize: "clamp(2.375rem, 1.1rem + 4.2vw, 4.75rem)"
    fontWeight: 500
    lineHeight: 1.02
    letterSpacing: "-0.04em"
  statement:
    fontFamily: "Hanken Grotesk, ui-sans-serif, system-ui, sans-serif"
    fontSize: "clamp(2.25rem, 1.2rem + 3.6vw, 4.25rem)"
    fontWeight: 500
    lineHeight: 1.04
    letterSpacing: "-0.04em"
  headline:
    fontFamily: "Hanken Grotesk, ui-sans-serif, system-ui, sans-serif"
    fontSize: "clamp(2rem, 1.25rem + 2.6vw, 3.5rem)"
    fontWeight: 500
    lineHeight: 1.06
    letterSpacing: "-0.035em"
  figure:
    fontFamily: "Hanken Grotesk, ui-sans-serif, system-ui, sans-serif"
    fontSize: "clamp(2rem, 1.4rem + 1.8vw, 3rem)"
    fontWeight: 500
    lineHeight: 1.05
    letterSpacing: "-0.035em"
    fontFeature: "\"tnum\" 1"
  title:
    fontFamily: "Hanken Grotesk, ui-sans-serif, system-ui, sans-serif"
    fontSize: "clamp(1.375rem, 1rem + 1.2vw, 2rem)"
    fontWeight: 500
    lineHeight: 1.15
    letterSpacing: "-0.03em"
  lead:
    fontFamily: "Hanken Grotesk, ui-sans-serif, system-ui, sans-serif"
    fontSize: "clamp(1.125rem, 1rem + 0.5vw, 1.375rem)"
    fontWeight: 400
    lineHeight: 1.45
    letterSpacing: "-0.01em"
  body:
    fontFamily: "Hanken Grotesk, ui-sans-serif, system-ui, sans-serif"
    fontSize: "1.0625rem"
    fontWeight: 400
    lineHeight: 1.5
  small:
    fontFamily: "Hanken Grotesk, ui-sans-serif, system-ui, sans-serif"
    fontSize: "0.875rem"
    fontWeight: 400
    lineHeight: 1.45
  caption:
    fontFamily: "Hanken Grotesk, ui-sans-serif, system-ui, sans-serif"
    fontSize: "0.8125rem"
    fontWeight: 400
    lineHeight: 1.45
  terminal:
    fontFamily: "Geist Mono, ui-monospace, SF Mono, Menlo, monospace"
    fontSize: "clamp(0.71875rem, 0.55rem + 0.75vw, 0.9375rem)"
    fontWeight: 400
    lineHeight: 1.65
    fontFeature: "\"liga\" 0, \"calt\" 0"
rounded:
  highlight: "3px"
  key: "5px"
  focus: "6px"
  button: "8px"
  panel: "10px"
  pill: "999px"
spacing:
  gutter: "clamp(16px, 4vw, 32px)"
  column: "1080px"
  section: "clamp(112px, 13vw, 184px)"
  terminal-max: "760px"
  panel-pad: "24px 28px"
  panel-pad-compact: "18px 16px"
components:
  button-pill:
    backgroundColor: "{colors.graphite}"
    textColor: "{colors.paper}"
    rounded: "{rounded.pill}"
    padding: "0 26px"
    height: "48px"
  button-pill-hover:
    opacity: 0.88
  button-copy:
    textColor: "{colors.graphite-2}"
    rounded: "{rounded.button}"
    size: "34px"
  button-copy-hover:
    backgroundColor: "{colors.paper-3}"
    textColor: "{colors.graphite}"
  link-text:
    textColor: "{colors.graphite}"
  terminal-panel:
    backgroundColor: "{colors.paper-2}"
    textColor: "{colors.graphite-2}"
    typography: "{typography.terminal}"
    rounded: "{rounded.panel}"
    padding: "{spacing.panel-pad}"
    width: "{spacing.terminal-max}"
  terminal-ghost:
    textColor: "{colors.graphite-3}"
  terminal-key:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.graphite}"
    rounded: "{rounded.key}"
  keycap:
    backgroundColor: "{colors.paper-2}"
    textColor: "{colors.graphite}"
    rounded: "{rounded.key}"
  source-highlight:
    backgroundColor: "{colors.amber}"
    textColor: "{colors.graphite}"
    rounded: "{rounded.highlight}"
  install-panel:
    backgroundColor: "{colors.paper-2}"
    textColor: "{colors.graphite}"
    typography: "{typography.terminal}"
    rounded: "{rounded.panel}"
    padding: "14px 14px 14px 28px"
    width: "{spacing.terminal-max}"
---

# Design System: wisp

## Overview

**Creative North Star: "The Paper Terminal"**

A calm, warm-paper page where the only machine in sight is a light terminal, and the only colour is the moment a suggestion is accepted. The system belongs to the HEYLOLA brand: warm off-white paper, graphite ink, one amber that never carries text. It is light only; there is no dark mode and no dark terminal.

Density is low and centred. Every section sits on one 1080px column with a generous vertical rhythm, headings set in Hanken Grotesk at medium weight with tight negative tracking, and long grey copy kept to short measures. The terminal panels are the only boxes on the page; everything else is separated by 1px hairlines or by space. Depth is tonal (paper, paper-2, paper-3), never shadowed.

Motion is the product demonstrated, not decoration. Terminal demos replay real completions as an abortable state machine whose signature is the accept beat: grey ghost text, a keycap press, the line turning solid and an amber block growing behind the source token above. The static HTML always carries the settled frame, so the page is complete without JavaScript.

**Key Characteristics:**
- Light only: warm paper ground, graphite text, no dark surfaces besides the graphite pill.
- One accent, amber, reserved for caret, focus halo, selection and the source-token highlight. Never text.
- Hanken Grotesk for everything; Geist Mono only inside terminals, code and install commands.
- Medium-weight headings with tight tracking; nothing bold.
- Flat: tonal panels and 1px hairlines, zero shadows.
- Motion on transform and opacity only, paused off-screen, on hidden tabs and by the user.

## Colors

A warm, low-chroma paper-and-graphite neutral range with a single saturated amber used as a signal, never as a voice.

### Primary
- **Accept Amber** (amber): the caret, the 3px focus halo, `::selection`, and the highlight block that grows behind the source token at accept. Graphite sits on it at 7.88:1. It is never a text colour and never a fill for buttons or panels.

### Neutral
- **Paper** (paper): the page ground, the terminal keycap face inside demos, and the text colour on the graphite pill (15.4:1).
- **Paper Two** (paper-2): terminal and install panels, inline keycaps. The only raised tone.
- **Paper Three** (paper-3): hover fill of the copy buttons only.
- **Line** (line): every 1px hairline: panel borders, keycap borders, footer rule, figures-row and names-row dividers, link underlines at rest, scrollbar thumb.
- **Graphite** (graphite): headings, body emphasis, links, typed and accepted terminal text, the pill fill, the focus outline.
- **Graphite Two** (graphite-2): secondary copy (lead, body, nav, footer, figure labels) and terminal output text. 5.94:1 on paper-2, 6.55:1 on paper.
- **Graphite Three** (graphite-3): ghost suggestions, the `$` prompt sigil, small captions and footnotes. 5.12:1 on paper, 4.64:1 on paper-2.

### Named Rules
**The Amber Is A Signal Rule.** Amber marks focus, the caret and the instant of accept. It never sets type, never fills a button, and never appears in the no-JS or reduced-motion frame of a demo.

**The Solid Ghost Rule.** Ghost text is solid graphite-3, never graphite at reduced opacity. Its contrast is a measured 4.64:1 on paper-2; fading it would break that.

## Typography

**Display Font:** Hanken Grotesk, variable 100-900, self-hosted latin + latin-ext, OFL (with ui-sans-serif, system-ui, sans-serif)
**Body Font:** Hanken Grotesk (same file)
**Label/Mono Font:** Geist Mono, variable 100-900, self-hosted latin, OFL (with ui-monospace, SF Mono, Menlo, monospace)

**Character:** One humanist grotesk carries the whole page at two weights, tight and quiet at size; a clean mono appears only where a real terminal would, with ligatures off so commands read literally.

### Hierarchy
- **Display** (500, clamp(2.375rem, 1.1rem + 4.2vw, 4.75rem), 1.02, -0.04em): the hero headline, centred, balanced, max 20ch.
- **Statement** (500, clamp(2.25rem, 1.2rem + 3.6vw, 4.25rem), 1.04, -0.04em): the single privacy statement, revealed line by line.
- **Headline** (500, clamp(2rem, 1.25rem + 2.6vw, 3.5rem), 1.06, -0.035em): section headings, balanced.
- **Figure** (500, clamp(2rem, 1.4rem + 1.8vw, 3rem), 1.05, -0.035em, tabular numerals): the three labelled figures.
- **Title** (500, clamp(1.375rem, 1rem + 1.2vw, 2rem), 1.15, -0.03em): terminal names in the names row.
- **Lead** (400, clamp(1.125rem, 1rem + 0.5vw, 1.375rem), 1.45, -0.01em): graphite-2 subheads, 34-40ch.
- **Body** (400, 1.0625rem, 1.5): graphite-2 section copy, max 46ch.
- **Small** (400, 0.875rem, 1.45): graphite-3 footnotes and name notes, max 62ch.
- **Caption** (400, 0.8125rem, 1.45): demo disclaimers, the Pause control, "Copied".
- **Terminal** (Geist Mono 400, clamp(0.71875rem, 0.55rem + 0.75vw, 0.9375rem), 1.65, ligatures off): all terminal and install-command text.
- **Wordmark** (500, 1.375rem header / 1.0625rem footer, -0.04em): "wisp" set in the body face, not a logo file.

### Named Rules
**The Medium Weight Rule.** Headings are 500 with -0.03em to -0.04em tracking. Nothing on the page is bold.

**The Mono Means Machine Rule.** Geist Mono appears only inside terminal panels, `code`/`kbd`, and the install commands. Keycap letters inside prose use the sans at 0.75rem/500.

## Layout

One centred column: 1080px content width plus a fluid gutter of clamp(16px, 4vw, 32px). Sections are separated by clamp(112px, 13vw, 184px) of top padding; the last section also takes it at the bottom. The hero is centred with a reduced top of clamp(40px, 6vw, 88px); its terminal (max 760px) starts clamp(48px, 6vw, 72px) under the actions so it begins above the fold at desktop sizes. Section heads are centred; copy measures stay short (lead 34-40ch, body 46ch).

Responsive behaviour:
- **S2 reads, >=960px with motion allowed:** a sticky 100svh stage (5fr copy / 7fr terminals, max 1180px) pinned over a 270svh scroll track; three terminals crossfade in place with three 6px progress dots. Otherwise the three terminals stack in a 760px column, and this is the full no-JS layout.
- **S3 pair:** two terminals side by side at >=900px, stacked below.
- **Names row / figures row:** horizontal with hairline dividers; at <=720px names become a 2-column grid and figures a single 360px column with top hairlines.
- **Terminal and install panels** tighten padding at <=560px; the demo footer stacks.
- Supported from 360px.

## Elevation & Depth

Flat. There are no shadows anywhere. Depth is tonal: paper ground, paper-2 for the terminal and install panels and keycaps, paper-3 for the copy-button hover, each bounded by a 1px line hairline. The one `box-shadow` in the build is the focus halo, which is a state ring, not elevation.

### Named Rules
**The Tone Not Shadow Rule.** A surface rises by one paper step plus a 1px hairline, never by a shadow.

**The Focus Ring Rule.** Every focusable element shows a 2px graphite outline offset 3px with a 3px amber halo (`box-shadow: 0 0 0 3px amber`) and a 6px radius.

## Shapes

Soft, small radii stepping with object size: 3px on the source highlight block, 5px on keycaps (both in-demo and inline), 8px on the square copy buttons, 10px on terminal and install panels, full pill (999px) on the primary button and skip link. Borders are always 1px line. Dividers in the names row are pseudo-element hairlines inset 8px top and bottom, so they never touch the type; the figures row uses full-height 1px borders. Icons are 16px-grid line SVGs (stroke 1.3-1.5, round caps).

## Components

### Buttons
Quiet and certain: one dark pill per action group, everything else is a text link.
- **Shape:** full pill (999px), 48px minimum height.
- **Primary (pill):** graphite fill, paper text, 500 weight, 1rem, -0.005em, 0 26px padding.
- **Hover / Focus / Active:** hover eases to 0.88 opacity on fine pointers only; active scales to 0.97 over 160ms on the accept ease; focus is the shared ring.
- **Text link:** graphite text with a 1px line-coloured underline at 0.25em offset; the underline turns graphite on hover (150ms).
- **Copy button:** 34px square, 8px radius, transparent, 16px line icon in graphite-2; hover fills paper-3 and darkens the icon; active scales 0.94; on copy a "Copied" caption fades in to its left over the ghost timing and the icon drops to 0.35 opacity for 1.6s. Hidden until JS is present.
- **Pause / Play toggle:** a caption-sized underlined text button in graphite-2 under the hero demo.

### Navigation
Header: 72px bar on the content column, wordmark left, three graphite-2 links at 0.9375rem right, darkening to graphite on hover. Footer: a 1px top hairline, wordmark, plain links, "MIT", and the HEYLOLA mark at 15px (13px on small screens).

### Terminal Panel (signature)
A light terminal, not a dark one.
- **Container:** paper-2, 1px line border, 10px radius, 24px 28px padding (18px 16px under 560px), terminal type in graphite-2, max 760px.
- **Lines:** output in graphite-2; commands the user ran and the live prompt in graphite; the `$` sigil in graphite-3.
- **Prompt:** two stacked layers in one grid cell, a live layer (typed text, amber 2px caret, graphite-3 ghost) and a done layer (the accepted command, solid graphite), with an in-terminal keycap (paper face, 1px line, 5px radius, arrow icon) on the right.
- **Source token:** the name on screen that the suggestion completes; at accept it turns graphite and a 3px-radius amber block scales from 0 to 1 on X behind it, left origin.
- **Scenes:** multi-scene demos stack scenes in one cell and crossfade at 400ms on the settle ease.
- **Static frame:** the HTML ships ghost + keycap visible and no amber. This is what no-JS and reduced-motion visitors see.

### The Accept Beat (signature motion)
Typing at 55 +/- 20ms per character; the ghost appears 140ms after the last key (60ms debounce plus ~80ms cached latency) over 180ms on cubic-bezier(0.2, 0, 0, 1); the keycap fades in, presses to scale 0.94 for 80ms, then the done layer fades over the live one in 320ms on cubic-bezier(0.32, 0.72, 0, 1) while the amber block grows behind the source token; the keycap fades out after 240ms and the caret resumes blinking 400ms later. Output lines settle in at 640ms on cubic-bezier(0.16, 1, 0.3, 1), translateY(6px) to 0, staggered 40ms per line. Demos run as an abortable timeline per panel: they pause off-screen, on a hidden tab and via the Pause control, and any interrupt jumps to the accepted end frame. The only other entrance on the page is the statement's line-by-line reveal (translateY(105%) to 0, 640ms settle, 80ms between lines).

### Keycaps (inline)
Inline `kbd` in prose: paper-2 face, 1px line border, 5px radius, min 2em wide, 0.5em 0.6em padding; arrow as a 0.75rem line icon, letters as sans 0.75rem/500; a graphite-3 "+" between chord keys.

### Install Panel
The terminal container in its static form: paper-2, 1px line, 10px radius, max 760px. Each command is a Geist Mono line in graphite with a graphite-3 `$ ` prefix and hanging indent so wrapped commands align, paired with a copy button. The final check command sits below a 1px hairline. A graphite pill below copies all commands; text links sit beside it.

### Names Row
Terminal names at title size with a small graphite-3 note under each (inline `code` in graphite-2), separated by inset 1px hairline pseudo-elements; wraps to a 2-column grid at <=720px.

### Figures Row
Three labelled figures on a 1px top hairline, max 880px, separated by 1px vertical hairlines; the figure (tabular numerals) sits above a graphite-2 label at 0.9375rem; a small graphite-3 footnote states the source. Collapses to one 360px column with per-figure top hairlines.

## Do's and Don'ts

### Do:
- **Do** keep the page light: paper ground, paper-2 panels, graphite text.
- **Do** separate with 1px line hairlines or space; the terminal and install panels are the only boxes.
- **Do** set headings in Hanken Grotesk 500 with -0.03em to -0.04em tracking.
- **Do** ship every demo's settled frame in HTML (ghost + keycap, no amber) so the page works without JS and under reduced motion.
- **Do** animate only transform and opacity, using the settle (640ms), ghost (180ms) and accept (320ms) tokens, and stop all demos off-screen, on hidden tabs and on Pause.
- **Do** use the shared focus ring: 2px graphite outline, 3px offset, 3px amber halo.

### Don't:
- **Don't** add a dark mode or a dark terminal.
- **Don't** use shadows for depth; the focus halo is the only box-shadow.
- **Don't** set any text in amber or fill buttons or panels with it.
- **Don't** fade ghost text with opacity; it is solid graphite-3.
- **Don't** put eyebrows or kicker labels above section headings.
- **Don't** use Geist Mono outside terminal panels, inline code and the install commands.
- **Don't** use bold weights; 500 is the heaviest.
