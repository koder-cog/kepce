<script>
  import { onMount } from 'svelte';
  import { api } from '@/api/index.js';
  import EmptyState from '@/components/ui/EmptyState.svelte';
  import Loader from '@/components/ui/Loader.svelte';
  import { icon } from '@/components/ui/icons.js';
  import * as ui from '@/components/ui/forms.js';
  import { showToast } from '@/components/ui/toast.js';
  import { createModal } from '@/components/features/modal.js';

  let searchQuery = $state('');
  let isLoading = $state(true);
  let users = $state([]);
  let errorMsg = $state(null);
  let sortState = $state({ column: 'date', asc: false });
  let searchTimeout;
  let currentLoadToken = 0;

  async function fetchUsers(query = '') {
    isLoading = true;
    errorMsg = null;
    users = [];
    const token = ++currentLoadToken;
    try {
      const data = await api.getUsers(query);
      if (token !== currentLoadToken) return;
      users = data;
    } catch (err) {
      if (token !== currentLoadToken) return;
      errorMsg = err.message || 'Kullanıcılar yüklenirken bir hata oluştu.';
    } finally {
      if (token === currentLoadToken) {
        isLoading = false;
      }
    }
  }

  onMount(() => {
    fetchUsers();
  });

  $effect(() => {
    return () => {
      clearTimeout(searchTimeout);
    };
  });

  function handleSearchInput(e) {
    searchQuery = e.target.value;
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
      fetchUsers(searchQuery.trim());
    }, 300);
  }

  function handleSort(column) {
    if (sortState.column === column) {
      sortState.asc = !sortState.asc;
    } else {
      sortState.column = column;
      sortState.asc = true;
    }
  }

  let sortedUsers = $derived([...users].sort((a, b) => {
    let valA, valB;
    if (sortState.column === 'username') { valA = a.username.toLocaleLowerCase('tr-TR'); valB = b.username.toLocaleLowerCase('tr-TR'); }
    else if (sortState.column === 'email') { valA = (a.email || '').toLocaleLowerCase('tr-TR'); valB = (b.email || '').toLocaleLowerCase('tr-TR'); }
    else if (sortState.column === 'status') { valA = a.is_verified ? 1 : 0; valB = b.is_verified ? 1 : 0; }
    else { valA = new Date(a.created_at).getTime(); valB = new Date(b.created_at).getTime(); }

    if (valA < valB) return sortState.asc ? -1 : 1;
    if (valA > valB) return sortState.asc ? 1 : -1;
    return 0;
  }));

  async function toggleVerify(user) {
    const newVal = !user.is_verified;
    try {
      await api.updateUser(user.id, { is_verified: newVal });
      showToast(newVal ? 'Kullanıcı onaylandı.' : 'Kullanıcının onayı kaldırıldı.');
      fetchUsers(searchQuery.trim());
    } catch (err) { showToast(err.message, 'error'); }
  }

  async function toggleAdmin(user) {
    const newVal = !user.is_admin;
    try {
      await api.updateUser(user.id, { is_admin: newVal });
      showToast(newVal ? 'Kullanıcı admin yapıldı.' : 'Kullanıcının adminliği alındı.');
      fetchUsers(searchQuery.trim());
    } catch (err) { showToast(err.message, 'error'); }
  }

  async function toggleBan(user) {
    const newVal = !user.is_banned;
    try {
      await api.updateUser(user.id, { is_banned: newVal });
      showToast(newVal ? 'Kullanıcı yasaklandı.' : 'Kullanıcının yasağı kaldırıldı.');
      fetchUsers(searchQuery.trim());
    } catch (err) { showToast(err.message, 'error'); }
  }

  function handleWarnUser(user) {
    createModal({
      title: 'Kullanıcıyı Uyar',
      iconHtml: icon('alert-triangle', 24),
      iconColor: 'warning',
      contentHtml: `
        <div class="c-modal__form-group">
          <label class="c-modal__label">Uyarı mesajı</label>
          <textarea id="warning-message" class="c-modal__input" rows="4" placeholder="Kullanıcıya gönderilecek uyarı mesajını yazın..."></textarea>
        </div>
        <p class="u-text-xs u-color-muted u-mt-sm">Bu mesaj doğrudan kullanıcının e-posta adresine iletilecek ve sistemde loglanacaktır.</p>
      `,
      buttons: [
        { label: 'İptal', variant: 'secondary' },
        {
          label: 'Uyarıyı Gönder', variant: 'warning', onClick: async () => {
            const message = document.getElementById('warning-message').value.trim();
            if (!message) {
              showToast('Lütfen bir uyarı mesajı yazın.', 'error');
              return false;
            }
            try {
              await api.warnUser(user.id, message);
              showToast('Uyarı başarıyla gönderildi.');
            } catch (err) { showToast(err.message, 'error'); }
          }
        }
      ]
    });
  }
