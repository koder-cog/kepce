<script>
  import { onMount } from "svelte";
  import { api } from "@/api/index.js";
  import Loader from "@/components/ui/Loader.svelte";
  import EmptyState from "@/components/ui/EmptyState.svelte";
  import Pagination from "@/components/ui/Pagination.svelte";
  import { icon } from "@/components/ui/icons.js";
  import { showToast } from "@/components/ui/toast.js";
  import { createModal } from "@/components/features/modal.js";

  let activeTab = $state("browser"); // 'browser' | 'sql'

  // ── Tablo Gezgini Durumları ──────────────────────────────
  let tables = $state([]);
  let isTablesLoading = $state(true);
  let tableSearch = $state("");
  let selectedTable = $state(null);

  let tableData = $state(null);
  let isDataLoading = $state(false);
  let currentPage = $state(1);
  let limit = $state(25);

  let filteredTables = $derived.by(() => {
    if (!tableSearch.trim()) return tables;
    const q = tableSearch.trim().toLowerCase();
    return tables.filter((t) => t.name.toLowerCase().includes(q));
  });

  async function loadTables() {
    isTablesLoading = true;
    try {
      tables = await api.getDatabaseTables();
      if (tables.length > 0 && !selectedTable) {
        selectTable(tables[0].name);
      }
    } catch (err) {
      showToast(err.message || "Tablo listesi yüklenemedi.", "error");
    } finally {
      isTablesLoading = false;
    }
  }

  async function selectTable(tableName, page = 1) {
    selectedTable = tableName;
    currentPage = page;
    isDataLoading = true;
    try {
      tableData = await api.getTableData(tableName, { page, limit });
    } catch (err) {
      showToast(err.message || "Tablo verileri yüklenemedi.", "error");
    } finally {
      isDataLoading = false;
    }
  }

  function handlePageChange(newPage) {
    if (selectedTable) {
      selectTable(selectedTable, newPage);
    }
  }

  function openEditCellModal(row, column) {
    if (!tableData || tableData.primary_keys.length === 0) {
      showToast("Tabloda birincil anahtar bulunmadığı için doğrudan düzenleme yapılamaz.", "warning");
      return;
    }

    const pkCol = tableData.primary_keys[0];
    const pkVal = row[pkCol];
    const currentVal = row[column.name];

    let editVal = currentVal !== null && currentVal !== undefined ? (typeof currentVal === 'object' ? JSON.stringify(currentVal) : String(currentVal)) : "";

    createModal({
      title: `${tableData.table_name} Satır Düzenleme`,
      iconHtml: icon("edit", 20),
      contentHtml: `
        <div class="u-mb-md">
          <p class="u-text-sm u-color-muted u-mb-xs">Birincil Anahtar (${pkCol}): <strong>${pkVal}</strong></p>
          <p class="u-text-sm u-color-muted u-mb-md">Sütun: <strong>${column.name}</strong> (${column.data_type})</p>
          <label class="label u-text-xs">Yeni Değer</label>
          <textarea id="modal-edit-val" class="input u-width-full" rows="4" style="font-family: var(--font-mono); font-size: var(--text-xs);">${editVal}</textarea>
        </div>
      `,
      buttons: [
        { label: "İptal", variant: "secondary" },
        {
          label: "Kaydet",
          variant: "primary",
          onClick: async (close) => {
            const inputEl = document.getElementById("modal-edit-val");
            const rawVal = inputEl ? inputEl.value : "";
            try {
              let parsedVal = rawVal;
              if (column.data_type.includes("json")) {
                try { parsedVal = JSON.parse(rawVal); } catch (_) { parsedVal = rawVal; }
              } else if (column.data_type.includes("bool")) {
                parsedVal = rawVal.toLowerCase() === "true";
              } else if (column.data_type.includes("int") || column.data_type.includes("numeric")) {
                const num = Number(rawVal);
                if (!isNaN(num)) parsedVal = num;
              }

              await api.updateTableRow(tableData.table_name, pkCol, pkVal, column.name, parsedVal);
              showToast("Kayıt güncellendi.", "success");
              close();
              selectTable(tableData.table_name, currentPage);
            } catch (err) {
              showToast(err.message || "Güncelleme hatası.", "error");
            }
          },
        },
      ],
    });
  }

  function promptDeleteRow(row) {
    if (!tableData || tableData.primary_keys.length === 0) {
      showToast("Tabloda birincil anahtar bulunmadığı için doğrudan satır silinemez.", "warning");
      return;
    }

    const pkCol = tableData.primary_keys[0];
    const pkVal = row[pkCol];

    createModal({
      title: "Satır Silinsin mi?",
      iconHtml: icon("trash", 20),
      iconColor: "danger",
      contentHtml: `
        <p><strong>${tableData.table_name}</strong> tablosundaki <code>${pkCol} = ${pkVal}</code> satırı kalıcı olarak silinecektir. Bu işlem geri alınamaz.</p>
      `,
      buttons: [
        { label: "Vazgeç", variant: "secondary" },
        {
          label: "Evet, Sil",
          variant: "danger",
          onClick: async (close) => {
            try {
              await api.deleteTableRow(tableData.table_name, pkCol, pkVal);
              showToast("Satır silindi.", "danger");
              close();
              selectTable(tableData.table_name, currentPage);
            } catch (err) {
              showToast(err.message || "Silme hatası.", "error");
            }
          },
        },
      ],
    });
  }

  // ── SQL Konsolu Durumları ───────────────────────────────
  let sqlQuery = $state("SELECT * FROM menus ORDER BY id DESC LIMIT 15;");
  let writeMode = $state(false);
  let isExecuting = $state(false);
  let queryResult = $state(null);
  let queryError = $state(null);

  const QUICK_QUERIES = [
    { label: "Son Menüler", query: "SELECT id, city_id, date, meal_type, is_approved, created_at FROM menus ORDER BY id DESC LIMIT 20;" },
    { label: "Son Kullanıcılar", query: "SELECT id, username, email, role, is_banned, created_at FROM users ORDER BY created_at DESC LIMIT 20;" },
    { label: "İletişim Mesajları", query: "SELECT id, email, category, subject, source, status, created_at FROM contact_messages ORDER BY id DESC LIMIT 20;" },
    { label: "Tablo Canlı Satırları", query: "SELECT relname AS tablo, n_live_tup AS canli_satir FROM pg_stat_user_tables ORDER BY n_live_tup DESC;" },
  ];

  async function runQuery() {
    if (!sqlQuery.trim()) return;
    isExecuting = true;
    queryError = null;
    queryResult = null;

    try {
      const res = await api.executeDatabaseQuery(sqlQuery, writeMode);
      queryResult = res;
    } catch (err) {
      queryError = err.message || "Sorgu yürütülemedi.";
    } finally {
      isExecuting = false;
    }
  }

  function handleKeydown(e) {
    if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
      e.preventDefault();
      runQuery();
    }
  }

  onMount(() => {
    loadTables();
  });
