import useSWR from "swr"
import { z } from "zod"

import { DataResponseSchema, ResponseSchema, get, post } from "./api"
import { File } from "./books"

export const LibraryListsSchema = z.enum([
	"listening",
	"want_to_listen",
	"finished",
	"abandoned",
])
export type LibraryLists = z.infer<typeof LibraryListsSchema>

// Add a book to user's library.
export const addBookToLibrary = async (
	bookId: number,
	list: LibraryLists,
	file?: File,
	progress?: number,
) => {
	const res = await post(`/book/${bookId}/library`, {
		list,
		fileId: file?.id,
		progress,
	})
	return ResponseSchema.parse(res)
}

export const LibraryEntrySchema = z.object({
	id: z.number(),
	fileId: z.number(),
	progress: z.number(),
	list: LibraryListsSchema,
	created: z.string(), // ISO 8601 date string
	modified: z.string(), // ISO 8601 date string
})
export type LibraryEntry = z.infer<typeof LibraryEntrySchema>

// Update progress of a book in user's library.
export const updateProgress = async (
	bookId: number,
	fileId: number,
	progress: number,
) => {
	const res = await post(`/book/${bookId}/progress`, { fileId, progress })
	return ResponseSchema.parse(res)
}

// Get user's library.
export const LibrarySchema = z.array(
	z.object({
		id: z.number(),
		title: z.string(),
		author: z.string(),
		list: LibraryListsSchema,
		progress: z.number(),
	}),
)
export type Library = z.infer<typeof LibrarySchema>

export const LibraryResponseSchema = DataResponseSchema(LibrarySchema)

export const getLibrary = async (id: number) => {
	const res = await get(`/user/${id}/library`)
	return LibraryResponseSchema.parse(res)
}

export const useLibrary = (id: number) => {
	const {
		data,
		error: parseError,
		isLoading,
	} = useSWR(`/user/${id}/library`, () => getLibrary(id))

	let error
	if (parseError) {
		console.error(parseError)
		error = parseError
	} else if (!data && !isLoading) {
		error = parseError
	}

	let library
	if (data?.success) {
		library = data.data
	} else {
		error = data
	}

	return { library, error, isLoading }
}
