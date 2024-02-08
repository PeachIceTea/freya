import {
	Alert,
	Button,
	ButtonGroup,
	Container,
	Dropdown,
	Image,
	ListGroup,
} from "react-bootstrap"
import { useParams } from "wouter"

import { bookCoverURL, useBook } from "../api/books"
import type { BookDetails } from "../api/books"
import {
	LibraryLists,
	LibraryListsSchema,
	addBookToLibrary as _addBookToLibrary,
} from "../api/library"
import { capitalize, formatDuration, fromSnakeCase, useTitle } from "../common"
import { useLocale } from "../locales"
import { useStore } from "../store"

export default function BookDetails() {
	const t = useLocale()
	const state = useStore()

	const { id } = useParams()

	const { book, error, isLoading, mutate } = useBook(+id!)

	// Setup page title.
	let translationString = "book-details--title-placeholder"
	let translationData
	if (book) {
		translationString = "book-details--title"
		translationData = { title: book.title, author: book.author }
	}
	useTitle(translationString, translationData)

	// Guards.
	if (isLoading) {
		return null
	}

	if (error) {
		return (
			<Container>
				<Alert variant="error">{error.errorCode}</Alert>
			</Container>
		)
	}

	if (!book) {
		return (
			<Container>
				<Alert variant="error">Book not found</Alert>
			</Container>
		)
	}

	// List out the files.
	const fileList = book.files.map(file => (
		<ListGroup.Item key={file.id}>
			<div className="ms-2 me-auto d-flex justify-content-between align-items-center">
				<div>
					<div className="fw-bold">{file.name}</div>
					<div className="text-secondary">
						Duration: {formatDuration(file.duration)}
					</div>
				</div>
			</div>
		</ListGroup.Item>
	))

	// User library data.
	const hasListened =
		book.library &&
		(book.library.progress > 0 || book.library.fileId !== book.files[0].id)

	// List button.
	async function addBookToLibrary(bookId: number, list: LibraryLists) {
		await _addBookToLibrary(bookId, list)
		mutate()
	}

	const listDropdown = Object.values(LibraryListsSchema.Values)
		.filter(list => list !== book.library?.list)
		.map(list => (
			<Dropdown.Item key={list} onClick={() => addBookToLibrary(book.id, list)}>
				{t("book-details--add-to")}{" "}
				<span className="fst-italic">{capitalize(fromSnakeCase(list))}</span>
			</Dropdown.Item>
		))

	const listButton = (
		<Button
			variant="success"
			onClick={() =>
				!book.library?.list &&
				addBookToLibrary(book.id, LibraryListsSchema.Values.want_to_listen)
			}
			style={{
				pointerEvents: book.library?.list ? "none" : "auto",
			}}
		>
			{t(book.library?.list ? "book-details--is-in" : "book-details--add-to")}{" "}
			<span className="fst-italic">
				{capitalize(
					fromSnakeCase(
						book.library?.list ?? LibraryListsSchema.Values.want_to_listen,
					),
				)}
			</span>
		</Button>
	)

	// Function for start listening button.
	async function startListening() {
		// Create library entry.
		await addBookToLibrary(book!.id, LibraryListsSchema.Values.listening)

		// Fetch the book again.
		try {
			const res = await mutate(false)

			if (
				res === undefined ||
				res.success === false ||
				res.data.library === undefined
			) {
				// This should never happen.
				console.error(
					"Book had no library data after adding to listening list.",
					res,
				)
				return
			}

			// Play the book.
			// Not sure why we to cast here, but Typescript complains about library being possibly
			// undefined, even after we explicitly check for that in the if statement before.
			state.playBook(res.data as Required<BookDetails>)
		} catch (e) {
			// TODO: Inform user that something broke.
			console.error(e)
		}
	}

	return (
		<Container className="grid">
			<div
				className="g-col-12 g-col-md-4 sticky-md-top"
				style={{
					height: "fit-content",
				}}
			>
				<Image
					src={bookCoverURL(book.id)}
					alt={book.title}
					className="img-fluid rounded mt-2 shadow-sm"
				/>
				<div className="mt-2">
					<h4>{book.title}</h4>
					<h5 className="text-secondary">{book.author}</h5>
				</div>
				<div className="mt- d-flex gap-3">
					<Button variant="primary" onClick={startListening}>
						{t(
							hasListened
								? "book-details--continue-listening"
								: "book-details--start-listening",
						)}
					</Button>
					<Dropdown as={ButtonGroup}>
						{listButton}
						<Dropdown.Toggle split variant="success" />
						<Dropdown.Menu align={"end"}>{listDropdown}</Dropdown.Menu>
					</Dropdown>
				</div>
			</div>
			<div className="g-col-12 g-col-md-8 mt-2">
				<ListGroup className="shadow-sm">{fileList}</ListGroup>
			</div>
		</Container>
	)
}
