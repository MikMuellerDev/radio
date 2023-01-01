import type { ConfigAction } from '@smui/snackbar/kitchen'
import { get, writable } from 'svelte/store'
import type { Writable } from 'svelte/store'

// eslint-disable-next-line @typescript-eslint/no-empty-function
export const createSnackbar: Writable<(message: string, actions?: ConfigAction[]) => void> = writable(() => {})

// Given an arbitrary input color, the function decides whether text on the color should be white or black
export function contrast(color: string): 'black' | 'white' {
    const r = parseInt(color.slice(1, 3), 16)
    const g = parseInt(color.slice(3, 5), 16)
    const b = parseInt(color.slice(5, 7), 16)
    const a = [r, g, b].map(v => {
        v /= 255
        return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4)
    })
    const luminance = a[0] * 0.2126 + a[1] * 0.7152 + a[2] * 0.0722
    const [darker, brighter] = [1.05, luminance + 0.05].sort()
    return brighter / darker <= 4.5 ? 'black' : 'white'
}
