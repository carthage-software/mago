(() => {
  const root = document.getElementById("home-sponsors");
  if (!root) return;

  const sources = [];
  const explicit = root.dataset.source;
  if (explicit) sources.push(explicit);
  sources.push("/sponsors.json", "../../../sponsors.json");

  const tryFetch = async () => {
    for (const url of sources) {
      try {
        const response = await fetch(url, { cache: "no-cache" });
        if (response.ok) return await response.json();
      } catch {
      }
    }
    return null;
  };

  const escape = (value) => String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");

  const render = (data) => {
    if (!data || !Array.isArray(data.tiers)) {
      root.innerHTML = "";
      return;
    }

    const tiers = data.tiers
      .filter((tier) => Array.isArray(tier.sponsors) && tier.sponsors.length > 0)
      .map((tier) => {
        const grid = tier.dense ? "sponsors__grid sponsors__grid--dense" : "sponsors__grid";
        const size = Number.isFinite(tier.size) ? tier.size : 64;
        const cards = tier.sponsors.map((sponsor) => {
          const avatar = sponsor.avatar ? `${sponsor.avatar}${sponsor.avatar.includes("?") ? "&" : "?"}s=${size * 2}` : "";
          return `<a href="${escape(sponsor.url)}" class="sponsor sponsor--${escape(tier.id)}" target="_blank" rel="noopener"><img src="${escape(avatar)}" alt="${escape(sponsor.name)}" loading="lazy" width="${size}" height="${size}"><span>${escape(sponsor.name)}</span></a>`;
        }).join("");
        return `<div class="sponsors__tier sponsors__tier--${escape(tier.id)}"><div class="sponsors__tier-label">${escape(tier.label)}</div><div class="${grid}">${cards}</div></div>`;
      })
      .join("");

    root.innerHTML = `<div class="sponsors">${tiers}</div>`;
  };

  tryFetch().then(render);
})();
