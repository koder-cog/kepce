import { request } from './client.js';

export const favoritesApi = {
  getFavorites: () => request('/auth/me/favorites'),
  toggleFavorite: (dishId) =>
    request('/auth/me/favorites/toggle', {
      method: 'POST',
      body: JSON.stringify({ dish_id: dishId }),
    }),
  togglePinned: (dishId) =>
    request('/auth/me/pinned/toggle', {
      method: 'POST',
      body: JSON.stringify({ dish_id: dishId }),
    }),
};
