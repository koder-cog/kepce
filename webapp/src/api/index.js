import { authApi } from './auth.js';
import { menusApi } from './menus.js';
import { commentsApi } from './comments.js';
import { favoritesApi } from './favorites.js';
import { statisticsApi } from './statistics.js';
import { moderationApi } from './moderation.js';
import { adminApi } from './admin.js';
import { systemApi, reportsApi, contactApi } from './system.js';
import { notificationsApi } from './notifications.js';
import { request, API_BASE, HOST_BASE } from './client.js';

export { API_BASE, HOST_BASE };

export const api = {
  ...authApi,
  ...menusApi,
  ...commentsApi,
  ...favoritesApi,
  ...statisticsApi,
  ...moderationApi,
  ...adminApi,
  ...systemApi,
  ...reportsApi,
  ...contactApi,
  ...notificationsApi,
};
