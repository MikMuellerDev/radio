<script lang="ts">
    import Page from '../../Page.svelte'
    import { createSnackbar } from '../../global'
    import Progress from '../../components/Progress.svelte'
    import IconButton from '@smui/icon-button'
    import { Icon } from '@smui/button'
    import List, { Graphic, Item, PrimaryText, SecondaryText, Text } from '@smui/list'
    import Slider from '@smui/slider'
    import FormField from '@smui/form-field'
    import { onMount } from 'svelte'

    let pendingStationId: string = null
    let selectedStation: string = undefined
    let stations: Station[] = []
    let currStation: Station = undefined

    $: currStation = stations.find(s => s.id == selectedStation)

    let loading = false
    let volume = 100

    // eslint-disable-next-line no-undef
    let timer: NodeJS.Timeout

    interface Station {
        id: string
        name: string
        description: string
        url: string
        image_file: string
    }

    async function setVolume() {
        if (timer) clearTimeout(timer)
        timer = setTimeout(() => {
            fetch('/api/volume', {
                body: JSON.stringify({ volume }),
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            })
        }, 100)
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

    async function fetchStatus() {
        loading = true
        try {
            let status = await (await fetch('/api/status')).json()
            selectedStation = status.stationId
            volume = status.volume
        } catch (err) {
            $createSnackbar(`Could not fetch status information: ${err}`)
        }
        loading = false
    }

    async function setPlaying(stationId: string) {
        pendingStationId = stationId
        loading = true
        try {
            let res = await fetch('/api/play', {
                body: JSON.stringify({ stationId }),
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            })

            switch (res.status) {
                case 200:
                    let playRes = await res.json()
                    if (playRes.error !== null && playRes.error !== '') {
                        throw playRes.error
                    }
                    selectedStation = stationId
                    break
                case 503:
                    selectedStation = null
                    throw (await res.json()).error
                default:
                    throw res.statusText
            }
        } catch (err) {
            $createSnackbar(`Could not start playing: ${err}`)
        }
        loading = false
    }

    async function stopPlaying() {
        pendingStationId = null
        loading = true
        try {
            let res = await fetch('/api/stop', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
            })

            switch (res.status) {
                case 200:
                    let stopRes = await res.json()
                    if (stopRes.error !== null && stopRes.error !== '') {
                        throw stopRes.error
                    }
                    selectedStation = null
                    break
                case 503:
                    throw (await res.json()).error
                default:
                    throw res.statusText
            }
        } catch (err) {
            $createSnackbar(`Could not stop playback: ${err}`)
        }
        setTimeout(() => {
            loading = false
        }, 500)
    }

    onMount(async () => {
        await fetchStations()
        await fetchStatus()
    })
</script>

