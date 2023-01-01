<script lang="ts">
    import Page from '../../Page.svelte'
    import { createSnackbar } from '../../global'
    import Progress from '../../components/Progress.svelte'
    import { onMount } from 'svelte'

    let stations: Station[] = []
    let loading = false

    interface Station {
        url: string
        name: string
        image_file: string
    }

    async function fetchStations() {
        loading = true
        try {
            stations = await (await fetch('/api/stations')).json()
        } catch (err) {
            $createSnackbar(`Could not fetch stations: ${err}`)
        }
        loading = false
    }

    onMount(fetchStations)
</script>

<Page pageId="dash">
    <Progress bind:loading />
    {#each stations as station}
        {station.name}
        <br />
    {/each}
</Page>
