import holidays from '../data/holidays.json';

/**
 * Returns the theme for a given date based on whether it's a holiday, weekend, or regular day.
 * @param {string|Date} date - The date to check (can be a YYYY-MM-DD string or a Date object).
 * @returns {string} - The theme class modifier ('accent-disclaimer', 'sunken', or 'default').
 */
export function getDayTheme(date) {
    if (!date) return 'default';
    
    let dateObj = typeof date === 'string' ? new Date(date) : date;
    
    // Validate date
    if (isNaN(dateObj.getTime())) return 'default';
    
    const year = dateObj.getFullYear();
    const month = String(dateObj.getMonth() + 1).padStart(2, '0');
    const day = String(dateObj.getDate()).padStart(2, '0');
    
    const fullDateString = `${year}-${month}-${day}`;
    const shortDateString = `${month}-${day}`;
    
    // Check holidays first (highest priority)
    if (holidays[fullDateString]) {
        return holidays[fullDateString].theme;
    }
    if (holidays[shortDateString]) {
        return holidays[shortDateString].theme;
    }
    
    // Check weekends (sunken)
    const dayOfWeek = dateObj.getDay(); // 0 is Sunday, 6 is Saturday
    if (dayOfWeek === 0 || dayOfWeek === 6) {
        return 'sunken';
    }
    
    return 'default';
}