</script>

<svelte:head>
  <title>Kullanıcılar - Moderasyon - Kepçe</title>
</svelte:head>

    <div class="admin-search-bar u-flex-grow u-mb-md u-mt-md">
      <span class="admin-search-bar__icon">
        {@html icon('search', 16)}
      </span>
      <input 
        type="text" 
        class="admin-search-bar__input u-text-base" 
        placeholder="Kullanıcı ara (Email veya Kullanıcı Adı)..." 
        autocomplete="off"
        value={searchQuery}
        oninput={handleSearchInput}
      >
    </div>

<div id="user-list-container" class="u-mt-md">
  {#if isLoading}
    <div class="stats-placeholder">
      <Loader size={48} />
    </div>
  {:else if errorMsg}
    <EmptyState statusCode={500} desc={errorMsg} />
  {:else if users.length === 0}
    <EmptyState iconName={'users'} title={'Kullanıcı Bulunamadı'} desc={'Arama kriterlerine uygun kimse yok.'} />
  {:else}
    <div class="admin-table-wrapper">
      <table class="admin-table admin-table--hybrid">
        <thead>
          <tr>
            <th class="sortable {sortState.column === 'username' ? (sortState.asc ? 'sort-asc' : 'sort-desc') : ''}" onclick={() => handleSort('username')}>Kullanıcı</th>
            <th class="sortable {sortState.column === 'email' ? (sortState.asc ? 'sort-asc' : 'sort-desc') : ''}" onclick={() => handleSort('email')}>Email</th>
            <th class="sortable {sortState.column === 'status' ? (sortState.asc ? 'sort-asc' : 'sort-desc') : ''}" onclick={() => handleSort('status')}>Durum</th>
            <th>Admin?</th>
            <th class="sortable {sortState.column === 'date' ? (sortState.asc ? 'sort-asc' : 'sort-desc') : ''}" onclick={() => handleSort('date')}>Kayıt</th>
            <th class="col-actions">Aksiyonlar</th>
          </tr>
        </thead>
        <tbody>
          {#each sortedUsers as user (user.id)}
            <tr class={user.is_banned ? 'u-opacity-50' : ''}>
              <td>
                <div class="u-flex u-items-center u-gap-sm">
                  <div class="user-avatar-mini" style="--bg-image: url('{api.getAvatarUrl(user.avatar_url) || '/assets/img/default-avatar.png'}');"></div>
                  <strong>{user.username}</strong>
                </div>
              </td>
              <td>
                {#if user.email}
                  {user.email}
                {:else}
                  <span class="u-text-muted">Gizli</span>
                {/if}
              </td>
              <td>
                {@html ui.createBadge({ label: user.is_verified ? 'Onaylı' : 'Bekliyor', variant: user.is_verified ? 'success' : 'warning', size: 'sm' })}
                {#if user.is_banned}
                  {@html ui.createBadge({ label: 'Yasaklı', variant: 'danger', size: 'sm' })}
                {/if}
              </td>
              <td>{@html user.is_admin ? ui.createBadge({ label: 'Evet', variant: 'primary', size: 'sm' }) : 'Hayır'}</td>
              <td>{new Date(user.created_at).toLocaleDateString('tr-TR')}</td>
              <td class="col-actions">
                <div class="dish-actions">
                  <button class="btn-icon toggle-verify" title={user.is_verified ? 'Onayı Kaldır' : 'Onayla'} onclick={() => toggleVerify(user)}>
                    {@html icon(user.is_verified ? 'close' : 'check', 14)}
                  </button>
                  <button class="btn-icon toggle-admin" title={user.is_admin ? 'Adminliği Al' : 'Admin Yap'} onclick={() => toggleAdmin(user)}>
                    {@html icon('user', 14)}
                  </button>
                  <button class="btn-icon ban-user" title={user.is_banned ? 'Yasağı Kaldır' : 'Yasakla'} onclick={() => toggleBan(user)}>
                    {@html icon('slash', 14)}
                  </button>
                  <button class="btn-icon warn-user" title="Uyarı Ver" onclick={() => handleWarnUser(user)}>
                    {@html icon('alert-triangle', 14)}
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
