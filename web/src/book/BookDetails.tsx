import { useEffect, useRef } from "react"
import {
	Alert,
	Button,
	ButtonGroup,
	Container,
	Dropdown,
	Image,
	ListGroup,
} from "react-bootstrap"
import { TbPlayerPauseFilled, TbPlayerPlayFilled } from "react-icons/tb"
import { useParams } from "wouter"

import { bookCoverURL, useBook } from "../api/books"
import type { BookDetails, BookDetailsResponse, File } from "../api/books"
import { LibraryListsSchema, addBookToLibrary } from "../api/library"
import { capitalize, formatDuration, fromSnakeCase, useTitle } from "../common"
import { useLocale } from "../locales"
import { useStore } from "../store"

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

	// Scroll to currently playing file once on load.
	const activeFileListItem = useRef<HTMLDivElement>(null)
	useEffect(() => {
		if (activeFileListItem.current) {
			activeFileListItem.current.scrollIntoView({
				behavior: "smooth",
				block: "center",
			})
		}
	}, [activeFileListItem, book?.id])

	// Function to play book.
	async function playBook(file?: File) {
		if (!book) {
			return
		}

		// Create library entry if it doesn't exist.
		let selectedBook: Required<BookDetails>
		if (!book.library || (file && book.library.fileId !== file?.id)) {
			await addBookToLibrary(
				book.id,
				// TODO: This should probably happen on the server.
				book.library?.list ?? LibraryListsSchema.Values.listening,
				file,
			)
			const updatedBook = await mutate(false)
			if (!updatedBook || !updatedBook.success || !updatedBook.data.library) {
				// This should never happen.
				console.error(
					"Book had no library data after adding to listening list.",
				)
				return
			}
			selectedBook = updatedBook.data as Required<BookDetails>
		} else {
			selectedBook = book as Required<BookDetails>
		}

		// Play the book.
		state.playBook(selectedBook)
	}

	// Get library from either the selected book or the book we are viewing.
	const library =
		state.selectedBook?.id === book.id
			? state.selectedBook.library
			: book.library
	const playingFileIndex = book.files.findIndex(
		file => file.id === library?.fileId,
	)
	// List of files.
	const fileList = book.files.map(file => {
		const fileIndex = book.files.indexOf(file)
		const progress =
			library?.fileId === file.id
				? (library.progress / file.duration) * 100
				: fileIndex < playingFileIndex
					? 100
					: 0

		const ref =
			fileIndex === playingFileIndex ? { ref: activeFileListItem } : undefined
		return (
			<ListGroup.Item key={file.id} className="p-0">
				<div
					className="mx-3 my-2 me-auto d-flex justify-content-between align-items-center"
					{...ref}
				>
					<div>
						<div className="fw-bold">{file.name}</div>
						<div className="text-secondary">
							Duration: {formatDuration(file.duration)}
						</div>
					</div>
					<div
						className="details-control me-2"
						role="button"
						onClick={() => {
							if (
								fileIndex === playingFileIndex &&
								state.selectedBook?.id === book.id
							) {
								state.togglePlay()
							} else {
								playBook(file)
							}
						}}
					>
						{state.playing &&
						state.selectedBook &&
						book?.id === state.selectedBook.id &&
						fileIndex === playingFileIndex ? (
							<TbPlayerPauseFilled />
						) : (
							<TbPlayerPlayFilled />
						)}
					</div>
				</div>
				<div
					style={{
						height: "0.25em",
						backgroundColor: "var(--bs-primary)",
						width: `${progress}%`,
					}}
				></div>
			</ListGroup.Item>
		)
	})

	// User library data.
	const hasListened =
		library && (library.progress > 0 || library.fileId !== book.files[0].id)

	// List button.
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
				<ListGroup className="shadow-sm">{fileList}</ListGroup>
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
