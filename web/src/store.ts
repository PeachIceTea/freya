import { create } from "zustand"
import { immer } from "zustand/middleware/immer"

import { SessionInfo } from "./api/authentication"
import { Theme } from "./common"

type State = {
	theme: Theme
	sessionInfo: SessionInfo | null
}

type Actions = {
	setSessionInfo: (sessionInfo: SessionInfo) => void
	setTheme: (theme: Theme) => void
	reset: () => void
}

const initialState: State = {
	theme: "system",
	sessionInfo: null,
}

export const useStore = create<State & Actions>()(
	immer(set => ({
		...initialState,
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
				return { ...initialState }
			}),
	})),
)
