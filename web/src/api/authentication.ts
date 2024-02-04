import { z } from "zod"

import { useStore } from "../store"
import {
	DataResponseSchema,
	Response,
	ResponseSchema,
	ServerConnectionError,
	del,
	get,
	post,
} from "./api"

// Send a login request to the API.
export const login = async (
	username: string,
	password: string,
): Promise<Response> => {
	try {
		const res = ResponseSchema.parse(
			await post("/login", { username, password }),
		)
		return res
	} catch (error) {
		console.error(error)
		return ServerConnectionError
	}
}

// Send a request to the API to get the current session info.
export const SessionInfoSchema = z.object({
	user_id: z.number().int(),
	last_accessed: z.string(), // ISO8601 date string
	username: z.string(),
	admin: z.boolean(),
})
export type SessionInfo = z.infer<typeof SessionInfoSchema>
const SessionInfoResponseSchema = DataResponseSchema(SessionInfoSchema)
type SessionInfoResponse = z.infer<typeof SessionInfoResponseSchema>

export const getSessionInfo = async (): Promise<SessionInfoResponse> => {
	const res = await get("/info")
	const info = SessionInfoResponseSchema.parse(res)
	if (info.success) {
		useStore.getState().setSessionInfo(info.data)
	}
	return info
}

export const checkSession = async () => {
	const res = await getSessionInfo()
	if (res.success) {
		const store = useStore.getState()
		store.setSessionInfo(res.data)
		store.finishInit()
	}
}
// Send a request to the API to log out the current user.
export const logout = async () => {
	try {
		await del("/logout")
		useStore.getState().reset()
	} catch (error) {
		console.error(error)
		return ServerConnectionError
	}
}
