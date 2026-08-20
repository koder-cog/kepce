<script>
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { onMount } from "svelte";
    import Loader from "@/components/ui/Loader.svelte";

    let params = $derived(page.params);

    onMount(() => {
        const id = params?.id;
        const threadId = params?.threadId || page.url.searchParams.get("thread");
        const target = threadId ? `/menu/${id}/${threadId}` : `/menu/${id}`;
        goto(target, { replaceState: true });
    });
</script>

<div class="u-flex u-justify-center u-items-center" style="min-height: 40vh;">
    <Loader size={48} />
</div>
