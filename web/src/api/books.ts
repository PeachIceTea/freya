import useSWR from "swr"
import { z } from "zod"

import {
	DataResponseSchema,
	ServerConnectionError,
	get,
	post,
	postMultipart,
} from "./api"

// Schema of book in the database.
export const BookSchema = z.object({
	id: z.number().int(),

	title: z.string(),
	author: z.string(),

	created: z.string(), // ISO8601 date string
	updated: z.string().optional(), // ISO8601 date string
})
export type Book = z.infer<typeof BookSchema>

export const Books = z.array(BookSchema)
export type Books = z.infer<typeof Books>

// Response schema for a list of books.
export const BooksResponseSchema = DataResponseSchema(Books)
export type BooksResponse = z.infer<typeof BooksResponseSchema>

// Send request
export const getBooks = async (): Promise<BooksResponse> => {
	const res = await get("/book")
	return BooksResponseSchema.parse(res)
}

export const useBooks = () => {
	const { data, error: parseError, isLoading } = useSWR("/book", getBooks)

	let error
	if (parseError) {
		console.error(parseError)
		error = ServerConnectionError
	} else if (!data && !isLoading) {
		error = ServerConnectionError
	}

	let books
	if (data?.success) {
		books = data.data
	} else {
		error = data
	}

	return { books, error, isLoading }
}

// Generate URL for book cover image.
export const bookCoverURL = (id: number) => `/api/book/${id}/cover`

// Upload book to server.
export const BookUploadSchema = z.object({
	book_id: z.number().int(),
})
export const BookUploadResponseSchema = DataResponseSchema(BookUploadSchema)
export type BookUploadResponse = z.infer<typeof BookUploadResponseSchema>

export const uploadBook = async (data: FormData) => {
	const res = await postMultipart("/book", data)
	return BookUploadResponseSchema.parse(res)
}
