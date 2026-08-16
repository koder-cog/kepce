<script>
  /**
   * TODO: FUTURE ENHANCEMENTS (Gelecek Planlaması)
   * 1. Toast Entegrasyonu: Yeni bildirim geldiğinde _toast.css kullanılarak sağ alttan popup çıkartılabilir.
   * 2. Pagination / Infinite Scroll: Bildirim sayısı arttığında performansı korumak için IntersectionObserver ile sonsuz kaydırma (load more) eklenebilir.
   * 3. Swipe-to-Dismiss: Mobilde kullanıcıların kartı sağa/sola kaydırarak "Okundu" olarak işaretleyebilmesi (touch events ile) eklenebilir.
   * 4. Real-time (WebSockets/SSE): Sayfa yenilenmeden arka planda (SSE veya WebSocket) bildirimlerin anında UI'a düşmesi sağlanabilir.
   * 5. Web Push API: Sekme kapalıyken bile tarayıcı/işletim sistemi üzerinden "Yorumunuza yanıt geldi" bildirimi gönderilebilir.
   */

  import { api } from '@/api/index.js';
  import { icon } from '@/components/ui/icons.js';
  import EmptyState from '@/components/ui/EmptyState.svelte';
  import Loader from '@/components/ui/Loader.svelte';
  import TabBar from '@/components/ui/TabBar.svelte';
  import { onMount } from 'svelte';
  import { globalState } from '@/state.svelte.js';
  import { slide } from 'svelte/transition';
  import { getDuration } from '@/lib/dom/motion.js';
  import Seo from '@/components/ui/Seo.svelte';

  let user = $derived(globalState?.user);
  let notifications = $state([]);
  let isLoading = $state(true);
  let errorMsg = $state(null);
  
  let activeTab = $state('all'); // 'all' | 'unread'

  // Derivated states
  let unreadCount = $derived(notifications.filter(n => !n.is_read).length);
  let filteredNotifications = $derived(
    activeTab === 'unread' 
      ? notifications.filter(n => !n.is_read) 
      : notifications
  );

  // Grouping logic
  let groupedNotifications = $derived.by(() => {
    const groups = {
      today: { label: 'Bugün', items: [] },
      yesterday: { label: 'Dün', items: [] },
      older: { label: 'Daha Eski', items: [] }
    };

    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
    const yesterday = today - 86400000;

    filteredNotifications.forEach(n => {
      const date = new Date(n.created_at).getTime();
      if (date >= today) {
        groups.today.items.push(n);
      } else if (date >= yesterday) {
        groups.yesterday.items.push(n);
      } else {
        groups.older.items.push(n);
      }
    });

    return [groups.today, groups.yesterday, groups.older].filter(g => g.items.length > 0);
  });

  let hasFetched = false;

  $effect(() => {
    if (globalState.isReady && !hasFetched) {
      if (user) {
        hasFetched = true;
        loadNotifications();
      } else {
        isLoading = false;
        errorMsg = 'Bildirimleri görüntülemek için giriş yapmalısınız.';
      }
    }
  });

  async function loadNotifications() {
    isLoading = true;
    errorMsg = null;
    try {
      notifications = await api.getNotifications();
    } catch (err) {
      errorMsg = 'Bildirimler yüklenirken bir hata oluştu.';
    } finally {
      isLoading = false;
    }
  }

  async function handleMarkAsRead(id) {
    const n = notifications.find(n => n.id === id);
    if (!n || n.is_read) return;
    
    n.is_read = true; // Optimistic update
    try {
      await api.markAsRead(id);
    } catch (err) {
      n.is_read = false; // Revert
      console.error(err);
    }
  }

  async function handleMarkAllAsRead() {
    if (unreadCount === 0) return;
    
    const previousState = notifications.map(n => ({...n}));
    notifications = notifications.map(n => ({...n, is_read: true}));
    
    try {
      await api.markAllAsRead();
    } catch (err) {
      notifications = previousState;
      console.error(err);
    }
  }

  function getIconForType(type) {
    switch (type) {
      case 'system': return icon('info', 24);
      case 'achievement': return icon('star', 24);
      case 'comment': return icon('chat', 24);
      case 'moderation': return icon('check', 24);
      default: return icon('bell', 24);
    }
  }

  function formatTimeAgo(isoString) {
    const diff = Date.now() - new Date(isoString).getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(minutes / 60);

    if (minutes < 60) return `${Math.max(1, minutes)} dk önce`;
    if (hours < 24) return `${hours} saat önce`;
    
    const d = new Date(isoString);
    return `${d.getDate().toString().padStart(2, '0')}.${(d.getMonth() + 1).toString().padStart(2, '0')}.${d.getFullYear()}`;
  }
</script>

<Seo
  title="Bildirimler - Kepçe"
  description="Kullanıcı bildirimleri ve güncellemeler."
  noindex={true}
/>

<div class="notification-page">
  <div class="notification-page__header">
    <div class="notification-page__header-left">
      <h1 class="notification-page__title">Bildirimler</h1>
    </div>
    {#if unreadCount > 0}
      <button class="btn btn--outline btn--sm" onclick={handleMarkAllAsRead}>
        Tümünü okundu işaretle
      </button>
    {/if}
  </div>

  <TabBar
    bind:activeId={activeTab}
    tabs={[
      { id: 'all', label: 'Tümü', icon: icon('bell', 18) },
      { id: 'unread', label: 'Okunmayanlar', icon: icon('eyeSlash', 18), badge: unreadCount > 0 ? (unreadCount > 99 ? '99+' : unreadCount) : undefined }
    ]}
  />

  {#if isLoading}
    <div class="stats-placeholder">
      <Loader size={48} />
    </div>
  {:else if errorMsg}
    <EmptyState 
      statusCode={user ? 500 : 401} 
      desc={errorMsg} 
      actionLabel={!user ? "Giriş Yap" : "Tekrar Dene"}
      onAction={!user ? () => window.location.href = '/giris' : loadNotifications}
    />
  {:else if filteredNotifications.length === 0}
    <EmptyState 
      desc={activeTab === 'unread' ? "Okunmamış bildiriminiz bulunmuyor." : "Henüz hiç bildiriminiz yok."} 
      iconHtml={icon('checkCircle', 48)}
    />
  {:else}
    {#each groupedNotifications as group (group.label)}
      <div class="notification-group" transition:slide={{ duration: getDuration('standard') }}>
        <h2 class="notification-group__title">{group.label}</h2>
        <div class="notification-list">
          {#each group.items as item (item.id)}
            <div 
              class="notification-card {item.is_read ? '' : 'notification-card--unread'}"
              transition:slide={{ duration: getDuration('fast') }}
            >
              <div class="notification-card__icon">
                {@html getIconForType(item.type)}
              </div>
              
              <div class="notification-card__content">
                <div class="notification-card__header">
                  <h3 class="notification-card__title">{item.title}</h3>
                  <span class="notification-card__time">{formatTimeAgo(item.created_at)}</span>
                </div>
                
                <p class="notification-card__message">{item.message}</p>
                
                <div class="notification-card__actions">
                  {#if item.action}
                    <a href={item.action.href} class="btn btn--sm btn--primary">
                      {item.action.label}
                    </a>
                  {/if}
                  {#if !item.is_read}
                    <button class="btn btn--sm btn--ghost" onclick={() => handleMarkAsRead(item.id)}>
                      Okundu işaretle
                    </button>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/each}
  {/if}
</div>
