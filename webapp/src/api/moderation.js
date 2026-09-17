import { request, buildQuery } from './client.js';

export const moderationApi = {
  getPendingMenus: () => request('/moderation/pending'),
  getMenus: (status = '', city = '', month = '') => {
    return request(`/moderation/menus${buildQuery({ status, city_slug: city, month })}`);
  },
  createMenu: (payload) =>
    request('/moderation/menus', {
      method: 'POST',
      body: JSON.stringify(payload),
    }),
  getMenuDishIds: (menuId) => request(`/moderation/${menuId}/items`),
  approveMenu: (menuId, notes = '') =>
    request(`/moderation/${menuId}/approve`, {
      method: 'POST',
      body: JSON.stringify({ notes }),
    }),
  rejectMenu: (menuId, notes = '') =>
    request(`/moderation/${menuId}/reject`, {
      method: 'POST',
      body: JSON.stringify({ notes }),
    }),
  bulkUpdateMenuStatus: (menuIds, status) =>
    request('/moderation/menus/bulk-status', {
      method: 'POST',
      body: JSON.stringify({ menu_ids: menuIds, status }),
    }),
  updateMenuCommentary: (menuId, bot_commentary) =>
    request(`/moderation/menus/${menuId}/commentary`, {
      method: 'PUT',
      body: JSON.stringify({ content: bot_commentary }),
    }),
  getPendingVotes: () => request('/moderation/votes/pending'),
  getAllVotes: (search = '', limit = 20, offset = 0) => request(`/moderation/votes/all${buildQuery({ search, limit, offset })}`),
  getComplaints: () => request('/moderation/votes/complaints'),
  approveVote: (voteId) => request(`/moderation/votes/${voteId}/approve`, { method: 'POST' }),
  rejectVote: (voteId) => request(`/moderation/votes/${voteId}/reject`, { method: 'POST' }),
  resetVote: (voteId) => request(`/moderation/votes/${voteId}/reset`, { method: 'POST' }),
  purgeVote: (voteId) => request(`/moderation/votes/${voteId}/purge`, { method: 'DELETE' }),

  getUsers: (search = '') => request(`/moderation/users${buildQuery({ search })}`),
  updateUserStatus: (userId, status) =>
    request(`/moderation/users/${userId}/status`, {
      method: 'PUT',
      body: JSON.stringify({ status }),
    }),
  updateUser: (userId, data) =>
    request(`/moderation/users/${userId}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  banUser: (userId) => request(`/moderation/users/${userId}/ban`, { method: 'POST' }),
  warnUser: (userId, message) => request(`/moderation/users/${userId}/warn`, { method: 'POST', body: JSON.stringify({ message }) }),

  getTags: () => request('/moderation/tags'),
  createTag: (tag) =>
    request('/moderation/tags', {
      method: 'POST',
      body: JSON.stringify(tag),
    }),
  updateTag: (tagId, tag) =>
    request(`/moderation/tags/${tagId}`, {
      method: 'PUT',
      body: JSON.stringify(tag),
    }),
  deleteTag: (tagId) => request(`/moderation/tags/${tagId}`, { method: 'DELETE' }),

  updateMenuItems: (menuId, payload) =>
    request(`/moderation/${menuId}/items`, {
      method: 'PUT',
      body: JSON.stringify(Array.isArray(payload) ? { dish_ids: payload } : payload),
    }),
    
  exportMonthlyMenuForBot: (city, month) => request(`/moderation/bot/export-monthly${buildQuery({ city_slug: city, month })}`),
  injectBotComments: (city_slug, comments) => request('/moderation/bot/inject', {
    method: 'POST',
    body: JSON.stringify({ city_slug, comments })
  }),
  
  getIncidents: () => request('/moderation/incidents'),
  createIncident: (incident) => request('/moderation/incidents', {
    method: 'POST',
    body: JSON.stringify(incident)
  }),
  updateIncident: (id, payload) => request(`/moderation/incidents/${id}`, {
    method: 'PUT',
    body: JSON.stringify(payload)
  }),
  deleteIncident: (id) => request(`/moderation/incidents/${id}`, {
    method: 'DELETE'
  }),
  getSubmissions: (status = '') => request(`/moderation/submissions${buildQuery({ status })}`),
  updateSubmissionStatus: (submissionId, status) =>
    request(`/moderation/submissions/${submissionId}/status`, {
      method: 'POST',
      body: JSON.stringify({ status }),
    }),

  getKitchenCoverage: (year, month) =>
    request(`/moderation/kitchen/coverage${buildQuery({ year, month })}`),

  getDatabaseTables: () => request('/moderation/database/tables'),
  getTableData: (tableName, params = {}) =>
    request(`/moderation/database/tables/${tableName}${buildQuery(params)}`),
  deleteTableRow: (tableName, pkColumn, pkValue) =>
    request(`/moderation/database/tables/${tableName}/row`, {
      method: 'DELETE',
      body: JSON.stringify({ pk_column: pkColumn, pk_value: pkValue }),
    }),
  updateTableRow: (tableName, pkColumn, pkValue, updateColumn, newValue) =>
    request(`/moderation/database/tables/${tableName}/row`, {
      method: 'PUT',
      body: JSON.stringify({
        pk_column: pkColumn,
        pk_value: pkValue,
        update_column: updateColumn,
        new_value: newValue,
      }),
    }),
  executeDatabaseQuery: (query, writeMode = false) =>
    request('/moderation/database/query', {
      method: 'POST',
      body: JSON.stringify({ query, write_mode: writeMode }),
    }),
};
