(() => {
  const root = document.documentElement;
  const toggle = document.getElementById("theme-toggle");
  const KEY = "mago-theme";

  if (toggle) {
    toggle.addEventListener("click", () => {
      const current = root.dataset.theme === "dark"
        ? "dark"
        : root.dataset.theme === "light"
          ? "light"
          : (matchMedia && matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
      const next = current === "dark" ? "light" : "dark";
      root.dataset.theme = next;
      try { localStorage.setItem(KEY, next); } catch {}
    });
  }

  if (new URLSearchParams(location.search).get("untranslated") === "1") {
    const banner = document.getElementById("untranslated-banner");
    if (banner) banner.hidden = false;
  }
})();
