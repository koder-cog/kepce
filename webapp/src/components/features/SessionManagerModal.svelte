<script>
    import { onMount } from "svelte";
    import Modal from "@/components/ui/Modal.svelte";
    import { icon } from "@/components/ui/icons.js";
    import { api } from "@/api/index.js";
    import { showToast } from "@/components/ui/toast.js";
    import { parseUserAgent } from "@/utils/device.js";

    let { onClose } = $props();

    let sessions = $state([]);
    let loading = $state(true);

    onMount(async () => {
        try {
            sessions = await api.getSessions();
        } catch (e) {
            showToast("Oturumlar yüklenemedi.", "error");
        } finally {
            loading = false;
        }
    });

    async function handleRevoke(sessionId) {
        try {
            await api.revokeSession(sessionId);
            sessions = sessions.filter(s => s.id !== sessionId);
            showToast("Cihaz oturumu başarıyla kapatıldı.", "success");
        } catch (e) {
            showToast(e.message || "Oturum kapatılamadı.", "error");
        }
    }
</script>

<Modal options={{ title: "Kayıtlı cihazlar", iconHtml: icon("laptop", 24) }} {onClose}>
    {#snippet children()}
        <div class="c-boxed-list session-list">
            {#if loading}
                <div class="u-text-center u-p-md u-color-muted">Yükleniyor...</div>
            {:else if sessions.length === 0}
                <div class="u-text-center u-p-md u-color-muted">Aktif oturum bulunamadı.</div>
            {:else}
                {#each sessions as session (session.id)}
                    {@const device = parseUserAgent(session.user_agent)}
                    <div class="c-list-row session-row">
                        <div class="c-list-row__info u-flex u-gap-sm session-row-info">
                            <div class="session-icon-wrap">
                                {@html icon(device.icon, 20)}
                            </div>
                            <div class="session-details">
                                <div class="c-list-row__title session-title">
                                    {device.os} - {device.browser}
                                    {#if session.is_current}
                                        <span class="c-badge session-badge">Şu anki cihaz</span>
                                    {/if}
                                </div>
                                <div class="c-list-row__desc session-desc">
                                    {session.ip_address || "Bilinmeyen IP"} • {new Date(session.last_used_at).toLocaleDateString("tr-TR", { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}
                                </div>
                            </div>
                            {#if !session.is_current}
                                <div class="c-list-row__control session-control">
                                    <button class="btn btn--danger btn--icon btn--squish" onclick={() => handleRevoke(session.id)} data-tooltip="Oturumu kapat">
                                        {@html icon("log-out", 18)}
                                    </button>
                                </div>
                            {/if}
                        </div>
                    </div>
                {/each}
            {/if}
        </div>
        <p class="u-text-sm u-color-muted u-mt-md session-hint">
            Şüpheli bir etkinlik görürseniz, ilgili cihazın oturumunu hemen kapatın ve şifrenizi değiştirin.
        </p>
    {/snippet}
    {#snippet footer()}
        <button class="btn btn--secondary" onclick={onClose}>Kapat</button>
    {/snippet}
</Modal>

<style>
    .session-list {
        margin: 0;
        padding: 0;
    }
    .session-row {
        align-items: center;
    }
    .session-row-info {
        align-items: center;
        width: 100%;
    }
    .session-icon-wrap {
        background: var(--color-surface-sunken);
        color: var(--color-text);
        border-radius: var(--radius-sm);
        padding: 8px;
        display: flex;
        align-items: center;
        justify-content: center;
        margin-right: 8px;
    }
    .session-details {
        flex: 1;
    }
    .session-title {
        display: flex;
        align-items: center;
        gap: 8px;
    }
    .session-badge {
        background: var(--color-accent-primary-light);
        color: var(--color-accent-primary);
        font-size: 0.7em;
        padding: 2px 6px;
    }
    .session-desc {
        font-size: 0.85em;
        opacity: 0.8;
        margin-top: 4px;
    }
    .session-control {
        margin-left: auto;
    }
    .session-hint {
        margin-top: var(--space-md);
    }
</style>
