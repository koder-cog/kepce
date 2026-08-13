<script>
    let { type = 'mealCard' } = $props();
</script>

{#if type === 'mealCard'}
    <div class="meal-card-skeleton">
        <div class="meal-card-skeleton__header">
            <div class="skeleton meal-card-skeleton__title"></div>
            <div class="skeleton meal-card-skeleton__badge"></div>
        </div>
        <div class="meal-card-skeleton__items">
            <div class="meal-card-skeleton__row">
                <div class="skeleton meal-card-skeleton__text skeleton--w80"></div>
                <div class="skeleton meal-card-skeleton__action"></div>
            </div>
            <div class="meal-card-skeleton__row">
                <div class="skeleton meal-card-skeleton__text skeleton--w60"></div>
                <div class="skeleton meal-card-skeleton__action"></div>
            </div>
            <div class="meal-card-skeleton__row">
                <div class="skeleton meal-card-skeleton__text skeleton--w70"></div>
                <div class="skeleton meal-card-skeleton__action"></div>
            </div>
            <div class="meal-card-skeleton__row">
                <div class="skeleton meal-card-skeleton__text skeleton--w50"></div>
                <div class="skeleton meal-card-skeleton__action"></div>
            </div>
            <div class="meal-card-skeleton__row">
                <div class="skeleton meal-card-skeleton__text skeleton--w65"></div>
                <div class="skeleton meal-card-skeleton__action"></div>
            </div>
            <div class="meal-card-skeleton__row">
                <div class="skeleton meal-card-skeleton__text skeleton--w45"></div>
                <div class="skeleton meal-card-skeleton__action"></div>
            </div>
            <div class="meal-card-skeleton__row">
                <div class="skeleton meal-card-skeleton__text skeleton--w75"></div>
                <div class="skeleton meal-card-skeleton__action"></div>
            </div>
        </div>
        <div class="meal-card-skeleton__footer">
            <div class="skeleton meal-card-skeleton__votes"></div>
            <div class="meal-card-skeleton__actions">
                <div class="skeleton meal-card-skeleton__action-btn"></div>
                <div class="skeleton meal-card-skeleton__action-btn"></div>
            </div>
        </div>
    </div>
{:else if type === 'botCorner'}
    <div class="bot-card ai-element skeleton--bot-card">
        <div class="skeleton skeleton--bot-title"></div>
        <div class="skeleton skeleton--bot-line"></div>
        <div class="skeleton skeleton--bot-line"></div>
        <div class="skeleton skeleton--bot-line-80"></div>
        <div class="bot-card__footer">
            <div class="skeleton skeleton--bot-action"></div>
            <div class="skeleton skeleton--bot-report"></div>
        </div>
    </div>
{:else if type === 'timeline'}
    <div class="timeline skeleton--timeline">
        <div class="timeline__line"></div>
        <div class="timeline__slot">
            <div class="timeline__time"><div class="skeleton skeleton--text skeleton--time-w"></div></div>
            <div class="timeline__content timeline__content--60-40">
                <div class="timeline__meal-wrapper">
                    <!-- Self reference for mealCard type via Svelte recursively doesn't work easily if we just write HTML, so we just duplicate or use components -->
                    <!-- Svelte allows <svelte:self> but here we can just write the markup directly or use an internal snippet, but copying is fine for skeletons -->
                    <div class="meal-card-skeleton">
                        <div class="meal-card-skeleton__header">
                            <div class="skeleton meal-card-skeleton__title"></div>
                            <div class="skeleton meal-card-skeleton__badge"></div>
                        </div>
                        <div class="meal-card-skeleton__items">
                            <div class="meal-card-skeleton__row"><div class="skeleton meal-card-skeleton__text skeleton--w80"></div><div class="skeleton meal-card-skeleton__action"></div></div>
                            <div class="meal-card-skeleton__row"><div class="skeleton meal-card-skeleton__text skeleton--w60"></div><div class="skeleton meal-card-skeleton__action"></div></div>
                            <div class="meal-card-skeleton__row"><div class="skeleton meal-card-skeleton__text skeleton--w70"></div><div class="skeleton meal-card-skeleton__action"></div></div>
                            <div class="meal-card-skeleton__row"><div class="skeleton meal-card-skeleton__text skeleton--w50"></div><div class="skeleton meal-card-skeleton__action"></div></div>
                            <div class="meal-card-skeleton__row"><div class="skeleton meal-card-skeleton__text skeleton--w65"></div><div class="skeleton meal-card-skeleton__action"></div></div>
                            <div class="meal-card-skeleton__row"><div class="skeleton meal-card-skeleton__text skeleton--w45"></div><div class="skeleton meal-card-skeleton__action"></div></div>
                            <div class="meal-card-skeleton__row"><div class="skeleton meal-card-skeleton__text skeleton--w75"></div><div class="skeleton meal-card-skeleton__action"></div></div>
                        </div>
                        <div class="meal-card-skeleton__footer">
                            <div class="skeleton meal-card-skeleton__votes"></div>
                            <div class="meal-card-skeleton__actions">
                                <div class="skeleton meal-card-skeleton__action-btn"></div>
                                <div class="skeleton meal-card-skeleton__action-btn"></div>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="bot-wrapper">
                    <div class="bot-card ai-element skeleton--bot-card">
                        <div class="skeleton skeleton--bot-title"></div>
                        <div class="skeleton skeleton--bot-line"></div>
                        <div class="skeleton skeleton--bot-line"></div>
                        <div class="skeleton skeleton--bot-line-80"></div>
                        <div class="bot-card__footer">
                            <div class="skeleton skeleton--bot-action"></div>
                            <div class="skeleton skeleton--bot-report"></div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
{/if}
