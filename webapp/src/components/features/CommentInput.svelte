<script>
  import { globalState, authActions } from '../../state.svelte.js';
  import { onMount } from 'svelte';

    import { sentiments, recommendations, resolveLabel } from '../../utils/feedback-data.js';
    import { icon } from '../ui/icons.js';
    import { api } from '../../api/index.js';
    import Dropdown from './Dropdown.svelte';
    import { showToast } from '../ui/toast.js';
    import { normalizeItems, groupItems } from '../../utils/menu.js';
    import { openSpamInfoModal } from '../../lib/dom/spam-modal.js';

    let { menuObj = {}, parentId = null } = $props();

    let mode = $state('quick');
    let selectedFood = $state('all');
    let selectedSentimentId = $state(null);
    let selectedRecommendationId = $state(null);
    let freeText = $state('');
    let isSubmitting = $state(false);

    // Görev #16: Yazılan yorum, kullanıcı sayfadan yanlışlıkla ayrılsa bile
    // kaybolmasın diye menü+thread bazlı localStorage taslağı tutulur.
    let draftKey = $derived(
        menuObj?.id ? `kepce_comment_draft:${menuObj.id}:${parentId || 'root'}` : null
    );

    onMount(() => {
        if (!draftKey) return;
        try {
            const saved = localStorage.getItem(draftKey);
            if (saved) {
                freeText = saved;
                // Görev #16 Hata Çözümü: Taslak yüklendiyse kullanıcının
                // bunu görebilmesi için doğrudan "Serbest" moda geçir.
                mode = 'freetext';
            }
        } catch {}
    });

    // Debounce'lu taslak kaydı: her tuşta değil, yazım duraklayınca yaz.
    function saveDraftNow() {
        if (!draftKey) return;
        const text = freeText;
        try {
            if (text.trim()) {
                localStorage.setItem(draftKey, text);
            } else {
                localStorage.removeItem(draftKey);
            }
        } catch {}
    }

    $effect(() => {
        if (!draftKey) return;
        // Serbest yazım değiştiğinde, ancak kullanıcı gerçekten bir şey yazdıysa
        // (örneğin sadece silme yaptıysa da kaydetmeli). 
        // Burada derived bağımlılık `freeText` üzerine.
        const t = setTimeout(saveDraftNow, 400);
        return () => clearTimeout(t);
    });

    // Görev #16: Kullanıcı sayfadan aniden çıkarsa 400ms beklemeyi es geçip
    // hemen kaydet, böylece hiçbir zaman data kaybı (data loss) olmaz.
    function handleBeforeUnload() {
        saveDraftNow();
    }

    import { onDestroy } from 'svelte';
    onDestroy(() => {
        // Bileşen (yani sayfa) kapatılırken taslak varsa toast göster.
        if (draftKey && freeText.trim()) {
            saveDraftNow();
            showToast('Gönderilmeyen yorumunuz taslak olarak kaydedildi.', { timeout: 4000 });
        }
    });

    let isVerified = $derived(globalState?.user?.is_verified);
    let isUserLoggedIn = $derived(!!globalState?.user);

    let normalizedItems = $derived(groupItems(normalizeItems(menuObj)));
    let flattenedDishes = $derived(normalizedItems.flatMap(item => item.dishes || []));

    let foodOptions = $derived([
        { value: 'all', label: 'Tüm öğün' },
        ...flattenedDishes.map(d => ({ value: d.name, label: d.name }))
    ]);

    let sentimentOptions = $derived(sentiments.map(s => ({
        value: s.id.toString(),
        label: resolveLabel(s.label, selectedFood).trim()
    })));

    let recommendationOptions = recommendations.map(r => ({ value: r.id.toString(), label: r.label }));

    let submitDisabled = $derived(
        mode === 'quick' 
            ? !selectedSentimentId 
            : (!freeText.trim() || freeText.length > 280)
    );

    let submitTooltipMsg = $derived(
        mode === 'quick' && !selectedSentimentId ? "Lütfen bir durum seçin" :
        mode === 'freetext' && !freeText.trim() ? "Lütfen yorumunuzu yazın" :
        mode === 'freetext' && freeText.length > 280 ? "Yorumunuz en fazla 280 karakter olabilir" :
        "Gönder"
    );

    // Eski kodda default `selectedFood` `'Yemek'` string'iydi ve placeholder
    // için kullanılıyordu. Şu anki default `'all'`; placeholder ise Dropdown
    // bileşeninde ayrı tutuluyor. Yani `'Yemek'` koşulu artık hiçbir zaman
    // doğrulanmıyor ve eskiden "ilk gerçek yemeği otomatik seç" davranışını
    // sağlayan blok ölü koddu. Kullanıcı ilk yemeği seçtiğinde dropdown'ın
    // seçili öğesi zaten güncelleniyor; ekstra bir otomatik seçim gereksiz.
    // (Refactor sonrası blok bilinçli olarak kaldırıldı.)

    function handleTextInput(e) {
        const cleanValue = e.target.value.replace(/\n\n\n+/g, '\n\n');
        if (e.target.value !== cleanValue) {
            e.target.value = cleanValue;
        }
        freeText = e.target.value;
        e.target.style.height = 'auto';
        e.target.style.height = Math.min(e.target.scrollHeight, 200) + 'px';
    }

    function handleTextKeydown(e) {
        // Çok satırlı giriş serbest (Enter çalışır)
    }

    async function handleSubmit() {
        if (submitDisabled || isSubmitting) return;

        if (!isUserLoggedIn) {
            authActions.triggerLogin();
            return;
        }

        // `dish_id` artık yalnızca gerçekten bir yemek seçildiğinde
        // gönderiliyor; `1` sabit fallback'i, boş menü kartından gelen
        // yorumun ID=1 olan (çoğunlukla "Kahvaltı" gibi placeholder)
        // kayda sapmasına neden oluyordu. Kullanıcı "Tüm öğün" seçtiyse
        // backend zaten `dish_id` olmadan kabul ediyor.
        let payload = {
            menu_id: menuObj.id,
            sentiment: 'neutral',
            parent_id: parentId,
            is_tabldot: mode === 'quick'
        };

        if (selectedFood !== 'all') {
            const matchedDish = flattenedDishes.find(d => d.name === selectedFood);
            if (matchedDish?.id) {
                payload.dish_id = matchedDish.id;
            }
        }

        if (mode === 'quick') {
            if (!selectedSentimentId) {
                showToast('Lütfen bir durum değerlendirmesi seçin.', 'error');
                return;
            }
            const sId = parseInt(selectedSentimentId);
            const rId = selectedRecommendationId === 'clear' ? null : parseInt(selectedRecommendationId);
            const s = sentiments.find(s => s.id === sId);
            const r = recommendations.find(r => r.id === rId);

            payload.sentiment = s.sentiment;
            payload.tag_ids = [sId, rId].filter(id => id !== null && !isNaN(id));

            let comment = '';
            if (selectedFood === 'all') {
                const mealName = menuObj.meal_type === 'breakfast' ? 'Kahvaltı' : 'Yemek';
                const baseLabel = resolveLabel(s.label, mealName).trim();
                comment = `${mealName} ${baseLabel}.`;
            } else {
                comment = `${selectedFood}${resolveLabel(s.label, selectedFood)}.`;
            }
            
            if (r) {
                comment += ` ${r.label}.`;
            }
            payload.comment = comment;
        } else {
            if (!freeText.trim()) {
                showToast('Lütfen bir yorum yazın.', 'error');
                return;
            }
            if (freeText.length > 280) {
                showToast('Yorum 280 karakterden uzun olamaz.', 'error');
                return;
            }
            payload.comment = freeText;
        }

        try {
            isSubmitting = true;
            await api.submitVote(payload);
            showToast('Gönderildi!', 'success');
            
            selectedSentimentId = null;
            selectedRecommendationId = null;
            freeText = '';
            // Başarılı gönderimde taslak artık gereksiz (#16)
            if (draftKey) {
                try { localStorage.removeItem(draftKey); } catch {}
            }
            
            window.dispatchEvent(new CustomEvent('comment-submitted', { detail: { menuId: menuObj.id } }));
        } catch (err) {
            const isSpam = err.message && err.message.includes('spam');
            showToast(err.message, {
                type: 'error',
                action: isSpam ? {
                    text: 'Bilgi',
                    callback: () => openSpamInfoModal()
                } : null
            });
        } finally {
            isSubmitting = false;
        }
    }
