import { z } from "zod"

export const LibraryListsSchema = z.enum([
	"listening",
	"want to listen",
	"finished",
])
export type LibraryLists = z.infer<typeof LibraryListsSchema>