</script>

<svelte:head>
  <title>Veritabanı Konsolu - Moderasyon - Kepçe</title>
</svelte:head>

<div class="admin-header u-mb-lg">
  <div>
    <h2 class="admin-title">Veritabanı Konsolu</h2>
    <p class="admin-subtitle">
      PostgreSQL tablo gezgini ve kontrollü SQL yürütücü
    </p>
  </div>

  <div class="coverage-filters">
    <button
      class="btn btn--squish"
      class:btn--primary={activeTab === "browser"}
      class:btn--secondary={activeTab !== "browser"}
      onclick={() => (activeTab = "browser")}
    >
      {@html icon("grid", 16)}
      Tablo Gezgini
    </button>
    <button
      class="btn btn--squish"
      class:btn--primary={activeTab === "sql"}
      class:btn--secondary={activeTab !== "sql"}
      onclick={() => (activeTab = "sql")}
    >
      {@html icon("terminal", 16)}
      SQL Konsolu
    </button>
  </div>
</div>

{#if activeTab === "browser"}
  <div class="db-console-layout">
    <!-- SOL TABLO LİSTESİ -->
    <aside class="db-table-list">
      <div class="u-mb-sm">
        <input
          type="text"
          class="input input--sm u-width-full"
          placeholder="Tablo ara..."
          bind:value={tableSearch}
        />
      </div>

      {#if isTablesLoading}
        <div class="u-p-md u-flex u-flex-center">
          <Loader size={24} />
        </div>
      {:else}
        {#each filteredTables as table (table.name)}
          <button
            type="button"
            class="db-table-item"
            class:db-table-item--active={selectedTable === table.name}
            onclick={() => selectTable(table.name)}
          >
            <span>{table.name}</span>
            <span class="u-color-muted u-text-xs">
              {table.estimated_rows.toLocaleString("tr-TR")}
            </span>
          </button>
        {/each}
      {/if}
    </aside>

    <!-- SAĞ VERİ ALANI -->
    <main class="db-main-content">
      {#if isDataLoading}
        <div class="stats-placeholder">
          <Loader size={40} />
        </div>
      {:else if tableData}
        <div class="u-flex u-flex-justify-between u-flex-align-center u-mb-md">
          <div class="u-flex u-flex-align-center u-gap-sm">
            <h3 class="u-text-base u-color-text u-font-mono">
              {tableData.table_name}
            </h3>
            <span class="badge badge--neutral">
              Toplam: {tableData.total_rows.toLocaleString("tr-TR")} satır
            </span>
          </div>

          <button
            class="btn btn--secondary btn--sm btn--squish"
            onclick={() => selectTable(tableData.table_name, currentPage)}
            title="Yenile"
          >
            {@html icon("refresh", 14)}
            Yenile
          </button>
        </div>

        <div class="admin-table-wrapper" role="region" aria-label="{tableData.table_name} Veri Tablosu">
          <table class="admin-table">
            <thead>
              <tr>
                <th class="col-actions">İşlem</th>
                {#each tableData.columns as col}
                  <th>
                    <span class:u-color-primary={col.is_primary_key}>
                      {col.name}
                    </span>
                    {#if col.is_primary_key}
                      <span class="badge badge--primary u-ml-xs">PK</span>
                    {/if}
                  </th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#if tableData.rows.length === 0}
                <tr>
                  <td colspan={tableData.columns.length + 1} class="u-text-center u-p-lg u-color-muted">
                    Bu tabloda henüz kayıt bulunmuyor.
                  </td>
                </tr>
              {:else}
                {#each tableData.rows as row, idx (idx)}
                  <tr>
                    <td class="col-actions">
                      <button
                        class="btn btn--secondary btn--sm btn--squish u-mr-xs"
                        onclick={() => openEditCellModal(row, tableData.columns[0])}
                        title="Düzenle"
                      >
                        {@html icon("edit", 12)}
                      </button>
                      <button
                        class="btn btn--danger btn--sm btn--squish"
                        onclick={() => promptDeleteRow(row)}
                        title="Sil"
                      >
                        {@html icon("trash", 12)}
                      </button>
                    </td>
                    {#each tableData.columns as col}
                      {@const val = row[col.name]}
                      <td
                        class="db-cell-mono"
                        title={val !== null && val !== undefined ? String(val) : "NULL"}
                        ondblclick={() => openEditCellModal(row, col)}
                      >
                        {#if val === null || val === undefined}
                          <span class="u-color-muted">NULL</span>
                        {:else if typeof val === "boolean"}
                          <span class:u-color-primary={val}>{val ? "true" : "false"}</span>
                        {:else if typeof val === "object"}
                          <span>{JSON.stringify(val)}</span>
                        {:else}
                          <span>{String(val)}</span>
                        {/if}
                      </td>
                    {/each}
                  </tr>
                {/each}
              {/if}
            </tbody>
          </table>
        </div>

        {#if Math.ceil(tableData.total_rows / limit) > 1}
          <Pagination
            page={currentPage}
            totalPages={Math.ceil(tableData.total_rows / limit)}
            totalItems={tableData.total_rows}
            onPageChange={handlePageChange}
          />
        {/if}
      {:else}
        <EmptyState
          iconName="grid"
          title="Tablo Seçilmedi"
          desc="İncelemek istediğiniz bir tabloyu sol menüden seçiniz."
        />
      {/if}
    </main>
  </div>
{:else}
  <!-- SQL KONSOLU -->
  <div class="db-sql-section">
    <div class="u-mb-sm u-flex u-flex-justify-between u-flex-align-center">
      <label for="db-sql-input" class="label u-text-xs u-color-muted">
        SQL Sorgusu (Çalıştırmak için <code>Ctrl+Enter</code>)
      </label>

      <div class="coverage-filters">
        {#each QUICK_QUERIES as q}
          <button
            class="btn btn--secondary btn--sm btn--squish"
            onclick={() => (sqlQuery = q.query)}
          >
            {q.label}
          </button>
        {/each}
      </div>
    </div>

    <textarea
      id="db-sql-input"
      class="db-sql-editor"
      bind:value={sqlQuery}
      onkeydown={handleKeydown}
      spellcheck="false"
    ></textarea>

    <div class="db-sql-toolbar">
      <div class="u-flex u-flex-align-center u-gap-md">
        <label class="u-flex u-flex-align-center u-gap-xs u-text-sm u-color-text">
          <input type="checkbox" bind:checked={writeMode} />
          <strong>Yazma İzni (Write Mode)</strong>
        </label>
        <span class="u-text-xs u-color-muted">
          {writeMode ? "⚠️ Veri değiştiren sorgular çalıştırılabilir." : "🔒 Salt okunur mod devrede (SELECT)"}
        </span>
      </div>

      <div class="u-flex u-flex-align-center u-gap-sm">
        <button
          class="btn btn--secondary btn--squish"
          onclick={() => { sqlQuery = ""; queryResult = null; queryError = null; }}
        >
          Temizle
        </button>
        <button
          class="btn btn--primary btn--squish"
          disabled={isExecuting}
          onclick={runQuery}
        >
          {#if isExecuting}
            <Loader size={16} />
            Çalıştırılıyor...
          {:else}
            {@html icon("play", 16)}
            Çalıştır
          {/if}
        </button>
      </div>
    </div>

    {#if queryError}
      <div class="u-mt-lg alert alert--danger">
        <strong>Sorgu Başarısız:</strong>
        <p class="u-mt-xs u-font-mono u-text-xs">{queryError}</p>
      </div>
    {/if}

    {#if queryResult}
      <div class="u-mt-lg">
        <div class="u-flex u-flex-justify-between u-flex-align-center u-mb-sm">
          <span class="badge badge--neutral">
            Süre: {queryResult.duration_ms} ms &middot; Etkilenen / Dönen: {queryResult.affected_rows} satır
          </span>
        </div>

        <div class="admin-table-wrapper" role="region" aria-label="Sorgu Sonuçları">
          <table class="admin-table">
            <thead>
              <tr>
                {#each queryResult.columns as col}
                  <th class="u-font-mono">{col}</th>
                {/each}
              </tr>
            </thead>
            <tbody>
              {#if queryResult.rows.length === 0}
                <tr>
                  <td colspan={queryResult.columns.length || 1} class="u-text-center u-p-lg u-color-muted">
                    Sorgu başarıyla çalıştı, döndürülen satır yok.
                  </td>
                </tr>
              {:else}
                {#each queryResult.rows as row, idx (idx)}
                  <tr>
                    {#each queryResult.columns as col}
                      {@const val = row[col]}
                      <td class="db-cell-mono">
                        {#if val === null || val === undefined}
                          <span class="u-color-muted">NULL</span>
                        {:else if typeof val === "boolean"}
                          <span class:u-color-primary={val}>{val ? "true" : "false"}</span>
                        {:else if typeof val === "object"}
                          <span>{JSON.stringify(val)}</span>
                        {:else}
                          <span>{String(val)}</span>
                        {/if}
                      </td>
                    {/each}
                  </tr>
                {/each}
              {/if}
            </tbody>
          </table>
        </div>
      </div>
    {/if}
  </div>
{/if}
