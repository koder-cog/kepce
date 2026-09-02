<script>
  let { answer } = $props();

  // Döviz Çevirici için canlı dinamik etkileşim
  let fromAmount = $state(1);

  $effect(() => {
    if (answer?.type === "currency") {
      fromAmount = answer.fromAmount;
    }
  });

  let toAmount = $derived(
    answer?.type === "currency"
      ? (fromAmount * (answer.toRate / answer.fromRate)).toLocaleString("tr-TR", {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2
        })
      : ""
  );
</script>

{#if answer}
  <div class="c-answer-card" role="region" aria-label="Hızlı Yanıt">
    {#if answer.type === "currency"}
      <!-- Döviz / Kur Kartı -->
      <div class="c-answer-card__currency">
        <div class="c-answer-card__head">
          <span class="c-answer-card__currency-tag">{answer.fromCurrency} / {answer.toCurrency}</span>
          <span class="c-answer-card__date">{answer.date} Verisi</span>
        </div>

        <div class="c-answer-card__main-val">
          <span class="c-answer-card__target-num">{toAmount}</span>
          <span class="c-answer-card__target-unit">{answer.toCurrencyName}</span>
        </div>

        <div class="c-answer-card__rate-sub">
          1 {answer.fromCurrency} = {(answer.toRate / answer.fromRate).toLocaleString("tr-TR", { minimumFractionDigits: 4, maximumFractionDigits: 4 })} {answer.toCurrency}
        </div>

        <div class="c-answer-card__inputs">
          <div class="c-answer-input-group">
            <input
              type="number"
              class="c-answer-input"
              bind:value={fromAmount}
              min="0"
              step="any"
              aria-label={answer.fromCurrencyName}
            />
            <span class="c-answer-input-symbol">{answer.fromCurrency}</span>
          </div>
          <span class="c-answer-eq">=</span>
          <div class="c-answer-input-group">
            <input
              type="text"
              class="c-answer-input is-readonly"
              value={toAmount}
              readonly
              aria-label={answer.toCurrencyName}
            />
            <span class="c-answer-input-symbol">{answer.toCurrency}</span>
          </div>
        </div>
      </div>

    {:else if answer.type === "calculator"}
      <!-- Hesap Makinesi / Matematik Kartı -->
      <div class="c-answer-card__calc">
        <div class="c-answer-card__expr">{answer.expression} =</div>
        <div class="c-answer-card__calc-result">{answer.result}</div>
      </div>

    {:else}
      <!-- Standart / Genel Hızlı Yanıt -->
      <div class="c-answer-card__generic">
        <div class="c-answer-card__generic-title">{answer.title || "Hızlı Yanıt"}</div>
        <div class="c-answer-card__generic-text">{answer.content}</div>
      </div>
    {/if}
  </div>
{/if}
