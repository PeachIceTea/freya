import { z } from "zod"

export const LibraryListsSchema = z.enum([
	"listening",
	"want to listen",
	"finished",
	"abandoned",
])
export type LibraryLists = z.infer<typeof LibraryListsSchema>
