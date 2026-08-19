(() => {
  const trigger = document.getElementById("search-trigger");
  const root = document.getElementById("search-root");
  if (!trigger || !root) return;

  if (location.protocol === "file:") {
    trigger.style.display = "none";
    return;
  }

  const firstSegment = location.pathname.split("/").filter(Boolean)[0] || "";
  const base = firstSegment ? `/${firstSegment}` : "";

  let initialised = false;

  const open = async () => {
    if (!initialised) {
      const script = document.createElement("script");
      script.src = `${base}/pagefind/pagefind-ui.js`;
      await new Promise((resolve, reject) => {
        script.onload = resolve;
        script.onerror = reject;
        document.head.appendChild(script);
      });

      const link = document.createElement("link");
      link.rel = "stylesheet";
      link.href = `${base}/pagefind/pagefind-ui.css`;
      document.head.appendChild(link);

      root.innerHTML = `
        <div class="search-modal">
          <div id="pagefind-mount"></div>
          <p class="search-help">Type to search the documentation. Press <kbd>Esc</kbd> to close.</p>
        </div>
      `;

      new PagefindUI({
        element: "#pagefind-mount",
        showImages: false,
        showSubResults: true,
        autofocus: true,
        translations: {
          placeholder: "Search the documentation",
          zero_results: "No results for `[SEARCH_TERM]`",
          many_results: "[COUNT] results for `[SEARCH_TERM]`",
          one_result: "1 result for `[SEARCH_TERM]`",
        },
      });

      initialised = true;
    }
    root.hidden = false;
  };

  const close = () => {
    root.hidden = true;
  };

  trigger.addEventListener("click", open);

  root.addEventListener("click", (event) => {
    if (event.target === root) close();
  });

  document.addEventListener("keydown", (event) => {
    if (event.key === "/" && !event.metaKey && !event.ctrlKey && !event.altKey) {
      const target = event.target;
      const tag = (target && target.tagName) || "";
      if (tag === "INPUT" || tag === "TEXTAREA") return;
      if (target && typeof target.closest === "function" && target.closest(".cm-editor, [contenteditable=\"true\"]")) return;
      event.preventDefault();
      open();
    }
    if (event.key === "Escape") close();
  });
})();
