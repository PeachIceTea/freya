import { create } from "zustand"
import { immer } from "zustand/middleware/immer"

import { SessionInfo } from "./api/authentication"
import { BookDetails } from "./api/books"
import {
	PlaybackSpeedLocalStorageKey,
	Theme,
	VolumeLocalStorageKey,
	getPlaybackSpeedFromLocalStorage,
	getThemeFromLocalStorage,
	getVolumeFromLocalStorage,
} from "./common"

type State = {
	theme: Theme
	sessionInfo: SessionInfo | null

	// Player state.
	selectedBook: BookDetails | null
	playing: boolean
	volume: number
	playbackSpeed: number
}

type Actions = {
	setSessionInfo: (sessionInfo: SessionInfo) => void
	setTheme: (theme: Theme) => void
	reset: () => void

	// Player actions.
	playBookFromStart: (book: BookDetails) => void
	play(): void
	pause(): void
	nextFile(): boolean
	prevFile(): boolean
	setVolume: (volume: number) => void
	setPlaybackSpeed: (speed: number) => void
	updateProgress: (fileId: number, progress: number) => void
}

const initialState = (): State => ({
	theme: getThemeFromLocalStorage(),
	sessionInfo: null,

	// Player state.
	selectedBook: null,
	selectedFileIndex: null,
	playing: false,
	volume: getVolumeFromLocalStorage(),
	playbackSpeed: getPlaybackSpeedFromLocalStorage(),
})

export const useStore = create<State & Actions>()(
	immer(set => ({
		...initialState(),
		setTheme: (theme: Theme) =>
			set(state => {
				state.theme = theme
			}),
		setSessionInfo: (sessionInfo: SessionInfo) =>
			set(state => {
				state.sessionInfo = sessionInfo
			}),
		reset: () =>
			set(() => {
				return initialState()
			}),

		// Player actions.
		playBookFromStart: (book: BookDetails) =>
			set(state => {
				state.selectedBook = book
				state.playing = true
			}),
		play: () =>
			set(state => {
				state.playing = true
			}),
		pause: () =>
			set(state => {
				state.playing = false
			}),
		nextFile: () => {
			let result = false
			set(state => {
				if (state.selectedBook && state.selectedFileIndex !== null) {
					if (state.selectedFileIndex < state.selectedBook.files.length - 1) {
						state.playing = true
						result = true
					}
				}
			})
			return result
		},
		prevFile: () => {
			let result = false
			set(state => {
				if (state.selectedBook && state.selectedFileIndex !== null) {
					if (state.selectedFileIndex > 0) {
						state.selectedFileIndex = state.selectedFileIndex - 1
						state.playing = true
						result = true
					}
				}
			})
			return result
		},
		setVolume: (volume: number) =>
			set(state => {
				state.volume = volume
				localStorage.setItem(VolumeLocalStorageKey, volume.toString())
			}),
		setPlaybackSpeed: (speed: number) =>
			set(state => {
				state.playbackSpeed = speed
				localStorage.setItem(PlaybackSpeedLocalStorageKey, speed.toString())
			}),
		updateProgress: (progress: number) =>
			set(state => {
				if (state.selectedBook?.library) {
					state.selectedBook.library.progress = progress
				}
			}),
	})),
)
