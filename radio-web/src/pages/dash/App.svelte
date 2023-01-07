<script lang="ts">
    //import Page from '../../Page.svelte'
    import { createSnackbar } from '../../global'
    import Progress from '../../components/Progress.svelte'
    import { Icon } from '@smui/button'
    import List, { Graphic, Item, PrimaryText, SecondaryText, Text } from '@smui/list'
    import { onMount } from 'svelte'

    let stations: Station[] = []
    let loading = false

    interface Station {
        id: string
        name: string
        description: string
        url: string
        image_file: string
    }

    let selectionIndex = 0
    let selectedStation = ''

    async function fetchStations() {
        loading = true
        try {
            stations = await (await fetch('/api/stations')).json()
        } catch (err) {
            $createSnackbar(`Could not fetch stations: ${err}`)
        }
        loading = false
    }

    async function fetchCurrentlyPlaying() {
        loading = true
        try {
            stations = await (await fetch('/api/status')).json()
        } catch (err) {
            $createSnackbar(`Could not fetch status information: ${err}`)
        }
        loading = false
    }

    onMount(() => {
        fetchCurrentlyPlaying()
        fetchStations()
    })
</script>

<!-- <Page pageId="dash"> -->
<Progress bind:loading />
<List twoLine avatarList singleSelection bind:selectionIndex>
    {#each stations as station}
        <Item
            on:SMUI:action={() => (selectedStation = station.id)}
            selected={selectedStation === station.id}
        >
            <Graphic>
                <img
                    class="station-image"
                    src={`/images/${station.image_file}`}
                    alt="Logo of the station"
                />
            </Graphic>
            <Text>
                <PrimaryText>
                    {station.name}
                </PrimaryText>
                <SecondaryText>{station.description}</SecondaryText>
            </Text>
        </Item>
    {/each}
</List>

<!-- </Page> -->
<style lang="scss">
    .station-image {
        max-height: 3.4rem;
    }
</style>
