// Scenarios from wisp's eval. `completion` is what Qwen2.5-Coder produced; `output` is the
// fixture the model saw above the prompt, with `source` marking where the name came from.
// `seen` is the same fixture cut down to the lines and columns that fit above the hero prompt.
const SCENARIOS = {
  kubectl: {
    output: [
      "$ kubectl get pods -n billing",
      "NAME                        READY   STATUS             RESTARTS",
      "api-7d9f8b6c4-x2kqp         0/1     CrashLoopBackOff   12",
      "worker-5c8d7f9b6-mn4lz      1/1     Running            0",
    ],
    seen: [
      "$ kubectl get pods -n billing",
      "api-7d9f8b6c4-x2kqp      0/1  CrashLoopBackOff",
      "worker-5c8d7f9b6-mn4lz   1/1  Running",
    ],
    source: "api-7d9f8b6c4-x2kqp",
    typed: "kubectl logs -n billing api",
    completion: "-7d9f8b6c4-x2kqp",
    ghosts: ["-worker-5c8d", "-gateway", "-api-v2"],
  },
  compose: {
    output: ["$ ls", "README.md  docker-compose.prod.yml  docker-compose.yml  src/  scripts/"],
    seen: ["$ ls", "README.md  docker-compose.prod.yml", "docker-compose.yml  scripts/  src/"],
    source: "docker-compose.prod.yml",
    typed: "docker compose -f docker-compose.p",
    completion: "rod.yml up -d",
    ghosts: ["docker-compose.yml", "README.md", "scripts/"],
  },
  git: {
    output: [
      "$ git branch -a",
      "  main",
      "* develop",
      "  feat/payments-refactor",
      "  remotes/origin/feat/payments-refactor",
    ],
    seen: ["$ git branch -a", "* develop", "  feat/payments-refactor"],
    source: "feat/payments-refactor",
    typed: "git checkout feat/p",
    completion: "ayments-refactor",
    ghosts: ["main", "develop", "origin/feat/payments-refactor"],
  },
  trace: {
    output: [
      "$ python -m app",
      "Traceback (most recent call last):",
      '  File "/srv/app/src/billing/invoice_service.py", line 142, in total',
      "    return sum(l.amount for l in lines)",
      "TypeError: unsupported operand type(s)",
    ],
    seen: [
      "$ python -m app",
      'File "/srv/app/src/billing/invoice_service.py"',
      "TypeError: unsupported operand type(s)",
    ],
    source: "src/billing/invoice_service.py",
    typed: "nvim src/b",
    completion: "illing/invoice_service.py",
    ghosts: ["src/", "scripts/", "README.md"],
  },
  lsof: {
    output: [
      "$ lsof -i :3000",
      "COMMAND   PID USER   FD   TYPE DEVICE SIZE/OFF NODE NAME",
      "node    48213 dev    23u  IPv6 0x1a2b      0t0  TCP *:hbci (LISTEN)",
    ],
    seen: ["$ lsof -i :3000", "COMMAND   PID USER   NAME", "node    48213 dev    TCP *:hbci (LISTEN)"],
    source: "48213",
    typed: "kill ",
    completion: "48213",
    ghosts: ["node", "3000", "-9"],
  },
  docker: {
    output: [
      "$ docker ps",
      "CONTAINER ID   IMAGE         NAMES",
      "3f2a9c1b7e4d   postgres:16   pgl-db-1",
      "9b8e7d6c5a4f   redis:7       pgl-cache-1",
    ],
    seen: ["$ docker ps", "postgres:16   pgl-db-1", "redis:7       pgl-cache-1"],
    source: "pgl-db-1",
    typed: "docker exec -it pgl-d",
    completion: "b-1 bash",
    ghosts: ["pgl-cache-1", "postgres:16", "redis:7"],
  },
};

const ORDER = Object.keys(SCENARIOS);
// First paint (and any no-JS or headless capture) is the finished first example.
const FIRST_HOLD_MS = 4200;
const HOLD_MS = 3000;
const KEY_MS = 55;
// The real debounce is 60 ms; the demo waits a readable beat before the strike.
const STRIKE_DELAY_MS = 420;
const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

document.documentElement.classList.add("js");