<Page pageId="dash">
    <div id="container">
        <div id="container__other">
            <div id="container__other__banner" class="mdc-elevation--z3">
                <div id="container__other__banner__header">
                    <span class="text-hint">Currently Playing</span>
                    {#if currStation !== undefined}
                        <IconButton href={currStation.url} target="_blank" class="material-icons"
                            >open_in_new</IconButton
                        >
                    {/if}
                </div>

                <div id="container__other__banner__content">
                    <div
                        id="container__other__banner__img-div"
                        class="mdc-elevation--z5"
                        class:loading
                        style={currStation === undefined
                            ? ''
                            : `background-image: url("/images/${currStation.image_file}")`}
                    >
                        <div id="container__other__banner__img-div__inner">
                            {#if currStation !== undefined}
                                <img
                                    id="container__other__banner__img-div__inner__img"
                                    alt="Logo of the current station"
                                    src={`/images/${currStation.image_file}`}
                                />
                            {:else}
                                <i class="material-icons">volume_off</i>
                            {/if}
                        </div>
                    </div>

                    <div id="container__other__banner__texts">
                        {#if currStation !== undefined}
                            <h6 class:text-disabled={loading}>{currStation.name}</h6>
                            <span class:text-disabled={loading} class="text-hint"
                                >{currStation.description}</span
                            >
                        {:else}
                            <h6 class:text-disabled={loading}>Off</h6>
                            <span class:text-disabled={loading} class="text-hint"
                                >Nothing is playing</span
                            >
                        {/if}
                    </div>
                </div>
            </div>
            <div id="container__other__volume">
                <span class="text-hint">Volume Control</span>
                <div id="container__other__volume__container">
                    <div id="container__other__volume__container__left">
                        <FormField align="end" style="display: flex;">
                            <Slider
                                style="flex-grow: 1;"
                                bind:value={volume}
                                on:SMUISlider:change={setVolume}
                            />
                        </FormField>
                    </div>
                    <div id="container__other__volume__container__right">
                        <span class="text-hint">{volume}%</span>
                        <i class="material-icons">
                            {#if volume === 0}
                                volume_off
                            {:else if volume < 33}
                                volume_mute
                            {:else if volume < 66}
                                volume_down
                            {:else}
                                volume_up
                            {/if}
                        </i>
                    </div>
                </div>
            </div>
        </div>
        <div id="container__selection">
            <div id="container__selection__header">
                <span class="text-hint">Select Station</span>
            </div>
            <List twoLine avatarList singleSelection>
                {#each stations as station}
                    <Item
                        on:SMUI:action={() => {
                            if (selectedStation !== station.id) {
                                setPlaying(station.id)
                            }
                        }}
                        selected={selectedStation === station.id}
                    >
                        <Graphic>
                            {#if loading && pendingStationId === station.id}
                                <Progress bind:loading type="circular" />
                            {:else}
                                <img
                                    class="station-image"
                                    src={`/images/${station.image_file}`}
                                    alt="Logo of the station"
                                />
                            {/if}
                        </Graphic>
                        <Text>
                            <PrimaryText>
                                {station.name}
                            </PrimaryText>
                            <SecondaryText>{station.description}</SecondaryText>
                        </Text>
                    </Item>
                {/each}
                {#if stations.length > 0}
                    <Item
                        on:SMUI:action={() => {
                            if (selectedStation !== null) {
                                stopPlaying()
                            }
                        }}
                        selected={selectedStation === null}
                    >
                        <Graphic>
                            {#if loading && pendingStationId === null}
                                <Progress bind:loading type="circular" />
                            {:else}
                                <Icon class="material-icons">volume_off</Icon>
                            {/if}
                        </Graphic>
                        <Text>
                            <PrimaryText>Off</PrimaryText>
                            <SecondaryText>Play nothing</SecondaryText>
                        </Text>
                    </Item>
                {/if}
            </List>
        </div>
    </div>
</Page>

<style lang="scss">
    @use '../../mixins' as *;
    .station-image {
        max-height: 2.5rem;
        border-radius: 50%;
    }

    #container {
        display: flex;
        padding: 2rem 2.5rem;
        gap: 1rem;
        height: calc(100vh /* navbar */ - 64px /* container padding */ - 4rem);

        @include not-widescreen {
            height: auto;
            flex-direction: column;
        }

        @include mobile {
            padding: 1rem 1.5rem;
        }

        &__selection {
            width: 60%;
            height: 100%;
            background-color: var(--clr-height-0-3);
            border-radius: 0.3rem;

            &__header {
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding-left: 2.5rem;
                padding-top: 1rem;
                height: 3rem;
            }

            @include not-widescreen {
                width: 100%;
            }
        }

        &__other {
            width: 40%;
            display: flex;
            flex-direction: column;
            gap: 1rem;

            @include not-widescreen {
                width: 100%;
            }

            @include mobile {
                flex-direction: column-reverse;
            }

            &__banner {
                background-color: var(--clr-height-0-3);
                border-radius: 0.3rem;

                &__header {
                    display: flex;
                    align-items: center;
                    justify-content: space-between;
                    padding-left: 2.5rem;
                    padding-right: 0.8rem;
                    padding-top: 1rem;
                    height: 3rem;
                }

                &__content {
                    padding: 0 2.5rem;
                    padding-bottom: 2rem;
                    display: flex;
                    gap: 3rem;

                    @include not-widescreen {
                        margin-top: 1rem;
                        gap: 1rem;
                        flex-direction: column;
                        align-items: center;
                    }
                }

                &__img-div {
                    width: 9rem;
                    height: 9rem;
                    background-position: center;
                    background-size: cover;
                    transition-duration: 0.1s;
                    transition-property: filter;

                    &.loading {
                        filter: grayscale(100);
                    }

                    @include mobile {
                        width: 7rem;
                        height: 7rem;
                    }

                    &__inner {
                        background-color: rgba($color: #ffffff, $alpha: 0.04);
                        width: 100%;
                        height: 100%;

                        display: flex;
                        justify-content: center;
                        align-items: center;

                        i {
                            font-size: 5rem;
                            text-align: center;
                        }

                        &__img {
                            width: 100%;
                            height: 100%;
                            object-fit: cover;
                            backdrop-filter: blur(21px);
                        }
                    }
                }

                &__texts {
                    @include not-widescreen {
                        text-align: center;
                    }

                    h6 {
                        margin: 0.5rem 0;

                        @include mobile {
                            margin-bottom: 0.1rem;
                            font-size: 1.1rem;
                        }
                    }
                    span {
                        font-size: 0.9rem;
                    }
                }
            }

            &__volume {
                background-color: var(--clr-height-0-3);
                border-radius: 0.3rem;
                padding: 2rem 2.5rem;

                @include mobile {
                    padding: 1rem 1.5rem;
                }

                &__container {
                    display: flex;
                    align-items: center;

                    @include mobile {
                        flex-direction: column;
                        align-items: flex-start;
                    }

                    &__left {
                        width: 100%;
                    }

                    &__right {
                        display: flex;
                        align-items: center;
                        padding-left: 1rem;

                        i {
                            color: var(--clr-text-hint);
                            padding-left: 1rem;
                        }

                        span {
                            min-width: 2.1rem;
                        }
                    }
                }
            }
        }
    }
</style>
