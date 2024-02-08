import { Card, Container } from "react-bootstrap"
import { Link } from "wouter"

import { bookCoverURL, useBooks } from "../api/books"
import { useTitle } from "../common"
import { useLocale } from "../locales"

export default function Books() {
	useTitle("books--title")
	const t = useLocale()

	const { books, error, isLoading } = useBooks()

	if (isLoading) {
		return <h1>Loading...</h1>
	}

	if (error) {
		return (
			<Container>
				<h1>Books</h1>
				<p>Error: {t(error.errorCode)}</p>
			</Container>
		)
	}

	const booksList = books?.map(book => (
		<Card
			key={book.id}
			className="g-col-12 g-col-md-6 g-col-xl-4 shadow-sm"
			role="button"
		>
			<Card.Img variant="top" src={bookCoverURL(book.id)} />
			<Card.Body>
				<Card.Title>{book.title}</Card.Title>
				<Card.Text>{book.author}</Card.Text>
			</Card.Body>
			<Link to={`/book/${book.id}`} className="stretched-link"></Link>
		</Card>
	))

	return (
		<Container>
			<div className="d-flex justify-content-between align-items-center">
				<h1>{t("books--title")}</h1>
			</div>
			<div className="grid">{booksList}</div>
		</Container>
	)
}
