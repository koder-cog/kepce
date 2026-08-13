<script>
  import { onMount } from 'svelte';
  import { api } from '@/api/index.js';
  import Loader from '@/components/ui/Loader.svelte';
  import { icon } from '@/components/ui/icons.js';
  import { showToast } from '@/components/ui/toast.js';
  import Dropdown from "@/components/features/Dropdown.svelte";

  let cities = $state([]);
  let botCitySelect = $state('');
  
  const now = new Date();
  let botMonthSelect = $state(String(now.getMonth() + 1).padStart(2, '0'));
  let botYearSelect = $state(now.getFullYear().toString());

  const monthsTR = [
    "Ocak", "Şubat", "Mart", "Nisan", "Mayıs", "Haziran", 
    "Temmuz", "Ağustos", "Eylül", "Ekim", "Kasım", "Aralık"
  ];

  let cityOptions = $derived(cities.map(c => ({ label: c.name, value: c.slug })));
  let monthOptions = $derived(monthsTR.map((m, i) => ({ label: m, value: (i + 1).toString().padStart(2, '0') })));
  let yearOptions = $derived([now.getFullYear() - 1, now.getFullYear(), now.getFullYear() + 1].map(y => ({ label: y.toString(), value: y.toString() })));

  let isLoading = $state(true);
  let cachedBotData = null;

  let aiOutputJson = $state('');
  let previewEntries = $state(null);
  let injectionCity = $state('');

  onMount(async () => {
    try {
      cities = await api.getCities();
      cities = [...cities].sort((a, b) => a.name.localeCompare(b.name, 'tr'));
      if (cities.length > 0) botCitySelect = cities[0].slug;
    } catch (err) {
      console.error(err);
    } finally {
      isLoading = false;
    }
  });

  async function getBotData() {
    const month = `${botYearSelect}-${botMonthSelect}`;
    if (!cachedBotData || cachedBotData._city !== botCitySelect || cachedBotData._month !== month) {
      cachedBotData = await api.exportMonthlyMenuForBot(botCitySelect, month);
      cachedBotData._city = botCitySelect;
      cachedBotData._month = month;
    }
    return cachedBotData;
  }

  async function exportForAI() {
    try {
      const data = await getBotData();
      await navigator.clipboard.writeText(JSON.stringify(data.prompt, null, 2));
      showToast('İçerik/Girdi kopyalandı.');
    } catch (err) { showToast(err.message, 'error'); }
  }

  async function exportSchema() {
    try {
      const data = await getBotData();
      await navigator.clipboard.writeText(JSON.stringify(data.schema, null, 2));
      showToast('JSON Şeması kopyalandı.');
    } catch (err) { showToast(err.message, 'error'); }
  }

  function processAIOutput() {
    let raw = aiOutputJson.trim();
    if (!raw) return;
    try {
      const match = raw.match(/```(?:json)?\s*([\s\S]*?)\s*```/i);
      if (match && match[1]) {
        raw = match[1].trim();
      } else {
        const startIdx = Math.min(
          raw.indexOf('{') === -1 ? Infinity : raw.indexOf('{'),
          raw.indexOf('[') === -1 ? Infinity : raw.indexOf('[')
        );
        const endIdx = Math.max(raw.lastIndexOf('}'), raw.lastIndexOf(']'));
        if (startIdx !== Infinity && endIdx !== -1 && endIdx >= startIdx) {
          raw = raw.substring(startIdx, endIdx + 1);
        }
      }

      const data = JSON.parse(raw);
      const entries = Array.isArray(data) ? data : (data.yorum_listesi || data.gunler || []);

      if (entries.length === 0) throw new Error('Geçerli yorum verisi bulunamadı.');

      previewEntries = entries;
      injectionCity = botCitySelect;
      showToast('JSON başarıyla çözümlendi.');
    } catch (err) { showToast('JSON geçersiz: ' + err.message, 'error'); }
  }

  async function confirmInjection() {
    const comments = previewEntries.map(entry => ({
      date: entry.date || entry.tarih,
      commentary: entry.comment || entry.yorum
    }));

    try {
      const res = await api.injectBotComments(injectionCity, comments);
      showToast(`${res.updated_count} menü kaydı bot yorumuyla güncellendi!`);
      previewEntries = null;
      aiOutputJson = '';
    } catch (err) { showToast(err.message, 'error'); }
  }
