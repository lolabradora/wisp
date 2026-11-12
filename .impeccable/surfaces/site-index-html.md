---
version: 1
slug: "site-index-html"
primary_target: "site/index.html"
related_targets: []
---

Scope: site/index.html, the wisp landing page. Mode: Persuade.
Audience: developers on macOS or Linux who live in zsh, arriving skeptical from Hacker News or GitHub. Action: copy the install command (hero), follow the four install steps, or open the repo.
Proof on hand: measured latencies, the 8-scenario eval, memory, and real completions (PRODUCT.md Evidence). Nothing else is claimed.
Constraints: static HTML/CSS/JS in site/ with no build step and no external requests (strict CSP, no inline script or style). Served from the sub-path https://lolabradora.github.io/wisp/, so every URL is relative. Repo: github.com/lolabradora/wisp, public, MIT.
Related files: site/404.html, site/robots.txt, site/assets/og.png (rendered from .impeccable/og/og.html).

## Direction contract

THESIS: The first screen is one command line whose possible completions hang as unlit ghosts behind bronze gauze, with the one wisp picks struck forward in cathode orange. It refuses the terminal-window mockup plus feature-card grid.
OWN-WORLD: An ink-blue black plane (#0a0a18) running past every edge under an undulating bronze wire gauze plate. Two hues: ink blue and cathode orange (#ff6a1a) with a halo. Bone-white type. Barlow Condensed (medium) for display, M PLUS 1 Code for commands, M PLUS 1 for prose. (Contract first named League Gothic from the font ranking; the comp lettering is a condensed grotesque, so the build uses Barlow Condensed — accepted by the finish review.) No boxes, rules, cards or dividers: grouping comes from depth and gauze. The only control shape is an orange-outlined pill.
STORY (0.2.0 relaunch): A real command completes with a name that was on the screen, and the hero shows that screen. Then: the screen it came from (six scenarios), which terminals it reads (tmux, WezTerm, kitty experimental, custom command) and what works without one, how it works, the measured numbers, that it runs locally, the limits, and the install steps with `wisp doctor`.
FIRST VIEWPORT: As comp-1 measured (1536x1024).
- Wordmark at top left, GitHub at top right.
- A 50px mono command at 16%/38%.
- Ghosts stacked at 60%/37%, with the struck completion at 60%/49%.
- A two-line Barlow Condensed headline at 4.5%/67.5%.
- A mono subline at 80%.
- The orange pill "Install from GitHub" at 85%.
0.2.0 relaunch changes to that frame, made without a new comp: three lines of ash terminal output above the command; the pitch is bottom-anchored and its pill became the install command with a "Copy command" primary pill, a quiet GitHub pill and a link to the install steps; the hero is max(100svh, 880 comp-px) tall so that row stays above the fold at 1440x900; a "Pause demo" control sits bottom right.
Signature: the completion strikes (snaps, never tweens) and the previous glow decays over one beat, as the hero plays real completions: output arrives, the command is typed, the completion strikes and the same name lights in the output.
FORM: Struck Cathode Gauze, a catalog challenger rated competitive in the hand, chosen by the user over the assigned vellum direction. Seed key 8097ae80; approved comp .impeccable/mocks/comp-1.png.
RELAUNCH DECISIONS (made by the designer in place of user questions, for a skeptical senior developer): kept the headline; made the install command the primary CTA; put limits before install; added a full figures table under the readouts; replaced the heading "Measured, not promised." with "Measured on one Mac."; removed hover-pause (it froze the demo for most desktop visitors) in favour of an explicit pause control and off-screen pausing.
FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
