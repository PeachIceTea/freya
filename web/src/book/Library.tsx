import { useState } from "react"
import { Alert, Container, Form } from "react-bootstrap"
import { useParams } from "wouter"

import {
	type Library,
	LibraryLists,
	LibraryListsSchema,
	useLibrary,
} from "../api/library"
import { capitalize, fromSnakeCase, useTitle } from "../common"
import { useLocale } from "../locales"
import { useStore } from "../store"
import BookList from "./components/BookList"

function LibraryComponent({ library }: { library: Library }) {
	const t = useLocale()
	useTitle("library--title")

	const [listFilter, setListFilter] = useState<LibraryLists | null>("listening")
	const [searchFilter, setSearchFilter] = useState<string | null>(null)

	let filteredLibrary = searchFilter
		? library.filter(book =>
				`${book.title} ${book.author}`
					.toLowerCase()
					.includes(searchFilter.toLowerCase()),
			)
		: library

	const libraryLists = Object.values(LibraryListsSchema.Values).map(list => (
		<option key={list} value={list}>
			{capitalize(fromSnakeCase(list))} (
			{filteredLibrary.filter(book => book.list === list).length})
		</option>
	))

	filteredLibrary = filteredLibrary.filter(book =>
		listFilter === null ? true : book.list === listFilter,
	)

	return (
		<Container>
			<div className="d-flex flex-column flex-md-row justify-content-between align-items-center">
				<h1>{t("library--title")}</h1>
				<div className="d-flex flex-column flex-lg-row gap-2 mb-2">
					<Form.Control
						type="text"
						placeholder="Search"
						value={searchFilter || ""}
						onChange={e => setSearchFilter(e.target.value)}
					/>
					<Form.Select
						className="form-select w-auto"
						value={listFilter || ""}
						onChange={e => setListFilter(e.target.value as LibraryLists)}
					>
						{libraryLists}
					</Form.Select>
				</div>
			</div>
			<BookList books={filteredLibrary} />
		</Container>
	)
}

export default function Library() {
	const t = useLocale()
	const { id } = useParams()
	const state = useStore()

	const userId = id ? Number.parseInt(id, 10) : state.sessionInfo!.userId
	const { library, error, isLoading } = useLibrary(userId)

	if (isLoading) {
		return null
	}

	if (error) {
		return (
			<Container>
				<Alert variant="danger">{t(error.errorCode)}</Alert>
			</Container>
		)
	}

	if (!library) {
		return (
			<Container>
				<Alert variant="danger">Library not found</Alert>
			</Container>
		)
	}

	return <LibraryComponent library={library} />
}
