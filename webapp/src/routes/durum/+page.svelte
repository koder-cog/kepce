<script>
  import "@/styles/pages/_status.css";
  import { api } from "@/api/index.js";
  import Loader from "@/components/ui/Loader.svelte";
  import { afterNavigate } from "$app/navigation";
  import { icon } from "@/components/ui/icons.js";
  import { onDestroy, tick } from "svelte";
  import Seo from "@/components/ui/Seo.svelte";

  let current = $state(null);
  let history = $state(null);
  let errorMessage = $state(null);
  let historyDays = $state(30);

  function updateHistoryDays() {
    if (window.innerWidth <= 600) historyDays = 30;
    else if (window.innerWidth <= 900) historyDays = 60;
    else historyDays = 90;
  }

  let resizeCleanup = null;

  afterNavigate(() => {
    updateHistoryDays();

    // Resize listener'ı yalnızca ilk seferinde bağla
    if (!resizeCleanup) {
      window.addEventListener("resize", updateHistoryDays);
      resizeCleanup = () =>
        window.removeEventListener("resize", updateHistoryDays);
    }

    const loadData = async () => {
      try {
        const [c, h] = await Promise.all([
          api.getStatus(),
          api.getStatusHistory(90),
        ]);
        current = c;
        history = h;
      } catch (err) {
        errorMessage = err.message;
      }
    };

    loadData();
  });

  onDestroy(() => {
    resizeCleanup?.();
  });

  function getOverallStatusLabel(status) {
    return (
      {
        aktif: "Tüm Sistemler Çalışıyor",
        yavas: "Kısmi Yavaşlama Mevcut",
        kesinti: "Sistemde Kesinti Var",
      }[status] || "Durum Belirlenemiyor"
    );
  }

  function formatDuration(start, end) {
    const diffMins = Math.round((new Date(end) - new Date(start)) / 60000);
    if (diffMins < 60) return `${diffMins} dk.`;
    const hrs = Math.floor(diffMins / 60);
    const mins = diffMins % 60;
    if (mins === 0) return `${hrs} sa.`;
    return `${hrs} sa. ${mins} dk.`;
  }

  let groupedIncidents = $derived.by(() => {
    if (!current?.incidents) return [];
    const acc = {};
    current.incidents.forEach((incident) => {
      const dateStr = new Date(incident.started_at).toLocaleDateString(
        "tr-TR",
        {
          month: "long",
          day: "numeric",
          year: "numeric",
        },
      );
      if (!acc[dateStr]) acc[dateStr] = [];
      acc[dateStr].push(incident);
    });
    return Object.entries(acc);
  });
</script>

