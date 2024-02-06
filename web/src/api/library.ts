import { z } from "zod"

import { Response, post } from "./api"

export const LibraryListsSchema = z.enum([
	"listening",
	"want to listen",
	"finished",
	"abandoned",
])
export type LibraryLists = z.infer<typeof LibraryListsSchema>

// Add a book to user's library.
export const addBookToLibrary = async (
	bookId: number,
	list: LibraryLists,
): Response => {
	const res = await post(`/book/${bookId}/library`, { list })
}
