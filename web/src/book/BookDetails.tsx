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
import type {
	BookDetails,
	BookDetailsResponse,
	Chapter,
	File,
} from "../api/books"
import {
	LibraryLists,
	LibraryListsSchema,
	addBookToLibrary as _addBookToLibrary,
} from "../api/library"
import { capitalize, fromSnakeCase, useTitle } from "../common"
import { useLocale } from "../locales"
import { useStore } from "../store"
import FileList from "./components/FileList"

function BookDetailsComponent({
	book,
	mutate,
}: {
	book: BookDetails
	mutate(shouldRevalidate?: boolean): Promise<BookDetailsResponse | undefined>
}) {
	const t = useLocale()
	const state = useStore()

	// Setup page title.
	let translationString = "book-details--title-placeholder"
	let translationData
	if (book) {
		translationString = "book-details--title"
		translationData = { title: book.title, author: book.author }
	}
	useTitle(translationString, translationData)

	// Get library from either the selected book or the book we are viewing.
	const library =
		state.selectedBook?.id === book.id
			? state.selectedBook.library
			: book.library

	// User library data.
	const hasListened =
		library && (library.progress > 0 || library.fileId !== book.files[0].id)
	const isFinished =
		book.library && book.library.list === LibraryListsSchema.Values.finished

	// Function to play book.
	async function playBook(item?: File | Chapter) {
		if (!book) {
			return
		}

		let playableBook
		if (item && "duration" in item) {
			playableBook = await ensureBookIsPlayable(item)
		} else {
			// Reset progress and put book in listening list if it's finished.
			if (book.library?.list === LibraryListsSchema.Values.finished) {
				await _addBookToLibrary(
					book.id,
					LibraryListsSchema.Values.listening,
					book.files[0],
				)
				await mutate(true)
			}

			playableBook = await ensureBookIsPlayable()
		}

		state.playBook(playableBook)
		if (item && "start" in item) {
			state.seekTo(item.start)
		}
	}

	async function addBookToLibrary(library: LibraryLists, file?: File) {
		await _addBookToLibrary(book.id, library, file)
		await mutate(true)
	}

	async function ensureBookIsPlayable(
		file?: File,
	): Promise<Required<BookDetails>> {
		if (!book) {
			throw new Error("Book not found")
		}

		if (book.library) {
			return book as Required<BookDetails>
		}

		// Create library entry.
		await addBookToLibrary(LibraryListsSchema.Values.listening, file)
		const res = await mutate(true)
		if (!res?.success || !res.data.library) {
			throw new Error("Failed to create library entry")
		}

		return res.data as Required<BookDetails>
	}

	// List button.
	const listDropdown = Object.values(LibraryListsSchema.Values)
		.filter(list => list !== book.library?.list)
		.map(list => (
			<Dropdown.Item key={list} onClick={() => addBookToLibrary(list)}>
				{t("book-details--add-to")}{" "}
				<span className="fst-italic">{capitalize(fromSnakeCase(list))}</span>
			</Dropdown.Item>
		))

	const listButton = (
		<Button
			variant="success"
			onClick={async () => {
				!book.library?.list &&
					(await addBookToLibrary(LibraryListsSchema.Values.want_to_listen))
			}}
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

	return (
		<Container className="grid mb-2">
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
				<div className="d-flex gap-3">
					<Button
						variant="primary"
						onClick={() => playBook()}
						disabled={state.playing && state.selectedBook?.id === book.id}
					>
						{t(
							state.playing && state.selectedBook?.id === book.id
								? "book-details--is-playing"
								: isFinished
									? "book-details--listen-again"
									: hasListened
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
				<ListGroup className="shadow-sm">
					<FileList
						book={
							book.id === state.selectedBook?.id ? state.selectedBook : book
						}
						currentlyPlaying={
							book.id === state.selectedBook?.id && state.playing
						}
						onClick={playBook}
					/>
				</ListGroup>
			</div>
		</Container>
	)
}

export default function BookDetails() {
	const t = useLocale()
	const { id } = useParams()

	const { book, error, isLoading, mutate } = useBook(+id!)

	if (error) {
		console.error(error)
		return (
			<Container>
				<Alert variant="danger">{t(error.errorCode)}</Alert>
			</Container>
		)
	}

	if (isLoading) {
		return null
	}

	if (!book) {
		return (
			<Container>
				<Alert variant="danger">Book not found</Alert>
			</Container>
		)
	}

	return <BookDetailsComponent book={book} mutate={mutate} />
}
