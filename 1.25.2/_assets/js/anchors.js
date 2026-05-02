(() => {
  const article = document.querySelector(".doc-page");
  if (!article) return;

  article.querySelectorAll("summary .rule__anchor").forEach((anchor) => {
    anchor.addEventListener("click", (event) => event.stopPropagation());
  });

  const openAncestorDetailsFor = (id) => {
    if (!id) return null;
    const target = document.getElementById(id);
    if (!target) return null;
    let cursor = target;
    while (cursor) {
      if (cursor instanceof HTMLDetailsElement) {
        cursor.open = true;
      }

      cursor = cursor.parentElement;
    }

    return target;
  };

  const revealHashTarget = () => {
    const id = decodeURIComponent(location.hash.slice(1));
    const target = openAncestorDetailsFor(id);
    if (target) {
      requestAnimationFrame(() =>
        target.scrollIntoView({ block: "start", behavior: "auto" }),
      );
    }
  };

  revealHashTarget();
  window.addEventListener("hashchange", revealHashTarget);

  if (article.classList.contains("home-article")) {
    article.querySelectorAll(".home-section__head").forEach((head) => {
      const num = head.querySelector(".home-section__num");
      const heading = head.querySelector("h2[id]");
      if (!num || !heading) return;

      const anchor = document.createElement("a");
      anchor.className = "home-section__num-link";
      anchor.href = "#" + heading.id;
      while (num.firstChild) anchor.appendChild(num.firstChild);
      num.appendChild(anchor);
    });
    return;
  }

  const headings = article.querySelectorAll("h2[id], h3[id]");
  let h2Counter = 0;
  let h3Counter = 0;

  headings.forEach((heading) => {
    if (heading.tagName === "H2") {
      h2Counter += 1;
      h3Counter = 0;
    } else {
      h3Counter += 1;
    }

    const label =
      heading.tagName === "H2"
        ? "§ " + String(h2Counter).padStart(2, "0")
        : "§ " + String(h2Counter).padStart(2, "0") + "." + h3Counter;

    const anchor = document.createElement("a");
    anchor.className = "heading-anchor";
    anchor.href = "#" + heading.id;
    anchor.textContent = label;
    heading.insertBefore(anchor, heading.firstChild);
  });

  const toc = document.querySelector(".doc-toc");
  if (toc && headings.length > 0 && "IntersectionObserver" in window) {
    const linkById = new Map();
    toc.querySelectorAll(".doc-toc__link").forEach((link) => {
      const href = link.getAttribute("href");
      if (href && href.startsWith("#")) {
        linkById.set(href.slice(1), link);
      }
    });

    const visible = new Set();
    let activeId = null;

    const setActive = (id) => {
      if (id === activeId) return;
      if (activeId) {
        const previous = linkById.get(activeId);
        if (previous) previous.classList.remove("is-active");
      }
      activeId = id;
      if (activeId) {
        const link = linkById.get(activeId);
        if (link) link.classList.add("is-active");
      }
    };

    const pickActive = () => {
      if (visible.size === 0) return;
      let pick = null;
      let pickTop = Infinity;
      visible.forEach((heading) => {
        const top = heading.getBoundingClientRect().top;
        if (top < pickTop) {
          pickTop = top;
          pick = heading;
        }
      });

      if (pick) setActive(pick.id);
    };

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) visible.add(entry.target);
          else visible.delete(entry.target);
        });
        pickActive();
      },
      {
        rootMargin: "-10% 0% -70% 0%",
        threshold: 0,
      },
    );

    headings.forEach((heading) => observer.observe(heading));
  }
})();
