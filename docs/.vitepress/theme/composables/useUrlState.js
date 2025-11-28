import { ref } from 'vue';

const MAX_URL_LENGTH = 2000;

async function compressState(state) {
  const json = JSON.stringify(state);
  const stream = new Blob([json])
    .stream()
    .pipeThrough(new CompressionStream('gzip'));
  const compressed = await new Response(stream).arrayBuffer();

  return btoa(String.fromCharCode(...new Uint8Array(compressed)))
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
}

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

  async function generateShareUrl(state) {
    shareError.value = null;
    shareSuccess.value = false;

    try {
      const encoded = await compressState(state);
      const url = `${window.location.origin}${window.location.pathname}#${encoded}`;

      if (url.length > MAX_URL_LENGTH) {
        throw new Error(
          `Code is too long to share (${url.length} characters). ` +
          `Please reduce the code size to share via URL.`
        );
      }

      return url;
    } catch (e) {
      shareError.value = e.message || 'Failed to generate share URL';
      throw e;
    }
  }

  async function loadFromUrl() {
    const hash = window.location.hash.slice(1);
    if (!hash) return null;

    try {
      return await decompressState(hash);
    } catch (e) {
      console.warn('Failed to load state from URL:', e);
      return null;
    }
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
    generateShareUrl,
    loadFromUrl,
    copyToClipboard,
  };
}
