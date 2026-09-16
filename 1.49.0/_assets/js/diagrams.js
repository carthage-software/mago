(() => {
  "use strict";

  const mermaidApi = window.mermaid;
  const sources = [...document.querySelectorAll("pre > code.language-mermaid")];
  if (!mermaidApi || sources.length === 0) return;

  const icons = {
    in: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 3v10M3 8h10"/></svg>',
    out: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8h10"/></svg>',
    reset:
      '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3.5 5.5V2.5M3.5 2.5h3M3.7 3.2a5.5 5.5 0 1 1-1 6.3"/></svg>',
    full: '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M2.5 6V2.5H6M10 2.5h3.5V6M13.5 10v3.5H10M6 13.5H2.5V10"/></svg>',
  };

  const isDark = () => {
    const selected = document.documentElement.dataset.theme;
    if (selected) return selected === "dark";
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  };

  const themeVariables = () =>
    isDark()
      ? {
          background: "#0a0a0a",
          primaryColor: "#161616",
          primaryTextColor: "#fafafa",
          primaryBorderColor: "#a0a0a0",
          secondaryColor: "#17351f",
          secondaryTextColor: "#fafafa",
          secondaryBorderColor: "#5fb677",
          tertiaryColor: "#1f1f1f",
          tertiaryTextColor: "#fafafa",
          tertiaryBorderColor: "#777777",
          lineColor: "#a0a0a0",
          clusterBkg: "#101010",
          clusterBorder: "#777777",
          noteBkgColor: "#1f1f1f",
          noteTextColor: "#fafafa",
          noteBorderColor: "#777777",
          actorBkg: "#161616",
          actorBorder: "#a0a0a0",
          actorTextColor: "#fafafa",
          signalColor: "#a0a0a0",
          signalTextColor: "#fafafa",
          labelBoxBkgColor: "#161616",
          labelBoxBorderColor: "#777777",
          labelTextColor: "#fafafa",
          loopTextColor: "#fafafa",
        }
      : {
          background: "#ffffff",
          primaryColor: "#f5f5f5",
          primaryTextColor: "#000000",
          primaryBorderColor: "#666666",
          secondaryColor: "#e8f3ec",
          secondaryTextColor: "#000000",
          secondaryBorderColor: "#3c8d58",
          tertiaryColor: "#ffffff",
          tertiaryTextColor: "#000000",
          tertiaryBorderColor: "#8c8c8c",
          lineColor: "#666666",
          clusterBkg: "#fafafa",
          clusterBorder: "#8c8c8c",
          noteBkgColor: "#ebebeb",
          noteTextColor: "#000000",
          noteBorderColor: "#8c8c8c",
          actorBkg: "#f5f5f5",
          actorBorder: "#666666",
          actorTextColor: "#000000",
          signalColor: "#666666",
          signalTextColor: "#000000",
          labelBoxBkgColor: "#f5f5f5",
          labelBoxBorderColor: "#8c8c8c",
          labelTextColor: "#000000",
          loopTextColor: "#000000",
        };

  const configureMermaid = () => {
    mermaidApi.initialize({
      startOnLoad: false,
      securityLevel: "strict",
      theme: "base",
      themeVariables: {
        fontFamily: '"IBM Plex Sans", sans-serif',
        fontSize: "14px",
        ...themeVariables(),
      },
      flowchart: {
        curve: "basis",
        htmlLabels: true,
        nodeSpacing: 36,
        rankSpacing: 54,
        padding: 14,
        useMaxWidth: false,
      },
      sequence: {
        actorMargin: 36,
        boxMargin: 8,
        diagramMarginX: 12,
        diagramMarginY: 24,
        messageMargin: 24,
        useMaxWidth: false,
      },
    });
  };

  const findTitle = (element, index) => {
    let sibling = element.previousElementSibling;
    while (sibling) {
      if (/^H[1-4]$/.test(sibling.tagName)) {
        const heading = sibling.cloneNode(true);
        heading
          .querySelectorAll(".heading-anchor")
          .forEach((anchor) => anchor.remove());
        return heading.textContent?.trim() || `Diagram ${index + 1}`;
      }
      sibling = sibling.previousElementSibling;
    }
    return `Diagram ${index + 1}`;
  };

  const button = (name, icon, action) => {
    const control = document.createElement("button");
    control.className = "doc-diagram__control";
    control.type = "button";
    control.title = name;
    control.setAttribute("aria-label", name);
    control.innerHTML = icon;
    control.addEventListener("click", action);
    return control;
  };

  const createViewer = (source, title, index) => {
    const figure = document.createElement("figure");
    figure.className = "doc-diagram";

    const header = document.createElement("div");
    header.className = "doc-diagram__header";

    const identity = document.createElement("div");
    identity.className = "doc-diagram__identity";

    const kind = document.createElement("span");
    kind.className = "doc-diagram__kind";
    kind.textContent = "System map";

    const caption = document.createElement("figcaption");
    caption.className = "doc-diagram__title";
    caption.id = `doc-diagram-title-${index}`;
    caption.textContent = title;

    const controls = document.createElement("div");
    controls.className = "doc-diagram__controls";

    const viewport = document.createElement("div");
    viewport.className = "doc-diagram__viewport";
    viewport.tabIndex = 0;
    viewport.setAttribute("role", "group");
    viewport.setAttribute("aria-labelledby", caption.id);
    viewport.setAttribute(
      "aria-description",
      "Interactive diagram. Drag or use arrow keys to pan. Use Control or Command and scroll, plus, or minus to zoom. Press zero to reset.",
    );

    const canvas = document.createElement("div");
    canvas.className = "doc-diagram__canvas";
    viewport.appendChild(canvas);

    const footer = document.createElement("div");
    footer.className = "doc-diagram__footer";

    const hint = document.createElement("span");
    hint.textContent =
      "Drag to pan · Ctrl/Cmd + scroll to zoom · Double-click to reset";

    const status = document.createElement("span");
    status.className = "doc-diagram__status";
    status.setAttribute("aria-live", "polite");

    identity.append(kind, caption);
    header.append(identity, controls);
    footer.append(hint, status);
    figure.append(header, viewport, footer);

    const state = {
      canvas,
      figure,
      index,
      naturalHeight: 320,
      naturalWidth: 640,
      scale: 1,
      source,
      status,
      tx: 0,
      ty: 0,
      viewport,
    };

    const apply = () => {
      canvas.style.transform = `translate(${state.tx}px, ${state.ty}px) scale(${state.scale})`;
      status.textContent = `${Math.round(state.scale * 100)}%`;
    };

    const fit = () => {
      const inset = 48;
      const availableWidth = Math.max(1, viewport.clientWidth - inset);
      const availableHeight = Math.max(
        1,
        (figure.classList.contains("doc-diagram--focused")
          ? viewport.clientHeight
          : 620) - inset,
      );
      state.scale = Math.min(
        1,
        availableWidth / state.naturalWidth,
        availableHeight / state.naturalHeight,
      );
      const renderedWidth = state.naturalWidth * state.scale;
      const renderedHeight = state.naturalHeight * state.scale;
      state.tx = Math.max(24, (viewport.clientWidth - renderedWidth) / 2);
      state.ty = 24;
      if (!figure.classList.contains("doc-diagram--focused")) {
        viewport.style.height = `${Math.max(260, Math.min(620, renderedHeight + 48))}px`;
      }
      apply();
    };

    const zoom = (
      factor,
      x = viewport.clientWidth / 2,
      y = viewport.clientHeight / 2,
    ) => {
      const next = Math.min(4, Math.max(0.25, state.scale * factor));
      state.tx = x - (x - state.tx) * (next / state.scale);
      state.ty = y - (y - state.ty) * (next / state.scale);
      state.scale = next;
      apply();
    };

    controls.append(
      button("Zoom in", icons.in, () => zoom(1.25)),
      button("Zoom out", icons.out, () => zoom(0.8)),
      button("Reset view", icons.reset, fit),
    );

    const focusControl = button("Focus diagram", icons.full, () => {
      const focused = figure.classList.toggle("doc-diagram--focused");
      document.documentElement.classList.toggle("has-focused-diagram", focused);
      focusControl.setAttribute("aria-pressed", String(focused));
      focusControl.setAttribute(
        "aria-label",
        focused ? "Exit diagram focus" : "Focus diagram",
      );
      focusControl.title = focused ? "Exit diagram focus" : "Focus diagram";
      requestAnimationFrame(fit);
    });
    focusControl.setAttribute("aria-pressed", "false");
    controls.append(focusControl);

    viewport.addEventListener(
      "wheel",
      (event) => {
        if (!event.ctrlKey && !event.metaKey) return;
        event.preventDefault();
        const rect = viewport.getBoundingClientRect();
        zoom(
          event.deltaY < 0 ? 1.12 : 1 / 1.12,
          event.clientX - rect.left,
          event.clientY - rect.top,
        );
      },
      { passive: false },
    );

    let dragging = false;
    let pointerX = 0;
    let pointerY = 0;
    viewport.addEventListener("pointerdown", (event) => {
      if (event.button !== 0) return;
      dragging = true;
      pointerX = event.clientX;
      pointerY = event.clientY;
      viewport.setPointerCapture(event.pointerId);
      viewport.focus({ preventScroll: true });
    });
    viewport.addEventListener("pointermove", (event) => {
      if (!dragging) return;
      state.tx += event.clientX - pointerX;
      state.ty += event.clientY - pointerY;
      pointerX = event.clientX;
      pointerY = event.clientY;
      apply();
    });
    const stopDragging = () => {
      dragging = false;
    };
    viewport.addEventListener("pointerup", stopDragging);
    viewport.addEventListener("pointercancel", stopDragging);
    viewport.addEventListener("dblclick", fit);
    viewport.addEventListener("keydown", (event) => {
      const distance = event.shiftKey ? 80 : 28;
      if (event.key === "+" || event.key === "=") zoom(1.25);
      else if (event.key === "-") zoom(0.8);
      else if (event.key === "0") fit();
      else if (event.key === "ArrowLeft") state.tx += distance;
      else if (event.key === "ArrowRight") state.tx -= distance;
      else if (event.key === "ArrowUp") state.ty += distance;
      else if (event.key === "ArrowDown") state.ty -= distance;
      else return;
      event.preventDefault();
      apply();
    });

    document.addEventListener("keydown", (event) => {
      if (
        event.key === "Escape" &&
        figure.classList.contains("doc-diagram--focused")
      ) {
        figure.classList.remove("doc-diagram--focused");
        document.documentElement.classList.remove("has-focused-diagram");
        focusControl.setAttribute("aria-pressed", "false");
        focusControl.setAttribute("aria-label", "Focus diagram");
        focusControl.title = "Focus diagram";
        requestAnimationFrame(fit);
      }
    });

    let resizeFrame = 0;
    new ResizeObserver(() => {
      cancelAnimationFrame(resizeFrame);
      resizeFrame = requestAnimationFrame(fit);
    }).observe(viewport);

    state.apply = apply;
    state.fit = fit;
    return state;
  };

  const viewers = sources.map((code, index) => {
    const source = code.textContent.trim();
    const pre = code.parentElement;
    const viewer = createViewer(source, findTitle(pre, index), index);
    pre.replaceWith(viewer.figure);
    return viewer;
  });

  let renderGeneration = 0;
  const render = async () => {
    const generation = ++renderGeneration;
    configureMermaid();
    for (const viewer of viewers) {
      try {
        const id = `mago-diagram-${generation}-${viewer.index}`;
        const result = await mermaidApi.render(id, viewer.source);
        if (generation !== renderGeneration) return;
        viewer.canvas.innerHTML = result.svg;
        result.bindFunctions?.(viewer.canvas);
        const svg = viewer.canvas.querySelector("svg");
        if (!svg) throw new Error("Mermaid returned no SVG element.");
        const viewBox = svg.viewBox?.baseVal;
        viewer.naturalWidth =
          viewBox?.width || Number.parseFloat(svg.getAttribute("width")) || 640;
        viewer.naturalHeight =
          viewBox?.height ||
          Number.parseFloat(svg.getAttribute("height")) ||
          320;
        svg.setAttribute("width", String(viewer.naturalWidth));
        svg.setAttribute("height", String(viewer.naturalHeight));
        svg.setAttribute("role", "img");
        svg.setAttribute(
          "aria-labelledby",
          `doc-diagram-title-${viewer.index}`,
        );
        viewer.canvas.style.width = `${viewer.naturalWidth}px`;
        viewer.canvas.style.height = `${viewer.naturalHeight}px`;
        viewer.fit();
      } catch (error) {
        viewer.canvas.textContent = `Diagram could not be rendered: ${error instanceof Error ? error.message : String(error)}`;
        viewer.canvas.style.padding = "24px";
        viewer.status.textContent = "Error";
      }
    }
  };

  let themeFrame = 0;
  const scheduleThemeRender = () => {
    cancelAnimationFrame(themeFrame);
    themeFrame = requestAnimationFrame(render);
  };
  new MutationObserver(scheduleThemeRender).observe(document.documentElement, {
    attributeFilter: ["data-theme"],
    attributes: true,
  });
  window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", () => {
      if (!document.documentElement.dataset.theme) scheduleThemeRender();
    });

  render();
})();
