import { request } from './client.js';

export const commentsApi = {
  submitVote: (voteData) =>
    request('/comments', {
      method: 'POST',
      body: JSON.stringify(voteData),
    }),
  postComment: (menuId, comment, parentId = null) =>
    request('/comments', {
      method: 'POST',
      body: JSON.stringify({ menu_id: menuId, comment, parent_id: parentId, sentiment: "neutral" }),
    }),
  reportComment: (commentId, description = "Yorum şikayet edildi") =>
    request('/reports', {
      method: 'POST',
      body: JSON.stringify({ 
        reported_comment_id: commentId, 
        reason: description 
      }),
    }),
  getMenuComments: (menuId) => request(`/comments/menu/${menuId}`),
  getComments: (menuId) => request(`/comments/menu/${menuId}`),
  voteComment: (voteId, type) =>
    request('/comments/react', {
      method: 'POST',
      body: JSON.stringify({ vote_id: voteId, type: type }),
    }),
  updateComment: (commentId, comment) =>
    request(`/comments/${commentId}`, {
      method: 'PUT',
      body: JSON.stringify({ comment }),
    }),
  deleteComment: (voteId) => request(`/comments/${voteId}`, { method: 'DELETE' }),
  purgeComment: (voteId) => request(`/moderation/votes/${voteId}/purge`, { method: 'DELETE' }),
};
