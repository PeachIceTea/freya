import useSWR from "swr"
import { z } from "zod"

import {
	DataResponseSchema,
	ResponseSchema,
	ServerConnectionError,
	get,
	patch,
	post,
} from "./api"

export const UserSchema = z.object({
	id: z.number(),
	name: z.string(),
	admin: z.boolean(),
	created: z.string(), // ISO 8601 date string
	modified: z.string(), // ISO 8601 date string
})
const UserResponseSchema = DataResponseSchema(UserSchema)
export type User = z.infer<typeof UserSchema>

export const UsersSchema = z.array(UserSchema)
export const UsersResponseSchema = DataResponseSchema(UsersSchema)

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

// Get a single user.
export const getUser = async (id: number) => {
	const res = await get(`/user/${id}`)
	return UserResponseSchema.parse(await res)
}

export const useUser = (id: number) => {
	const {
		data,
		error: parseError,
		isLoading,
	} = useSWR(`/user/${id}`, () => getUser(id))

	let error
	if (parseError) {
		console.error(parseError)
		error = ServerConnectionError
	} else if (!data && !isLoading) {
		error = ServerConnectionError
	}

	let user
	if (data?.success) {
		user = data.data
	} else {
		error = data
	}

	return { user, error, isLoading }
}

// Create a user.
export const createUser = async (
	data: Omit<User, "id" | "created" | "modified"> & { password: string },
) => {
	const res = await post("/user", data)
	return ResponseSchema.parse(await res)
}

// Update a user.
export const updateUser = async (
	id: number,
	data: Partial<User> & { password?: string },
) => {
	const res = await patch(`/user/${id}`, data)
	return ResponseSchema.parse(await res)
}
