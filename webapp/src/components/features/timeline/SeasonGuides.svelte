<script>
    import { icon } from "@/components/ui/icons.js";
</script>

<div class="season-guides">
    <a
        href="/kyk-beslenme-yardimi"
        class="card season-guides__card"
        data-link
    >
        <div class="season-guides__content">
            <span class="season-guides__title">Beslenme Yardımı</span>
            <span class="season-guides__desc">
                KYK yurtlarında beslenme yardımı, tabldot ve alakart farkı, kota ve limit aşımı kuralları.
            </span>
        </div>
        <span class="season-guides__arrow" aria-hidden="true">
            {@html icon("chevronRight", 20)}
        </span>
    </a>

    <a
        href="/kyk-yemek-saatleri"
        class="card season-guides__card"
        data-link
    >
        <div class="season-guides__content">
            <span class="season-guides__title">Yemekhane Saatleri</span>
            <span class="season-guides__desc">
                KYK yurtlarında kahvaltı ve akşam yemeği saatleri kaçta başlıyor, kaçta bitiyor?
            </span>
        </div>
        <span class="season-guides__arrow" aria-hidden="true">
            {@html icon("chevronRight", 20)}
        </span>
    </a>
</div>
