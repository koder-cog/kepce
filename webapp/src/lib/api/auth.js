import { request, HOST_BASE, buildQuery } from './client.js';

export const authApi = {
  register: async (
    nickname,
    password,
    default_city_slug = null,
    diet_mode = null
  ) => {
    const cleanNickname =
      nickname && typeof nickname === 'string' && nickname.trim() ? nickname.trim() : null;
    return await request('/auth/register', {
      method: 'POST',
      body: JSON.stringify({
        nickname: cleanNickname,
        password,
        default_city_slug,
        diet_mode
      })
    });
  },
  login: async (identifier, password, remember = false) => {
    return await request('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ identifier, password, remember })
    });
  },
  confirmRecovery: async (last_block) => {
    return await request('/auth/confirm-recovery', {
      method: 'POST',
      body: JSON.stringify({ last_block })
    });
  },
  regenerateRecovery: async (password) => {
    return await request('/auth/regenerate-recovery', {
      method: 'POST',
      body: JSON.stringify({ password })
    });
  },
  resetPasswordWithRecovery: async (nickname, recovery_key, new_password) => {
    return await request('/auth/password-reset', {
      method: 'POST',
      body: JSON.stringify({ nickname, recovery_key, new_password })
    });
  },
  logout: async () => {
    await request('/auth/logout', { method: 'POST' });
  },
  getMe: () => request('/auth/me'),
  getSessions: () => request('/auth/me/sessions'),
  revokeSession: (id) => request(`/auth/me/sessions/${id}`, { method: 'DELETE' }),
  deleteAccount: (password) =>
    request('/auth/me', {
      method: 'DELETE',
      body: JSON.stringify({ password })
    }),
  getProjects: () => request('/auth/projects'),
  getApiUsage: (projectId = 'all', days = 28) =>
    request(`/auth/projects/usage${buildQuery({ project_id: projectId, days })}`),
  createProject: (name) =>
    request('/auth/projects', {
      method: 'POST',
      body: JSON.stringify({ name })
    }),
  updateProject: (id, name) =>
    request(`/auth/projects/${id}`, {
      method: 'PUT',
      body: JSON.stringify({ name })
    }),
  deleteProject: (id) => request(`/auth/projects/${id}`, { method: 'DELETE' }),
  blockUser: (blockedId) => request(`/profile/block/${blockedId}`, { method: 'POST' }),
  unblockUser: (blockedId) => request(`/profile/block/${blockedId}`, { method: 'DELETE' }),
  getApiKeys: () => request('/auth/apikeys'),
  createApiKey: (projectId, name) =>
    request('/auth/apikeys', {
      method: 'POST',
      body: JSON.stringify({ project_id: projectId, name })
    }),
  revokeApiKey: (id) => request(`/auth/apikeys/${id}`, { method: 'DELETE' }),
  getPublicProfile: (nickname) => request(`/profile/${nickname}`),
  updatePinnedBadges: (pinned_badges) =>
    request('/profile/me/pinned-badges', {
      method: 'PUT',
      body: JSON.stringify({ pinned_badges })
    }),
  updateProfile: (data) =>
    request('/auth/me', {
      method: 'PUT',
      body: JSON.stringify(data)
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
  }
};
