<script>
    import Modal from "../ui/Modal.svelte";

    let { controller, onClose } = $props();

    const modalOptions = {
        title: "Yorumum neden yayınlanmadı?",
    };
</script>

<Modal options={modalOptions} {controller} {onClose}>
    <p class="u-mb-sm">
        Sistem, yazdığınız yorumu otomatik güvenlik ve spam filtresine takıldığı
        için yayınlayamadı.
    </p>

    <p class="u-mb-sm">
        Yorum alanının temiz ve güvenilir kalması adına harici bağlantı içeren,
        aşırı harf tekrarı barındıran veya reklam şüphesi taşıyan iletiler
        otomatik olarak durdurulur.
    </p>

    <p class="u-mb-md">
        Yorumunuzu sadeleştirip bağlantıları kaldırarak tekrar göndermeyi
        deneyebilirsiniz.
    </p>

    <p class="color-muted text-sm">
        Filtrenin hatalı çalıştığını düşünüyorsanız <a href="/iletisim"
            >iletişime geçebilirsiniz</a
        >.
    </p>

    {#snippet footer()}
        <button class="btn btn--primary" onclick={() => controller.close()}>
            Anladım
        </button>
    {/snippet}
</Modal>