function escapeHtml(text) {
  return text.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

// Marks the first occurrence of the name, where wisp found it.
function markSource(lines, source) {
  const name = escapeHtml(source);
  let marked = false;
  return lines
    .map(escapeHtml)
    .map((line) => {
      if (marked || !line.includes(name)) return line;
      marked = true;
      return line.replace(name, `<mark>${name}</mark>`);
    })
    .join("\n");
}

function setupHeroDemo() {
  const tube = document.querySelector("[data-cycle]");
  if (!tube) return;
  const toggle = tube.querySelector("[data-demo-toggle]");
  if (reducedMotion) {
    toggle.hidden = true;
    return;
  }
  const seen = tube.querySelector("[data-seen]");
  const typed = tube.querySelector("[data-typed]");
  const stack = tube.querySelector("[data-stack]");
  const ghosts = tube.querySelector("[data-ghosts]");
  const struck = tube.querySelector("[data-struck]");
  const afterglow = tube.querySelector("[data-afterglow]");
  const hero = tube.closest(".hero");
  let userPaused = false;
  let visible = true;

  const running = () => !userPaused && visible && !document.hidden;
  const sleep = (ms) => new Promise((resolve) => window.setTimeout(resolve, ms));
  // Waits `ms` of running time; a pause holds the demo on the current frame.
  const beat = async (ms) => {
    await sleep(ms);
    while (!running()) await sleep(200);
  };

  const play = async (name) => {
    const scenario = SCENARIOS[name];
    // The outgoing completion decays while the next screen arrives.
    afterglow.textContent = struck.textContent;
    afterglow.classList.remove("decaying");
    void afterglow.offsetWidth;
    afterglow.classList.add("decaying");
    stack.classList.add("unstruck");
    seen.innerHTML = markSource(scenario.seen, scenario.source);
    typed.textContent = "";
    ghosts.innerHTML = scenario.ghosts.map((g) => `<li>${escapeHtml(g)}</li>`).join("");
    struck.textContent = scenario.completion;
    await beat(700);
    for (let i = 1; i <= scenario.typed.length; i += 1) {
      typed.textContent = scenario.typed.slice(0, i);
      await beat(KEY_MS);
    }
    await beat(STRIKE_DELAY_MS);
    // Strike: the completion snaps in and the name on screen lights with it.
    stack.classList.remove("unstruck");
    seen.querySelector("mark")?.classList.add("lit");
  };

  const loop = async () => {
    await beat(FIRST_HOLD_MS);
    for (let index = 1; ; index = (index + 1) % ORDER.length) {
      await play(ORDER[index]);
      await beat(HOLD_MS);
    }
  };

  toggle.addEventListener("click", () => {
    userPaused = !userPaused;
    toggle.setAttribute("aria-pressed", String(userPaused));
    toggle.textContent = userPaused ? "Play demo" : "Pause demo";
  });
  if ("IntersectionObserver" in window) {
    new IntersectionObserver(([entry]) => { visible = entry.isIntersecting; }, { threshold: 0.2 }).observe(hero);
  }
  loop();
}

function setupScreen() {
  const screen = document.querySelector("[data-screen]");
  if (!screen) return;
  const tabs = [...screen.querySelectorAll("[data-scenario]")];
  const panel = screen.querySelector("[data-panel]");
  const output = screen.querySelector("[data-output]");
  const typed = screen.querySelector("[data-prompt-typed]");
  const completion = screen.querySelector("[data-prompt-completion]");

  const show = (name) => {
    const scenario = SCENARIOS[name];
    output.innerHTML = markSource(scenario.output, scenario.source);
    typed.textContent = scenario.typed;
    completion.textContent = scenario.completion;
    for (const tab of tabs) {
      const selected = tab.dataset.scenario === name;
      tab.setAttribute("aria-selected", String(selected));
      tab.tabIndex = selected ? 0 : -1;
      if (selected) panel.setAttribute("aria-labelledby", tab.id);
    }
  };

  tabs.forEach((tab, i) => {
    tab.addEventListener("click", () => show(tab.dataset.scenario));
    tab.addEventListener("keydown", (event) => {
      const target = {
        ArrowRight: (i + 1) % tabs.length,
        ArrowLeft: (i - 1 + tabs.length) % tabs.length,
        Home: 0,
        End: tabs.length - 1,
      }[event.key];
      if (target === undefined) return;
      event.preventDefault();
      tabs[target].focus();
      show(tabs[target].dataset.scenario);
    });
  });
}

function setupCopy() {
  const status = document.querySelector("[data-status]");
  for (const button of document.querySelectorAll("[data-copy]")) {
    const label = button.textContent;
    const code = button.closest(".cmd").querySelector("code");
    let reset;
    button.addEventListener("click", async () => {
      let message = "Copied";
      try {
        await navigator.clipboard.writeText(code.textContent);
      } catch {
        // No clipboard access: select the command so one keystroke copies it.
        const range = document.createRange();
        range.selectNodeContents(code);
        const selection = window.getSelection();
        selection.removeAllRanges();
        selection.addRange(range);
        message = "Selected";
      }
      button.textContent = message;
      button.classList.add("copied");
      if (status) status.textContent = `${message}: ${code.textContent}`;
      window.clearTimeout(reset);
      reset = window.setTimeout(() => {
        button.textContent = label;
        button.classList.remove("copied");
        if (status) status.textContent = "";
      }, 1600);
    });
  }
}

setupHeroDemo();
setupScreen();
setupCopy();
