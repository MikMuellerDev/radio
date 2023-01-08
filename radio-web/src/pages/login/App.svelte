<script lang="ts">
    import Textfield from '@smui/textfield'
    import Button from '@smui/button'
    import type { KitchenComponentDev } from '@smui/snackbar/kitchen'
    import Progress from '../../components/Progress.svelte'
    import { createSnackbar } from '../../global'
    import Kitchen from '@smui/snackbar/kitchen'
    import TopAppBar, { Row, Section, Title, AutoAdjust } from '@smui/top-app-bar'

    let username = ''
    let password = ''
    let loading = false
    let invalid = false
    let topAppBar: TopAppBar

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
                    invalid = true
                    setTimeout(() => (invalid = false), 1000)
                    throw 'Invalid username or password'
            }
        } catch (err) {
            if (typeof err === 'string') $createSnackbar(err)
            else $createSnackbar('An unknown error occurred. Please try again')
        }
        loading = false
    }
</script>

<TopAppBar bind:this={topAppBar} variant="standard">
    <Row>
        <Section>
            <Title>Radio</Title>
        </Section>
    </Row>
</TopAppBar>

<AutoAdjust {topAppBar}>
    <Progress bind:loading />
    <div id="container">
        <div id="container__login">
            <div id="container__login__textfields">
                <Textfield bind:value={username} bind:invalid label="Username" />
                <Textfield bind:value={password} bind:invalid type="password" label="Password" />
            </div>
            <Button on:click={login} variant="raised">Login</Button>
        </div>
    </div>
</AutoAdjust>

<Kitchen bind:this={kitchen} dismiss$class="material-icons" />

<style lang="scss">
    @use '../../mixins' as *;

    #container {
        display: flex;
        justify-content: center;
        align-items: center;
        height: calc(100vh /* navbar */ - 64px);

        &__login {
            background-color: var(--clr-height-0-6);
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 1rem;
            padding: 1.5rem 1.7rem;
            border-radius: 0.3rem;

            &__textfields {
                display: flex;
                flex-direction: column;
                align-items: center;
                margin-bottom: 1rem;
                gap: 1rem;
            }
        }
    }
</style>
