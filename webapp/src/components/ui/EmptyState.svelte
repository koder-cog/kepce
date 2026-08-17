<script>
    import { icon } from './icons.js';

    let {
        title,
        desc,
        iconName,
        statusCode,
        className = '',
        compact = false,
        headingLevel = 'h2',
        children
    } = $props();

    const ERROR_REGISTRY = {
        "4xx": [
            { code: 400, title: "Kötü istek", icon: "keyboard", desc: "Gönderilen istek sunucu tarafında bir kafa karışıklığına yol açtı. Parametreleri kontrol etmekte fayda var." },
            { code: 401, title: "Yetkisiz", icon: "login", desc: "Buradan geçmek için önce kim olduğunuzu göstermeniz gerekiyor. Giriş yapıp tekrar deneyin." },
            { code: 402, title: "Ödeme gerekli", icon: "creditCard", desc: "Ödeme gerektiren bir durum var gibi görünüyor ama biz de bir şey satmıyoruz." },
            { code: 403, title: "Yasak", icon: "login", desc: "Kim olduğunuzu biliyoruz ama bu içeriğe erişim izniniz bulunmuyor." },
            { code: 404, title: "Yok böyle bişii", icon: "ghost", desc: "Aradığınız sayfa veya veri bulunamadı. Belki hiç var olmadı, belki de biz bir yerlerde kaybettik." },
            { code: 405, title: "Yöntem yasak", icon: "keyboard", desc: "Bu işlem için kullandığınız yöntem burada geçersizdir." },
            { code: 406, title: "Kabul edilemez", icon: "keyboard", desc: "Sunucu, istediğiniz formatta bir yanıt üretemiyor; menüde bu seçenek yok." },
            { code: 408, title: "Zaman aşımı", icon: "timeout", desc: "Sunucu isteği beklerken zaman aşımına uğradı. Bağlantınızı kontrol edip tekrar deneyin." },
            { code: 409, title: "Çakışma", icon: "keyboard", desc: "Gönderdiğiniz veriler sunucudaki mevcut durumla bir çakışma yaşıyor." },
            { code: 410, title: "Artık yok", icon: "ghost", desc: "Bu içerik artık aramızda değil, sunucudan kalıcı olarak kaldırılmış." },
            { code: 411, title: "Uzunluk gerekli", icon: "keyboard", desc: "Gönderdiğiniz verinin boyutu belirtilmemiş, sunucu ne kadar bekleyeceğini bilemiyor." },
            { code: 412, title: "Ön koşul başarısız", icon: "login", desc: "İstekte belirttiğiniz şartlar sunucu tarafından karşılanamadı." },
            { code: 413, title: "İstek çok büyük", icon: "keyboard", desc: "Gönderdiğiniz veri sunucunun kapasitesini aşıyor. Biraz zayıflatıp tekrar deneyin." },
            { code: 414, title: "URL Çok Uzun", icon: "keyboard", desc: "İstek adresi işlenemeyecek kadar uzun, bir roman yazmış olabilirsiniz." },
            { code: 415, title: "Desteklenmeyen medya", icon: "keyboard", desc: "Gönderdiğiniz dosya türünü işleyemiyoruz." },
            { code: 422, title: "İşlenemeyen varlık", icon: "keyboard", desc: "İstek formatı doğru ama içeriği işlerken bir sorun çıktı." },
            { code: 429, title: "Çok fazla istek", icon: "timeout", desc: "Kısa sürede çok fazla istek gönderdiniz. Biraz soluklanıp tekrar deneyin." }
        ],
        "5xx": [
            { code: 500, title: "Sunucu içi hata", icon: "server", desc: "Sunucu tarafında beklenmedik bir aksilik çıktı. Arka planda bir şeyler ters gitmiş olmalı." },
            { code: 501, title: "Uygulanmadı", icon: "server", desc: "Sunucu bu isteği yerine getirecek yeteneğe henüz sahip değil." },
            { code: 502, title: "Kötü ağ geçidi", icon: "server", desc: "Sunucular arası iletişimde bir kopukluk yaşandı." },
            { code: 503, title: "Hizmet yok", icon: "server", desc: "Şu an servis veremiyoruz. Muhtemelen bakım yapıyoruz veya içerisi çok kalabalık." },
            { code: 504, title: "Ağ geçidi zaman aşımı", icon: "timeout", desc: "Üst sunucudan zamanında yanıt alınamadı, arka taraf biraz meşgul görünüyor." },
            { code: 505, title: "HTTP Sürümü Yok", icon: "server", desc: "Kullanılan HTTP sürümü sunucu tarafından desteklenmiyor." }
        ]
    };

    let finalTitle = $derived.by(() => {
        if (statusCode) {
            const group = statusCode >= 500 ? '5xx' : '4xx';
            const errorData = ERROR_REGISTRY[group]?.find(e => e.code === statusCode);
            if (errorData) return title || `${statusCode}: ${errorData.title}`;
        }
        return title || 'Sonuç bulunamadı';
    });

    let finalDesc = $derived.by(() => {
        if (statusCode) {
            const group = statusCode >= 500 ? '5xx' : '4xx';
            const errorData = ERROR_REGISTRY[group]?.find(e => e.code === statusCode);
            if (errorData) return desc || errorData.desc;
        }
        return desc || '';
    });

    let finalIcon = $derived.by(() => {
        if (statusCode) {
            const group = statusCode >= 500 ? '5xx' : '4xx';
            const errorData = ERROR_REGISTRY[group]?.find(e => e.code === statusCode);
            if (errorData) return iconName || errorData.icon;
        }
        return iconName || 'info';
    });
</script>

<div class="empty-state {compact ? 'empty-state--compact' : ''} {className}">
    <div class="empty-state__icon">
        {@html icon(finalIcon, compact ? 28 : 64)}
    </div>
    <svelte:element this={headingLevel} class="empty-state__title">{finalTitle}</svelte:element>
    <p class="empty-state__desc">{finalDesc}</p>
    {#if children}
        <div class="empty-state__actions">
            {@render children()}
        </div>
    {/if}
</div>
