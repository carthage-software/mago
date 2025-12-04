import { ref } from 'vue';

const API_BASE_URL = 'https://carthage.software/api/playground';

const UUID_REGEX = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

async function decompressState(encoded) {
  let base64 = encoded.replace(/-/g, '+').replace(/_/g, '/');
  while (base64.length % 4) base64 += '=';

  const binary = atob(base64);
  const bytes = Uint8Array.from(binary, (c) => c.charCodeAt(0));

  const stream = new Blob([bytes])
    .stream()
    .pipeThrough(new DecompressionStream('gzip'));

  const text = await new Response(stream).text();
  return JSON.parse(text);
}

export function useUrlState() {
  const shareError = ref(null);
  const shareSuccess = ref(false);
  const isSharing = ref(false);
  const shareUrl = ref(null);

  let lastStateHash = null;

  async function generateShareUrl(state) {
    const stateHash = JSON.stringify(state);

    if (shareUrl.value && lastStateHash === stateHash) {
      return shareUrl.value;
    }

    shareError.value = null;
    shareSuccess.value = false;
    isSharing.value = true;

    try {
      const [response] = await Promise.all([
        fetch(`${API_BASE_URL}/share`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ state }),
        }),
        new Promise((r) => setTimeout(r, 500)),
      ]);

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Failed to create share link');
      }

      const { uuid } = await response.json();
      shareUrl.value = `${window.location.origin}${window.location.pathname}#${uuid}`;
      lastStateHash = stateHash;

      window.history.replaceState(null, '', `#${uuid}`);

      return shareUrl.value;
    } catch (e) {
      shareError.value = e.message || 'Failed to generate share URL';
      throw e;
    } finally {
      isSharing.value = false;
    }
  }

  function clearShareUrl() {
    shareUrl.value = null;
    lastStateHash = null;

    if (window.location.hash) {
      window.history.replaceState(null, '', window.location.pathname);
    }
  }

  async function loadFromUrl() {
    const hash = window.location.hash.slice(1);
    if (!hash) return null;

    try {
      let state;
      if (UUID_REGEX.test(hash)) {
        const response = await fetch(`${API_BASE_URL}/share/${hash}`);
        if (!response.ok) {
          console.warn('Failed to load shared state:', response.status);
          return null;
        }
        const data = await response.json();
        state = data.state;
      } else {
        state = await decompressState(hash);
      }

      if (state) {
        shareUrl.value = window.location.href;
        lastStateHash = JSON.stringify(state);
      }

      return state;
    } catch (e) {
      console.warn('Failed to load state from URL:', e);
      return null;
    }
  }

  function shouldClearShareUrl(currentState) {
    if (!shareUrl.value) return false;
    const currentHash = JSON.stringify(currentState);
    return currentHash !== lastStateHash;
  }

  async function copyToClipboard(url) {
    try {
      await navigator.clipboard.writeText(url);
      shareSuccess.value = true;
      setTimeout(() => {
        shareSuccess.value = false;
      }, 2000);
      return true;
    } catch (e) {
      shareError.value = 'Failed to copy to clipboard';
      return false;
    }
  }

  return {
    shareError,
    shareSuccess,
    isSharing,
    shareUrl,
    generateShareUrl,
    loadFromUrl,
    copyToClipboard,
    clearShareUrl,
    shouldClearShareUrl,
  };
}
