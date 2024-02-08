import { z } from "zod"

import { ResponseSchema, post } from "./api"

export const LibraryListsSchema = z.enum([
	"listening",
	"want_to_listen",
	"finished",
	"abandoned",
])
export type LibraryLists = z.infer<typeof LibraryListsSchema>

// Add a book to user's library.
export const addBookToLibrary = async (bookId: number, list: LibraryLists) => {
	const res = await post(`/book/${bookId}/library`, { list })
	return ResponseSchema.parse(res)
}

export const LibraryEntry = z.object({
	id: z.number(),
	fileId: z.number(),
	progress: z.number(),
	list: LibraryListsSchema,
	created: z.string(), // ISO 8601 date string
	modified: z.string(), // ISO 8601 date string
})
export type LibraryEntry = z.infer<typeof LibraryEntry>

// Update progress of a book in user's library.
export const updateProgress = async (
	bookId: number,
	fileId: number,
	progress: number,
) => {
	const res = await post(`/book/${bookId}/progress`, { fileId, progress })
	return ResponseSchema.parse(res)
}