</script>

<svelte:window onbeforeunload={handleBeforeUnload} />


    <div class="ci-container {mode === 'freetext' ? 'ci-container--freetext' : ''}">
        {#if !isUserLoggedIn || !isVerified}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div 
                class="ci-auth-overlay" 
                onclick={(e) => { 
                    e.stopPropagation(); 
                    if (!isUserLoggedIn) {
                        authActions.triggerLogin(); 
                    } else {
                        showToast('Yorum yapabilmek için e-postanızı onaylamalısınız.', { type: 'warning' });
                    }
                }}
            ></div>
        {/if}
        <div class="ci-capsule">
            {#if mode === 'quick'}
                <div class="ci-left">
                    <div class="ci-pills">
                        <div class="ci-pill">
                            <Dropdown options={foodOptions} bind:value={selectedFood} placeholder="Yemek" variant="ghost" />
                        </div>
                        <div class="ci-pill">
                            {#key selectedFood}
                                <Dropdown options={sentimentOptions} bind:value={selectedSentimentId} placeholder="Durum..." variant="ghost" />
                            {/key}
                        </div>
                        <div class="ci-pill">
                            <Dropdown 
                                options={recommendationOptions} 
                                bind:value={selectedRecommendationId} 
                                placeholder="Tavsiye..." 
                                variant="ghost" 
                                specialItem={{ label: 'Seçimi temizle' }}
                                onSpecialClick={() => selectedRecommendationId = null}
                            />
                        </div>
                    </div>
                </div>
            {:else}
                <textarea 
                    class="ci-left ci-input" 
                    placeholder="Tartışmaya katıl..." 
                    rows="1"
                    value={freeText}
                    oninput={handleTextInput}
                    onkeydown={handleTextKeydown}
                ></textarea>
            {/if}

            <div class="ci-actions">
                {#if mode === 'freetext'}
                    <span class="ci-char-count {freeText.length > 280 ? 'is-error' : ''}">
                        {freeText.length}/280
                    </span>
                {/if}
                <div class="ci-mode-trigger">
                    <Dropdown 
                        options={[
                            { value: 'quick', label: 'Tabldot' },
                            { value: 'freetext', label: 'Serbest' }
                        ]} 
                        bind:value={mode} 
                        placeholder="Mod seç" 
                        variant="ghost" 
                    />
                </div>
                <button 
                    class="ci-submit" 
                    class:is-disabled={submitDisabled || isSubmitting}
                    data-tooltip={submitTooltipMsg}
                    onclick={handleSubmit}
                >
                    {@html icon('send', 20)}
                </button>
            </div>
        </div>
    </div>