</script>

<svelte:head>
  <title>Kepçe Bot - Moderasyon - Kepçe</title>
</svelte:head>

{#if isLoading}
  <div class="stats-placeholder">
    <Loader size={48} />
  </div>
{:else}
  <div class="kepce-bot-page">
    <!-- ADIM 1: Dışa aktar -->
    <section class="kepce-bot-step">
      <div class="kepce-bot-step__header">
        <div class="kepce-bot-step__title">
          <h3 class="u-text-md">1. Dışa aktar</h3>
          <p class="u-text-sm">
            Bot için aylık menü girdisini ve çıktı şemasını kopyala.
          </p>
        </div>
      </div>

      <div class="card admin-preview-card">
        <div class="form-group u-mb-md">
          <label for="bot-city" class="form-label">Şehir</label>
          <Dropdown options={cityOptions} bind:value={botCitySelect} />
        </div>

        <div class="u-flex u-gap-sm u-mb-md">
          <div class="form-group u-flex-1">
            <label for="bot-month" class="form-label">Ay</label>
            <Dropdown options={monthOptions} bind:value={botMonthSelect} />
          </div>
          <div class="form-group u-flex-1">
            <label for="bot-year" class="form-label">Yıl</label>
            <Dropdown options={yearOptions} bind:value={botYearSelect} />
          </div>
        </div>

        <div class="kepce-bot-actions">
          <button class="btn btn--primary u-flex-1" onclick={exportForAI}>
            Girdiyi kopyala
          </button>
          <button class="btn btn--secondary u-flex-1" onclick={exportSchema}>
            Şemayı kopyala
          </button>
        </div>
      </div>
    </section>

    <!-- ADIM 2: Çözümle -->
    <section class="kepce-bot-step">
      <div class="kepce-bot-step__header">
        <div class="kepce-bot-step__title">
          <h3 class="u-text-md">2. Çözümle</h3>
          <p class="u-text-sm">
            Modelin JSON çıktısını çözümle ve önizlemeyi oluştur.
          </p>
        </div>
      </div>

      <div class="card admin-preview-card">
        <div class="form-group u-mb-md">
          <textarea
            id="bot-json"
            bind:value={aiOutputJson}
            rows="8"
            class="form-textarea--resizable form-input"
            placeholder={'[{"tarih": "1 Nisan 2026", "yorum": "..."}]'}
          ></textarea>
        </div>

        <button class="btn btn--secondary btn--full" onclick={processAIOutput}>
          Çözümle
        </button>
      </div>
    </section>

    <!-- ADIM 3: Enjekte et -->
    {#if previewEntries && previewEntries.length > 0}
      <section class="kepce-bot-step" id="kepce-bot-preview-container">
        <div class="kepce-bot-step__header">
          <div class="kepce-bot-step__title">
            <h3 class="u-text-md">3. Enjekte et</h3>
            <p class="u-text-sm">
              Önizlenen bot yorumlarını onayla ve veritabanına yaz.
            </p>
          </div>
        </div>

        <div class="card admin-preview-card">
          <div class="u-flex u-flex-align-center u-justify-between u-mb-md">
            <h4 class="u-text-md">
              Önizleme ({previewEntries.length} Gün)
            </h4>
            <button class="btn btn--primary" onclick={confirmInjection}>
              Veritabanına yaz
            </button>
          </div>

          <div class="admin-preview-list">
            {#each previewEntries as entry}
              <div class="admin-preview-card__header">
                <strong>{entry.date || entry.tarih}:</strong>
              </div>
              <article class="admin-preview-card kepce-bot-preview-card">
                <p class="kepce-bot-preview-card__comment">
                  {entry.comment || entry.yorum}
                </p>
              </article>
            {/each}
          </div>
        </div>
      </section>
    {/if}
  </div>
{/if}
