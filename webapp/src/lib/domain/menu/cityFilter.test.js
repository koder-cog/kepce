import { describe, it, expect } from 'vitest';
import { filterCitiesByViewPeriod } from './cityFilter.js';

describe('filterCitiesByViewPeriod', () => {
  const mockCities = [
    {
      id: 1,
      slug: 'istanbul',
      name: 'İstanbul',
      latest_menu_date: '2026-10-25'
    },
    {
      id: 2,
      slug: 'ankara',
      name: 'Ankara',
      latest_menu_date: '2026-09-15' // ~43 gün önce (90 gün içinde)
    },
    {
      id: 3,
      slug: 'izmir',
      name: 'İzmir',
      latest_menu_date: '2026-08-10' // ~79 gün önce (90 gün içinde)
    },
    {
      id: 4,
      slug: 'bursa',
      name: 'Bursa',
      latest_menu_date: '2026-05-01' // ~180 gün önce (>90 gün, eski)
    },
    {
      id: 5,
      slug: 'edirne',
      name: 'Edirne',
      latest_menu_date: null // Hiç verisi yok
    }
  ];

  // Referans tarih: 28 Ekim 2026
  const refToday = new Date(2026, 9, 28); // viewMonth 9 = Ekim

  it('güncel ayda (Ekim 2026) yalnızca bu ay menüsü olan şehirleri döndürür', () => {
    const result = filterCitiesByViewPeriod({
      cities: mockCities,
      currentCity: 'istanbul',
      viewYear: 2026,
      viewMonth: 9, // Ekim
      today: refToday
    });

    const slugs = result.map((c) => c.slug);
    expect(slugs).toContain('istanbul');
    expect(slugs).not.toContain('ankara');
    expect(slugs).not.toContain('bursa');
    expect(slugs).not.toContain('edirne');
  });

  it('1-3 ay önceki görünümde (Eylül 2026) son 90 gün içinde menüsü olanları döndürür', () => {
    const result = filterCitiesByViewPeriod({
      cities: mockCities,
      currentCity: 'ankara',
      viewYear: 2026,
      viewMonth: 8, // Eylül (monthDiff = 1)
      today: refToday
    });

    const slugs = result.map((c) => c.slug);
    expect(slugs).toContain('istanbul'); // Ekim (3 gün önce)
    expect(slugs).toContain('ankara'); // Eylül (~43 gün önce)
    expect(slugs).toContain('izmir'); // Ağustos (~79 gün önce)
    expect(slugs).not.toContain('bursa'); // Mayıs (>90 gün)
    expect(slugs).not.toContain('edirne'); // null
  });

  it('3 aydan eski görünümde (Ocak 2026, monthDiff = 9) tüm şehirleri döndürür', () => {
    const result = filterCitiesByViewPeriod({
      cities: mockCities,
      currentCity: 'istanbul',
      viewYear: 2026,
      viewMonth: 0, // Ocak (monthDiff = 9)
      today: refToday
    });

    expect(result.length).toBe(mockCities.length);
    const slugs = result.map((c) => c.slug);
    expect(slugs).toEqual(['istanbul', 'ankara', 'izmir', 'bursa', 'edirne']);
  });

  it('seçili şehir filtrede elense bile koruma kuralıyla listeye dahil edilir', () => {
    // Kullanıcı Bursa (eski şehir) sayfasındayken Ekim 2026'ya bakıyor
    const result = filterCitiesByViewPeriod({
      cities: mockCities,
      currentCity: 'bursa',
      viewYear: 2026,
      viewMonth: 9,
      today: refToday
    });

    const slugs = result.map((c) => c.slug);
    expect(slugs).toContain('istanbul');
    expect(slugs).toContain('bursa'); // Güvenlik koruması devrede
  });

  it('şehirlerin hiçbirinde latest_menu_date yoksa tüm liste korunur', () => {
    const citiesWithoutDates = [
      { id: 1, slug: 'istanbul', name: 'İstanbul' },
      { id: 2, slug: 'ankara', name: 'Ankara' }
    ];

    const result = filterCitiesByViewPeriod({
      cities: citiesWithoutDates,
      currentCity: 'istanbul',
      viewYear: 2026,
      viewMonth: 9,
      today: refToday
    });

    expect(result.length).toBe(2);
  });

  it('boş veya geçersiz parametrelerde güvenli döner', () => {
    expect(filterCitiesByViewPeriod({ cities: [] })).toEqual([]);
    expect(filterCitiesByViewPeriod({})).toEqual([]);
  });
});
