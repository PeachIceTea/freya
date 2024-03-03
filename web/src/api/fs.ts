import useSWR from "swr"
import { z } from "zod"

import { DataResponseSchema, ServerConnectionError, get } from "./api"

// List directory entries.
export const FileCategorySchema = z.enum([
	"directory",
	"audio",
	"image",
	"file",
])
export type FileCategory = z.infer<typeof FileCategorySchema>

export const EntrySchema = z.object({
	name: z.string(),
	path: z.string(),
	category: FileCategorySchema,
})
export const EntriesSchema = z.array(EntrySchema)
export type Entry = z.infer<typeof EntrySchema>
export type Entries = z.infer<typeof EntriesSchema>

export const EntriesResponseSchema = DataResponseSchema(
	z.object({
		path: z.string(),
		parentPath: z.string(),
		directory: EntriesSchema,
	}),
)
export type EntriesResponse = z.infer<typeof EntriesResponseSchema>

export const getDirectoryEntries = async (
	path: string,
): Promise<EntriesResponse> => {
	path = encodeURIComponent(path)
	const res = await get(`/fs?path=${path}`)
	return EntriesResponseSchema.parse(res)
}

export const useDirectoryEntries = (path: string) => {
	const {
		data,
		error: parseError,
		isLoading,
	} = useSWR(`/fs?path=${path}`, () => getDirectoryEntries(path))

	let error
	if (parseError) {
		console.error(parseError)
		error = ServerConnectionError
	} else if (!data && !isLoading) {
		error = ServerConnectionError
	}

	let info
	if (data?.success) {
		info = data.data
	} else {
		error = data
	}

	return { data: info, error, isLoading }
}

// Get ffprobe data for a file.
export const FileInfoSchema = z.object({
	title: z.string().optional(),
	author: z.string().optional(),
	cover: z.string().optional(),
})
export type FileInfo = z.infer<typeof FileInfoSchema>

export const FileInfoResponseSchema = DataResponseSchema(
	z.object({
		path: z.string(),
		info: FileInfoSchema,
	}),
)

export const getFileInfo = async (path: string) => {
	path = encodeURIComponent(path)
	const res = await get(`/fs/info?path=${path}`)
	return FileInfoResponseSchema.parse(res)
}

// Get URL for temporary cover image.
export const getTmpCoverImageURL = (name?: string) => {
	if (!name) {
		return undefined
	}

	if (name.startsWith("extracted-file://")) {
		name = name.slice(17)
	}

	console.log("getTmpCoverImageURL", name)

	return `/api/fs/tmp-cover?name=${name}`
}
