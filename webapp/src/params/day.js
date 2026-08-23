/**
 * Validates if the route parameter is a calendar date in YYYY-MM-DD format.
 * Used by SvelteKit route matching [date=day] to prevent collision with
 * other segments under [city_slug=city].
 *
 * Sadece FORMAT doğrular; takvimde olmayan tarihler (2026-02-30 gibi)
 * load() içinde NaiveDate/Date parse'ı ile 404'e düşürülür.
 *
 * @param {string} param
 * @returns {boolean}
 */
export function match(param) {
	return typeof param === 'string' && /^\d{4}-\d{2}-\d{2}$/.test(param);
}
