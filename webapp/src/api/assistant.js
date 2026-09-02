import { API_BASE } from './client.js';

/**
 * Streams chat completion chunks from the local Gemma 4 assistant API.
 * 
 * @param {Object} payload - { messages: Array<{role: string, content: string}>, city?: string }
 * @param {(chunk: string) => void} onChunk - Callback for received text tokens
 * @param {() => void} onDone - Callback when streaming completes
 * @param {(err: Error) => void} onError - Callback on network or response error
 * @returns {() => void} abort - Function to abort the active stream
 */
export function streamAssistant(payload, onChunk, onDone, onError) {
  const controller = new AbortController();

  (async () => {
    try {
      const res = await fetch(`${API_BASE}/assistant`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'text/event-stream'
        },
        body: JSON.stringify(payload),
        signal: controller.signal
      });

      if (!res.ok) {
        throw new Error(`Asistan API hatası: ${res.status}`);
      }

      if (!res.body) {
        throw new Error('Yanıt akışı açılamadı.');
      }

      const reader = res.body.getReader();
      const decoder = new TextDecoder('utf-8');
      let buffer = '';

      while (true) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          if (line.startsWith('data:')) {
            let dataStr = line.slice(5).trim();
            if (dataStr === '[DONE]') {
              onDone();
              return;
            }
            if (dataStr) {
              try {
                const parsed = JSON.parse(dataStr);
                onChunk(parsed);
              } catch {
                onChunk({ type: 'content', text: dataStr });
              }
            }
          }
        }
      }

      onDone();
    } catch (err) {
      if (err.name === 'AbortError') {
        onDone();
        return;
      }
      onError(err);
    }
  })();

  return () => controller.abort();
}
