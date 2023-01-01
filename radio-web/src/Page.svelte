<script lang="ts">
    import type { KitchenComponentDev } from '@smui/snackbar/kitchen'
    import Kitchen from '@smui/snackbar/kitchen'
    import type { ConfigAction } from '@smui/snackbar/kitchen'
    import TopAppBar, { Row, Section, Title, AutoAdjust } from '@smui/top-app-bar'
    import IconButton from '@smui/icon-button'
    import Drawer, { AppContent, Content, Header, Subtitle, Scrim } from '@smui/drawer'
    import { Item, Text, Graphic } from '@smui/list'
    import { createSnackbar } from './global'

    let open = false
    let topAppBar: TopAppBar

    export let pageId: 'dash' | 'settings' | 'logout' = 'dash'

    //document.documentElement.style.setProperty('--clr-primary-dark', '#ff0000')
    /* document.documentElement.style.setProperty(
        '--clr-on-primary-dark',
        contrast('#ff0000') === 'black' ? '#121212' : '#ffffff',
    ) */

    let kitchen: KitchenComponentDev
    $createSnackbar = (message: string, actions?: ConfigAction[]) => {
        kitchen.push({
            label: message,
            dismissButton: true,
            actions,
        })
    }
</script>

<TopAppBar bind:this={topAppBar} variant="standard">
    <Row>
        <Section>
            <IconButton class="material-icons" on:click={() => (open = !open)}>menu</IconButton>
            <Title>Radio</Title>
        </Section>
        <Section align="end" toolbar>
            <IconButton class="material-icons" aria-label="Download">file_download</IconButton>
            <IconButton class="material-icons" aria-label="Print this page">print</IconButton>
            <IconButton class="material-icons" aria-label="Bookmark this page">bookmark</IconButton>
        </Section>
    </Row>
</TopAppBar>

<AutoAdjust {topAppBar}>
    <Drawer variant="modal" fixed={true} bind:open>
        <Header>
            <br />
            <Title>Radio</Title>
            <Subtitle>An internet radio written in Rust</Subtitle>
        </Header>
        <Content>
            <div class="drawer-items">
                <div>
                    <Item href="/" activated={pageId === 'dash'}>
                        <Graphic class="material-icons" aria-hidden="true">home</Graphic>
                        <Text>Home</Text>
                    </Item>
                    <Item href="/settings" activated={pageId === 'settings'}>
                        <Graphic class="material-icons" aria-hidden="true">settings</Graphic>
                        <Text>Settings</Text>
                    </Item>
                </div>
                <div>
                    <Item href="/logout" activated={pageId === 'logout'}>
                        <Graphic class="material-icons" aria-hidden="true">logout</Graphic>
                        <Text>Logout</Text>
                    </Item>
                </div>
            </div>
        </Content>
    </Drawer>
    <Scrim fixed={true} />

    <AppContent class="app-content">
        <main class="main-content">
            <slot />
        </main>
    </AppContent>
</AutoAdjust>

<Kitchen bind:this={kitchen} dismiss$class="material-icons" />

<style lang="scss">
    :global(body) {
        margin: 0;
    }

    .drawer-items {
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        height: calc(100vh - 11rem);
        padding: 1rem 0;
    }
</style>
