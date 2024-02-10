import { Badge, Card } from "react-bootstrap"
import { Link } from "wouter"

import { bookCoverURL } from "../../api/books"
import { capitalize, fromSnakeCase } from "../../common"

export default function BookList({ books }: { books: Book[] }) {
	return (
		<div className="grid">
			{books.map(book => (
				<Card
					key={book.id}
					className="g-col-12 g-col-md-6 g-col-xl-4 shadow-sm"
					role="button"
				>
					<Card.Img variant="top" src={bookCoverURL(book.id)} />
					<Card.Body>
						<Card.Title>{book.title}</Card.Title>
						<Card.Text className="d-flex justify-content-between align-items-center">
							{book.author}{" "}
							{book.list && (
								<Badge bg="primary">
									{capitalize(fromSnakeCase(book.list))}
								</Badge>
							)}
						</Card.Text>
					</Card.Body>
					<Link to={`/book/${book.id}`} className="stretched-link"></Link>
					{book.progress !== undefined && (
						<div className="rounded-bottom overflow-hidden">
							<div
								className="bg-primary"
								style={{
									height: "0.25em",
									width: `${book.progress * 100}%`,
								}}
							></div>
						</div>
					)}
				</Card>
			))}
		</div>
	)
}

interface Book {
	id: number
	title: string
	author: string
	list?: string
	progress?: number
}
