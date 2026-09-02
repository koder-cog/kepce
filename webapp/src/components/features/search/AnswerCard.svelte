<script>
  let { answer } = $props();

  const CURRENCY_LIST = [
    { code: "TRY", name: "Türk Lirası" },
    { code: "USD", name: "Amerikan Doları" },
    { code: "EUR", name: "Euro" },
    { code: "GBP", name: "İngiliz Sterlini" },
    { code: "JPY", name: "Japon Yeni" },
    { code: "CHF", name: "İsviçre Frangı" }
  ];

  let fromCurrency = $state("USD");
  let toCurrency = $state("TRY");
  let fromAmount = $state(1);
  let toAmount = $state(0);
  let rates = $state({ USD: 1, TRY: 48.27, EUR: 0.86, GBP: 0.74, JPY: 160.0, CHF: 0.81 });

  function calculateTo() {
    const fRate = rates[fromCurrency] || 1;
    const tRate = rates[toCurrency] || 1;
    const rate = tRate / fRate;
    toAmount = parseFloat((fromAmount * rate).toFixed(2));
  }

  function calculateFrom() {
    const fRate = rates[fromCurrency] || 1;
    const tRate = rates[toCurrency] || 1;
    const rate = tRate / fRate;
    if (rate > 0) {
      fromAmount = parseFloat((toAmount / rate).toFixed(2));
    }
  }

  $effect(() => {
    if (answer?.type === "currency") {
      fromCurrency = answer.fromCurrency;
      toCurrency = answer.toCurrency;
      fromAmount = answer.fromAmount;
      if (answer.allRates) {
        rates = answer.allRates;
      } else {
        rates = {
          ...rates,
          [answer.fromCurrency]: answer.fromRate,
          [answer.toCurrency]: answer.toRate
        };
      }
      calculateTo();
    }
  });

  function handleFromInput(e) {
    fromAmount = parseFloat(e.target.value) || 0;
    calculateTo();
  }

  function handleToInput(e) {
    toAmount = parseFloat(e.target.value) || 0;
    calculateFrom();
  }

  function handleFromCurrencyChange(e) {
    fromCurrency = e.target.value;
    calculateTo();
  }

  function handleToCurrencyChange(e) {
    toCurrency = e.target.value;
    calculateTo();
  }

  let fromCurrencyName = $derived(
    CURRENCY_LIST.find((c) => c.code === fromCurrency)?.name || fromCurrency
  );
  let toCurrencyName = $derived(
    CURRENCY_LIST.find((c) => c.code === toCurrency)?.name || toCurrency
  );
  let singleRate = $derived(
    ((rates[toCurrency] || 1) / (rates[fromCurrency] || 1)).toLocaleString("tr-TR", {
      minimumFractionDigits: 2,
      maximumFractionDigits: 4
    })
  );
  let formattedToAmount = $derived(
    toAmount.toLocaleString("tr-TR", { minimumFractionDigits: 2, maximumFractionDigits: 2 })
  );
</script>

{#if answer}
  <div class="c-answer-card" role="region" aria-label="Hızlı Yanıt">
    {#if answer.type === "currency"}
      <!-- Google Tarzı Sade Döviz Çevirici -->
      <div class="c-answer-card__currency-clean">
        <p class="c-answer-card__sub-title">
          {fromAmount.toLocaleString("tr-TR")} {fromCurrencyName} eşittir
        </p>

        <h2 class="c-answer-card__headline">
          {formattedToAmount} {toCurrencyName}
        </h2>

        <p class="c-answer-card__single-rate">
          1 {fromCurrency} = {singleRate} {toCurrency}
        </p>

        <!-- Çift Yönlü İnteraktif Dönüştürücü Kutuları -->
        <div class="c-answer-fx-grid">
          <!-- Kaynak Para Birimi -->
          <div class="c-answer-fx-box">
            <input
              type="number"
              class="c-answer-fx-input"
              value={fromAmount}
              oninput={handleFromInput}
              min="0"
              step="any"
              aria-label="Kaynak Tutar"
            />
            <select
              class="c-answer-fx-select"
              value={fromCurrency}
              onchange={handleFromCurrencyChange}
              aria-label="Kaynak Para Birimi"
            >
              {#each CURRENCY_LIST as c}
                <option value={c.code}>{c.name}</option>
              {/each}
            </select>
          </div>

          <!-- Hedef Para Birimi -->
          <div class="c-answer-fx-box">
            <input
              type="number"
              class="c-answer-fx-input"
              value={toAmount}
              oninput={handleToInput}
              min="0"
              step="any"
              aria-label="Hedef Tutar"
            />
            <select
              class="c-answer-fx-select"
              value={toCurrency}
              onchange={handleToCurrencyChange}
              aria-label="Hedef Para Birimi"
            >
              {#each CURRENCY_LIST as c}
                <option value={c.code}>{c.name}</option>
              {/each}
            </select>
          </div>
        </div>
      </div>

    {:else if answer.type === "calculator"}
      <!-- Hesap Makinesi / Matematik Kartı -->
      <div class="c-answer-card__calc">
        <div class="c-answer-card__sub-title">{answer.expression} =</div>
        <div class="c-answer-card__headline">{answer.result}</div>
      </div>

    {:else}
      <!-- Standart / Genel Hızlı Yanıt -->
      <div class="c-answer-card__generic">
        <div class="c-answer-card__sub-title">{answer.title || "Hızlı Yanıt"}</div>
        <div class="c-answer-card__headline">{answer.content}</div>
      </div>
    {/if}
  </div>
{/if}
