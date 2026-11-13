// wisp landing v2: demo state machine, reveals and copy buttons.
// The HTML carries every demo's settled frame, so the page is complete without this file.
(() => {
  "use strict";

  const root = document.documentElement;
  const reduce = matchMedia("(prefers-reduced-motion: reduce)");
  const status = document.querySelector("[data-status]");
  const motion = !reduce.matches && "IntersectionObserver" in window;

  const announce = (msg) => {
    status.textContent = "";
    requestAnimationFrame(() => { status.textContent = msg; });
  };

  // ---------- copy buttons (independent of motion) ----------

  const copyText = async (text) => {
    try { await navigator.clipboard.writeText(text); return true; }
    catch { return false; }
  };
  const flash = (btn) => {
    btn.classList.add("is-copied");
    clearTimeout(btn._t);
    btn._t = setTimeout(() => btn.classList.remove("is-copied"), 1600);
  };
  const rows = [...document.querySelectorAll(".cmds code")].map((c) => c.textContent);

  document.querySelectorAll("[data-copy]").forEach((btn) => {
    btn.hidden = false;
    btn.addEventListener("click", async () => {
      const ok = await copyText(btn.parentElement.querySelector("code").textContent);
      if (ok) { flash(btn); announce("Copied to clipboard."); }
      else announce("Copy failed. Select the command and copy it by hand.");
    });
  });
  const all = document.querySelector("[data-copy-all]");
  if (all) {
    all.hidden = false;
    all.addEventListener("click", async () => {
      const ok = await copyText(rows.slice(0, 4).join("\n") + "\n");
      if (ok) {
        all.textContent = "Copied all four";
        clearTimeout(all._t);
        all._t = setTimeout(() => { all.textContent = "Copy all four commands"; }, 1800);
        announce("Copied all four install commands to clipboard.");
      } else announce("Copy failed. Select the commands and copy them by hand.");
    });
  }

  if (!motion) return;
  root.classList.add("motion");

  // ---------- timing helpers ----------

  const wait = (ms, signal) => new Promise((resolve, reject) => {
    if (signal.aborted) return reject(signal.reason);
    const t = setTimeout(resolve, ms);
    signal.addEventListener("abort", () => { clearTimeout(t); reject(signal.reason); }, { once: true });
  });
  const jitter = () => 55 + Math.round((Math.random() * 2 - 1) * 20);

  // ---------- Scene: one terminal frame ----------
  // phases: empty -> out -> typing -> ghost -> key -> accept (settled)

  class Scene {
    constructor(el) {
      this.el = el;
      this.typed = el.querySelector(".typed");
      this.text = this.typed.textContent;
      this.hasSrc = !!el.querySelector(".src");
      this.accepted = false;
      el.querySelectorAll(".ln").forEach((ln, i) => ln.style.setProperty("--i", i));
    }
    set(cls, on = true) { this.el.classList.toggle(cls, on); }

    // Jump to the starting frame with no transitions. Output stays when keepOut is set.
    clear(keepOut = false) {
      const el = this.el;
      el.classList.add("no-tx");
      el.classList.toggle("has-out", keepOut);
      el.classList.remove("has-ghost", "has-key", "has-accept", "has-lit", "is-typing", "is-press");
      this.typed.textContent = "";
      this.accepted = false;
      void el.offsetWidth;
      el.classList.remove("no-tx");
    }

    // Settle to the end frame (accepted line, source lit), with transitions. Interrupt target.
    settle() {
      this.set("is-typing", false);
      this.set("is-press", false);
      this.typed.textContent = this.text;
      ["has-out", "has-ghost", "has-key", "has-accept"].forEach((c) => this.set(c));
      this.set("has-lit", this.hasSrc);
      this.accepted = true;
    }

    async play(signal, pace = 1, keepOut = false) {
      const lines = this.el.querySelectorAll(".ln").length;
      this.clear(keepOut);
      await wait(40, signal);
      if (!keepOut) {
        this.set("has-out");
        await wait((lines * 40 + 520) * pace, signal);
      } else await wait(360 * pace, signal);

      if (this.text) {
        this.set("is-typing");
        for (let i = 1; i <= this.text.length; i++) {
          this.typed.textContent = this.text.slice(0, i);
          await wait(jitter(), signal);
        }
        this.set("is-typing", false);
        await wait(140, signal); // 60 ms pause + ~80 ms cached model latency
      } else {
        await wait(900 * pace, signal); // empty prompt: the caret idles, then the prediction lands
      }

      this.set("has-ghost");
      await wait(720 * pace, signal);
      this.set("has-key");
      await wait(420 * pace, signal);
      this.set("is-press");
      await wait(90, signal);
      this.set("is-press", false);
      this.accepted = true;
      this.set("has-accept");
      if (this.hasSrc) this.set("has-lit");
    }
  }

  // ---------- Runner: one abortable timeline per demo ----------

  let paused = false;
  const runners = [];

  class Runner {
    constructor(script) { this.script = script; this.ctl = null; this.wanted = false; }
    get running() { return !!this.ctl; }
    want(on) { this.wanted = on; this.sync(); }
    sync() {
      const go = this.wanted && !paused && !document.hidden;
      if (go && !this.ctl) this.start();
      else if (!go && this.ctl) this.stop();
      else if (paused && !this.ctl) this.script.interrupt(); // paused before it ever played: show the settled frame
    }
    async start() {
      const ctl = (this.ctl = new AbortController());
      try { await this.script.run(ctl.signal); }
      catch (e) { if (!ctl.signal.aborted) throw e; }
      if (this.ctl === ctl) this.ctl = null;
    }
    stop() {
      const ctl = this.ctl;
      this.ctl = null;
      ctl.abort();
      this.script.interrupt();
    }
  }
  const runner = (script) => { const r = new Runner(script); runners.push(r); return r; };

  // Hero: loops through its scenes forever (while in view and not paused).
  const loopEl = document.querySelector("[data-loop]");
  if (loopEl) {
    const scenes = [...loopEl.querySelectorAll("[data-scene]")].map((el) => new Scene(el));
    let idx = 0;
    const show = (i) => scenes.forEach((s, j) => s.set("is-off", j !== i));
    // first paint: start from an empty terminal, not from the settled frame
    scenes.forEach((s) => s.clear());
    show(0);
    let fresh = true;

    const hero = runner({
      async run(signal) {
        for (;;) {
          const s = scenes[idx];
          if (!fresh) { // fade the settled frame away before replaying
            s.set("is-off");
            await wait(420, signal);
          }
          fresh = false;
          s.clear();
          show(idx);
          await s.play(signal);
          await wait(1800, signal);
          s.set("is-off");
          await wait(420, signal);
          idx = (idx + 1) % scenes.length;
          s.clear();
          fresh = true;
        }
      },
      interrupt() {
        const s = scenes[idx];
        show(idx);
        s.settle();
        fresh = false;
      },
    });

    new IntersectionObserver(([e]) => {
      if (e.intersectionRatio >= 0.5) hero.want(true);
      else if (!e.isIntersecting) hero.want(false);
    }, { threshold: [0, 0.5] }).observe(loopEl);
  }

  // One-shot scene: plays once when shown, then stays settled.
  // Its output is already on screen, so a waiting panel is never an empty box.
  const oneShot = (el, pace = 1) => {
    const s = new Scene(el);
    s.clear(true);
    let done = false;
    return runner({
      async run(signal) {
        if (done) return;
        await s.play(signal, pace, true);
        done = true;
      },
      interrupt() { s.settle(); done = true; },
    });
  };

  // S2: sticky stage on desktop, stacked one-shots otherwise.
  const reads = document.querySelector("[data-reads]");
  if (reads) {
    const shots = [...reads.querySelectorAll(".shot")];
    const shotRunners = shots.map((f) => oneShot(f.querySelector("[data-scene]"), 0.7));
    const pinned = matchMedia("(min-width: 960px)");
    let active = 0, inStage = false;

    const apply = () => {
      shotRunners.forEach((r, i) => r.want(pinned.matches ? inStage && i === active : r._visible));
    };
    const setActive = (i) => {
      if (i === active && reads.dataset.active) return;
      active = i;
      reads.dataset.active = String(i);
      apply();
    };
    reads.dataset.active = "0";

    const centre = new IntersectionObserver((entries) => {
      entries.forEach((e) => { if (e.isIntersecting) setActive(+e.target.dataset.marker); });
    }, { rootMargin: "-50% 0px -50% 0px" });
    reads.querySelectorAll("[data-marker]").forEach((m) => centre.observe(m));

    new IntersectionObserver(([e]) => { inStage = e.isIntersecting; apply(); }, { threshold: 0.25 })
      .observe(reads.querySelector(".reads-shots"));

    const each = new IntersectionObserver((entries) => {
      entries.forEach((e) => {
        const r = shotRunners[shots.indexOf(e.target)];
        if (e.intersectionRatio >= 0.35) r._visible = true;
        else if (!e.isIntersecting) r._visible = false;
      });
      apply();
    }, { threshold: [0, 0.35] });
    shots.forEach((f) => each.observe(f));
    pinned.addEventListener("change", apply);
  }

  // S3: each compact demo plays once when it is well in view.
  document.querySelectorAll("[data-once]").forEach((fig) => {
    const r = oneShot(fig.querySelector("[data-scene]"));
    new IntersectionObserver(([e]) => {
      if (e.intersectionRatio >= 0.35) r.want(true);
      else if (!e.isIntersecting) r.want(false);
    }, { threshold: [0, 0.35] }).observe(fig);
  });

  // ---------- Pause / Play and hidden tabs ----------

  const toggle = document.querySelector("[data-toggle]");
  if (toggle) {
    toggle.hidden = false;
    toggle.addEventListener("click", () => {
      paused = !paused;
      root.classList.toggle("is-paused", paused);
      toggle.setAttribute("aria-pressed", String(paused));
      toggle.textContent = paused ? "Play demos" : "Pause demos";
      runners.forEach((r) => r.sync());
    });
  }
  document.addEventListener("visibilitychange", () => runners.forEach((r) => r.sync()));

  // ---------- S5 statement: line-by-line reveal, once ----------

  const lines = document.querySelector("[data-lines]");
  if (lines) {
    new IntersectionObserver(([e], io) => {
      if (!e.isIntersecting) return;
      lines.classList.add("is-in");
      io.disconnect();
    }, { threshold: 0.2 }).observe(lines);
  }

  reduce.addEventListener("change", () => { if (reduce.matches) location.reload(); });
})();
