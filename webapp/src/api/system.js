import { request, buildQuery } from './client.js';

export const systemApi = {
  getSystemHealth: () => request('/system/health'),
  verifyTree: () => request('/system/verify'),
  getStatus: () => request('/system/status'),
  getStatusHistory: (days = 90) => request(`/system/status/history${buildQuery({ days })}`).catch(() => []),
};

export const reportsApi = {
  submitReport: (data) =>
    request('/reports', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
  getReports: (status = '') => {
    return request(`/reports${buildQuery({ status })}`);
  },
  getContactMessages: () => request('/reports/contact'),
  updateContactMessageStatus: (id, status) =>
    request(`/reports/contact/${id}`, {
      method: 'PATCH',
      body: JSON.stringify({ status }),
    }),
  deleteContactMessage: (id) =>
    request(`/reports/contact/${id}`, { method: 'DELETE' }),
  updateReportStatus: (reportId, status) =>
    request(`/reports/${reportId}`, {
      method: 'PATCH',
      body: JSON.stringify({ status }),
    }),
  deleteReport: (reportId) => request(`/reports/${reportId}`, { method: 'DELETE' }),
};

export const contactApi = {
  submitContactForm: (data) =>
    request('/public/contact', {
      method: 'POST',
      body: JSON.stringify(data),
    }),
};
