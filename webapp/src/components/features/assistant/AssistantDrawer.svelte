<script>
  import { onMount, tick } from "svelte";
  import { icon } from "../../ui/icons.js";
  import { streamAssistant } from "../../../api/assistant.js";
  import { openBotReportModal } from "@/components/features/report-modal.js";
  import { page } from "$app/stores";
  import { timelineState } from "@/stores/timeline.svelte.js";
  const STORAGE_KEY = "kepce_assistant_chat";

  const LOADING_PHRASES = [
    "Bahaneleri sıralıyor...",
    "Kafasında bir mantığa oturtuyor...",
    "Aşçı ablayla pazarlık yapıyor...",
    "Yemekhane kurallarını tarıyor...",
    "Tabldot matematiği yapıyor...",
    "Sıraya kaynak yaparken gerekçe arıyor...",
    "Tencerenin dibini yokluyor...",
    "Yine bulgur çıkmasın diye ihtimalleri hesaplıyor...",
    "Ekmek sepetinde mantık arıyor...",
    "Menüde et arıyor...",
    "Yemekhane sırasını kolaçan ediyor...",
    "Turnikede kart basıyor...",
    "Çorbanın dibini sıyırıyor...",
  ];

  let isOpen = $state(false);
  let inputVal = $state("");
  let editingText = $state("");
  let isStreaming = $state(false);
  let messages = $state([]);
  let chatContainer = $state(null);
  let inputElement = $state(null);
  let editInputElement = $state(null);
  let loadingPhrase = $state(LOADING_PHRASES[0]);
  let phraseInterval = null;
  let abortStream = null;
  let editingIndex = $state(null);
  let isConfirmingClear = $state(false);
  let copiedMessageIndex = $state(null);
  let copyTimeout = null;
  let userScrolledUp = $state(false);

  let citySlug = $derived(
    $page.params.city_slug || timelineState?.currentCity || "istanbul",
  );

  function loadMessages() {
    if (typeof window === "undefined") return [];
    try {
      const stored = sessionStorage.getItem(STORAGE_KEY);
      return stored ? JSON.parse(stored) : [];
    } catch {
      return [];
    }
  }

  function saveMessages(msgs) {
    if (typeof window === "undefined") return;
    try {
      sessionStorage.setItem(STORAGE_KEY, JSON.stringify(msgs));
    } catch {}
  }

  function formatContent(text) {
    if (!text) return "";
    let formatted = text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");

    // Markdown linkleri [başlık](url)
    formatted = formatted.replace(
      /\[(.*?)\]\((https?:\/\/[^\s\)]+)\)/g,
      '<a href="$2" target="_blank" rel="noopener noreferrer" class="c-assistant-citation-link">$1</a>'
    );

    // Açık URL'ler (eğer zaten link içinde değilse)
    formatted = formatted.replace(
      /(^|[\s(])(https?:\/\/[^\s<)]+)/g,
      '$1<a href="$2" target="_blank" rel="noopener noreferrer" class="c-assistant-citation-link">$2</a>'
    );

    // Dipnot numaraları [1], [2] vb.
    formatted = formatted.replace(
      /\[(\d+)\]/g,
      '<span class="c-assistant-citation-badge">[$1]</span>'
    );

    // Kalın & italik
    return formatted
      .replace(/\*\*(.*?)\*\*/g, "<strong>$1</strong>")
      .replace(/\*(.*?)\*/g, "<em>$1</em>");
  }

  function handleScroll() {
    if (!chatContainer) return;
    const threshold = 80;
    const isNear =
      chatContainer.scrollHeight -
        chatContainer.scrollTop -
        chatContainer.clientHeight <
      threshold;
    userScrolledUp = !isNear;
  }

  function scrollToBottom() {
    if (chatContainer) {
      requestAnimationFrame(() => {
        if (chatContainer) {
          chatContainer.scrollTop = chatContainer.scrollHeight;
        }
      });
    }
  }

  function smartScroll() {
    if (!userScrolledUp) {
      scrollToBottom();
    }
  }

  export function open() {
    if (
      typeof document !== "undefined" &&
      document.documentElement.classList.contains("hide-ai")
    ) {
      return;
    }
    isOpen = true;
    userScrolledUp = false;
    if (typeof document !== "undefined") {
      document.body.style.overflow = "hidden";
    }
    scrollToBottom();
    setTimeout(() => {
      if (inputElement) inputElement.focus();
    }, 100);
  }

  export function close() {
    isOpen = false;
    if (typeof document !== "undefined") {
      document.body.style.overflow = "";
    }
    stopLoadingPhrase();
    isConfirmingClear = false;
    cancelEditing();
    if (abortStream) {
      abortStream();
      abortStream = null;
      isStreaming = false;
    }
  }

  function clearChat() {
    messages = [];
    saveMessages([]);
    isConfirmingClear = false;
    cancelEditing();
  }

  async function copyMessage(text, index) {
    if (!text || typeof navigator === "undefined") return;
    try {
      await navigator.clipboard.writeText(text);
      copiedMessageIndex = index;
      if (copyTimeout) clearTimeout(copyTimeout);
      copyTimeout = setTimeout(() => {
        copiedMessageIndex = null;
      }, 1500);
    } catch {}
  }

  function handleReportBot(e) {
    openBotReportModal(null, e?.currentTarget || null);
  }

  async function startEditing(index) {
    if (isStreaming) return;
    editingIndex = index;
    editingText = messages[index].content;
    await tick();
    if (editInputElement) {
      editInputElement.focus();
      autoResize(editInputElement);
    }
  }

  function cancelEditing() {
    editingIndex = null;
    editingText = "";
  }

  function saveAndResend() {
    const q = editingText.trim();
    if (!q || isStreaming) return;
    handleSubmit(q);
  }

  function stopStreaming() {
    if (abortStream) {
      abortStream();
      abortStream = null;
    }
    stopLoadingPhrase();
    isStreaming = false;
    if (messages.length > 0) {
      const lastIdx = messages.length - 1;
      if (messages[lastIdx].role === "assistant") {
        messages[lastIdx].isTyping = false;
        if (!messages[lastIdx].content && !messages[lastIdx].thought) {
          messages[lastIdx].content = "Yanıt durduruldu.";
        }
      }
    }
    saveMessages(messages);
  }

  function retryLastMessage() {
    if (isStreaming || messages.length === 0) return;
    let lastUserText = "";
    for (let i = messages.length - 1; i >= 0; i--) {
      if (messages[i].role === "user") {
        lastUserText = messages[i].content;
        messages = messages.slice(0, i);
        break;
      }
    }
    if (lastUserText) {
      handleSubmit(lastUserText);
    }
  }

  function shuffleArray(arr) {
    const copy = [...arr];
    for (let i = copy.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [copy[i], copy[j]] = [copy[j], copy[i]];
    }
    return copy;
  }

  function startLoadingPhrase() {
    let queue = shuffleArray(LOADING_PHRASES);
    let queueIdx = 0;
    loadingPhrase = queue[queueIdx];

    if (phraseInterval) clearInterval(phraseInterval);
    phraseInterval = setInterval(() => {
      queueIdx++;
      if (queueIdx >= queue.length) {
        queue = shuffleArray(LOADING_PHRASES);
        queueIdx = 0;
      }
      loadingPhrase = queue[queueIdx];
    }, 4000);
  }

  function stopLoadingPhrase() {
    if (phraseInterval) {
      clearInterval(phraseInterval);
      phraseInterval = null;
    }
  }

  function isLastUserMessage(index) {
    for (let i = messages.length - 1; i >= 0; i--) {
      if (messages[i].role === "user") {
        return i === index;
      }
    }
    return false;
  }

  function autoResize(el) {
    if (!el) return;
    el.style.height = "auto";
    el.style.height = Math.min(el.scrollHeight, 72) + "px";
  }

  function handleInputKeydown(e) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }

  function handleEditKeydown(e) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      const isUnchanged =
        editingIndex !== null &&
        editingText.trim() === messages[editingIndex]?.content.trim();
      if (editingText.trim() && !isUnchanged) {
        saveAndResend();
      }
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelEditing();
    }
  }

  function handleSubmit(overrideText) {
    const q = (overrideText || inputVal).trim();
    if (!q || isStreaming) return;

    if (editingIndex !== null) {
      messages = messages.slice(0, editingIndex);
      editingIndex = null;
      editingText = "";
    }

    inputVal = "";
    if (inputElement) {
      inputElement.style.height = "auto";
    }

    const userMsg = { role: "user", content: q };
    const botMsg = {
      role: "assistant",
      content: "",
      thought: "",
      thoughtOpen: false,
      isTyping: true,
      isOffline: false,
      isError: false,
    };

    messages = [...messages, userMsg, botMsg];
    saveMessages(messages);
    userScrolledUp = false;
    scrollToBottom();

    isStreaming = true;
    startLoadingPhrase();

    const apiPayload = {
      messages: messages
        .slice(0, -1)
        .map((m) => ({ role: m.role, content: m.content })),
      city: citySlug,
    };

    abortStream = streamAssistant(
      apiPayload,
      (chunk) => {
        stopLoadingPhrase();
        if (messages.length > 0) {
          const lastIdx = messages.length - 1;
          if (typeof chunk === "string") {
            if (chunk.startsWith("[offline_notice]")) {
              messages[lastIdx].isOffline = true;
              messages[lastIdx].content += chunk
                .replace("[offline_notice]", "")
                .trim();
            } else if (chunk.startsWith("[THOUGHT]")) {
              messages[lastIdx].thought =
                (messages[lastIdx].thought || "") + chunk.slice(9);
            } else {
              messages[lastIdx].content += chunk;
            }
          } else if (chunk && typeof chunk === "object") {
            if (chunk.type === "truncated") {
              messages[lastIdx].truncatedCount = chunk.dropped;
            } else if (chunk.type === "offline") {
              messages[lastIdx].isOffline = true;
              messages[lastIdx].content += chunk.text;
            } else if (chunk.type === "thought") {
              messages[lastIdx].thought =
                (messages[lastIdx].thought || "") + chunk.text;
            } else if (chunk.type === "content") {
              messages[lastIdx].content += chunk.text;
            }
          }
          smartScroll();
        }
      },
      () => {
        stopLoadingPhrase();
        if (messages.length > 0) {
          const lastIdx = messages.length - 1;
          messages[lastIdx].isTyping = false;
          if (!messages[lastIdx].content.trim()) {
            messages[lastIdx].content =
              "Yanıt oluşturulamadı. Lütfen tekrar deneyin.";
            messages[lastIdx].isError = true;
          }
        }
        saveMessages(messages);
        isStreaming = false;
        abortStream = null;
        smartScroll();
      },
      () => {
        stopLoadingPhrase();
        if (messages.length > 0) {
          const lastIdx = messages.length - 1;
          messages[lastIdx].isTyping = false;
          messages[lastIdx].isError = true;
          if (!messages[lastIdx].content.trim()) {
            messages[lastIdx].content =
              "Bağlantı kesildi veya model yanıt vermedi.";
          }
        }
        saveMessages(messages);
        isStreaming = false;
        abortStream = null;
        smartScroll();
      },
    );
  }

  function handleKeydown(e) {
    if (e.key === "Escape" && isOpen) {
      if (editingIndex !== null) {
        cancelEditing();
      } else if (isConfirmingClear) {
        isConfirmingClear = false;
      } else {
        close();
      }
    }
  }

  onMount(() => {
    messages = loadMessages();

    const handleOpenEvent = () => open();
    window.addEventListener("kepce:open-assistant", handleOpenEvent);

    return () => {
      window.removeEventListener("kepce:open-assistant", handleOpenEvent);
      stopLoadingPhrase();
      if (typeof document !== "undefined") {
        document.body.style.overflow = "";
      }
      if (copyTimeout) clearTimeout(copyTimeout);
      if (abortStream) abortStream();
    };
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Arka Plan Perdesi -->
<div
  class="c-assistant-backdrop {isOpen ? 'c-assistant-backdrop--open' : ''}"
  onclick={close}
  role="presentation"
></div>

<!-- Asistan Çekmecesi -->
<div
  class="c-assistant-drawer {isOpen ? 'c-assistant-drawer--open' : ''}"
  role="dialog"
  aria-modal="true"
  aria-label="Kepçe Bot"
>
  <div class="c-assistant-drawer__header">
    <div class="c-assistant-drawer__brand">
      <h3 class="c-assistant-drawer__title">Kepçe Bot</h3>
    </div>
    <div class="c-assistant-drawer__actions">
      {#if messages.length > 0}
        {#if isConfirmingClear}
          <div class="c-assistant-clear-confirm"><button type="button" class="c-assistant-clear-confirm__btn c-assistant-clear-confirm__btn--yes" onclick={clearChat} aria-label="Sohbeti Sil" title="Sohbeti Sil">{@html icon("check", 14)}</button><button type="button" class="c-assistant-clear-confirm__btn" onclick={() => (isConfirmingClear = false)} aria-label="İptal" title="İptal">{@html icon("close", 14)}</button></div>
        {:else}
          <button
            type="button"
            class="c-assistant-drawer__btn"
            onclick={() => (isConfirmingClear = true)}
            aria-label="Sohbeti Temizle"
            title="Sohbeti Temizle"
            disabled={isStreaming || editingIndex !== null}
          >
            {@html icon("trash", 18)}
          </button>
        {/if}
      {/if}
      <button
        type="button"
        class="c-assistant-drawer__btn"
        onclick={close}
        aria-label="Kapat"
        title="Kapat"
      >
        {@html icon("close", 20)}
      </button>
    </div>
  </div>

  <div class="c-assistant-drawer__body" bind:this={chatContainer} onscroll={handleScroll}>
    {#if messages.length === 0}
      <div class="c-assistant-welcome">
        <p class="c-assistant-welcome__title">Merhaba</p>
        <p class="c-assistant-welcome__desc">
          Günün menüsü, yemek saatleri, beslenme yardımı kuralları veya yurt
          hakkında merak ettiklerini sorabilirsin.
        </p>
      </div>
    {/if}

    {#each messages as msg, i (i)}
      <div
        class="c-assistant-msg {msg.role === 'user'
          ? 'c-assistant-msg--user-wrapper'
          : 'c-assistant-msg--bot-wrapper'}"
      >
        {#if msg.role === "user"}
          {#if editingIndex === i}
            <div class="c-assistant-msg--user c-assistant-msg--editing">
              <textarea
                bind:this={editInputElement}
                class="c-assistant-edit-textarea"
                rows="1"
                bind:value={editingText}
                oninput={(e) => autoResize(e.currentTarget)}
                onkeydown={handleEditKeydown}
                aria-label="Mesajı düzenle"
              ></textarea>
              <div class="c-assistant-edit-actions">
                <button
                  type="button"
                  class="c-assistant-edit-btn c-assistant-edit-btn--save"
                  onclick={saveAndResend}
                  disabled={!editingText.trim() || (editingIndex !== null && editingText.trim() === messages[editingIndex]?.content.trim())}
                >
                  Kaydet ve Gönder
                </button>
                <button
                  type="button"
                  class="c-assistant-edit-btn c-assistant-edit-btn--cancel"
                  onclick={cancelEditing}
                >
                  İptal
                </button>
              </div>
            </div>
          {:else}
            <div class="c-assistant-msg--user">
              <span class="c-assistant-msg__text">{msg.content}</span>
            </div>
            {#if isLastUserMessage(i) && !isStreaming && editingIndex === null}
              <div class="c-assistant-msg__user-actions"><button type="button" class="c-assistant-action-btn" onclick={() => startEditing(i)} aria-label="Mesajı Düzenle" title="Mesajı Düzenle">{@html icon("edit", 13)}</button></div>
            {/if}
          {/if}
        {:else}
          <div class="c-assistant-msg--bot">{#if msg.truncatedCount}<span class="c-assistant-msg__notice">Hafıza sınırı: İlk {msg.truncatedCount} eski mesaj işlenmedi.</span>{/if}{#if msg.isOffline}<span class="c-assistant-msg__notice">Model çevrimdışı</span>{/if}{#if msg.thought && msg.thought.trim()}<div class="c-assistant-thought"><button type="button" class="c-assistant-thought__toggle" onclick={() => { msg.thoughtOpen = !msg.thoughtOpen; }} aria-expanded={msg.thoughtOpen}><span class="c-assistant-thought__icon {msg.thoughtOpen ? 'c-assistant-thought__icon--open' : ''}">{@html icon("chevronRight", 13)}</span><span class="c-assistant-thought__label">Düşünce Süreci</span></button>{#if msg.thoughtOpen}<div class="c-assistant-thought__body"><p class="c-assistant-thought__text">{msg.thought.trim()}</p></div>{/if}</div>{/if}{#if !msg.content && msg.isTyping && isStreaming && (!msg.thought || !msg.thought.trim())}<span class="c-assistant-msg__loading">{loadingPhrase}</span>{:else if msg.content}<span class="c-assistant-msg__text">{@html formatContent(msg.content)}</span>{:else if msg.isTyping && isStreaming && msg.thought}<span class="c-assistant-msg__loading">Düşünüyor...</span>{/if}</div>

          {#if !msg.isTyping && msg.content && editingIndex === null}
            <div class="c-assistant-msg__bot-actions"><button type="button" class="c-assistant-action-btn" onclick={() => copyMessage(msg.content, i)} aria-label={copiedMessageIndex === i ? "Kopyalandı" : "Kopyala"} title={copiedMessageIndex === i ? "Kopyalandı" : "Kopyala"}>{#if copiedMessageIndex === i}{@html icon("check", 13)}{:else}{@html icon("copy", 13)}{/if}</button><button type="button" class="c-assistant-action-btn" onclick={handleReportBot} aria-label="Yanıtı Bildir" title="Yanıtı Bildir">{@html icon("flag", 13)}</button>{#if (msg.isError || (!isStreaming && i === messages.length - 1))}<button type="button" class="c-assistant-action-btn" onclick={retryLastMessage} aria-label="Yeniden dene" title="Yeniden dene">{@html icon("refresh", 13)}</button>{/if}</div>
          {/if}
        {/if}
      </div>
    {/each}
  </div>

  <div class="c-assistant-drawer__footer">
    <form
      class="c-assistant-form {editingIndex !== null
        ? 'c-assistant-form--locked'
        : ''}"
      onsubmit={(e) => {
        e.preventDefault();
        handleSubmit();
      }}
    >
      <textarea
        bind:this={inputElement}
        class="c-assistant-input"
        rows="1"
        placeholder={editingIndex !== null
          ? "Önce düzenlemeyi tamamlayın..."
          : "Menü, yemek veya saat sor..."}
        bind:value={inputVal}
        oninput={(e) => autoResize(e.currentTarget)}
        onkeydown={handleInputKeydown}
        disabled={isStreaming || editingIndex !== null}
        aria-label="Asistana soru sorun"
      ></textarea>
      {#if isStreaming}
        <button
          type="button"
          class="c-assistant-submit c-assistant-submit--stop"
          onclick={stopStreaming}
          aria-label="Durdur"
          title="Durdur"
        >
          {@html icon("stop", 14)}
        </button>
      {:else}
        <button
          type="submit"
          class="c-assistant-submit"
          disabled={!inputVal.trim() || editingIndex !== null}
          aria-label="Gönder"
          title="Gönder"
        >
          {@html icon("send", 16)}
        </button>
      {/if}
    </form>
    <p class="c-assistant-footer-note">
      Kepçe Bot yapay zeka tabanlıdır; resmi bağlayıcılığı yoktur ve hata yapabilir. Sohbet geçmişi
      tarayıcınızda geçici saklanır.
    </p>
  </div>
</div>
