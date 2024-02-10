import { useState } from "react"
import { Alert, Container, Form } from "react-bootstrap"

import { useBooks } from "../api/books"
import { useLibrary } from "../api/library"
import { useTitle } from "../common"
import { useLocale } from "../locales"
import { useStore } from "../store"
import BookList from "./components/BookList"

export default function Books() {
	useTitle("books--title")
	const t = useLocale()
	const state = useStore()

	const [filterLibraryBooks, setFilterLibraryBooks] = useState<boolean>(true)
	const [searchFilter, setSearchFilter] = useState<string | null>(null)

	const { books, error, isLoading } = useBooks()
	const {
		library,
		error: libraryError,
		isLoading: libraryLoading,
	} = useLibrary(state.sessionInfo!.userId)

	if (isLoading || libraryLoading) {
		return <h1>Loading...</h1>
	}

	if (error || libraryError) {
		return (
			<Container>
				<h1>Books</h1>
				<Alert variant="danger">
					{t(libraryError?.errorCode ?? error?.errorCode)}
				</Alert>
			</Container>
		)
	}

	if (!books) {
		return (
			<Container>
				<h1>Books</h1>
				<Alert variant="danger">Couldn't get books.</Alert>
			</Container>
		)
	}

	let filteredBooks = searchFilter
		? books.filter(book =>
				`${book.title} ${book.author}`
					.toLowerCase()
					.includes(searchFilter.toLowerCase()),
			)
		: books

	filteredBooks =
		filterLibraryBooks && library
			? filteredBooks.filter(
					book => !library.some(libraryBook => libraryBook.id === book.id),
				)
			: filteredBooks

	return (
		<Container className="mb-2">
			<div className="d-flex flex-column flex-md-row justify-content-between align-items-center">
				<h1>{t("books--title")}</h1>
				<div className="d-flex flex-column flex-lg-row gap-2 align-items-center mb-2">
					<Form.Control
						type="text"
						placeholder="Search"
						className="w-auto"
						value={searchFilter || ""}
						onChange={e => setSearchFilter(e.target.value)}
					/>
					<div className="form-check form-switch">
						<input
							className="form-check-input"
							type="checkbox"
							id="flexSwitchCheckDefault"
							checked={filterLibraryBooks}
							onChange={() => setFilterLibraryBooks(!filterLibraryBooks)}
						/>
						<label
							className="form-check-label"
							htmlFor="flexSwitchCheckDefault"
						>
							{t("books--filter-library")}
						</label>
					</div>
				</div>
			</div>
			<BookList books={filteredBooks} />
		</Container>
	)
}
