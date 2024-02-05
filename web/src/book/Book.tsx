import { Alert, Container, Image, ListGroup } from "react-bootstrap"
import { useParams } from "wouter"

import { bookCoverURL, useBook } from "../api/books"
import { formatDuration, useTitle } from "../common"
import { useLocale } from "../locales"

export default function Book() {
	const t = useLocale()
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
			<div className="ms-2 me-auto">
				<div className="fw-bold">{file.name}</div>
				<div>Duration: {formatDuration(file.duration)}</div>
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
				<h4>
					{t("book-details--title", {
						title: book.title,
						author: book.author,
					})}
				</h4>
			</div>
			<div className="g-col-12 g-col-md-8 mt-2">
				<ListGroup>{fileList}</ListGroup>
			</div>
		</Container>
	)
}
