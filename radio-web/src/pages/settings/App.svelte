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
    <Progress bind:loading />

    <div id="device">
        <h6>Output Devices</h6>
        <ul>
            {#each outputDevices as device}
                <li>
                    <code>
                        {device}
                    </code>
                </li>
            {/each}
        </ul>

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
</Page>

<style lang="scss">
    #device {
        margin: 1rem 1.5rem;

        h6 {
            margin: 0;
        }
    }
</style>
