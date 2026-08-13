<!-- 
  Kepçe Webapp — Bileşen: Switch (Toggle)
  =======================================
  
  Kural 2: "Daha fazla alan sağladığı için checkbox yerine switch kullanılır."
  Tüm formlarda (Ayarlar vb.) bu bileşen kullanılacak.
-->
<script>
  let { checked = $bindable(false), label = "" } = $props();
</script>

<label class="toggle-switch squish-effect">
  <input type="checkbox" bind:checked class="c-input-hidden">
  <span class="slider"></span>
  <span class="label-text">{label}</span>
</label>
