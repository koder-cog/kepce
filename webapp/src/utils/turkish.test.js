import { describe, it, expect } from 'vitest';
import {
  resolveCityFromQuery,
  formatFullTurkishDate,
  getAblativeSuffix,
  getDativeSuffix,
  normalizeTurkishText
} from './turkish.js';

describe('normalizeTurkishText', () => {
  it('Türkçe büyük/küçük I ve İ harflerini birleştirme işareti olmadan normalize eder', () => {
    expect(normalizeTurkishText('İSTANBUL')).toBe('istanbul');
    expect(normalizeTurkishText('IĞDIR')).toBe('igdir');
    expect(normalizeTurkishText('ISPARTA')).toBe('isparta');
    expect(normalizeTurkishText('Şanlıurfa')).toBe('sanliurfa');
    expect(normalizeTurkishText('Diyarbakır')).toBe('diyarbakir');
  });
});

describe('resolveCityFromQuery', () => {
  it('resmi il adlarını ve sluglarını doğrudan çözer', () => {
    expect(resolveCityFromQuery('istanbul kyk yemek')?.slug).toBe('istanbul');
    expect(resolveCityFromQuery('ankara kyk yemek')?.slug).toBe('ankara');
    expect(resolveCityFromQuery('izmir yemek menüsü')?.slug).toBe('izmir');
    expect(resolveCityFromQuery('bursa tabldot')?.slug).toBe('bursa');
    expect(resolveCityFromQuery('canakkale yemek')?.slug).toBe('canakkale');
  });

  it('Türkçe çekim eklerini (bulunma, ayrılma, yönelme, tamlayan) başarıyla çözer', () => {
    expect(resolveCityFromQuery("istanbul'da kyk yemek")?.slug).toBe('istanbul');
    expect(resolveCityFromQuery('istanbuldaki yemekhane')?.slug).toBe('istanbul');
    expect(resolveCityFromQuery('ankaranın kyk menüsü')?.slug).toBe('ankara');
    expect(resolveCityFromQuery('izmire ait yemek')?.slug).toBe('izmir');
    expect(resolveCityFromQuery('bursadan yemek')?.slug).toBe('bursa');
    expect(resolveCityFromQuery('adapazarında kyk')?.slug).toBe('sakarya');
    expect(resolveCityFromQuery('afyonda yemek')?.slug).toBe('afyonkarahisar');
  });

  it('yaygın halk ağzı ve şehir kısaltmalarını doğru ile eşleştirir', () => {
    expect(resolveCityFromQuery('afyon kyk yemek')?.slug).toBe('afyonkarahisar');
    expect(resolveCityFromQuery('urfa tabldot menüsü')?.slug).toBe('sanliurfa');
    expect(resolveCityFromQuery('maraş kyk yemek')?.slug).toBe('kahramanmaras');
    expect(resolveCityFromQuery('maras kyk yemek')?.slug).toBe('kahramanmaras');
    expect(resolveCityFromQuery('antep yemek listesi')?.slug).toBe('gaziantep');
    expect(resolveCityFromQuery('izmit kyk yemek')?.slug).toBe('kocaeli');
    expect(resolveCityFromQuery('adapazarı kyk yemek')?.slug).toBe('sakarya');
    expect(resolveCityFromQuery('antakya kyk yemek')?.slug).toBe('hatay');
    expect(resolveCityFromQuery('içel kyk yemek')?.slug).toBe('mersin');
    expect(resolveCityFromQuery('icel kyk yemek')?.slug).toBe('mersin');
    expect(resolveCityFromQuery('dersim kyk yemek')?.slug).toBe('tunceli');
    expect(resolveCityFromQuery('elaziz kyk yemek')?.slug).toBe('elazig');
  });

  it('iki kelime olarak ayrı yazılan şehir adlarını doğru çözer', () => {
    expect(resolveCityFromQuery('şanlı urfa kyk yemek')?.slug).toBe('sanliurfa');
    expect(resolveCityFromQuery('sanli urfa yemek')?.slug).toBe('sanliurfa');
    expect(resolveCityFromQuery('gazi antep kyk')?.slug).toBe('gaziantep');
    expect(resolveCityFromQuery('kahraman maraş yemek')?.slug).toBe('kahramanmaras');
    expect(resolveCityFromQuery('afyon karahisar menü')?.slug).toBe('afyonkarahisar');
  });

  it('popüler üniversite adlarını ilgili şehirle doğru eşleştirir', () => {
    expect(resolveCityFromQuery('itü kyk yemek')?.slug).toBe('istanbul');
    expect(resolveCityFromQuery('odtü kyk yemek')?.slug).toBe('ankara');
    expect(resolveCityFromQuery('boğaziçi kyk yemek')?.slug).toBe('istanbul');
    expect(resolveCityFromQuery('hacettepe kyk menü')?.slug).toBe('ankara');
    expect(resolveCityFromQuery('akdeniz kyk yemek')?.slug).toBe('antalya');
    expect(resolveCityFromQuery('uludağ kyk menüsü')?.slug).toBe('bursa');
    expect(resolveCityFromQuery('anadolu kyk yemek')?.slug).toBe('eskisehir');
    expect(resolveCityFromQuery('dokuz eylül kyk')?.slug).toBe('izmir');
    expect(resolveCityFromQuery('çukurova kyk yemek')?.slug).toBe('adana');
    expect(resolveCityFromQuery('selçuk kyk yemek')?.slug).toBe('konya');
    expect(resolveCityFromQuery('karadeniz teknik kyk')?.slug).toBe('trabzon');
    expect(resolveCityFromQuery('ondokuz mayıs kyk')?.slug).toBe('samsun');
  });

  it('kısa şehir adlarında (Van, Muş) sahte eşleşmeleri (false-positive) kesinlikle engeller', () => {
    // "vantilatör" Van ile eşleşmemeli
    expect(resolveCityFromQuery('vantilatör kyk yurdunda yasak mı')).toBeNull();
    // "musluk" Muş ile eşleşmemeli
    expect(resolveCityFromQuery('musluk bozuldu kyk')).toBeNull();
    // "orduevi" Ordu ile eşleşmemeli
    expect(resolveCityFromQuery('orduevi kyk')).toBeNull();
    
    // Fakat gerçek Van ve Muş eşleşmeli
    expect(resolveCityFromQuery('van kyk yemek')?.slug).toBe('van');
    expect(resolveCityFromQuery('vanda kyk yemek')?.slug).toBe('van');
    expect(resolveCityFromQuery('muş kyk yemek')?.slug).toBe('mus');
    expect(resolveCityFromQuery('muşta kyk yemek')?.slug).toBe('mus');
  });

  it('şehir içermeyen aramalarda null döner', () => {
    expect(resolveCityFromQuery('fiyat hesaplama tabldot')).toBeNull();
    expect(resolveCityFromQuery('')).toBeNull();
    expect(resolveCityFromQuery(null)).toBeNull();
  });
});
