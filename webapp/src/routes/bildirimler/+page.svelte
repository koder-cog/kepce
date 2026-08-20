<script>
  /**
   * TODO: FUTURE ENHANCEMENTS (Gelecek Planlaması)
   * 1. Toast Entegrasyonu: Yeni bildirim geldiğinde _toast.css kullanılarak sağ alttan popup çıkartılabilir.
   * 2. Pagination / Infinite Scroll: Bildirim sayısı arttığında performansı korumak için IntersectionObserver ile sonsuz kaydırma (load more) eklenebilir.
   * 3. Swipe-to-Dismiss: Mobilde kullanıcıların kartı sağa/sola kaydırarak "Okundu" olarak işaretleyebilmesi (touch events ile) eklenebilir.
   * 4. Real-time (WebSockets/SSE): Sayfa yenilenmeden arka planda (SSE veya WebSocket) bildirimlerin anında UI'a düşmesi sağlanabilir.
   * 5. Web Push API: Sekme kapalıyken bile tarayıcı/işletim sistemi üzerinden "Yorumunuza yanıt geldi" bildirimi gönderilebilir.
   */

  import { api } from "@/api/index.js";
  import { icon } from "@/components/ui/icons.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Loader from "@/components/ui/Loader.svelte";
  import TabBar from "@/components/ui/TabBar.svelte";
  import { onMount } from "svelte";
  import { globalState } from "@/state.svelte.js";
  import { slide } from "svelte/transition";
  import { getDuration } from "@/lib/dom/motion.js";
  import { showToast } from "@/components/ui/toast.js";
  import { createModal } from "@/components/features/modal.js";
  import Seo from "@/components/ui/Seo.svelte";
  import Pagination from "@/components/ui/Pagination.svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";

  let user = $derived(globalState?.user);
  let notifications = $state([]);
  let isLoading = $state(true);
  let errorMsg = $state(null);

  let activeTab = $state("all"); // 'all' | 'unread'
  let paginationMode = $derived(globalState.paginationMode || "sayfali");
  let urlPage = $derived(parseInt($page.url.searchParams.get("sayfa") || "1", 10) || 1);

  // Pagination states
  let limit = 20;
  let currentPage = $state(1);

  // Derivated states
  let unreadCount = $derived(notifications.filter((n) => !n.is_read).length);
  let filteredNotifications = $derived(
    activeTab === "unread"
      ? notifications.filter((n) => !n.is_read)
      : notifications,
  );
  let totalItems = $derived(filteredNotifications.length);
  let totalPages = $derived(Math.ceil(totalItems / limit) || 1);

  $effect(() => {
    currentPage = Math.max(1, Math.min(urlPage, totalPages));
  });

  let paginatedNotifications = $derived.by(() => {
    if (paginationMode === "sayfali") {
      const start = (currentPage - 1) * limit;
      return filteredNotifications.slice(start, start + limit);
    }
    return filteredNotifications;
  });

  // Grouping logic
  let groupedNotifications = $derived.by(() => {
    const groups = {
      today: { label: "Bugün", items: [] },
      yesterday: { label: "Dün", items: [] },
      older: { label: "Daha Eski", items: [] },
    };

    const now = new Date();
    const today = new Date(
      now.getFullYear(),
      now.getMonth(),
      now.getDate(),
    ).getTime();
    const yesterday = today - 86400000;

    paginatedNotifications.forEach((n) => {
      const date = new Date(n.created_at).getTime();
      if (date >= today) {
        groups.today.items.push(n);
      } else if (date >= yesterday) {
        groups.yesterday.items.push(n);
      } else {
        groups.older.items.push(n);
      }
    });

    return [groups.today, groups.yesterday, groups.older].filter(
      (g) => g.items.length > 0,
    );
  });

  function handlePageChange(newPage) {
    currentPage = newPage;
    const url = new URL(window.location.href);
    if (newPage > 1) {
      url.searchParams.set("sayfa", String(newPage));
    } else {
      url.searchParams.delete("sayfa");
    }
    goto(url.pathname + url.search, { keepFocus: true, noScroll: false });
  }

  let hasFetched = false;

  $effect(() => {
    if (globalState.isReady && !hasFetched) {
      if (user) {
        hasFetched = true;
        loadNotifications();
      } else {
        isLoading = false;
        errorMsg = "Bildirimleri görüntülemek için giriş yapmalısınız.";
      }
    }
  });

  async function loadNotifications() {
    isLoading = true;
    errorMsg = null;
    try {
      notifications = await api.getNotifications();
    } catch (err) {
      errorMsg = "Bildirimler yüklenirken bir hata oluştu.";
    } finally {
      isLoading = false;
    }
  }

  async function handleMarkAsRead(id) {
    const n = notifications.find((n) => n.id === id);
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

    const previousState = notifications.map((n) => ({ ...n }));
    notifications = notifications.map((n) => ({ ...n, is_read: true }));

    try {
      await api.markAllAsRead();
      showToast("Tüm bildirimler okundu olarak işaretlendi.", "success");
    } catch (err) {
      notifications = previousState;
      showToast("Bildirimler güncellenemedi.", "error");
    }
  }

  async function handleDeleteNotification(id) {
    const previousState = [...notifications];
    notifications = notifications.filter((n) => n.id !== id);
    try {
      await api.deleteNotification(id);
      showToast("Bildirim silindi.", "success");
    } catch (err) {
      notifications = previousState;
      showToast(err.message || "Bildirim silinemedi.", "error");
    }
  }

  function confirmDeleteAll() {
    createModal({
      title: "Bildirimleri Temizle",
      iconHtml: icon("trash", 24),
      contentHtml:
        '<p class="modal-confirm-text">Tüm bildirimlerini silmek istediğine emin misin? Bu işlem geri alınamaz.</p>',
      buttons: [
        { label: "Vazgeç", variant: "secondary" },
        {
          label: "Tümünü Sil",
          variant: "danger",
          onClick: async () => {
            try {
              await api.deleteAllNotifications();
              notifications = [];
              showToast("Tüm bildirimler temizlendi.", "success");
              return true;
            } catch (err) {
              showToast(err.message || "Bildirimler temizlenemedi.", "error");
              return false;
            }
          },
        },
      ],
    });
  }

  function getIconForType(type) {
    switch (type) {
      case "system":
        return icon("info", 24);
      case "achievement":
        return icon("star", 24);
      case "comment":
        return icon("chat", 24);
      case "moderation":
        return icon("check", 24);
      default:
        return icon("bell", 24);
    }
  }

  function formatTimeAgo(isoString) {
    const diff = Date.now() - new Date(isoString).getTime();
    const minutes = Math.floor(diff / 60000);
    const hours = Math.floor(minutes / 60);

    if (minutes < 60) return `${Math.max(1, minutes)} dk önce`;
    if (hours < 24) return `${hours} saat önce`;

    const d = new Date(isoString);
    return `${d.getDate().toString().padStart(2, "0")}.${(d.getMonth() + 1).toString().padStart(2, "0")}.${d.getFullYear()}`;
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
    <div class="notification-page__header-actions">
      {#if unreadCount > 0}
        <button
          class="btn btn--secondary btn--sm btn--squish"
          onclick={handleMarkAllAsRead}
        >
          {@html icon("check", 14)}
          Tümünü okundu işaretle
        </button>
      {/if}
      {#if notifications.length > 0}
        <button
          class="btn btn--outline btn--sm btn--squish"
          onclick={confirmDeleteAll}
        >
          {@html icon("trash", 14)}
          Tümünü temizle
        </button>
      {/if}
      {#if paginationMode === "sayfali" && totalPages > 1}
        <Pagination
          compact={true}
          page={currentPage}
          {totalPages}
          {totalItems}
          onPageChange={handlePageChange}
        />
      {/if}
    </div>
  </div>

  <TabBar
    bind:activeId={activeTab}
    tabs={[
      { id: "all", label: "Tümü", icon: icon("bell", 18) },
      {
        id: "unread",
        label: "Okunmayanlar",
        icon: icon("eyeSlash", 18),
        badge:
          unreadCount > 0
            ? unreadCount > 99
              ? "99+"
              : unreadCount
            : undefined,
      },
    ]}
  />

  {#if isLoading}
    <div class="stats-placeholder">
      <Loader size={48} />
    </div>
  {:else if !user}
    <EmptyState
      title="Oturum Açın"
      desc="Bildirimlerinizi görüntülemek, yorumlarınıza gelen yanıtları ve kazandığınız rozetleri takip etmek için giriş yapın."
      iconName="bell"
    >
      <div class="u-flex u-gap-md u-justify-center u-mt-md">
        <a href="/giris" class="btn btn--primary">Giriş Yap</a>
        <a href="/kayit" class="btn btn--secondary">Hesap Oluştur</a>
      </div>
    </EmptyState>
  {:else if errorMsg}
    <EmptyState
      statusCode={500}
      desc={errorMsg}
      actionLabel="Tekrar Dene"
      onAction={loadNotifications}
    />
  {:else if filteredNotifications.length === 0}
    <EmptyState
      desc={activeTab === "unread"
        ? "Okunmamış bildiriminiz bulunmuyor."
        : "Henüz hiç bildiriminiz yok."}
      iconHtml={icon("checkCircle", 48)}
    />
  {:else}
    {#each groupedNotifications as group (group.label)}
      <div
        class="notification-group"
        transition:slide={{ duration: getDuration("standard") }}
      >
        <h2 class="notification-group__title">{group.label}</h2>
        <div class="notification-list">
          {#each group.items as item (item.id)}
            <div
              class="notification-card {item.is_read
                ? ''
                : 'notification-card--unread'}"
              transition:slide={{ duration: getDuration("fast") }}
            >
              <div class="notification-card__icon">
                {@html getIconForType(item.type)}
              </div>

              <div class="notification-card__content">
                <div class="notification-card__header">
                  <h3 class="notification-card__title">{item.title}</h3>
                  <span class="notification-card__time"
                    >{formatTimeAgo(item.created_at)}</span
                  >
                </div>

                <p class="notification-card__message">{item.message}</p>

                <div class="notification-card__actions">
                  {#if item.action_href}
                    <a
                      href={item.action_href}
                      class="btn btn--sm btn--primary btn--squish"
                    >
                      {item.action_label || "Görüntüle"}
                    </a>
                  {/if}
                  {#if !item.is_read}
                    <button
                      class="btn btn--sm btn--secondary btn--squish"
                      onclick={() => handleMarkAsRead(item.id)}
                    >
                      Okundu işaretle
                    </button>
                  {/if}
                  <button
                    class="btn btn--sm btn--icon-only btn--ghost btn--squish notification-card__delete-btn"
                    title="Bildirimi sil"
                    onclick={() => handleDeleteNotification(item.id)}
                  >
                    {@html icon("trash", 14)}
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/each}

    {#if paginationMode === "sayfali" && totalPages > 1}
      <Pagination
        page={currentPage}
        {totalPages}
        {totalItems}
        onPageChange={handlePageChange}
      />
    {/if}
  {/if}
</div>
