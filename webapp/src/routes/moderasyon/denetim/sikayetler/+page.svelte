<script>
  import { onMount } from "svelte";
  import { api } from "@/api/index.js";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Loader from "@/components/ui/Loader.svelte";
  import { icon } from "@/components/ui/icons.js";
  import { sanitizeText } from "@/utils/sanitize.js";
  import { showToast } from "@/components/ui/toast.js";
  import { createModal } from "@/components/features/modal.js";
  import { slide, fade } from "svelte/transition";
  import { flip } from "svelte/animate";
  import { getDuration } from "@/lib/dom/motion.js";
  import { page } from "$app/stores";
  import Dropdown from "@/components/features/Dropdown.svelte";

  // Tab mapping:
  // "content": İçerik Şikayetleri (Comment, User)
  // "menu": Menü & Bot Hataları
  // "contact": İletişim Mesajları
  let complaintTab = $derived(
    $page.url.searchParams.get("tip") === "hata" ? "menu" : 
    ($page.url.searchParams.get("tip") === "iletisim" ? "contact" : "content")
  );

  let activeReportFilter = $state("pending"); // 'pending' | 'resolved' | 'dismissed'
  let sourceFilter = $state("all"); // 'all' | 'kepce' | 'ara'

  let isLoading = $state(true);
  let allReports = $state([]);
  let contactMessages = $state([]);
  import Pagination from "@/components/ui/Pagination.svelte";
  import { goto } from "$app/navigation";

  let errorMsg = $state(null);
  let limit = 20;
  let urlPage = $derived(parseInt($page.url.searchParams.get("sayfa") || "1", 10) || 1);
  let currentPage = $state(1);

  // Filter lists based on tab, status and source
  let items = $derived.by(() => {
    if (complaintTab === "content") {
      return allReports.filter(r => (r.type === 'comment' || r.type === 'user') && r.status === activeReportFilter);
    } else if (complaintTab === "menu") {
      return allReports.filter(r => (r.type === 'menu' || r.type === 'bot') && r.status === activeReportFilter);
    } else if (complaintTab === "contact") {
      return contactMessages.filter(m => {
        const matchesStatus = m.status === activeReportFilter;
        const matchesSource = sourceFilter === "all" || (m.source || "kepce") === sourceFilter;
        return matchesStatus && matchesSource;
      });
    }
    return [];
  });

  let totalItems = $derived(items.length);
  let totalPages = $derived(Math.ceil(totalItems / limit) || 1);

  $effect(() => {
    currentPage = Math.max(1, Math.min(urlPage, totalPages));
  });

  let paginatedItems = $derived.by(() => {
    const start = (currentPage - 1) * limit;
    return items.slice(start, start + limit);
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

  $effect(() => {
    let ignore = false;
    async function fetchData() {
      isLoading = true;
      errorMsg = null;
      try {
        if (complaintTab === "contact") {
          let result = await api.getContactMessages();
          if (!ignore) contactMessages = result;
        } else {
          let result = await api.getReports('');
          if (!ignore) allReports = result;
        }
      } catch (err) {
        if (!ignore) errorMsg = err.message || "Bir hata oluştu.";
      } finally {
        if (!ignore) isLoading = false;
      }
    }
    fetchData();
    return () => { ignore = true; };
  });

  // Generic Update Status Action
  async function changeReportStatus(id, newStatus, isContact = false) {
    try {
      if (isContact) {
        await api.updateContactMessageStatus(id, newStatus);
        contactMessages = contactMessages.map(m => m.id === id ? { ...m, status: newStatus } : m);
        showToast("İletişim mesajı güncellendi.", "success");
      } else {
        await api.updateReportStatus(id, newStatus);
        allReports = allReports.map(r => r.id === id ? { ...r, status: newStatus } : r);
        showToast("Durum güncellendi.", "success");
      }
    } catch(err) {
      showToast(err.message, "error");
    }
  }

  function deleteReportPrompt(id, isContact = false) {
    createModal({
      title: "Kalıcı silme",
      iconHtml: icon("alert", 24),
      iconColor: "danger",
      contentHtml: "<p>Kalıcı olarak silmek üzeresin. Bu işlem geri alınamaz. Emin misin?</p>",
      buttons: [
        { label: "İptal", variant: "secondary" },
        {
          label: "Evet, Sil",
          variant: "danger",
          onClick: async (close) => {
            try {
              if (isContact) {
                await api.deleteContactMessage(id);
                contactMessages = contactMessages.filter(m => m.id !== id);
              } else {
                await api.deleteReport(id);
                allReports = allReports.filter(r => r.id !== id);
              }
              showToast("Kalıcı olarak silindi.", "danger");
              close();
            } catch (err) { showToast(err.message, "error"); }
          }
        }
      ]
    });
  }

  let expandedReplies = $state({});
  let loadingReplies = $state({});

  async function toggleReplies(id) {
    if (expandedReplies[id]) {
      delete expandedReplies[id];
      expandedReplies = { ...expandedReplies };
      return;
    }

    loadingReplies[id] = true;
    try {
      const res = await api.getContactMessageReplies(id);
      expandedReplies[id] = res;
      expandedReplies = { ...expandedReplies };
    } catch (err) {
      showToast(err.message || "Yanıt geçmişi yüklenemedi.", "error");
    } finally {
      loadingReplies[id] = false;
    }
  }

  function openReplyModal(item) {
    createModal({
      title: `${item.email} Yanıtla`,
      iconHtml: icon("send", 20),
      contentHtml: `
        <div class="u-mb-md">
          <p class="u-text-xs u-color-muted u-mb-sm">Konu: <strong>${sanitizeText(item.subject)}</strong></p>
          <div class="u-mb-sm u-flex u-gap-xs u-flex-wrap">
            <button type="button" class="btn btn--secondary btn--sm btn--squish" onclick="document.getElementById('reply-text-input').value = 'Merhaba, bildirdiğiniz menü bilgisi incelenmiş ve sistemde güncellenmiştir. Geri bildiriminiz için teşekkür ederiz.';">Menü Güncellendi</button>
            <button type="button" class="btn btn--secondary btn--sm btn--squish" onclick="document.getElementById('reply-text-input').value = 'Merhaba, karşılaştığınız teknik aksaklık giderilmiştir. Deneyiminiz için teşekkür ederiz.';">Hata Giderildi</button>
            <button type="button" class="btn btn--secondary btn--sm btn--squish" onclick="document.getElementById('reply-text-input').value = 'Merhaba, geri bildiriminiz ve öneriniz için teşekkür ederiz. İncelemeye alınmıştır.';">Öneri Alındı</button>
          </div>
          <label class="label u-text-xs">E-posta Yanıtı</label>
          <textarea id="reply-text-input" class="input u-width-full" rows="5" placeholder="Kullanıcıya iletilecek e-posta metnini yazınız..."></textarea>
        </div>
      `,
      buttons: [
        { label: "Vazgeç", variant: "secondary" },
        {
          label: "Gönder ve Çözüldü İşaretle",
          variant: "primary",
          onClick: async (close) => {
            const inputEl = document.getElementById("reply-text-input");
            const text = inputEl ? inputEl.value.trim() : "";
            if (!text) {
              showToast("Lütfen bir yanıt metni yazınız.", "warning");
              return;
            }
            try {
              await api.replyToContactMessage(item.id, text);
              showToast("E-posta yanıtı gönderildi ve durum güncellendi.", "success");
              contactMessages = contactMessages.map(m => m.id === item.id ? { ...m, status: "resolved" } : m);
              close();
            } catch (err) {
              showToast(err.message || "Yanıt gönderilemedi.", "error");
            }
          }
        }
      ]
    });
  }

  function formatDate(isoString) {
    if (!isoString) return 'Bilinmeyen Tarih';
    return isoString.substring(0, 10).replace(/-/g, '.') + ' ' + isoString.substring(11, 16);
  }

  const typeLabels = {
    comment: "Yorum Şikayeti",
    user: "Kullanıcı Şikayeti",
    menu: "Menü Hatası",
    bot: "Yapay Zeka Hatası"
  };
</script>

<svelte:head>
  <title>Şikayetler - Moderasyon - Kepçe</title>
</svelte:head>

<div class="u-mb-lg u-flex u-flex-justify-between u-flex-align-center u-flex-wrap u-gap-md">
  <div class="u-flex u-flex-align-center u-gap-sm u-flex-wrap">
    <Dropdown
      options={[
        { label: "Bekleyenler", value: "pending" },
        { label: "Çözülenler", value: "resolved" },
        { label: "Göz Ardı Edilenler", value: "dismissed" },
      ]}
      bind:value={activeReportFilter}
    />

    {#if complaintTab === 'contact'}
      <Dropdown
        options={[
          { label: "Tüm Kaynaklar", value: "all" },
          { label: "Kepçe (Ana Site)", value: "kepce" },
          { label: "Kepçe Ara (ara.kepce.org)", value: "ara" },
        ]}
        bind:value={sourceFilter}
      />
    {/if}
  </div>

  {#if totalPages > 1}
    <Pagination
      compact={true}
      page={currentPage}
      {totalPages}
      {totalItems}
      onPageChange={handlePageChange}
    />
  {/if}
</div>

<div id="complaint-list-container">
  {#if isLoading}
    <div class="stats-placeholder">
      <Loader size={48} />
    </div>
  {:else if errorMsg}
    <EmptyState statusCode={500} desc={errorMsg} />
  {:else if items.length === 0}
    <EmptyState
      iconName={"check"}
      title="Liste Boş"
      desc="Bu kategoride şu an işlem bekleyen öğe bulunmuyor."
    />
  {:else}
    <div class="comment-list">
      {#each paginatedItems as item (item.id)}
        <article class="comment-card" data-id={item.id} animate:flip={{ duration: getDuration(250) }} in:fade={{ duration: getDuration(200) }} out:slide={{ duration: getDuration(200) }}>
          <header class="comment-card__header-group u-mb-md">
            <div class="comment-card__meta">
              <strong class="u-text-base u-color-text">
                {complaintTab === 'contact' ? item.email : (item.reporter_id || "Anonim")}
              </strong>
              <span class="u-color-muted u-text-sm">&middot;</span>
              <span class="comment-card__date u-text-sm u-color-muted">{formatDate(item.created_at)}</span>
            </div>
            <div class="comment-card__meta u-mt-xs u-flex u-flex-align-center u-gap-xs">
              <span class="u-text-sm u-color-muted">#{item.id.toString().substring(0, 8)}</span>
              {#if complaintTab === 'contact'}
                {#if item.source === 'ara'}
                  <span class="badge badge--warning">Kepçe Ara</span>
                {:else}
                  <span class="badge badge--neutral">Kepçe</span>
                {/if}
              {/if}
            </div>
          </header>

          <div class="comment-card__body u-mb-md">
            {#if complaintTab === 'contact'}
              <div class="u-mb-xs"><strong>Kategori:</strong> <span class="u-color-text">{item.category}</span></div>
              <div class="u-mb-xs"><strong>Konu:</strong> <span class="u-color-text">{item.subject}</span></div>
              {#if item.page_url}
                <div class="u-mb-xs u-text-xs">
                  <strong>Sayfa:</strong>
                  <a href={item.page_url} target="_blank" class="u-link u-color-primary">{item.page_url}</a>
                </div>
              {/if}
              <div class="u-mt-md">
                <strong>Mesaj:</strong>
                <p class="u-mt-xs">{item.message}</p>
              </div>

              {#if expandedReplies[item.id]}
                <div class="ticket-replies-box">
                  <strong class="u-text-xs u-color-muted u-mb-xs u-display-block">GÖNDERİLEN YANITLAR</strong>
                  {#if expandedReplies[item.id].length === 0}
                    <p class="u-text-xs u-color-muted">Henüz bu mesaja yanıt gönderilmemiş.</p>
                  {:else}
                    {#each expandedReplies[item.id] as rep}
                      <div class="u-mb-sm u-pb-sm u-border-bottom">
                        <div class="u-flex u-flex-justify-between u-text-xs u-color-muted u-mb-xs">
                          <span>{rep.responder_username || 'Yönetici'}</span>
                          <span>{formatDate(rep.created_at)}</span>
                        </div>
                        <p class="u-text-sm u-color-text ticket-reply-body">{rep.reply_body}</p>
                      </div>
                    {/each}
                  {/if}
                </div>
              {/if}
            {:else}
              <div class="u-mb-xs"><strong>Tip:</strong> <span class="u-color-text">{typeLabels[item.type] || item.type}</span></div>
              <div class="u-mb-xs"><strong>Hedef:</strong> 
                {#if item.type === 'comment'}Yorum ID: {item.reported_comment_id}{:else if item.type === 'user'}Kullanıcı ID: {item.reported_user_id}{:else if item.type === 'menu'}Menü ID: {item.menu_id}{:else}Bilinmiyor{/if}
              </div>
              <div class="u-mb-xs"><strong>Sebep:</strong> <span class="u-color-text">{item.reason}</span></div>
              {#if item.description}
                <div class="u-mt-md">
                  <strong>Açıklama:</strong>
                  <p class="u-mt-xs">{item.description}</p>
                </div>
              {/if}
            {/if}
          </div>

          <footer class="comment-card__footer u-flex u-flex-wrap u-gap-xs">
            {#if complaintTab === 'contact'}
              <button class="btn btn--primary btn--squish" onclick={() => openReplyModal(item)}>
                {@html icon("send", 14)}
                E-posta ile Yanıtla
              </button>
              <button class="btn btn--secondary btn--squish" onclick={() => toggleReplies(item.id)}>
                {expandedReplies[item.id] ? "Yanıtları Gizle" : (loadingReplies[item.id] ? "Yükleniyor..." : "Yanıt Geçmişi")}
              </button>
            {/if}

            {#if activeReportFilter === 'pending'}
              <button class="btn btn--secondary btn--squish" onclick={() => changeReportStatus(item.id, 'dismissed', complaintTab === 'contact')}>Göz ardı et</button>
              <button class="btn btn--secondary btn--squish" onclick={() => changeReportStatus(item.id, 'resolved', complaintTab === 'contact')}>Çözüldü işaretle</button>
            {:else}
              <button class="btn btn--secondary btn--squish" onclick={() => changeReportStatus(item.id, 'pending', complaintTab === 'contact')}>Geri al (inceleniyor)</button>
              {#if complaintTab !== 'contact'}
                <button class="btn btn--danger btn--squish" onclick={() => deleteReportPrompt(item.id, false)}>Kalıcı sil</button>
              {/if}
            {/if}
          </footer>
        </article>
      {/each}
    </div>

    {#if totalPages > 1}
      <Pagination
        page={currentPage}
        {totalPages}
        {totalItems}
        onPageChange={handlePageChange}
      />
    {/if}
  {/if}
</div>
