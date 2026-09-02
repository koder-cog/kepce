<script>
  import Modal from "@/components/ui/Modal.svelte";
  import { icon } from "@/components/ui/icons.js";

  let { isOpen = $bindable(false) } = $props();
</script>

{#if isOpen}
  <Modal
    options={{
      title: "Hakkında",
      iconHtml: icon("info", 24),
    }}
    onClose={() => (isOpen = false)}
  >
    {#snippet children()}
      <div class="u-flex u-flex-col u-gap-md">
        <p class="u-text-sm u-leading-comfortable">
          Kepçe Ara, bağımsız SearXNG altyapısını kullanan açık kaynaklı bir meta arama motorudur.
        </p>

        <p class="u-text-sm u-leading-comfortable">
          Aramalarınız profillenmez, takip edilmez ve arama geçmişiniz sunucularda saklanmaz. Arama yaptığınızda sorgunuz kimliğinizden arındırılarak onlarca farklı arama motoruna iletilir ve sonuçlar tarafsız bir şekilde listelenir.
        </p>

        <div class="card card--flat u-p-md u-flex u-flex-col u-gap-xs">
          <div class="u-text-xs u-font-bold u-color-text">Temel Prensipler:</div>
          <ul class="u-text-xs u-color-muted u-pl-md u-flex u-flex-col u-gap-2xs">
            <li>Kişisel arama geçmişi ve profil kaydı tutulmaz.</li>
            <li>Reklam hedeflemesi ve filtre balonu oluşturulmaz.</li>
            <li>AGPLv3 lisansıyla tamamen özgür ve açık kaynaklıdır.</li>
          </ul>
        </div>
      </div>
    {/snippet}

    {#snippet footer()}
      <button
        type="button"
        class="btn btn--primary"
        onclick={() => (isOpen = false)}
      >
        Anladım
      </button>
    {/snippet}
  </Modal>
{/if}
