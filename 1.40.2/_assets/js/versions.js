(() => {
  const root = document.querySelector("[data-version-select]");
  if (!root) return;

  const list = root.querySelector("[data-version-list]");
  if (!list) return;

  const currentVersion = root.dataset.currentVersion || "main";
  const currentLang = root.dataset.currentLang || "en";
  const logicalPath = root.dataset.logicalPath || "";
  const pathToRoot = root.dataset.pathToRoot || "";

  const escape = (value) =>
    String(value)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");

  const semverTuple = (id) => {
    const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(id);
    if (!match) return [-1, -1, -1];
    return [Number(match[1]), Number(match[2]), Number(match[3])];
  };

  const compareSemver = (a, b) => {
    const ta = semverTuple(a);
    const tb = semverTuple(b);
    for (let i = 0; i < 3; i++) {
      if (ta[i] !== tb[i]) return ta[i] - tb[i];
    }
    return 0;
  };

  const buildUrl = (versionId) => {
    const base = `${pathToRoot}${versionId}/${currentLang}/`;
    if (versionId === currentVersion && logicalPath) {
      return `${base}${logicalPath}`;
    }
    return base;
  };

  const sources = ["/versions.json", `${pathToRoot}versions.json`];

  const tryFetch = async () => {
    for (const url of sources) {
      try {
        const response = await fetch(url, { cache: "no-cache" });
        if (response.ok) return await response.json();
      } catch {}
    }
    return null;
  };

  const render = (data) => {
    if (!data || !Array.isArray(data.versions)) return;

    const entries = data.versions.filter((v) => v && typeof v.id === "string");
    if (entries.length === 0) return;

    const stable = entries
      .filter((v) => v.stable === true)
      .sort((a, b) => compareSemver(a.id, b.id));
    const latestStable = stable.length > 0 ? stable[stable.length - 1] : null;

    const items = [];
    if (latestStable) {
      items.push({
        id: "latest",
        label: `latest (${latestStable.id})`,
        url: buildUrl("latest"),
        active: false,
      });
    }
    for (const version of entries) {
      items.push({
        id: version.id,
        label: typeof version.label === "string" ? version.label : version.id,
        url: buildUrl(version.id),
        active: version.id === currentVersion,
      });
    }

    list.innerHTML = items
      .map(
        (item) =>
          `<li><a href="${escape(item.url)}"${item.active ? ' aria-current="true"' : ""}><span>${escape(item.label)}</span></a></li>`,
      )
      .join("");
  };

  tryFetch().then(render);
})();
