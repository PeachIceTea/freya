import { create } from "zustand"
import { immer } from "zustand/middleware/immer"

import { SessionInfo } from "./api/authentication"

type State = {
	initFinished: boolean
	sessionInfo: SessionInfo | null
}

type Actions = {
	finishInit: () => void
	setSessionInfo: (sessionInfo: SessionInfo) => void
	reset: () => void
}

const initialState: State = {
	initFinished: false,
	sessionInfo: null,
}

export const useStore = create<State & Actions>()(
	immer(set => ({
		...initialState,
		finishInit: () =>
			set(state => {
				state.initFinished = true
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
