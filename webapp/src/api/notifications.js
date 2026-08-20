import { request } from './client.js';

export const notificationsApi = {
  getNotifications: async () => {
    try {
      const data = await request('/auth/me/notifications');
      return Array.isArray(data) ? data : [];
    } catch (err) {
      if (err.status === 401 || err.message?.includes('authorization') || err.message?.includes('401')) {
        return [];
      }
      throw err;
    }
  },
  markNotificationRead: (id) =>
    request('/auth/me/notifications/mark-read', {
      method: 'POST',
      body: JSON.stringify({ notification_ids: [id] }),
    }),
  markAllNotificationsRead: () =>
    request('/auth/me/notifications/mark-all-read', {
      method: 'POST',
    }),
  deleteNotification: (id) =>
    request(`/auth/me/notifications/${id}`, {
      method: 'DELETE',
    }),
  clearAllNotifications: () =>
    request('/auth/me/notifications', {
      method: 'DELETE',
    }),
};
