import { z } from "zod"

// Send a DELETE request to the API.
export async function del(path: string) {
	return api(path, {
		method: "DELETE",
	})
}

// Send a PATCH request to the API.
export async function patch(path: string, body: unknown) {
	return api(path, {
		method: "PATCH",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify(body),
	})
}

// Send a POST multipart request to the API.
export async function postMultipart(path: string, body: FormData) {
	return api(path, {
		method: "POST",
		body,
	})
}

// Send a POST request to the API.
export async function post(path: string, body: unknown) {
	return api(path, {
		method: "POST",
		headers: {
			"Content-Type": "application/json",
		},
		body: JSON.stringify(body),
	})
}

// Send a GET request to the API.
export function get(path: string) {
	return api(path)
}

// Send a generic request to the API and return the parsed response.
export async function api(path: string, options: RequestInit = {}) {
	try {
		// Send request to API and return the parsed response.
		const response = await fetch(`/api${path}`, options)
		return response.json()
	} catch (e) {
		return ServerConnectionError
	}
}

// Generic error for when the server cannot be reached or returns an unexpected response.
export const ServerConnectionError: Error = {
	success: false,
	error_code: "server-connection-error",
}

// Schema for the error response from the API.
export const ErrorSchema = z.object({
	success: z.literal(false),
	error_code: z.string(),
	value: z.string().optional(),
})
export type Error = z.infer<typeof ErrorSchema>

// Schema for a successful response from the API.
export const SuccessResponseSchema = z.object({
	success: z.literal(true),
	message: z.string(),
	value: z.string().optional(),
})

// Schema for a successful response from the API with data.
export const SuccessDataResponseSchema = <T extends z.ZodTypeAny>(schema: T) =>
	z.object({
		success: z.literal(true),
		data: schema,
	})

// Schema for a response from the API.
export const ResponseSchema = z.union([SuccessResponseSchema, ErrorSchema])
export type Response = z.infer<typeof ResponseSchema>

// Schema for a response from the API with data.
export const DataResponseSchema = <T extends z.ZodTypeAny>(schema: T) =>
	z.union([SuccessDataResponseSchema(schema), ErrorSchema])
