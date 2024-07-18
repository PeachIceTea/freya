import { create } from "zustand"
import { immer } from "zustand/middleware/immer"

import { SessionInfo } from "./api/authentication"
import { BookDetails, File } from "./api/books"
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
	selectedBook: Required<BookDetails> | null
	playing: boolean
	volume: number
	playbackSpeed: number
	forceSeek: boolean
}

type Actions = {
	setSessionInfo: (sessionInfo: SessionInfo) => void
	setTheme: (theme: Theme) => void
	reset: () => void

	// Player actions.
	playBook: (book: Required<BookDetails>) => void
	playFile: (file: File) => void
	play(): void
	pause(): void
	togglePlay: () => void
	nextFile: () => boolean
	prevFile: () => boolean
	setVolume: (volume: number) => void
	setPlaybackSpeed: (speed: number) => void
	updateProgress: (progress: number) => void
	seekTo: (progress: number) => void
	seekComplete: () => void
}

const initialState = (): State => ({
	theme: getThemeFromLocalStorage(),
	sessionInfo: null,

	// Player state.
	selectedBook: null,
	playing: false,
	volume: getVolumeFromLocalStorage(),
	playbackSpeed: getPlaybackSpeedFromLocalStorage(),
	forceSeek: false,
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
		playBook: (book: Required<BookDetails>) =>
			set(state => {
				state.selectedBook = book
				state.playing = true
			}),
		playFile: (file: File) =>
			set(state => {
				state.playing = true
				state.selectedBook.library.fileId = file.id
				state.selectedBook.library.progress = 0
			}),
		play: () =>
			set(state => {
				state.playing = true
			}),
		pause: () =>
			set(state => {
				state.playing = false
			}),
		togglePlay: () =>
			set(state => {
				state.playing = !state.playing
			}),
		nextFile: () => {
			let result = false
			set(state => {
				// Check if a book is selected.
				if (!state.selectedBook) {
					return
				}

				// Get next file.
				const currentFile = state.selectedBook.files.find(
					file => file.id === state.selectedBook!.library.fileId,
				)
				const nextFile =
					state.selectedBook.files[
						state.selectedBook.files.indexOf(currentFile!) + 1
					]

				// Check if there is a next file.
				if (!nextFile) {
					return
				}

				// Update library.
				state.selectedBook.library.fileId = nextFile.id
				state.selectedBook.library.progress = 0
				state.playing = true
				result = true
			})
			return result
		},
		prevFile: () => {
			let result = false
			set(state => {
				// Check if a book is selected.
				if (!state.selectedBook) {
					return
				}

				// Get previous file.
				const currentFile = state.selectedBook.files.find(
					file => file.id === state.selectedBook!.library.fileId,
				)!
				const prevFile =
					state.selectedBook.files[
						state.selectedBook.files.indexOf(currentFile!) - 1
					]

				// Check if there is a previous file.
				if (!prevFile) {
					return
				}

				// Update library.
				state.selectedBook.library.fileId = prevFile.id
				state.selectedBook.library.progress = 0
				state.playing = true
				result = true
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
				// Check if a book is selected.
				if (!state.selectedBook) {
					return
				}

				state.selectedBook.library.progress = progress
			}),
		seekTo: (progress: number) =>
			set(state => {
				// Check if a book is selected.
				if (!state.selectedBook) {
					return
				}

				state.selectedBook.library.progress = progress
				state.forceSeek = true
			}),
		seekComplete: () =>
			set(state => {
				state.forceSeek = false
			}),
	})),
)
