(() => {
  const allPres = document.querySelectorAll(".doc-page pre");
  if (!allPres.length) return;

  const decorate = (pre) => {
    if (pre.dataset.codeReady) return;
    const code = pre.querySelector("code");
    if (!code) return;

    const languageClass = (code.className.match(/language-([\w-]+)/) || [])[1];
    if (languageClass && languageClass !== "plaintext") {
      pre.dataset.lang = languageClass;
    }

    pre.dataset.codeReady = "1";
  };

  allPres.forEach(decorate);

  const blocks = document.querySelectorAll('pre code[class*="language-"]');
  if (!blocks.length) return;

  const HLJS_BASE = "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0";

  const EXTRA_LANGUAGES = ["toml", "nginx", "dockerfile"];

  const loadScript = (src) =>
    new Promise((resolve, reject) => {
      const script = document.createElement("script");
      script.src = src;
      script.async = true;
      script.onload = () => resolve();
      script.onerror = () => reject(new Error(`failed to load ${src}`));
      document.head.appendChild(script);
    });

  (async () => {
    try {
      await loadScript(`${HLJS_BASE}/highlight.min.js`);
      await Promise.all(
        EXTRA_LANGUAGES.map((language) =>
          loadScript(`${HLJS_BASE}/languages/${language}.min.js`).catch(() => {
          })
        )
      );

      hljs.configure({ ignoreUnescapedHTML: true });
      blocks.forEach((block) => {
        hljs.highlightElement(block);
      });
    } catch (error) {
      console.warn("highlight.js failed to load:", error);
    }
  })();
})();
