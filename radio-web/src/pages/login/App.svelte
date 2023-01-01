<script lang="ts">
    import Textfield from '@smui/textfield'
    import Button from '@smui/button'
    import type { KitchenComponentDev } from '@smui/snackbar/kitchen'
    import Progress from '../../components/Progress.svelte'
    import { createSnackbar } from '../../global'
    import Kitchen from '@smui/snackbar/kitchen'

    let username = ''
    let password = ''
    let loading = false

    let kitchen: KitchenComponentDev
    $createSnackbar = (message: string) => {
        kitchen.push({
            label: message,
            dismissButton: true,
        })
    }

    async function login() {
        loading = true
        try {
            let res = await fetch('/api/login', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password }),
            })
            switch (res.status) {
                case 200:
                    window.location.href = '/'
                    return
                case 403:
                    throw 'Invalid username or password'
            }
        } catch (err) {
            if (typeof err === 'string') $createSnackbar(err)
            else $createSnackbar('An unknown error occurred. Please try again')
        }
        loading = false
    }
</script>

<div id="container">
    <Progress bind:loading />
    <Textfield variant="filled" bind:value={username} label="Username" />
    <Textfield variant="filled" bind:value={password} type="password" label="Password" />
    <Button on:click={login} variant="raised">Login</Button>
</div>
<Kitchen bind:this={kitchen} dismiss$class="material-icons" />

<style lang="scss">
    #container {
        background-color: var(--clr-height-0-6);

        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);

        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
        padding: 1rem;
        border-radius: 0.3rem;
    }
</style>
