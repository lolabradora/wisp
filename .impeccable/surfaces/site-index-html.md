---
version: 1
slug: "site-index-html"
primary_target: "site/index.html"
related_targets: []
---

Scope: site/index.html, the wisp landing page (v2). Mode: Persuade.
Audience: developers on macOS or Linux who live in zsh, arriving skeptical from Hacker News or GitHub. Action: reach the install panel and copy the four commands, or open the docs.
Proof on hand: real recorded completions (LANDING-V2 brief, PRODUCT.md Evidence) and labelled internal figures. Nothing else is claimed.
Constraints: static HTML/CSS/JS in site/, no build step, no external requests, strict CSP (no inline script or style), relative URLs for the sub-path, works without JS, reduced motion, AA contrast, 360px and up.
Related files: site/404.html, site/robots.txt, site/assets/og.png (rendered from .impeccable/og/og.html).

## Direction contract

THESIS: One calm page where a light terminal shows wisp finishing a name that is visibly on the screen above it, and the source of that name lights amber at the moment of accept. It refuses the dark terminal hero, the feature-card grid and the glowing-accent dev-tool look.
OWN-WORLD: HEYLOLA paper. Paper #f5f1ea ground, paper-2 terminal panels with a 1px Line hairline and 10px radius, graphite text, graphite-3 ghost text (solid, never faded), one amber #e2a254 used only for caret, focus and the source highlight. Hanken Grotesk 500 tight headings; Geist Mono only inside terminals. Pill buttons in graphite. No shadows, no boxes beyond the terminals.
STORY: wisp completes names already on screen (hero); it reads what scrolled past (three pinned steps); it suggests before and while you type; it works in the terminal you have; nothing leaves the machine, with three labelled figures; four commands plus wisp doctor.
FIRST VIEWPORT: Centred two-line headline at display size in the upper half, one-sentence subhead, graphite "Install" pill plus "Read the docs" text link; the light terminal panel (max 760px) centred below, starting before the fold at 1440x900, with a small disclaimer and Pause control under it.
Signature: the accept beat. Ghost text appears after the real 60 ms pause, a → keycap presses, the line turns solid and an amber block grows behind the source token in the output above.
FORM: pinned by the brief (HEYLOLA brand + lead's section/motion decisions), code-led; the roll's assignment and challengers were overridden by the pinned direction. seed:4709fdc194b5cb17
FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