<div class="content-page">
  <header class="content-page__header status-page-header">
    <h1 class="content-page__title">Kepçe Sistem Durumu</h1>
  </header>

  <div id="status-content">
    {#if errorMessage}
      <div class="status-summary status-summary--kesinti">
        <div class="pulse-dot"></div>
        <div class="status-summary__text">Bağlantı Hatası</div>
      </div>
      <p class="status-error-msg">{errorMessage}</p>
    {:else if current && history}
      <div class="status-banner-container">
        <div class="status-card--main status-card--{current.status}">
          {#if current.status === "aktif"}
            <svg
              width="48"
              height="48"
              viewBox="0 0 24 24"
              fill="none"
              stroke="var(--status-color, var(--color-success))"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><polyline points="20 6 9 17 4 12"></polyline></svg
            >
          {:else if current.status === "yavas"}
            <svg
              width="48"
              height="48"
              viewBox="0 0 24 24"
              fill="none"
              stroke="var(--status-color, var(--color-warning))"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><circle cx="12" cy="12" r="10"></circle><line
                x1="12"
                y1="8"
                x2="12"
                y2="12"
              ></line><line x1="12" y1="16" x2="12.01" y2="16"></line></svg
            >
          {:else}
            <svg
              width="48"
              height="48"
              viewBox="0 0 24 24"
              fill="none"
              stroke="var(--status-color, var(--color-error))"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><circle cx="12" cy="12" r="10"></circle><line
                x1="15"
                y1="9"
                x2="9"
                y2="15"
              ></line><line x1="9" y1="9" x2="15" y2="15"></line></svg
            >
          {/if}
          <div class="status-banner__text">
            {getOverallStatusLabel(current.status)}
          </div>
        </div>
      </div>

      <div class="status-list">
        {#each history as component}
          <div class="component-card">
            <div class="meal-card__title">{component.name}</div>
            <div class="status-timeline">
              <div class="timeline-pills">
                {#each (component.days || []).slice(-historyDays) as day}
                  <div
                    class="pill pill--{day.status}"
                    role="button"
                    tabindex="0"
                    data-tooltip-trigger="click"
                  >
                    <div class="tooltip-template u-hidden">
                      <div class="rich-status-tooltip">
                        <div class="rich-status-tooltip__header">
                          {new Date(day.date).toLocaleDateString("tr-TR", {
                            month: "short",
                            day: "numeric",
                            year: "numeric",
                          })}
                        </div>
                        {#if day.status === "aktif"}
                          <div class="rich-status-tooltip__body">
                            Bu gün için kayıtlı sorun yok
                          </div>
                        {:else}
                          {@const dayStr = day.date.split("T")[0]}
                          {@const dayIncident = current?.incidents?.find(
                            (i) =>
                              i.component === component.name &&
                              i.started_at.startsWith(dayStr),
                          )}
                          <div
                            class="rich-status-tooltip__badge rich-status-tooltip__badge--{day.status}"
                          >
                            {@html day.status === "kesinti"
                              ? icon("error")
                              : icon("warning")}
                            <span
                              >{day.status === "kesinti"
                                ? "Tam Kesinti"
                                : "Kısmi Yavaşlama Mevcut"}</span
                            >
                          </div>
                          <div class="rich-status-tooltip__related">
                            <div class="rich-status-tooltip__related-label">
                              İlgili
                            </div>
                            <div class="rich-status-tooltip__related-text">
                              {day.incident_title ||
                                day.title ||
                                dayIncident?.title ||
                                `${component.name} sorunları`}
                            </div>
                            {#if dayIncident && dayIncident.resolved_at}
                              <div class="rich-status-tooltip__duration">
                                {formatDuration(
                                  dayIncident.started_at,
                                  dayIncident.resolved_at,
                                )} sürdü
                              </div>
                            {/if}
                          </div>
                        {/if}
                      </div>
                    </div>
                  </div>
                {/each}
              </div>
              <div class="timeline-footer">
                <span>{historyDays} gün önce</span>
                <span>Bugün</span>
              </div>
            </div>
          </div>
        {/each}
      </div>

      <section class="incidents-section">
        <h2 class="incidents-title">Geçmiş Olaylar</h2>
        {#if current.incidents && current.incidents.length > 0}
          <div class="status-list">
            {#each groupedIncidents as [dateStr, incidents]}
              <div class="incident-day">
                <div class="incident-date">{dateStr}</div>
                {#each incidents as incident}
                  {@const resolvedDate = incident.resolved_at || incident.ended_at}
                  <div class="incident-card incident-card--{incident.status}">
                    <div class="incident-title">{incident.title}</div>

                    {#if resolvedDate}
                      <div class="incident-event">Düzeltildi</div>
                      <div class="incident-update-time">
                        {new Date(resolvedDate).toLocaleTimeString(
                          "tr-TR",
                          { hour: "2-digit", minute: "2-digit" },
                        )}
                        ({formatDuration(
                          incident.started_at,
                          resolvedDate,
                        )} sürdü)
                      </div>
                    {/if}

                    <div class="incident-event">
                      Fark edildi: {incident.message}
                    </div>
                    <div class="incident-update-time">
                      {new Date(incident.started_at).toLocaleTimeString(
                        "tr-TR",
                        { hour: "2-digit", minute: "2-digit" },
                      )}
                    </div>
                  </div>
                {/each}
              </div>
            {/each}
          </div>
        {:else}
          <p class="incidents-empty">
            Sistemimizde kayıtlı herhangi bir geçmiş olay bulunmamaktadır.
          </p>
        {/if}
      </section>
    {:else}
      <div class="status-loader-container">
        <Loader size={64} />
      </div>
    {/if}
  </div>
</div>

<Seo
  title="Sistem Durumu ve Çalışma Süresi - Kepçe"
  description="Kepçe API, web uygulaması, veri tabanı ve arka plan servislerinin anlık çalışma durumu ve geçmiş kesinti raporları."
  image="https://kepce.org/api/v1/public/og/page/durum"
/>
