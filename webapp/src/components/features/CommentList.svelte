<!-- 
  Kepçe Webapp - Bileşen: CommentList
  ===================================
  
  CommentCard'ları hiyerarşik (ağaç) yapıda render eder.
  Recursive (kendi kendini çağıran) bir Svelte bileşenidir.
-->
<script>
  import "@/styles/pages/_comments.css";
  import CommentCard from "./CommentCard.svelte";
  let { comments = [], depth = 0, menuId, onloadData } = $props();
</script>

<div class="comment-thread">
  {#each comments as comment}
    <CommentCard {comment} {depth} {menuId} {onloadData} />
  {/each}
</div>
