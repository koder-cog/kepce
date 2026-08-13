<script>
    import Modal from "../ui/Modal.svelte";
    import { icon } from "../ui/icons.js";

    let { controller, onClose } = $props();

    const modalOptions = {
        title: "Ne oldu?",
    };
</script>

<Modal options={modalOptions} {controller} {onClose}>
    <p class="u-mb-sm">
        Yorumunuz sistem tarafından spam veya kurallara aykırılık sezdiği için
        yayınlanmadı.
    </p>

    <p class="u-mb-sm">
        Sistemin yorumu spam olarak algılaması için aşağıdaki şartlardan birini
        karşılaması gerekir:
    </p>
    <ul class="u-mb-md list-disc list-inside u-pl-md">
        <li class="u-mb-xs">Bir yorumda 2 veya daha fazla link bulunması,</li>
        <li class="u-mb-xs">
            Yorumun tamamı (linkler hariç) 50 karakterden kısaysa, içerisinde
            link bulunması,
        </li>
        <li class="u-mb-xs">
            Aynı karakterin arka arkaya 10 defadan fazla tekrarlanması (örneğin: <em
                >"aaaaaaaaaaa..."</em
            >)
        </li>
    </ul>

    <p class="color-muted">
        Eğer yorumunuzun filtreye yanlışlıkla takıldığını veya kuralların aşırı
        sıkı olduğunu düşünüyorsanız <a href="/iletişim"
            >iletişime geçebilirsiniz</a
        >.
    </p>

    {#snippet footer()}
        <button class="btn btn--primary" onclick={() => controller.close()}
            >Anladım</button
        >
    {/snippet}
</Modal>
