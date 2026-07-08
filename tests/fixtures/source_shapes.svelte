<script lang="ts">
    export let title: string;
    const items = [{ id: "a", name: "Alpha" }];
    const load = Promise.resolve({ status: "ready" });
</script>

<section class="panel">
    <h1>{title}</h1>
    <Widget name="chart" />
    {#if items.length > 0}
        {#each items as item (item.id)}
            <Row {item} />
        {/each}
    {:else}
        <p>No items</p>
    {/if}
    {#await load}
        <p>Loading</p>
    {:then result}
        <p>{result.status}</p>
    {/await}
    {#key title}
        <Title value={title} />
    {/key}
    {#snippet summary(name)}
        <span>{name}</span>
    {/snippet}
    {@render summary(title)}
    {@const total = items.length}
</section>

<style>
    .panel {
        display: grid;
    }
</style>
