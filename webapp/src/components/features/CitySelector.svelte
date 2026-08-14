<script>
    import { globalState, authActions } from "../../state.svelte.js";

    import {
        detectCitySilent,
        detectCityPrecise,
        detectCityIP,
    } from "../../utils/geo.js";
    import { showToast } from "../../components/ui/toast.js";
    import Dropdown from "./Dropdown.svelte";
    import { createModal } from "./modal.js";
    import { setCurrentCity } from "../../stores/city.svelte.js";
    import { onMount } from "svelte";

    let {
        cities = [],
        value = $bindable(),
        variant = "secondary",
        onChange = () => {},
        showSpecial = true,
        localOnly = false,
    } = $props();

    let options = $derived(
        [...cities]
            .sort((a, b) =>
                new Intl.Collator("tr-TR", { sensitivity: "base" }).compare(
                    a.name,
                    b.name,
                ),
            )
            .map((c) => ({ value: c.slug, label: c.name })),
    );

    let slugs = $derived(cities.map((c) => c.slug));

    // Otomatik tespit ve kullanıcı seçiminde üst bileşene haber ver.
    // `$bindable` + parent'ta tek yönlü `value={...}` kombinasyonunda,
    // içeride yapılan atamalar üst bileşene OTOMATİK YAYILMAZ (Svelte 5'te
    // store getter/setter'larına `bind:` güvenilir değildir). Bu yüzden
    // her değişiklikte açık `onChange` callback'i çağırıyoruz; böylece
    // timeline store gibi üst state'ler tetiklenip `loadMenus()` çalışıyor.
    //
    // ÖNEMLİ: `newCity === value` guard'ı eskiden Dropdown'un `bind:value`
    // ile değeri zaten güncellemesinden sonra çağrıldığı için her zaman eşit
    // düşüp `onChange` hiç tetiklenmiyordu; bu, üst bileşenlerdeki (timeline,
    // /arşiv) state güncellemesini kırıyordu. Guard kaldırıldı; aynı değer
    // tekrar seçildiğinde de parent'a haber veriliyor.
    function commit(newCity) {
        if (!newCity) return;
        value = newCity;
        onChange(newCity);
    }

    onMount(async () => {
        const profileCity = globalState?.user?.default_city_slug;

        if (!value && !profileCity) {
            const detected = await detectCitySilent();
            if (detected && slugs.includes(detected)) {
                commit(detected);
            }
        }

        const finalCity = value || profileCity || "istanbul";

        if (finalCity !== value) {
            value = finalCity;
            // `localOnly` modda (örn. /arşiv) global şehir state'ini
            // kirletmemek için sadece localStorage'a yazmıyoruz; ama
            // localStorage'a yazmak kullanıcının profil bilgisi olmadan
            // sonraki ziyaretlerinde hatırlatması için mantıklı. Bu
            // yüzden sadece `setCurrentCity`'yi atlayıp localStorage'a
            // yazmaya devam ediyoruz.
            if (!localOnly) setCurrentCity(value);
            onChange(value);
        }

        if (value && slugs.length > 0 && !slugs.includes(value)) {
            value = "istanbul";
            if (!localOnly) setCurrentCity(value);
            onChange(value);
        }
    });

    async function handleActionClick() {
        // Önce sessizce IP'den (Cloudflare headers) bulmayı dene
        const detectedIP = await detectCityIP();
        if (detectedIP && slugs.includes(detectedIP)) {
            commit(detectedIP);
            if (!localOnly) setCurrentCity(detectedIP);
            showToast("Konumunuz otomatik olarak bulundu.", { timeout: 3000 });
            return;
        }

        // Bulunamazsa direkt tarayıcı izni istemek yerine önce uyarı (toast) ile sor
        showToast(
            "Otomatik konum bulunamadı. Kesin tespit için tarayıcı izni gerekiyor.",
            {
                timeout: 8000,
                action: {
                    text: "İzin Ver",
                    callback: async () => {
                        const detected = await detectCityPrecise(slugs);
                        if (detected) {
                            commit(detected);
                            if (!localOnly) setCurrentCity(detected);
                            showToast("Konumunuz güncellendi.", {
                                timeout: 3000,
                            });
                        } else {
                            showToast(
                                "Konum izni verilmedi veya şehir bulunamadı.",
                                { type: "error" },
                            );
                        }
                    },
                },
            },
        );
    }

    function handleSpecialClick() {
        createModal({
            title: "Şehrinizi göremiyor musunuz?",
            contentHtml:
                '<p>"Bizim şehir niye yok" diye dertlenmeden önce o ayki menüyü <a href="/menu-gonder">şuradan</a> gönderiniz. Eğer menü yoksa bayinizden ısrarla isteyiniz.</p>',
            buttons: [{ label: "Anladım", variant: "primary" }],
        });
    }

    function handleChange(newCity) {
        if (!localOnly) setCurrentCity(newCity);
        commit(newCity);
    }
</script>

<Dropdown
    {options}
    bind:value
    {variant}
    placeholder="Şehir"
    onChange={handleChange}
    actionItem={{ label: "Konumumu bul" }}
    onActionClick={handleActionClick}
    specialItem={showSpecial ? { label: "Şehrinizi göremiyor musunuz?" } : null}
    onSpecialClick={showSpecial ? handleSpecialClick : null}
/>
