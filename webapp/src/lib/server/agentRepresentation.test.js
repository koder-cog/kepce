import { describe, it, expect } from 'vitest';
import {
  negotiateMediaType,
  formatRootMarkdown,
  formatRootJson,
  formatJson404,
  ROOT_MEDIA_TYPES,
  ERROR_MEDIA_TYPES
} from './agentRepresentation.js';

describe('agentRepresentation - RFC 9110 Content Negotiation', () => {
  it('AgentGrade Prob 1: text/markdown istemci önceliğinde markdown seçilmelidir', () => {
    const req = new Request('https://kepce.org/', {
      headers: { Accept: 'text/markdown, text/html, */*' }
    });
    expect(negotiateMediaType(req, ROOT_MEDIA_TYPES)).toBe('text/markdown');
  });

  it('AgentGrade Prob 2: text/html istemci önceliğinde html seçilmelidir', () => {
    const req = new Request('https://kepce.org/', {
      headers: { Accept: 'text/html, text/markdown, */*' }
    });
    expect(negotiateMediaType(req, ROOT_MEDIA_TYPES)).toBe('text/html');
  });

  it('AgentGrade Prob 3: q-değeri üstünlüğünde (html q=1.0, markdown q=0.5) html seçilmelidir', () => {
    const req = new Request('https://kepce.org/', {
      headers: { Accept: 'text/markdown;q=0.5, text/html;q=1.0' }
    });
    expect(negotiateMediaType(req, ROOT_MEDIA_TYPES)).toBe('text/html');
  });

  it('AgentGrade Prob 4: application/json istemci önceliğinde json seçilmelidir', () => {
    const req = new Request('https://kepce.org/', {
      headers: { Accept: 'application/json, text/html, */*' }
    });
    expect(negotiateMediaType(req, ROOT_MEDIA_TYPES)).toBe('application/json');
  });

  it('Sınır durum: q=0 ile belirtilen tür reddedilmeli ve alternatif seçilmelidir', () => {
    const req = new Request('https://kepce.org/', {
      headers: { Accept: 'text/html;q=0, text/markdown' }
    });
    expect(negotiateMediaType(req, ROOT_MEDIA_TYPES)).toBe('text/markdown');
  });

  it('Sınır durum: joker türlerde q-değerleri doğru işletilmelidir', () => {
    const req = new Request('https://kepce.org/', {
      headers: { Accept: 'text/*;q=0.8, application/json;q=0.9' }
    });
    expect(negotiateMediaType(req, ROOT_MEDIA_TYPES)).toBe('application/json');
  });

  it('Sınır durum: Başlık yoksa veya boşsa varsayılan olarak text/html dönmelidir', () => {
    const reqEmpty = new Request('https://kepce.org/', { headers: { Accept: '' } });
    const reqNone = new Request('https://kepce.org/');
    expect(negotiateMediaType(reqEmpty, ROOT_MEDIA_TYPES)).toBe('text/html');
    expect(negotiateMediaType(reqNone, ROOT_MEDIA_TYPES)).toBe('text/html');
  });

  it('Gerçek dünya tarayıcı başlığı text/html seçmelidir', () => {
    const chromeReq = new Request('https://kepce.org/', {
      headers: {
        Accept:
          'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8'
      }
    });
    expect(negotiateMediaType(chromeReq, ROOT_MEDIA_TYPES)).toBe('text/html');
  });
});

describe('agentRepresentation - Çıktı Biçimlendirme ve Güvenlik', () => {
  it('Markdown çıktısı en az 20 bayt olmalı ve temel başlıkları içermelidir', () => {
    const markdown = formatRootMarkdown('2026-10-08', []);
    expect(markdown.length).toBeGreaterThan(20);
    expect(markdown).toContain('# Kepçe - KYK günün yemek menüsü');
    expect(markdown).toContain('2026-10-08');
  });

  it('JSON çıktısında en üst düzeyde error alanı bulunmamalıdır (AgentGrade kuralı)', () => {
    const json = formatRootJson('2026-10-08', []);
    expect(json.error).toBeUndefined();
    expect(json.name).toBe('Kepçe');
    expect(json.date).toBe('2026-10-08');
    expect(Array.isArray(json.menus)).toBe(true);
  });

  it('404 JSON yanıtı doğru durum kodu ve hata yapısı sunmalıdır', async () => {
    const res = formatJson404('/olmayan-sayfa');
    expect(res.status).toBe(404);
    expect(res.headers.get('content-type')).toContain('application/json');
    expect(res.headers.get('cache-control')).toContain('no-store');
    expect(res.headers.get('vary')).toBe('Accept');

    const body = await res.json();
    expect(body.error).toBe('not_found');
    expect(body.status).toBe(404);
    expect(body.path).toBe('/olmayan-sayfa');
  });

  it('404 içerik pazarlığında json talebi doğru tespit edilmelidir', () => {
    const jsonReq = new Request('https://kepce.org/olmayan', {
      headers: { Accept: 'application/json' }
    });
    const htmlReq = new Request('https://kepce.org/olmayan', {
      headers: { Accept: 'text/html' }
    });
    expect(negotiateMediaType(jsonReq, ERROR_MEDIA_TYPES)).toBe('application/json');
    expect(negotiateMediaType(htmlReq, ERROR_MEDIA_TYPES)).toBe('text/html');
  });
});
