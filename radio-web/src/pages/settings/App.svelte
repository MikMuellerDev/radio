<script lang="ts">
    import Page from '../../Page.svelte'
    import { createSnackbar } from '../../global'
    import Select, { Option } from '@smui/select'
    import { onMount } from 'svelte'
    import Progress from '../../components/Progress.svelte'
    import Button from '@smui/button'

    let outputDevices: String[] = []
    let selectedOutputDevice: String = undefined
    let currentOutputDevice: String = undefined
    let loading = false

    async function fetchOutputDevices() {
        try {
            outputDevices = await (await fetch('/api/devices')).json()
        } catch (err) {
            $createSnackbar(`Could not fetch output devices: ${err}`)
        }
    }

    async function fetchOutputDeviceIndex(): Promise<number> {
        try {
            let res = await (await fetch('/api/device')).json()
            return res.index
        } catch (err) {
            $createSnackbar(`Could not fetch device information: ${err}`)
        }
    }

    async function postOutputDevice() {
        loading = true
        try {
            let index = outputDevices.findIndex(device => device == selectedOutputDevice)
            let res = await (
                await fetch('/api/device', {
                    method: 'POST',
                    headers: { 'Content-Type': 'application/json' },
                    body: JSON.stringify({ index }),
                })
            ).json()

            if (res.error !== null) {
                throw res.error
            }

            currentOutputDevice = selectedOutputDevice
        } catch (err) {
            $createSnackbar(`Could not save output device: ${err}`)
        }
        loading = false
    }

    onMount(async () => {
        loading = true
        await fetchOutputDevices()
        let index = await fetchOutputDeviceIndex()
        loading = false

        selectedOutputDevice = outputDevices[index]
        currentOutputDevice = outputDevices[index]
    })
</script>

<Page pageId="settings">
    <div id="device">
        <div id="device__left">
            <div id="device__left__header">
                <span class="text-hint">Output Devices</span>
                <Progress bind:loading type="circular" />
            </div>
            <ul>
                {#each outputDevices as device}
                    <li>
                        <code>
                            {device}
                        </code>
                    </li>
                {/each}
            </ul>
        </div>
        <div id="device__right">
            <Select bind:value={selectedOutputDevice} label="Select device">
                {#each outputDevices as device}
                    <Option value={device}>{device}</Option>
                {/each}
            </Select>
            <Button
                on:click={postOutputDevice}
                disabled={currentOutputDevice === selectedOutputDevice || loading}>Save</Button
            >
        </div>
    </div>
</Page>

<style lang="scss">
    @use '../../mixins' as *;

    #device {
        margin: 1rem 1.5rem;
        background-color: var(--clr-height-0-3);
        border-radius: 0.3rem;
        padding: 1rem 1.5rem;
        display: flex;
        justify-content: space-between;
        box-sizing: border-box;
        height: calc(100vh - 64px - 2rem);

        @include mobile {
            flex-direction: column;
        }

        &__left {
            &__header {
                display: flex;
                align-items: center;
                gap: 1rem;
            }

            @include mobile {
                ul {
                    max-width: 60vw;
                    overflow-x: scroll;

                    li > code {
                        font-size: 0.8rem;
                    }
                }
            }
        }
    }
</style>
