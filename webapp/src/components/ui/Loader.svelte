<script>
  let {
    size = 'm3-loader--md',
    colorClass = 'm3-loader--accent',
    type = 'default' // 'default' or 'button'
  } = $props();

  let sizeClass = $derived.by(() => {
    if (typeof size === 'number') {
      if (size <= 24) return 'm3-loader--sm';
      if (size <= 48) return 'm3-loader--md';
      return 'm3-loader--lg';
    }
    return size;
  });
</script>

{#if type === 'button'}
  <div class="btn__loader">
    <div class="m3-loader m3-loader--sm m3-loader--current">
      <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
        <path d="M50,10 C72,10 90,28 90,50 C90,72 72,90 50,90 C28,90 10,72 10,50 C10,28 28,10 50,10 Z" fill="currentColor">
          <animate 
            attributeName="d" 
            dur="3s" 
            repeatCount="indefinite" 
            calcMode="spline"
            keyTimes="0; 0.33; 0.66; 1"
            keySplines="0.4 0 0.2 1; 0.4 0 0.2 1; 0.4 0 0.2 1"
            values="
                M50,10 C72,10 90,28 90,50 C90,72 72,90 50,90 C28,90 10,72 10,50 C10,28 28,10 50,10 Z;
                M50,10 C85,10 90,15 90,50 C90,85 85,90 50,90 C15,90 10,85 10,50 C10,15 15,10 50,10 Z;
                M50,10 C60,40 60,40 90,50 C60,60 60,60 50,90 C40,60 40,60 10,50 C40,40 40,40 50,10 Z;
                M50,10 C72,10 90,28 90,50 C90,72 72,90 50,90 C28,90 10,72 10,50 C10,28 28,10 50,10 Z
            "
          />
        </path>
      </svg>
    </div>
  </div>
{:else}
  <div class="m3-loader {sizeClass} {colorClass}">
    <svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
      <path d="M50,10 C72,10 90,28 90,50 C90,72 72,90 50,90 C28,90 10,72 10,50 C10,28 28,10 50,10 Z" fill="currentColor">
        <animate 
          attributeName="d" 
          dur="3s" 
          repeatCount="indefinite" 
          calcMode="spline"
          keyTimes="0; 0.33; 0.66; 1"
          keySplines="0.4 0 0.2 1; 0.4 0 0.2 1; 0.4 0 0.2 1"
          values="
              M50,10 C72,10 90,28 90,50 C90,72 72,90 50,90 C28,90 10,72 10,50 C10,28 28,10 50,10 Z;
              M50,10 C85,10 90,15 90,50 C90,85 85,90 50,90 C15,90 10,85 10,50 C10,15 15,10 50,10 Z;
              M50,10 C60,40 60,40 90,50 C60,60 60,60 50,90 C40,60 40,60 10,50 C40,40 40,40 50,10 Z;
              M50,10 C72,10 90,28 90,50 C90,72 72,90 50,90 C28,90 10,72 10,50 C10,28 28,10 50,10 Z
          "
        />
      </path>
    </svg>
  </div>
{/if}
