import useSWR from "swr"
import { z } from "zod"

import { DataResponseSchema, ServerConnectionError, get } from "./api"

export const UserSchema = z.object({
	id: z.number(),
	name: z.string(),
	admin: z.boolean(),
	created: z.string(), // ISO 8601 date string
	modified: z.string(), // ISO 8601 date string
})
export type User = z.infer<typeof UserSchema>

export const UsersResponseSchema = DataResponseSchema(z.array(UserSchema))

// Get all users.
export const getUsers = async () => {
	const res = await get("/user")
	return UsersResponseSchema.parse(await res)
}

export const useUsers = () => {
	const { data, error: parseError, isLoading } = useSWR("/user", getUsers)

	let error
	if (parseError) {
		console.error(parseError)
		error = ServerConnectionError
	} else if (!data && !isLoading) {
		error = ServerConnectionError
	}

	let users
	if (data?.success) {
		users = data.data
	} else {
		error = data
	}

	return { users, error, isLoading }
}
