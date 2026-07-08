<script lang="ts">
    import Row from "./Row.svelte";

    let count = $state(0);
    const items = [{ id: "one", name: "One" }];
    const ready = Promise.resolve({ message: "ok" });

    function increment() {
        count += 1;
    }
</script>

<main data-count={count}>
    <button onclick={increment}>Clicked {count}</button>
    <Icon name="plus" />
    {#if count > 0}
        {#each items as item, index (item.id)}
            <Row {item} {index} />
        {/each}
    {:else}
        <p>Empty</p>
    {/if}
    {#await ready}
        <p>Loading</p>
    {:then result}
        <p>{result.message}</p>
    {:catch err}
        <p>{err.message}</p>
    {/await}
    {#key count}
        <output>{count}</output>
    {/key}
    {#snippet label(name)}
        <span>{name}</span>
    {/snippet}
    {@render label("total")}
    {@const doubled = count * 2}
</main>

<style>
    main {
        display: grid;
        gap: 1rem;
    }
</style>
