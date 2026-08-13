import { request } from './client.js';

const getNotifications = async () => {
  return request('/auth/me/notifications');
};

const markAsRead = async (id) => {
  return request('/auth/me/notifications/mark-read', {
    method: 'POST',
    body: JSON.stringify({ id })
  });
};

const markAllAsRead = async () => {
  return request('/auth/me/notifications/mark-all-read', {
    method: 'POST'
  });
};

export const notificationsApi = {
  getNotifications,
  markAsRead,
  markAllAsRead
};
