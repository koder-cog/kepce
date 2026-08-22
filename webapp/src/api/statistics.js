import { request, buildQuery } from './client.js';

export const statisticsApi = {
  getTopDishes: (limit = 10, citySlug = '', timeframe = '') => {
    return request(`/statistics/top-dishes${buildQuery({ limit, city_slug: citySlug, timeframe })}`);
  },
  getWorstDishes: (limit = 10, citySlug = '', timeframe = '') => {
    return request(`/statistics/worst-dishes${buildQuery({ limit, city_slug: citySlug, timeframe })}`);
  },
  getTrendingTags: (limit = 10) => request(`/statistics/trending-tags${buildQuery({ limit })}`),
  getModerationActivity: (timeframe = '') => request(`/statistics/moderation${buildQuery({ timeframe })}`).catch(() => ({})),
  // #46: timeframe parametresi backend desteği eklenene dek yok sayılır
  getHumanityStats: (timeframe = '') => request(`/statistics/humanity${buildQuery({ timeframe })}`).catch(() => ({})),
  getGlobalTopComments: (limit = 10, timeframe = '') => request(`/statistics/comments/top${buildQuery({ limit, timeframe })}`).catch(() => []),
  // Kanonik yol /comments/recent (statistics altındaki kopya kaldırıldı)
  getGlobalRecentComments: (limit = 15) => request(`/comments/recent${buildQuery({ limit })}`).catch(() => []),
  getUserComments: (nickname, sort = 'new', limit = 20, offset = 0) => {
    const page = Math.floor(offset / limit) + 1;
    return request(`/profile/${nickname}/comments${buildQuery({ sort, limit, offset, page })}`);
  },
  getProfileDashboardStats: (nickname) => request(`/profile/${nickname}/stats/dashboard`),
};
