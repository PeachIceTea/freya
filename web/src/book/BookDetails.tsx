import {
	Alert,
	Button,
	ButtonGroup,
	Container,
	Dropdown,
	Image,
	ListGroup,
} from "react-bootstrap"
import { TbPlayerPlayFilled } from "react-icons/tb"
import { useParams } from "wouter"

import { bookCoverURL, useBook } from "../api/books"
import { LibraryListsSchema } from "../api/library"
import { capitalize, formatDuration, useTitle } from "../common"
import { useLocale } from "../locales"
import { useStore } from "../store"

export default function BookDetails() {
	const t = useLocale()
	const state = useStore()

	const { id } = useParams()

	const { book, error, isLoading } = useBook(+id!)

	// Setup page title.
	let translationString = "book-details--title-placeholder"
	let translationData
	if (book) {
		translationString = "book-details--title"
		translationData = { title: book.title, author: book.author }
	}
	useTitle(translationString, translationData)

	if (isLoading) {
		return null
	}

	if (error) {
		return (
			<Container>
				<Alert variant="error">{error.error_code}</Alert>
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

	return (
		<Container className="grid">
			<div
				className="g-col-12 g-col-md-4 sticky-md-top"
				style={{
					height: "fit-content",
					// Make sure it doesn't overlap the navbar. Bit hacky.
					// TODO: Find a better way. (Yeah, right.)
					top: `${document.querySelector(".navbar")?.clientHeight ?? 0}px`,
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
					<Button
						variant="primary"
						onClick={() => state.playBookFromStart(book)}
					>
						{t("book-details--start-listening")}
					</Button>
					<Dropdown as={ButtonGroup}>
						<Button variant="success">
							{t("book-details--add-to")}{" "}
							<span className="fst-italic">
								{capitalize(LibraryListsSchema.Values.listening)}
							</span>
						</Button>

						<Dropdown.Toggle split variant="success" />

						<Dropdown.Menu align={"end"}>
							{Object.values(LibraryListsSchema.Values).map(list => (
								<Dropdown.Item key={list}>
									{t("book-details--add-to")}{" "}
									<span className="fst-italic">{capitalize(list)}</span>
								</Dropdown.Item>
							))}
						</Dropdown.Menu>
					</Dropdown>
				</div>
			</div>
			<div className="g-col-12 g-col-md-8 mt-2">
				<ListGroup>{fileList}</ListGroup>
			</div>
		</Container>
	)
}
