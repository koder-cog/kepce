<script>
	import { page } from "$app/stores";
	import EmptyState from "@/components/ui/EmptyState.svelte";
	import Seo from "@/components/ui/Seo.svelte";

	let status = $derived($page.status);
</script>

<Seo
	title="{status} - Kepçe"
	description="Kepçe'de bir hata oluştu."
	noindex={true}
/>

<div class="error-page">
	<EmptyState statusCode={status}>
		<button
			class="btn btn--primary"
			onclick={() => (window.location.href = "/")}
		>
			Ana sayfaya dön
		</button>
	</EmptyState>
</div>
