<script>
  import { icon } from "@/components/ui/icons.js";

  let { answer } = $props();

  const CURRENCY_LIST = [
    { code: "TRY", name: "Türk Lirası" },
    { code: "USD", name: "Amerikan Doları" },
    { code: "EUR", name: "Euro" },
    { code: "GBP", name: "İngiliz Sterlini" },
    { code: "JPY", name: "Japon Yeni" },
    { code: "CHF", name: "İsviçre Frangı" }
  ];

  const DEFAULT_RATES = {
    USD: 1,
    TRY: 48.27,
    EUR: 0.86,
    GBP: 0.74,
    JPY: 160.0,
    CHF: 0.81
  };

  let fromCurrency = $state("USD");
  let toCurrency = $state("TRY");
  let fromAmount = $state(1);

  // Gelen yanıta göre başlangıç değerlerini senkronize et (Döngüsüz / Saf)
  $effect(() => {
    if (answer?.type === "currency") {
      fromCurrency = answer.fromCurrency || "USD";
      toCurrency = answer.toCurrency || "TRY";
      fromAmount = answer.fromAmount ?? 1;
    }
  });

  let rates = $derived(answer?.allRates || DEFAULT_RATES);

  // Güncel kur oranı
  let currentRate = $derived.by(() => {
    const f = rates[fromCurrency] || 1;
    const t = rates[toCurrency] || 1;
    return t / f;
  });

  // Hedef tutar (anlık türetilmiş)
  let toAmount = $derived(parseFloat((fromAmount * currentRate).toFixed(2)));

  function handleFromInput(e) {
    const val = parseFloat(e.target.value);
    fromAmount = isNaN(val) ? 0 : val;
  }

  function handleToInput(e) {
    const val = parseFloat(e.target.value);
    if (!isNaN(val) && currentRate > 0) {
      fromAmount = parseFloat((val / currentRate).toFixed(2));
    } else {
      fromAmount = 0;
    }
  }

  let fromCurrencyName = $derived(
    CURRENCY_LIST.find((c) => c.code === fromCurrency)?.name || fromCurrency
  );
  let toCurrencyName = $derived(
    CURRENCY_LIST.find((c) => c.code === toCurrency)?.name || toCurrency
  );

  let formattedToAmount = $derived(
    toAmount.toLocaleString("tr-TR", { minimumFractionDigits: 2, maximumFractionDigits: 2 })
  );

  let formattedSingleRate = $derived(
    currentRate.toLocaleString("tr-TR", { minimumFractionDigits: 2, maximumFractionDigits: 4 })
  );
</script>

{#if answer}
  <div class="c-answer-card" role="region" aria-label="Hızlı Yanıt">
    {#if answer.type === "currency"}
      <div class="c-answer-card__currency-clean">
        <div class="c-answer-card__left">
          <p class="c-answer-card__sub-title">
            {fromAmount.toLocaleString("tr-TR")} {fromCurrencyName} eşittir
          </p>

          <h2 class="c-answer-card__headline">
            {formattedToAmount} {toCurrencyName}
          </h2>

          <p class="c-answer-card__single-rate">
            1 {fromCurrency} = {formattedSingleRate} {toCurrency}
          </p>
        </div>

        <!-- Google Tarzı Dikey Yığılmış İki Dönüştürücü Kutusu (Sağ Taraf) -->
        <div class="c-answer-fx-stack">
          <!-- Üst Satır: Kaynak Tutar + Para Birimi -->
          <div class="c-answer-fx-row">
            <input
              type="number"
              class="c-answer-fx-input"
              value={fromAmount}
              oninput={handleFromInput}
              min="0"
              step="any"
              aria-label="Kaynak Tutar"
            />
            <div class="c-answer-fx-divider"></div>
            <div class="c-answer-fx-select-wrap">
              <select
                class="c-answer-fx-select"
                bind:value={fromCurrency}
                aria-label="Kaynak Para Birimi"
              >
                {#each CURRENCY_LIST as c}
                  <option value={c.code}>{c.name}</option>
                {/each}
              </select>
              <span class="c-answer-fx-arrow">
                {@html icon("chevronDown", 14)}
              </span>
            </div>
          </div>

          <!-- Alt Satır: Hedef Tutar + Para Birimi -->
          <div class="c-answer-fx-row">
            <input
              type="number"
              class="c-answer-fx-input"
              value={toAmount}
              oninput={handleToInput}
              min="0"
              step="any"
              aria-label="Hedef Tutar"
            />
            <div class="c-answer-fx-divider"></div>
            <div class="c-answer-fx-select-wrap">
              <select
                class="c-answer-fx-select"
                bind:value={toCurrency}
                aria-label="Hedef Para Birimi"
              >
                {#each CURRENCY_LIST as c}
                  <option value={c.code}>{c.name}</option>
                {/each}
              </select>
              <span class="c-answer-fx-arrow">
                {@html icon("chevronDown", 14)}
              </span>
            </div>
          </div>
        </div>
      </div>

    {:else if answer.type === "calculator"}
      <div class="c-answer-card__calc">
        <div class="c-answer-card__sub-title">{answer.expression} =</div>
        <div class="c-answer-card__headline">{answer.result}</div>
      </div>

    {:else}
      <div class="c-answer-card__generic">
        <div class="c-answer-card__sub-title">{answer.title || "Hızlı Yanıt"}</div>
        <div class="c-answer-card__headline">{answer.content}</div>
      </div>
    {/if}
  </div>
{/if}
