<script>
    import { timelineState } from "@/stores/timeline.svelte.js";
    import { setCurrentCity } from "@/stores/city.svelte.js";
    import DailyHeader from "@/components/features/timeline/DailyHeader.svelte";
    import CalendarSelector from "@/components/features/timeline/CalendarSelector.svelte";
    import TimelineView from "@/components/features/timeline/TimelineView.svelte";
    import { CITY_MAP } from "@/utils/turkish.js";
    import Seo from "@/components/ui/Seo.svelte";
    import { onMount } from "svelte";

    let { data } = $props();

    let citySlug = $derived(data?.citySlug || "istanbul");
    let cityName = $derived(CITY_MAP[citySlug] || citySlug);

    onMount(() => {
        if (citySlug) {
            setCurrentCity(citySlug);
        }
        timelineState.init();
    });

    let pageTitle = $derived(`${cityName} KYK Yemek Menüsü & Yemekhane Bilgileri | Kepçe`);
    let pageDescription = $derived(
        `${cityName} KYK yurtlarında bugün çıkan sabah kahvaltısı ve akşam yemeği menüsü, resmi yemek saatleri ve beslenme yardımı limitleri. Reklamsız, güncel yemek listeleri.`
    );
    let canonicalUrl = $derived(`https://kepce.org/${citySlug}`);
    let ogImage = $derived(`https://kepce.org/api/v1/public/og/city/${citySlug}`);

    let menuSchema = $derived.by(() => {
        const menus = timelineState.menusState || [];
        const baseGraphs = [
            {
                "@type": "WebSite",
                "@id": "https://kepce.org/#website",
                url: "https://kepce.org/",
                name: "Kepçe",
                description: "Bugün KYK'da Ne Yemek Var? Günlük KYK Yurt Menüleri",
                inLanguage: "tr-TR",
            },
            {
                "@type": "Organization",
                "@id": "https://kepce.org/#organization",
                name: "Kepçe",
                url: "https://kepce.org/",
                logo: "https://kepce.org/icon-512.png",
            },
            {
                "@type": "FAQPage",
                "@id": `https://kepce.org/${citySlug}#faq`,
                mainEntity: [
                    {
                        "@type": "Question",
                        name: `${cityName} KYK yurtlarında yemek saatleri kaçtır?`,
                        acceptedAnswer: {
                            "@type": "Answer",
                            text: `${cityName} KYK yurtlarında hafta içi kahvaltı 06:30 - 12:00, akşam yemeği 16:00 - 22:30 saatleri arasındadır. Hafta sonu kahvaltı 12:30'a kadar devam eder.`
                        }
                    },
                    {
                        "@type": "Question",
                        name: `${cityName} KYK beslenme yardımı ne kadar?`,
                        acceptedAnswer: {
                            "@type": "Answer",
                            text: `2025-2026 dönemi için günlük toplam 150 TL (Kahvaltı 45 TL, Akşam Yemeği 105 TL) beslenme yardımı tanımlanmaktadır.`
                        }
                    }
                ]
            }
        ];

        if (menus && menus.length > 0) {
            const sections = menus.map((m) => {
                const dishes = m.items || m.dishes || [];
                const dishNames = dishes.map((d) => (typeof d === "string" ? d : d.name)).filter(Boolean);
                const mealTitle = m.meal_type === "breakfast" ? "Sabah Kahvaltısı" : "Akşam Yemeği";
                return {
                    "@type": "MenuSection",
                    name: mealTitle,
                    hasMenuItem: dishNames.map((name) => ({
                        "@type": "MenuItem",
                        name: name,
                    })),
                };
            });

            baseGraphs.push({
                "@type": "Menu",
                "@id": `https://kepce.org/${citySlug}#menu`,
                name: `${cityName} KYK Günlük Yemek Menüsü`,
                inLanguage: "tr-TR",
                hasMenuSection: sections,
            });
        }

        return {
            "@context": "https://schema.org",
            "@graph": baseGraphs,
        };
    });
</script>

<Seo
    title={pageTitle}
    description={pageDescription}
    image={ogImage}
    canonical={canonicalUrl}
    schema={menuSchema}
/>

<h1 class="sr-only">
    {cityName} KYK Yurt Yemek Menüsü - Günlük Yemek Listesi
</h1>

<DailyHeader />
<CalendarSelector />
<TimelineView />

<!-- Programmatic Evergreen City Guide (Thin Content Protection) -->
<section class="disclaimer-card" style="margin-top: var(--space-2xl); margin-bottom: var(--space-2xl);" aria-label="{cityName} KYK Yemekhane Bilgileri">
    <div style="display: flex; flex-direction: column; gap: var(--space-sm);">
        <h2 style="font-size: var(--text-base); font-weight: var(--font-weight-bold); color: var(--color-text); margin: 0;">
            {cityName} KYK Yemekhane ve Beslenme Rehberi
        </h2>
        <p style="font-size: var(--text-xs); color: var(--color-text-secondary); line-height: 1.6; margin: 0;">
            {cityName} genelindeki tüm Gençlik ve Spor Bakanlığı (GSB) KYK yurtlarında yemekhane hizmeti günlük beslenme yardımı kotasıyla sunulmaktadır. Hafta içi sabah kahvaltısı <strong>06:30 – 12:00</strong>, akşam yemeği ise <strong>16:00 – 22:30</strong> saatleri arasındadır. Hafta sonu kahvaltı servisi <strong>12:30</strong>'a kadar devam eder.
        </p>
        <p style="font-size: var(--text-xs); color: var(--color-text-secondary); line-height: 1.6; margin: 0;">
            Günlük tanımlanan <strong>150 TL</strong> beslenme yardımını (Sabah 45 TL, Akşam 105 TL) ister standart fiks tabldot menüde 0 TL farkla kullanabilir, isterseniz <a href="/kyk-beslenme-yardimi" class="text-link" style="color: var(--color-accent-primary); font-weight: var(--font-weight-medium);">KYK Fiş Hesaplayıcı</a> ile alakart büfeden kendi tercihinize göre menü oluşturabilirsiniz. Detaylı çalışma düzeni için <a href="/kyk-yemek-saatleri" class="text-link" style="color: var(--color-accent-primary); font-weight: var(--font-weight-medium);">KYK Yemek Saatleri</a> sayfamızı inceleyebilirsiniz.
        </p>
    </div>
</section>
