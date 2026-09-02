<script>
  import Dropdown from "@/components/features/Dropdown.svelte";

  let { answer } = $props();

  const CURRENCY_LIST = [
    { code: "TRY", name: "Türk Lirası (₺)" },
    { code: "USD", name: "Amerikan Doları ($)" },
    { code: "EUR", name: "Euro (€)" },
    { code: "GBP", name: "İngiliz Sterlini (£)" },
    { code: "JPY", name: "Japon Yeni (¥)" },
    { code: "CHF", name: "İsviçre Frangı" },
    { code: "AUD", name: "Avustralya Doları" },
    { code: "CAD", name: "Kanada Doları" },
    { code: "CNY", name: "Çin Yuanı (¥)" },
    { code: "RUB", name: "Rus Rublesi (₽)" },
    { code: "SAR", name: "Suudi Arabistan Riyali" },
    { code: "AED", name: "BAE Dirhemi" },
    { code: "SEK", name: "İsveç Kronu" },
    { code: "NOK", name: "Norveç Kronu" },
    { code: "DKK", name: "Danimarka Kronu" },
    { code: "KRW", name: "Güney Kore Wonu (₩)" },
    { code: "INR", name: "Hindistan Rupisi (₹)" },
    { code: "BRL", name: "Brezilya Reali (R$)" },
    { code: "PLN", name: "Polonya Zlotisi" },
    { code: "CZK", name: "Çek Korunası" },
    { code: "BGN", name: "Bulgar Levası" },
    { code: "HUF", name: "Macar Forinti" },
    { code: "RON", name: "Rumen Leyi" },
    { code: "ILS", name: "İsrail Şekeli (₪)" },
    { code: "MXN", name: "Meksika Pezosu" },
    { code: "NZD", name: "Yeni Zelanda Doları" },
    { code: "SGD", name: "Singapur Doları" },
    { code: "HKD", name: "Hong Kong Doları" },
    { code: "ZAR", name: "Güney Afrika Randı" },
    { code: "THB", name: "Tayland Bahtı (฿)" },
    { code: "IDR", name: "Endonezya Rupiahı" },
    { code: "MYR", name: "Malezya Ringgiti" },
    { code: "PHP", name: "Filipinler Pezosu (₱)" }
  ];

  const dropdownOptions = CURRENCY_LIST.map((c) => ({
    value: c.code,
    label: c.name
  }));

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

  // Gelen yanıta göre başlangıç değerlerini senkronize et
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

        <!-- Google Tarzı Dikey Yığılmış İki Dönüştürücü Kutusu (Kepçe Dropdown ile) -->
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
            <div class="c-answer-fx-dropdown-wrap">
              <Dropdown
                variant="ghost"
                value={fromCurrency}
                options={dropdownOptions}
                onChange={(val) => {
                  fromCurrency = val;
                }}
              />
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
            <div class="c-answer-fx-dropdown-wrap">
              <Dropdown
                variant="ghost"
                value={toCurrency}
                options={dropdownOptions}
                onChange={(val) => {
                  toCurrency = val;
                }}
              />
            </div>
          </div>
        </div>
      </div>

    {:else if answer.type === "calculator"}
      <div class="c-answer-card__calc">
        <div class="c-answer-card__sub-title">{answer.expression} =</div>
        <div class="c-answer-card__headline">{answer.result}</div>
      </div>

    {:else if answer.type === "unit"}
      <div class="c-answer-card__unit">
        <div class="c-answer-card__sub-title">
          {answer.fromAmount} {answer.fromUnitName} ({answer.categoryName}) eşittir
        </div>
        <div class="c-answer-card__headline">
          {answer.toAmount.toLocaleString("tr-TR")} {answer.toUnitName}
        </div>
        <div class="c-answer-card__single-rate">
          {answer.formula}
        </div>
      </div>

    {:else if answer.type === "time"}
      <div class="c-answer-card__time">
        <div class="c-answer-card__sub-title">
          {answer.city}, {answer.country}
        </div>
        <div class="c-answer-card__headline">
          {answer.currentTime}
        </div>
        <div class="c-answer-card__single-rate">
          {answer.currentDate} · {answer.diffText}
        </div>
      </div>

    {:else if answer.type === "definition"}
      <div class="c-answer-card__definition">
        <div class="c-answer-card__sub-title">
          {answer.source}
        </div>
        <h2 class="c-answer-card__headline">
          {answer.word}
        </h2>
        <div class="c-answer-card__meanings">
          {#each answer.meanings as m}
            <div class="c-answer-def-item">
              <span class="c-answer-def-num">{m.index}.</span>
              <span class="c-answer-def-text">{m.meaning}</span>
              {#if m.example}
                <div class="c-answer-def-example">
                  "{m.example}" {#if m.author}<span class="c-answer-def-author">— {m.author}</span>{/if}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>

    {:else if answer.type === "crypto"}
      <div class="c-answer-card__crypto">
        <div class="c-answer-card__sub-title">
          {answer.name}
        </div>
        <h2 class="c-answer-card__headline">
          {answer.formattedPrice}
        </h2>
        {#if answer.change24h !== null}
          <p
            class="c-answer-card__single-rate"
            class:c-answer-crypto--up={answer.change24h >= 0}
            class:c-answer-crypto--down={answer.change24h < 0}
          >
            {answer.change24h >= 0 ? "+" : ""}{answer.change24h}% (Son 24 saat)
          </p>
        {/if}
      </div>

    {:else}
      <div class="c-answer-card__generic">
        <div class="c-answer-card__sub-title">{answer.title || "Hızlı Yanıt"}</div>
        <div class="c-answer-card__headline">{answer.content}</div>
      </div>
    {/if}
  </div>
{/if}
