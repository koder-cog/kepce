import { request, HOST_BASE, buildQuery } from './client.js';

export const authApi = {
  register: async (email, password, username = null, default_city_slug = null, diet_mode = null) => {
    const cleanUsername = (username && typeof username === 'string' && username.trim()) ? username.trim() : null;
    return await request('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ email, password, username: cleanUsername, default_city_slug, diet_mode }),
    });
  },
  login: async (username, password, remember = false) => {
    return await request('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ identifier: username, password, remember }),
    });
  },
  passwordless: async (email) => {
    return await request('/auth/passwordless', {
      method: 'POST',
      body: JSON.stringify({ email }),
    });
  },
  passwordlessLogin: async (token) => {
    return await request('/auth/passwordless-login', {
      method: 'POST',
      body: JSON.stringify({ token }),
    });
  },
  logout: async () => {
    await request('/auth/logout', { method: 'POST' });
  },
  getMe: () => request('/auth/me'),
  getSessions: () => request('/auth/me/sessions'),
  revokeSession: (id) => request(`/auth/me/sessions/${id}`, { method: 'DELETE' }),
  deleteAccount: (password) => request('/auth/me', { method: 'DELETE', body: JSON.stringify({ password }) }),
  verifyEmail: (token) => request(`/auth/verify${buildQuery({ token })}`),
  resendVerification: () => request('/auth/resend-verification', { method: 'POST' }),
  getProjects: () => request('/auth/projects'),
  getApiUsage: (projectId = 'all', days = 28) => 
    request(`/auth/projects/usage${buildQuery({ project_id: projectId, days })}`),
  createProject: (name) =>
    request('/auth/projects', {
      method: 'POST',
      body: JSON.stringify({ name }),
    }),
  updateProject: (id, name) =>
    request(`/auth/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify({ name }),
    }),
  deleteProject: (id) => request(`/auth/projects/${id}`, { method: 'DELETE' }),
  blockUser: (blockedId) => request(`/profile/block/${blockedId}`, { method: 'POST' }),
  unblockUser: (blockedId) => request(`/profile/block/${blockedId}`, { method: 'DELETE' }),
  getApiKeys: () => request('/auth/apikeys'),
  createApiKey: (projectId, name) =>
    request('/auth/apikeys', {
      method: 'POST',
      body: JSON.stringify({ project_id: projectId, name }),
    }),
  revokeApiKey: (id) => request(`/auth/apikeys/${id}`, { method: 'DELETE' }),
  getPublicProfile: (nickname) => request(`/profile/${nickname}`),
  updateProfile: (data) =>
    request('/auth/me', {
      method: 'PUT',
      body: JSON.stringify(data),
    }),
  uploadAvatar: async (formData) => {
    return await request('/auth/avatar', {
      method: 'POST',
      body: formData
    });
  },
  deleteAvatar: () => request('/auth/avatar', { method: 'DELETE' }),
  getAvatarUrl: (url) => {
    if (!url) return null;
    if (url.startsWith('http')) return url;
    if (url.startsWith('/static')) return `${HOST_BASE}${url}`;
    return url;
  },
  forgotPassword: (email) =>
    request('/auth/forgot-password', {
      method: 'POST',
      body: JSON.stringify({ email }),
    }),
  resetPassword: (token, new_password) =>
    request('/auth/reset-password', {
      method: 'POST',
      body: JSON.stringify({ token, new_password }),
    }),
};
