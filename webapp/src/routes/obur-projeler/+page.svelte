<script>
  import { icon } from "@/components/ui/icons.js";
  import ContentPage from "@/components/layout/ContentPage.svelte";
  import Seo from "@/components/ui/Seo.svelte";

  const oburProjelerSchema = {
    "@context": "https://schema.org",
    "@graph": [
      {
        "@type": "BreadcrumbList",
        itemListElement: [
          {
            "@type": "ListItem",
            position: 1,
            name: "Ana Sayfa",
            item: "https://kepce.org/",
          },
          {
            "@type": "ListItem",
            position: 2,
            name: "Öbür Projeler",
            item: "https://kepce.org/obur-projeler",
          },
        ],
      },
      {
        "@type": "WebPage",
        "@id": "https://kepce.org/obur-projeler#webpage",
        url: "https://kepce.org/obur-projeler",
        name: "Öbür Projeler | Kepçe",
        description:
          "Kepçe ekosistemi ve geliştiricilerinin yürüttüğü bağımsız açık kaynak projeler.",
        inLanguage: "tr-TR",
      },
      {
        "@type": "Organization",
        "@id": "https://kepce.org/#organization",
        name: "Kepçe",
        url: "https://kepce.org/",
        logo: "https://kepce.org/icon-512.png",
      },
    ],
  };
</script>

<Seo
  title="Öbür Projeler | Kepçe"
  description="Kepçe ekosistemi ve geliştiricilerinin yürüttüğü bağımsız açık kaynak projeler."
  image="https://kepce.org/api/v1/public/og/page/obur-projeler"
  schema={oburProjelerSchema}
/>

<ContentPage title="Öbür Projeler">
  <p>
    Bunlar Kepçe ekibinin veya diğer geliştiricilerinin internetin başka köşelerinde yürüttüğü bağımsız işlerdir. Kepçe'nin tabldot menüsüyle doğrudan bir bağı bulunmaz. Bu sitelerin ve servislerin yol açacağı her türlü sıkıntıyla ilgili kendi adreslerine başvurunuz.
  </p>

  <div class="project-cards">
    <article class="project-card">
      <div class="project-card__header">
        <h2 class="project-card__title">Kepçe Ara</h2>
      </div>

      <p class="project-card__desc">
        Gizlilik odaklı, kullanıcıyı takip etmeyen ve reklam barındırmayan bağımsız açık kaynak arama motoru.
      </p>

      <div class="project-card__meta">
        <span class="project-card__author">koder-cog</span>
      </div>

      <div class="project-card__footer">
        <a href="/ara" class="btn btn--secondary btn--squish" data-link>
          <span>Siteyi Aç</span>
          {@html icon("externalLink", 14)}
        </a>
      </div>
    </article>
  </div>
</ContentPage>
