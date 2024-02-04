import { useParams } from "wouter"

export default function Book() {
	const { id } = useParams()

	return <h1>Book {id}</h1>
}
