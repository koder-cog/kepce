import { request, buildQuery } from './client.js';

export const adminApi = {
  getDishStats: (search = '') => request(`/admin/dishes/stats${buildQuery({ search })}`),
  mergeDishes: (sourceId, targetId) =>
    request('/admin/dishes/merge', {
      method: 'POST',
      body: JSON.stringify({ source_dish_id: sourceId, target_dish_id: targetId }),
    }),
  updateDish: (dishId, payload) =>
    request(`/admin/dishes/${dishId}`, {
      method: 'PUT',
      body: JSON.stringify(payload),
    }),
  splitDish: (dishId, delimiter = '/') =>
    request('/admin/dishes/split', {
      method: 'POST',
      body: JSON.stringify({ dish_id: dishId, delimiter }),
    }),
  detachDish: (aliasId) =>
    request('/admin/dishes/detach', {
      method: 'POST',
      body: JSON.stringify({ alias_id: aliasId }),
    }),
  deleteDish: (dishId) =>
    request(`/admin/dishes/${dishId}`, { method: 'DELETE' }),
  createDish: (name, category = null) =>
    request('/admin/dishes', {
      method: 'POST',
      body: JSON.stringify({ name, category }),
    }),
};
